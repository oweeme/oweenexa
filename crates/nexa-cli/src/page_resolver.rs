//! Resuelve una ruta HTTP a un HTML compilado — compartido por `nexa
//! preview` y `nexa dev` (Fase 11), que solo difieren en si vale la pena
//! mirar primero un `dist/<ruta>/index.html` ya generado por un `nexa
//! build` previo (`preview`: sí, es más rápido y es su propósito; `dev`:
//! no, porque editar el código fuente nunca debe servir un HTML viejo).

use std::fs;
use std::path::{Path, PathBuf};

use nexa_router::{match_route, scan_pages};

use crate::pipeline::{compile_page, CompiledPage, PageError};

pub enum ResolveError {
    NotFound,
    Failed(String),
}

/// Encuentra qué ruta de `src/pages` coincide con `path` y la compila
/// desde cero — sin ningún efecto secundario en `dist/` (a diferencia de
/// `resolve_page`, que sí cachea chunks/CSS ahí). Lo usa `resolve_page`
/// internamente, y el endpoint de diagnóstico de `nexa dev`
/// (`/__nexa_dev__/diagnostics`, Fase 14), que necesita el
/// `CompiledPage` completo (avisos, manifiesto...), no solo su HTML.
pub fn compile_matched_page(pages_dir: &Path, path: &str, api_base: &str) -> Result<CompiledPage, ResolveError> {
    let routes = scan_pages(pages_dir).map_err(|e| ResolveError::Failed(e.to_string()))?;
    let (route, params) = match_route(&routes, path).ok_or(ResolveError::NotFound)?;

    compile_page(&route.file, &route.pattern, &params, api_base).map_err(|err| match err {
        PageError::NotFound => ResolveError::NotFound,
        // `{e:#}` (el formato "alternate" de anyhow), no `{e}`: sin esto
        // solo se ve el contexto más externo ("compilando
        // src/pages/index.tsx"), y se pierde el mensaje real del parser
        // (ej. "Unexpected token") que es la causa encadenada debajo.
        PageError::Other(e) => ResolveError::Failed(format!("{e:#}")),
    })
}

/// 1. Si `use_prebuilt_html` y ya existe `dist/.../index.html`, se sirve
///    tal cual (el caso rápido de `nexa preview`).
/// 2. Si no, ¿alguna ruta de `src/pages` coincide? Se compila al vuelo,
///    con los parámetros que haya capturado el router (ej. `slug`) y
///    ejecutando su `load()` si lo declara. Sus chunks JS (y el CSS de
///    `@nexa/ui` que use esta página) se escriben a `dist/assets/` como
///    efecto secundario — son deterministas a partir del código fuente,
///    no dependen de los datos, así que cachearlos ahí es seguro y hace
///    que la siguiente petición del navegador por ese archivo concreto ya
///    lo encuentre.
/// 3. Si su `load()` responde 404, se propaga un 404 real — nunca un 200
///    con contenido vacío.
pub fn resolve_page(
    pages_dir: &Path,
    path: &str,
    api_base: &str,
    use_prebuilt_html: bool,
) -> Result<String, ResolveError> {
    if use_prebuilt_html {
        if let Some(html) = read_prebuilt_page(path) {
            return Ok(html);
        }
    }

    let page = compile_matched_page(pages_dir, path, api_base)?;

    let _ = fs::create_dir_all("dist/assets");

    for chunk in &page.chunks {
        let _ = fs::write(format!("dist/assets/{}", chunk.filename), &chunk.content);
    }

    // Nota: esto sobrescribe con el CSS de *esta* página, no con la unión
    // de todo el sitio (eso solo lo sabe `nexa build`, que ve todas las
    // páginas a la vez). Suficiente para desarrollar una página a la vez;
    // `nexa build` es quien produce el bundle correcto para producción.
    if let Some(css) = nexa_ui::build_stylesheet_from_classes(&page.ui_used_classes) {
        let _ = fs::write("dist/assets/nexa-ui.css", css);
    }

    Ok(page.html)
}

fn read_prebuilt_page(path: &str) -> Option<String> {
    let dir = if path == "/" || path.is_empty() {
        PathBuf::from("dist")
    } else {
        Path::new("dist").join(path.trim_start_matches('/'))
    };
    fs::read_to_string(dir.join("index.html")).ok()
}

/// Chunks de nodos interactivos (`/assets/ProductPage-3.js`) y el CSS de
/// `@nexa/ui`: los escribe `nexa build`, o los deja escritos
/// `resolve_page` la primera vez que una ruta se renderiza al vuelo.
pub fn read_prebuilt_asset(path: &str) -> Option<String> {
    let rest = path.strip_prefix("/assets/")?;
    fs::read_to_string(Path::new("dist/assets").join(rest)).ok()
}

/// Último recurso antes de un 404: un archivo estático bajo `base`
/// (Fase 14) — binario, a diferencia de `read_prebuilt_asset`, que
/// asume texto (JS/CSS). `nexa preview` lo busca bajo `dist/` (lo que
/// `nexa build` ya copió de `public/`); `nexa dev` lo busca directo bajo
/// `public/` — consistente con que `nexa dev` nunca depende de que haya
/// corrido un build antes.
pub fn read_static_file(base: &Path, path: &str) -> Option<Vec<u8>> {
    let rest = path.trim_start_matches('/');
    if rest.is_empty() {
        return None;
    }
    let file = base.join(rest);
    if !file.is_file() {
        return None;
    }
    fs::read(file).ok()
}
