use std::path::Path;

use anyhow::{Context, Result};
use tiny_http::{Response, Server};

use crate::assets::framework_asset;
use crate::page_resolver::{read_prebuilt_asset, read_static_file, resolve_page, ResolveError};
use crate::serve::{
    css_content_type, guess_static_content_type, html_content_type, javascript_content_type, request_path,
    respond, respond_bytes, robots_or_generate, sitemap_or_generate, text_content_type, xml_content_type,
};

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

    println!("Nexa preview escuchando en http://{address}");
    println!(
        "Primero sirve lo que ya esté en dist/; lo que falte (típicamente rutas dinámicas) se \
         renderiza al vuelo, sin necesidad de un `nexa build` previo para esa ruta."
    );
    if api_base.is_empty() {
        println!(
            "Aviso: NEXA_API_URL no está definida — las páginas con `load()` fallarán al \
             intentar pedir datos."
        );
    }
    println!("Ctrl+C para detener.");

    for request in server.incoming_requests() {
        let path = request_path(request.url());
        let response = handle(pages_dir, &path, &api_base);

        let _ = request.respond(response);
    }

    Ok(())
}

fn handle(pages_dir: &Path, path: &str, api_base: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    if let Some(js) = framework_asset(path) {
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

    match resolve_page(pages_dir, path, api_base, true) {
        Ok(html) => respond(200, html_content_type(), html),
        Err(ResolveError::NotFound) => match read_static_file(Path::new("dist"), path) {
            Some(bytes) => respond_bytes(200, guess_static_content_type(path), bytes),
            None => respond(404, html_content_type(), not_found_page(path)),
        },
        Err(ResolveError::Failed(message)) => {
            respond(500, html_content_type(), server_error_page(&message))
        }
    }
}

fn not_found_page(path: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"es\"><head><meta charset=\"UTF-8\"><title>404</title></head>\
         <body><h1>404</h1><p>No existe una página para <code>{path}</code>.</p></body></html>\n"
    )
}

fn server_error_page(message: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"es\"><head><meta charset=\"UTF-8\"><title>500</title></head>\
         <body><h1>500</h1><p>{message}</p></body></html>\n"
    )
}
