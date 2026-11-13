//! Fase 60 — Modo App (SPA real).
//!
//! Nexa Core nunca ejecuta ni interpreta el código de una página: solo
//! reconoce patrones fijos. Eso es una fortaleza para sitios de
//! contenido y un muro para una aplicación. El Modo App no cambia esa
//! regla: añade un **segundo tipo de ruta** que convive con el primero.
//!
//! - `src/pages/**`  -> Nexa Core: HTML real, analizado, SEO.
//! - `src/app/pages/**` -> Modo App: TSX completo (`.map`, condicionales,
//!   componentes con props, estado) que se compila a JS plano y se
//!   ejecuta en el navegador sobre `@nexa/app` — DOM real con señales,
//!   sin Virtual DOM. El servidor solo entrega un *shell* HTML mínimo.
//!
//! Este módulo NO analiza el contenido de esos archivos: solo los
//! transpila (TypeScript + JSX -> JS) con `oxc` y reescribe sus imports
//! para que el navegador los resuelva (`./Card` -> `./Card.js`,
//! `@nexa/app` -> el runtime con hash).
//!
//! Estructura reconocida:
//!
//! ```text
//! src/app/pages/index.tsx          -> /app
//! src/app/pages/users/[id].tsx     -> /app/users/:id
//! src/app/components/*.tsx         -> importables con rutas relativas
//! src/app/config.ts (opcional)     -> export const guard / notFound / loading
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{JsxOptions, JsxRuntime, TransformOptions, Transformer};

use crate::manifest::Manifest;

pub const APP_DIR: &str = "src/app";
pub const DEFAULT_BASE: &str = "/app";
/// Dónde quedan los módulos compilados, bajo `dist/`.
pub const OUT_PREFIX: &str = "assets/app";

pub fn app_dir() -> PathBuf {
    PathBuf::from(APP_DIR)
}

/// ¿El proyecto usa el Modo App? Basta con que exista `src/app/pages/`.
pub fn is_enabled() -> bool {
    Path::new(APP_DIR).join("pages").is_dir()
}

/// Prefijo de URL de la SPA, normalizado: empieza con `/`, sin `/` final.
pub fn base_path(manifest: &Manifest) -> String {
    let raw = manifest.app.as_ref().and_then(|a| a.base.clone()).unwrap_or_else(|| DEFAULT_BASE.to_string());
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return DEFAULT_BASE.to_string();
    }
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

/// ¿`path` cae dentro de la SPA (`/app` o `/app/...`)?
pub fn owns_path(base: &str, path: &str) -> bool {
    path == base || path.strip_prefix(base).is_some_and(|rest| rest.starts_with('/'))
}

#[derive(Debug, Clone)]
pub struct AppRouteEntry {
    /// Patrón relativo a la base: `/users/:id`.
    pub pattern: String,
    /// URL absoluta del módulo compilado.
    pub module_url: String,
}

#[derive(Debug)]
pub struct AppBuild {
    pub routes: Vec<AppRouteEntry>,
    pub has_config: bool,
    /// Ruta relativa a `dist/` -> contenido, de cada módulo compilado.
    pub modules: BTreeMap<String, String>,
}

/// TypeScript + JSX -> JavaScript plano. JSX se compila a llamadas
/// `h(...)`/`Fragment` de `@nexa/app` (runtime clásico: sin
/// auto-import mágico, así el resultado es predecible).
pub fn transpile(source: &str, path: &Path) -> Result<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).map_err(|e| anyhow!("tipo de archivo no soportado {}: {e}", path.display()))?;

    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        let first = parsed
            .diagnostics
            .first()
            .map(|e| e.message.to_string())
            .unwrap_or_else(|| "error de sintaxis".to_string());
        bail!("{}: {first}", path.display());
    }
    let mut program = parsed.program;

    let semantic = SemanticBuilder::new().build(&program);
    let scoping = semantic.semantic.into_scoping();

    let mut options = TransformOptions::default();
    options.jsx = JsxOptions {
        runtime: JsxRuntime::Classic,
        pragma: Some("__nexa_h".to_string()),
        pragma_frag: Some("__nexa_Fragment".to_string()),
        pure: false,
        ..JsxOptions::default()
    };

    let transformed = Transformer::new(&allocator, path, &options).build_with_scoping(scoping, &mut program);
    if let Some(first) = transformed.diagnostics.first() {
        bail!("{}: {}", path.display(), first.message);
    }

    let code = Codegen::new().build(&program).code;

    // El usuario nunca escribe `import { h }`: se inyecta con nombres
    // propios (`__nexa_h`) para no chocar con un `h` suyo.
    let mut header = String::new();
    let uses_h = code.contains("__nexa_h(") || code.contains("__nexa_h,");
    let uses_frag = code.contains("__nexa_Fragment");
    if uses_h || uses_frag {
        let mut names = Vec::new();
        if uses_h {
            names.push("h as __nexa_h");
        }
        if uses_frag {
            names.push("Fragment as __nexa_Fragment");
        }
        header = format!("import {{ {} }} from \"@nexa/app\";\n", names.join(", "));
    }
    Ok(format!("{header}{code}"))
}

/// Reescribe los especificadores de import del JS ya transpilado:
/// - `@nexa/app` -> `/assets/<runtime con hash>`
/// - relativos (`./Card`, `../lib/api`) -> con `.js` o `/index.js` si el
///   archivo fuente (`.ts`/`.tsx`/`.js`/`.jsx`) existe junto al original.
///
/// Todo lo demás (URLs, nombres del import map de `[imports]`) queda
/// intacto — es el import map del shell quien lo resuelve.
pub fn rewrite_imports(js: &str, source_file: &Path, runtime_url: &str) -> String {
    let mut out = String::with_capacity(js.len());
    let mut rest = js;

    // Los tres lugares donde codegen escribe un especificador: `from "x"`,
    // `import "x"` y `import("x")`.
    const MARKERS: [&str; 3] = ["from \"", "import \"", "import(\""];

    loop {
        let next = MARKERS
            .iter()
            .filter_map(|m| rest.find(m).map(|i| (i, m.len())))
            .min_by_key(|(i, _)| *i);
        let Some((idx, mlen)) = next else {
            out.push_str(rest);
            break;
        };
        let spec_start = idx + mlen;
        out.push_str(&rest[..spec_start]);

        let after = &rest[spec_start..];
        let Some(end) = after.find('"') else {
            out.push_str(after);
            break;
        };
        let spec = &after[..end];
        out.push_str(&rewrite_specifier(spec, source_file, runtime_url));
        rest = &after[end..];
    }
    out
}

fn rewrite_specifier(spec: &str, source_file: &Path, runtime_url: &str) -> String {
    // Todo el ecosistema @nexa/* del navegador vive en un solo bundle.
    if spec == "@nexa/app" || spec == "@nexa/reactivity" {
        return runtime_url.to_string();
    }
    if !(spec.starts_with("./") || spec.starts_with("../")) {
        return spec.to_string();
    }
    let dir = source_file.parent().unwrap_or_else(|| Path::new("."));
    let target = dir.join(spec);

    // Si ya trae una extensión de código, se normaliza a `.js`.
    for ext in ["tsx", "ts", "jsx", "mjs"] {
        if let Some(stem) = spec.strip_suffix(&format!(".{ext}")) {
            return format!("{stem}.js");
        }
    }
    if spec.ends_with(".js") {
        return spec.to_string();
    }
    for ext in ["tsx", "ts", "jsx", "js"] {
        if target.with_file_name(format!("{}.{ext}", target.file_name().and_then(|n| n.to_str()).unwrap_or(""))).is_file() {
            return format!("{spec}.js");
        }
    }
    for ext in ["tsx", "ts", "jsx", "js"] {
        if target.join(format!("index.{ext}")).is_file() {
            return format!("{}/index.js", spec.trim_end_matches('/'));
        }
    }
    spec.to_string()
}

fn is_code_file(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()), Some("ts" | "tsx" | "js" | "jsx")) && !path.to_string_lossy().ends_with(".d.ts")
}

fn collect_code_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("leyendo {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_code_files(&path, out)?;
        } else if is_code_file(&path) {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

/// Ruta de salida (relativa a `dist/`) de un archivo fuente de `src/app`.
pub fn output_rel_path(app_root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(app_root).unwrap_or(file).with_extension("js");
    format!("{OUT_PREFIX}/{}", rel.to_string_lossy().replace('\\', "/"))
}

/// Transpila todo `src/app/**` y descubre las rutas de `src/app/pages`.
pub fn compile(app_root: &Path, runtime_url: &str) -> Result<AppBuild> {
    let pages_dir = app_root.join("pages");
    let routes = nexa_router::scan_pages(&pages_dir).with_context(|| format!("escaneando {}", pages_dir.display()))?;

    let mut files = Vec::new();
    collect_code_files(app_root, &mut files)?;

    let mut modules = BTreeMap::new();
    for file in &files {
        let source = fs::read_to_string(file).with_context(|| format!("leyendo {}", file.display()))?;
        let js = transpile(&source, file).with_context(|| format!("compilando {}", file.display()))?;
        let js = rewrite_imports(&js, file, runtime_url);
        modules.insert(output_rel_path(app_root, file), js);
    }

    let entries = routes
        .iter()
        .map(|route| AppRouteEntry { pattern: route.pattern.clone(), module_url: format!("/{}", output_rel_path(app_root, &route.file)) })
        .collect();

    let has_config = ["config.ts", "config.tsx", "config.js"].iter().any(|n| app_root.join(n).is_file());

    Ok(AppBuild { routes: entries, has_config, modules })
}

/// El HTML mínimo que sirve toda ruta de la SPA. No hay contenido
/// renderizado en servidor: la página la arma `@nexa/app` en el
/// navegador, así que este documento es idéntico para cualquier ruta.
pub fn shell_html(manifest: &Manifest, build: &AppBuild, base: &str, runtime_url: &str, desktop: bool) -> String {
    let title = manifest
        .app
        .as_ref()
        .and_then(|a| a.title.clone())
        .or_else(|| (!manifest.project.name.is_empty()).then(|| manifest.project.name.clone()))
        .unwrap_or_else(|| "App".to_string());

    let imports = crate::import_map::resolved(manifest);
    // Solo se declaran los imports de `[imports]` explícitos del proyecto:
    // los builtin (`platform`...) se resuelven igual que en una página Nexa.
    let importmap = crate::import_map::script_tag(&imports).unwrap_or_default();

    // Fase 63: en una app de escritorio la API no es del mismo origen.
    let proxy_table = if desktop { desktop_proxy(manifest) } else { BTreeMap::new() };
    let proxy_script = if proxy_table.is_empty() {
        String::new()
    } else {
        format!("<script>window.__NEXA_PROXY__={};</script>\n", serde_json::to_string(&proxy_table).unwrap_or_else(|_| "{}".into()))
    };

    let route_lines: Vec<String> = build
        .routes
        .iter()
        .map(|r| format!("    {{ pattern: {}, load: () => import({}) }},", js_str(&r.pattern), js_str(&r.module_url)))
        .collect();

    let (config_import, config_use) = if build.has_config {
        (
            format!("import * as config from {};\n", js_str(&format!("/{OUT_PREFIX}/config.js"))),
            "  guard: config.guard,\n  notFound: config.notFound ? (p) => config.notFound(p) : undefined,\n  loading: config.loading,\n",
        )
    } else {
        (String::new(), "")
    };

    format!(
        "<!doctype html>\n<html lang=\"es\">\n<head>\n<meta charset=\"UTF-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <link rel=\"icon\" href=\"data:,\">\n<title>{title}</title>\n{importmap}\n{proxy_script}</head>\n<body>\n<div id=\"app\"></div>\n\
         <script type=\"module\">\n\
         import {{ startApp }} from {runtime};\n{config_import}\
         startApp({{\n  base: {base},\n  routes: [\n{routes}\n  ],\n{config_use}}});\n\
         </script>\n</body>\n</html>\n",
        title = html_escape(&title),
        importmap = importmap,
        proxy_script = proxy_script,
        runtime = js_str(runtime_url),
        config_import = config_import,
        base = js_str(base),
        routes = route_lines.join("\n"),
        config_use = config_use,
    )
}

/// `[desktop.proxy]`, o `[proxy]` si el proyecto no declara una tabla propia.
pub fn desktop_proxy(manifest: &Manifest) -> BTreeMap<String, String> {
    match &manifest.desktop {
        Some(d) if !d.proxy.is_empty() => d.proxy.clone(),
        _ => manifest.proxy.clone(),
    }
}

fn js_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// URL pública del runtime con hash.
pub fn runtime_url() -> String {
    format!("/assets/{}", crate::assets::nexa_app_filename())
}

/// Escribe todo a `dist/`: runtime, módulos y shell.
pub fn write_to_dist(manifest: &Manifest, dist: &Path, desktop: bool) -> Result<AppBuild> {
    let url = runtime_url();
    let build = compile(&app_dir(), &url)?;
    let base = base_path(manifest);

    fs::create_dir_all(dist.join("assets"))?;
    fs::write(dist.join("assets").join(crate::assets::nexa_app_filename()), crate::assets::NEXA_APP_JS)?;

    for (rel, js) in &build.modules {
        let target = dist.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, js)?;
    }

    let shell = shell_html(manifest, &build, &base, &url, desktop);
    let shell_dir = dist.join(base.trim_start_matches('/'));
    fs::create_dir_all(&shell_dir)?;
    fs::write(shell_dir.join("index.html"), &shell)?;
    // Convención de hosts de SPA (Netlify, Cloudflare Pages...): un
    // archivo de respaldo para cualquier ruta profunda.
    fs::write(shell_dir.join("200.html"), &shell)?;
    Ok(build)
}

/// Lo que `nexa preview`/`nexa dev` responden para una ruta del Modo App.
pub enum AppResponse {
    Js(String),
    Html(String),
    /// Error de compilación de un módulo — se muestra legible en el navegador.
    Failure(String),
}

/// Sondeo de `nexa dev`: recarga la página completa cuando cambia algo
/// bajo `src/`. El shell de una SPA no se puede "reemplazar el body"
/// como una página Nexa Core (un `<script type=module>` insertado así
/// nunca se vuelve a ejecutar), así que acá recargar es lo correcto.
const DEV_RELOAD_SCRIPT: &str = "<script>(function(){var v=null;setInterval(function(){fetch('/__nexa_dev__/version').then(function(r){return r.text()}).then(function(t){if(v===null)v=t;else if(t!==v)location.reload()}).catch(function(){})},800)})()</script>";

/// Responde si `path` pertenece al Modo App (el shell, o un módulo
/// compilado bajo `/assets/app/`). `None` si no es suyo.
///
/// - `dev`: siempre recompila desde `src/app` (nunca sirve un `dist/` viejo).
/// - `preview`: sirve lo que `nexa build` ya dejó en `dist/`, y cae a
///   compilar al vuelo si falta (igual que las rutas dinámicas de Core).
pub fn serve(path: &str, manifest: &Manifest, dev: bool) -> Option<AppResponse> {
    if !is_enabled() {
        return None;
    }
    let base = base_path(manifest);

    if let Some(rest) = path.strip_prefix("/assets/app/") {
        let rel = format!("{OUT_PREFIX}/{rest}");
        if !dev {
            if let Ok(js) = fs::read_to_string(Path::new("dist").join(&rel)) {
                return Some(AppResponse::Js(js));
            }
        }
        return Some(match compile(&app_dir(), &runtime_url()) {
            Ok(build) => match build.modules.get(&rel) {
                Some(js) => AppResponse::Js(js.clone()),
                None => return None,
            },
            Err(e) => AppResponse::Failure(format!("{e:#}")),
        });
    }

    if owns_path(&base, path) {
        if !dev {
            let built = Path::new("dist").join(base.trim_start_matches('/')).join("index.html");
            if let Ok(html) = fs::read_to_string(built) {
                return Some(AppResponse::Html(html));
            }
        }
        return Some(match compile(&app_dir(), &runtime_url()) {
            Ok(build) => {
                let shell = shell_html(manifest, &build, &base, &runtime_url(), false);
                AppResponse::Html(if dev { shell.replace("</head>", &format!("{DEV_RELOAD_SCRIPT}\n</head>")) } else { shell })
            }
            Err(e) => AppResponse::Failure(format!("{e:#}")),
        });
    }

    None
}

/// Un proyecto que solo tiene `src/app/` (sin `src/pages/`) no tiene
/// una portada Core: `/` lleva a la SPA.
pub fn redirect_html(base: &str) -> String {
    let b = html_escape(base);
    format!(
        "<!doctype html>\n<html lang=\"es\"><head><meta charset=\"UTF-8\">\n\
         <meta http-equiv=\"refresh\" content=\"0; url={b}\">\n<title>Redirigiendo…</title></head>\n\
         <body><a href=\"{b}\">Ir a la aplicación</a></body></html>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nexa-app-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn base_is_normalized() {
        let mut m = Manifest::default();
        assert_eq!(base_path(&m), "/app");
        m.app = Some(crate::manifest::AppSection { base: Some("panel/".into()), title: None });
        assert_eq!(base_path(&m), "/panel");
    }

    #[test]
    fn owns_path_respects_segment_boundaries() {
        assert!(owns_path("/app", "/app"));
        assert!(owns_path("/app", "/app/users/1"));
        assert!(!owns_path("/app", "/apple"));
        assert!(!owns_path("/app", "/"));
    }

    #[test]
    fn transpile_strips_typescript_and_compiles_jsx_to_h_calls() {
        let src = r#"
            import { state } from "@nexa/app";
            interface P { n: number }
            export default function Page(props: P) {
                const c = state<number>(props.n);
                return <button class="x" onClick={() => c.value++}>{() => c.value}</button>;
            }
        "#;
        let js = transpile(src, Path::new("page.tsx")).unwrap();
        assert!(!js.contains("interface"), "{js}");
        assert!(!js.contains(": number"), "{js}");
        assert!(js.contains("__nexa_h(\"button\""), "{js}");
        assert!(js.starts_with("import { h as __nexa_h } from \"@nexa/app\""), "{js}");
        assert!(!js.contains("<button"), "{js}");
    }

    #[test]
    fn transpile_compiles_fragments_with_the_fragment_pragma() {
        let js = transpile("export default () => <><i/><b/></>;", Path::new("f.tsx")).unwrap();
        assert!(js.contains("__nexa_h(__nexa_Fragment"), "{js}");
    }

    #[test]
    fn transpile_reports_a_syntax_error_with_the_file_name() {
        let err = transpile("export default function ( {", Path::new("broken.tsx")).unwrap_err();
        assert!(format!("{err:#}").contains("broken.tsx"));
    }

    #[test]
    fn rewrite_imports_maps_runtime_and_relative_files() {
        let dir = scratch("rewrite");
        fs::create_dir_all(dir.join("ui")).unwrap();
        fs::write(dir.join("Card.tsx"), "").unwrap();
        fs::write(dir.join("ui/index.ts"), "").unwrap();
        fs::write(dir.join("api.ts"), "").unwrap();
        let file = dir.join("page.tsx");

        let js = "import { h } from \"@nexa/app\";\nimport Card from \"./Card\";\nimport { b } from \"./ui\";\nimport { a } from \"./api.ts\";\nimport z from \"zod\";\nconst m = import(\"./Card\");\n";
        let out = rewrite_imports(js, &file, "/assets/nexa-app.abc.js");
        assert!(out.contains("from \"/assets/nexa-app.abc.js\""), "{out}");
        assert!(out.contains("from \"./Card.js\""), "{out}");
        assert!(out.contains("from \"./ui/index.js\""), "{out}");
        assert!(out.contains("from \"./api.js\""), "{out}");
        assert!(out.contains("from \"zod\""), "{out}");
        assert!(out.contains("import(\"./Card.js\")"), "{out}");
    }

    #[test]
    fn compile_discovers_routes_and_transpiles_every_module() {
        let root = scratch("compile");
        fs::create_dir_all(root.join("pages/users")).unwrap();
        fs::create_dir_all(root.join("components")).unwrap();
        fs::write(root.join("pages/index.tsx"), "export default () => <p>hola</p>;").unwrap();
        fs::write(root.join("pages/users/[id].tsx"), "import C from \"../../components/C\";\nexport default (p) => <C id={p.params.id}/>;").unwrap();
        fs::write(root.join("components/C.tsx"), "export default (p: {id: string}) => <b>{p.id}</b>;").unwrap();

        let build = compile(&root, "/assets/rt.js").unwrap();
        let patterns: Vec<_> = build.routes.iter().map(|r| r.pattern.as_str()).collect();
        assert!(patterns.contains(&"/") && patterns.contains(&"/users/:id"), "{patterns:?}");
        assert_eq!(build.modules.len(), 3);
        let user = &build.modules["assets/app/pages/users/[id].js"];
        assert!(user.contains("../../components/C.js"), "{user}");
        assert!(!build.has_config);
    }

    #[test]
    fn shell_lists_the_routes_as_lazy_imports() {
        let build = AppBuild {
            routes: vec![AppRouteEntry { pattern: "/users/:id".into(), module_url: "/assets/app/pages/users/[id].js".into() }],
            has_config: true,
            modules: BTreeMap::new(),
        };
        let html = shell_html(&Manifest::default(), &build, "/app", "/assets/nexa-app.x.js", false);
        assert!(html.contains("<div id=\"app\"></div>"));
        assert!(html.contains("import(\"/assets/app/pages/users/[id].js\")"), "{html}");
        assert!(html.contains("base: \"/app\""));
        assert!(html.contains("config.guard"));
        assert!(!html.contains("__NEXA_PROXY__"), "en la web no hay tabla: el proxy es del mismo origen");
    }

    #[test]
    fn desktop_shell_declares_the_backend_table_and_prefers_desktop_proxy() {
        let build = AppBuild { routes: vec![], has_config: false, modules: BTreeMap::new() };
        let mut m = Manifest::default();
        m.proxy.insert("/api".into(), "http://127.0.0.1:9100".into());
        let html = shell_html(&m, &build, "/app", "/assets/rt.js", true);
        assert!(html.contains("window.__NEXA_PROXY__={\"/api\":\"http://127.0.0.1:9100\"}"), "{html}");

        m.desktop = Some(crate::manifest::DesktopSection { proxy: [("/api".to_string(), "https://api.miapp.com".to_string())].into() });
        let html = shell_html(&m, &build, "/app", "/assets/rt.js", true);
        assert!(html.contains("https://api.miapp.com") && !html.contains("9100"), "{html}");
        // y la web sigue sin tabla
        assert!(!shell_html(&m, &build, "/app", "/assets/rt.js", false).contains("__NEXA_PROXY__"));
    }
}
