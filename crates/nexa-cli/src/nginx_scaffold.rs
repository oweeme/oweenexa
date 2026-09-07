//! `nexa add nginx` (Fase 20): genera un `deploy/nginx.conf` real para
//! servir `dist/` — mismo patrón que `tauri_scaffold`/`capacitor_scaffold`
//! (Fase 13): un archivo real, generado a partir de una plantilla, no
//! texto suelto pegado a mano.
//!
//! **Sobre el cacheo de `/assets/`, a propósito NO agresivo:** los
//! archivos de `nexa build` (`nexa-runtime.js`, chunks de activación
//! como `Home-15.js`) todavía no llevan un hash de contenido en el
//! nombre — un chunk puede cambiar de contenido en un redeploy sin que
//! su nombre de archivo cambie. `Cache-Control: immutable` con un
//! `max-age` de un año (la receta típica para assets con hash) serviría
//! JS viejo indefinidamente a un visitante que vuelve después de un
//! redeploy. Por eso este `nginx.conf` usa un `max-age` corto — es
//! honesto con el estado actual de Nexa, no una plantilla genérica de
//! internet copiada sin pensar. El día que los nombres de archivo
//! lleven un hash, esta plantilla puede volverse agresiva de verdad.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

/// `server_name`/rutas son genéricas a propósito — el usuario las ajusta
/// a su dominio real. `try_files` sirve cualquier ruta estática que
/// `nexa build` haya generado (incluidas las rutas dinámicas
/// pre-renderizadas por `paths`, Fase 17); una ruta dinámica que NO
/// declaró `paths` no existe como archivo y cae al `404` — para esas
/// hace falta `nexa preview` corriendo detrás (ver el bloque comentado
/// de `proxy_pass` al final).
const TEMPLATE: &str = r#"# Generado por `nexa add nginx` — revisá y ajustá `server_name` y `root`
# antes de usarlo en producción. Pensado para el `dist/` que produce
# `nexa build`: HTML/CSS/JS estáticos + lo que `[pwa]` (Fase 18) haya
# generado (manifest.webmanifest, sw.js) + imágenes AVIF (Fase 19).
server {
    listen 80;
    listen [::]:80;
    server_name _;                      # <- poné tu dominio real acá
    root /var/www/{{NAME}}/dist;
    index index.html;

    # `text/html` no va en la lista: nginx ya lo comprime siempre que
    # `gzip on` está activo — declararlo de nuevo en `gzip_types` es un
    # error real de sintaxis, no cosmético (`nginx -t` lo marca como
    # "duplicate MIME type" — encontrado validando este archivo con un
    # nginx real).
    gzip on;
    gzip_min_length 512;
    gzip_types text/css application/javascript text/javascript application/json
               image/svg+xml application/manifest+json application/xml;

    # Cabeceras de seguridad — las mismas que ya aplica `nexa preview`/
    # `nexa dev` en desarrollo, para que el comportamiento sea consistente.
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Cacheo corto, no "immutable": ver el comentario del módulo que
    # genera este archivo (nginx_scaffold.rs) sobre por qué.
    location /assets/ {
        add_header Cache-Control "public, max-age=3600";
    }

    location / {
        try_files $uri $uri/ $uri/index.html =404;
    }

    # Si tu proyecto tiene rutas dinámicas SIN `paths` declarado (Fase
    # 17), `try_files` de arriba nunca las va a encontrar como archivo
    # — necesitás un `nexa preview` corriendo detrás y reenviarle esas
    # rutas. Ejemplo (ajustá el puerto y las rutas):
    #
    # location /products/ {
    #     try_files $uri $uri/index.html @nexa_preview;
    # }
    # location @nexa_preview {
    #     proxy_pass http://127.0.0.1:4321;
    #     proxy_set_header Host $host;
    # }
}
"#;

pub fn render(project_name: &str) -> String {
    TEMPLATE.replace("{{NAME}}", &sanitize_name(project_name))
}

/// Genera `<project_root>/deploy/nginx.conf`. Falla si ya existe — igual
/// que `tauri_scaffold`/`capacitor_scaffold`, no se sobrescribe trabajo
/// del usuario en silencio.
pub fn scaffold(project_root: &Path, project_name: &str) -> Result<()> {
    let deploy_dir = project_root.join("deploy");
    let out_path = deploy_dir.join("nginx.conf");
    if out_path.exists() {
        bail!("{} ya existe — bórralo primero si quieres regenerarlo", out_path.display());
    }

    fs::create_dir_all(&deploy_dir)?;
    fs::write(&out_path, render(project_name)).with_context(|| format!("escribiendo {}", out_path.display()))
}

fn sanitize_name(name: &str) -> String {
    let cleaned: String =
        name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    if cleaned.is_empty() {
        "app".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-nginx-scaffold-test-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn render_substitutes_the_project_name_into_the_root_path() {
        let conf = render("mi-tienda");
        assert!(conf.contains("root /var/www/mi-tienda/dist;"));
        assert!(!conf.contains("{{NAME}}"), "no debe quedar ningún placeholder sin sustituir");
    }

    #[test]
    fn render_sanitizes_a_name_with_spaces_and_uppercase() {
        let conf = render("Mi Tienda Real");
        assert!(conf.contains("root /var/www/Mi-Tienda-Real/dist;"));
    }

    #[test]
    fn render_includes_the_same_security_headers_that_serve_rs_applies() {
        let conf = render("app");
        assert!(conf.contains(r#"add_header X-Content-Type-Options "nosniff" always;"#));
        assert!(conf.contains(r#"add_header X-Frame-Options "SAMEORIGIN" always;"#));
        assert!(conf.contains(r#"add_header Referrer-Policy "strict-origin-when-cross-origin" always;"#));
    }

    #[test]
    fn render_never_lists_text_html_in_gzip_types() {
        // Bug real, encontrado validando el archivo con un nginx real
        // (`nginx -t`): nginx ya comprime text/html siempre que `gzip
        // on` está activo — declararlo de nuevo en `gzip_types` dispara
        // "duplicate MIME type" como warning de sintaxis real.
        let conf = render("app");
        let gzip_line = conf.lines().find(|line| line.trim_start().starts_with("gzip_types")).unwrap();
        let continuation = conf.lines().skip_while(|l| !l.contains("gzip_types")).nth(1).unwrap_or("");
        assert!(!gzip_line.contains("text/html"));
        assert!(!continuation.contains("text/html"));
    }

    #[test]
    fn render_never_recommends_a_year_long_immutable_cache_for_assets() {
        // Los nombres de archivo de nexa build no llevan hash de
        // contenido todavía — "immutable" serviría JS viejo para
        // siempre después de un redeploy. Se busca la línea real de
        // `Cache-Control` (no todo el archivo: el propio comentario que
        // explica esta decisión menciona la palabra "immutable" a
        // propósito, y no debería contar como una falla).
        let conf = render("app");
        let cache_control_line =
            conf.lines().find(|line| line.contains("Cache-Control")).expect("se espera una línea Cache-Control");
        assert!(!cache_control_line.contains("immutable"));
        assert!(!cache_control_line.contains("max-age=31536000"));
    }

    #[test]
    fn scaffold_writes_a_real_file_under_deploy() {
        let dir = scratch_dir();
        scaffold(&dir, "app").unwrap();
        assert!(dir.join("deploy/nginx.conf").is_file());
    }

    #[test]
    fn scaffold_refuses_to_overwrite_an_existing_config() {
        let dir = scratch_dir();
        scaffold(&dir, "app").unwrap();
        assert!(scaffold(&dir, "app").is_err());
    }
}
