//! `nexa dev`: cada petición recompila la página desde `src/` (nunca lee
//! un `dist/.../index.html` viejo, a diferencia de `nexa preview`), y
//! toda respuesta HTML lleva un `<script>` en `<head>` que sondea
//! `/__nexa_dev__/version` — cuando detecta que algo bajo `src/` cambió,
//! vuelve a pedir la página actual y reemplaza el `<body>` sin que el
//! usuario tenga que recargar a mano (ver `packages/dev-client`).
//!
//! El script vive en `<head>`, no en `<body>`: así sobrevive al propio
//! reemplazo del `<body>` que dispara, y el sondeo nunca se interrumpe.
//!
//! Los errores de compilación (ej. un TSX con un error de sintaxis
//! mientras se edita) se muestran como una página de error legible en el
//! propio navegador — y esa página *también* lleva el script de
//! auto-reload, así que en cuanto se corrige el archivo, la próxima
//! detección de cambio la reemplaza sola por la página real.

use std::path::Path;

use anyhow::{Context, Result};
use tiny_http::{Response, Server};

use crate::assets::{
    framework_asset, NEXA_DEVTOOLS_FILENAME, NEXA_DEVTOOLS_JS, NEXA_DEV_CLIENT_FILENAME, NEXA_DEV_CLIENT_JS,
};
use crate::bootstrap::insert_before_closing_tag;
use crate::dev_watch::version_marker;
use crate::devtools::diagnostics_json;
use crate::page_resolver::{compile_matched_page, read_prebuilt_asset, read_static_file, resolve_page, ResolveError};
use crate::serve::{
    css_content_type, guess_static_content_type, html_content_type, javascript_content_type, json_content_type,
    query_param, request_path, respond, respond_bytes, robots_or_generate, sitemap_or_generate, text_content_type,
    xml_content_type,
};

const VERSION_ENDPOINT: &str = "/__nexa_dev__/version";
const DIAGNOSTICS_ENDPOINT: &str = "/__nexa_dev__/diagnostics";

pub fn run(port: u16) -> Result<()> {
    let pages_dir = Path::new("src/pages");
    if !pages_dir.is_dir() {
        anyhow::bail!(
            "no se encontró {} — ¿estás dentro de un proyecto Nexa? (usa `nexa create <nombre>`)",
            pages_dir.display()
        );
    }

    let api_base = std::env::var("NEXA_API_URL").unwrap_or_default();

    let address = format!("127.0.0.1:{port}");
    let server = Server::http(&address)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("arrancando el servidor en {address}"))?;

    println!("Nexa dev escuchando en http://{address}");
    println!(
        "Cada página se recompila desde src/ en cada petición — nunca sirve un dist/ viejo. \
         Editar un archivo y recargar (o esperar el auto-reload) siempre refleja el cambio."
    );
    if api_base.is_empty() {
        println!(
            "Aviso: NEXA_API_URL no está definida — las páginas con `load()` fallarán al \
             intentar pedir datos."
        );
    }
    println!("Ctrl+C para detener.");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let response = handle(pages_dir, &url, &api_base);

        let _ = request.respond(response);
    }

    Ok(())
}

fn handle(pages_dir: &Path, url: &str, api_base: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let path = request_path(url);
    let path = path.as_str();

    if path == VERSION_ENDPOINT {
        return respond(200, text_content_type(), version_marker(Path::new("src")));
    }

    if path == DIAGNOSTICS_ENDPOINT {
        let target = query_param(url, "path").unwrap_or_else(|| "/".to_string());
        return match compile_matched_page(pages_dir, &target, api_base) {
            Ok(page) => respond(200, json_content_type(), diagnostics_json(&target, &page)),
            Err(_) => respond(404, json_content_type(), "{}".to_string()),
        };
    }

    if let Some(js) = serve_framework_asset(path) {
        return respond(200, javascript_content_type(), js.to_string());
    }

    if let Some(content) = read_prebuilt_asset(path) {
        let content_type = if path.ends_with(".css") { css_content_type() } else { javascript_content_type() };
        return respond(200, content_type, content);
    }

    if path == "/sitemap.xml" {
        return respond(200, xml_content_type(), sitemap_or_generate(pages_dir));
    }
    if path == "/robots.txt" {
        return respond(200, text_content_type(), robots_or_generate());
    }

    // `use_prebuilt_html: false` — a diferencia de `nexa preview`, `nexa
    // dev` nunca debe servir un `dist/.../index.html` que haya quedado de
    // un `nexa build` anterior: la razón de ser de este comando es que lo
    // que se ve siempre viene del `src/` actual.
    match resolve_page(pages_dir, path, api_base, false) {
        Ok(html) => respond(200, html_content_type(), with_dev_client(&html)),
        Err(ResolveError::NotFound) => match read_static_file(Path::new("public"), path) {
            Some(bytes) => respond_bytes(200, guess_static_content_type(path), bytes),
            None => respond(404, html_content_type(), with_dev_client(&not_found_page(path))),
        },
        Err(ResolveError::Failed(message)) => {
            respond(500, html_content_type(), with_dev_client(&compile_error_page(&message)))
        }
    }
}

/// `@nexa/runtime`, `@nexa/router`, `@nexa/forms` y `@nexa/platform`
/// están embebidos en el binario (igual que en `nexa preview`);
/// `@nexa/dev-client` y `@nexa/devtools` solo los sirve este comando.
fn serve_framework_asset(path: &str) -> Option<&'static str> {
    let trimmed = path.trim_start_matches('/');
    if trimmed == format!("assets/{NEXA_DEV_CLIENT_FILENAME}") {
        return Some(NEXA_DEV_CLIENT_JS);
    }
    if trimmed == format!("assets/{NEXA_DEVTOOLS_FILENAME}") {
        return Some(NEXA_DEVTOOLS_JS);
    }
    framework_asset(path)
}

/// `@nexa/devtools` (Fase 14) siempre va junto a `@nexa/dev-client`: no
/// hace falta declararlo, `nexa dev` en sí mismo ya es una herramienta
/// de desarrollo — mostrar sus propios diagnósticos es justo su trabajo.
fn with_dev_client(html: &str) -> String {
    let script = format!(
        "<script type=\"module\" src=\"/assets/{NEXA_DEV_CLIENT_FILENAME}\"></script>\n\
         <script type=\"module\">\n\
         import {{ initDevtools }} from \"/assets/{NEXA_DEVTOOLS_FILENAME}\";\n\
         initDevtools();\n\
         </script>\n"
    );
    insert_before_closing_tag(html, "head", &script)
}

fn not_found_page(path: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"es\"><head><meta charset=\"UTF-8\"><title>404 — Nexa dev</title></head>\
         <body><h1>404</h1><p>No existe una página para <code>{path}</code>.</p></body></html>\n"
    )
}

/// A diferencia del 500 plano de `nexa preview`, esta página está
/// pensada para mirarse mientras se edita: monoespaciada y legible, con
/// el mensaje de error completo.
fn compile_error_page(message: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"es\"><head><meta charset=\"UTF-8\">\
         <title>Error de compilación — Nexa dev</title>\
         <style>\
         body{{background:#1e1e1e;color:#f2f2f2;font-family:ui-monospace,Menlo,Consolas,monospace;\
         padding:2rem;line-height:1.5}}\
         h1{{color:#ff6b6b;font-size:1.1rem;margin:0 0 1rem}}\
         pre{{white-space:pre-wrap;background:#2a2a2a;padding:1rem;border-radius:6px;overflow-x:auto}}\
         p{{color:#999}}\
         </style></head>\
         <body data-nexa-dev-error=\"1\">\
         <h1>Nexa no pudo compilar esta página</h1>\
         <pre>{}</pre>\
         <p>Corrige el archivo y guarda — esta página se reemplaza sola cuando vuelva a compilar.</p>\
         </body></html>\n",
        html_escape(message)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_dev_client_inserts_the_script_in_head_not_body() {
        let html = "<html><head><title>t</title></head><body><p>hola</p></body></html>";
        let out = with_dev_client(html);

        assert!(out.contains(&format!("/assets/{NEXA_DEV_CLIENT_FILENAME}")));
        let script_pos = out.find("nexa-dev-client").unwrap();
        let head_close_pos = out.find("</head>").unwrap();
        let body_open_pos = out.find("<body>").unwrap();
        assert!(script_pos < head_close_pos);
        assert!(head_close_pos <= body_open_pos);
    }

    #[test]
    fn with_dev_client_also_loads_devtools_and_calls_init() {
        let html = "<html><head></head><body></body></html>";
        let out = with_dev_client(html);

        assert!(out.contains(&format!("/assets/{NEXA_DEVTOOLS_FILENAME}")));
        assert!(out.contains("initDevtools();"));
    }

    #[test]
    fn serve_framework_asset_recognizes_the_devtools_bundle() {
        assert!(serve_framework_asset(&format!("/assets/{NEXA_DEVTOOLS_FILENAME}")).is_some());
    }

    #[test]
    fn compile_error_page_escapes_the_message_and_is_marked_as_an_error() {
        let page = compile_error_page("<script>alert(1)</script> en home.tsx:3");
        assert!(!page.contains("<script>alert"));
        assert!(page.contains("&lt;script&gt;"));
        assert!(page.contains("data-nexa-dev-error"));
    }

    #[test]
    fn serve_framework_asset_recognizes_the_dev_client_bundle() {
        let js = serve_framework_asset(&format!("/assets/{NEXA_DEV_CLIENT_FILENAME}"));
        assert!(js.is_some());
    }
}
