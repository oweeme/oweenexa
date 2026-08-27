//! Piezas de servidor HTTP compartidas por `nexa preview` y `nexa dev`
//! (Fase 11): construir una `Response`, los `Content-Type` de siempre, y
//! `sitemap.xml`/`robots.txt` (que usan lo que `nexa build` ya haya
//! escrito en `dist/`, o los generan al vuelo si no).

use std::fs;
use std::path::Path;

use nexa_router::scan_pages;
use tiny_http::{Header, Response};

pub fn respond(status: u16, content_type: Header, body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_status_code(status).with_header(content_type)
}

/// Como `respond`, pero para contenido binario (imágenes, fuentes...) —
/// `Response::from_string` asumiría UTF-8 y corrompería esos bytes.
pub fn respond_bytes(status: u16, content_type: Header, body: Vec<u8>) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(body).with_status_code(status).with_header(content_type)
}

/// Adivina el `Content-Type` de un archivo estático de `public/`/`dist/`
/// por su extensión — no exhaustivo, cubre lo común (imágenes, fuentes,
/// favicons, JS/CSS). Lo que no reconoce cae a `application/octet-stream`:
/// un tipo genérico es mejor que uno inventado incorrectamente.
///
/// `js`/`mjs`/`css` faltaban hasta que un `<script type="module"
/// src="/vendor/....js">` real (un import map de la Fase 15 apuntando a
/// un archivo bajo `public/`) reveló el hueco: el navegador rechaza
/// ejecutar un script de módulo cuyo `Content-Type` no sea JS de verdad
/// ("Failed to load module script: ... MIME type of
/// 'application/octet-stream'") — encontrado corriendo un navegador real
/// (Chromium vía Playwright) contra un `nexa preview` real.
pub fn guess_static_content_type(path: &str) -> Header {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let mime: &str = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "pdf" => "application/pdf",
        "txt" => "text/plain; charset=utf-8",
        "json" => "application/json",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        _ => "application/octet-stream",
    };
    Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).expect("el header Content-Type siempre es válido")
}

pub fn request_path(url: &str) -> String {
    url.split('?').next().unwrap_or("/").to_string()
}

/// El valor de `key` en la query string de `url` (sin decodificar
/// porcentaje — suficiente para los usos actuales, todos rutas simples
/// como `/products/iphone-17` sin espacios ni caracteres especiales).
pub fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key).then(|| percent_decode(v))
    })
}

/// Decodifica `%XX` (y `+` como espacio, la convención de
/// `application/x-www-form-urlencoded` que usa `encodeURIComponent` en
/// el cliente) — encontrado como bug real: `@nexa/devtools` pide
/// `?path=%2F` para la ruta `/`, y sin decodificar esto nunca hacía
/// match contra ninguna ruta real (404 silencioso).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok());
                match hex {
                    Some(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    None => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// `nexa build` ya escribe `dist/sitemap.xml`; si no corrió, se genera
/// aquí mismo a partir de las rutas estáticas encontradas.
pub fn sitemap_or_generate(pages_dir: &Path) -> String {
    if let Ok(existing) = fs::read_to_string("dist/sitemap.xml") {
        return existing;
    }

    let site_url = std::env::var("NEXA_SITE_URL").ok();
    let routes = scan_pages(pages_dir).unwrap_or_default();
    let urls: Vec<String> = routes
        .iter()
        .filter(|r| !r.is_dynamic())
        .map(|r| match &site_url {
            Some(base) => format!("{}{}", base.trim_end_matches('/'), r.pattern),
            None => r.pattern.clone(),
        })
        .collect();

    nexa_seo::render_sitemap(&urls)
}

pub fn robots_or_generate() -> String {
    if let Ok(existing) = fs::read_to_string("dist/robots.txt") {
        return existing;
    }

    let sitemap_url =
        std::env::var("NEXA_SITE_URL").ok().map(|base| format!("{}/sitemap.xml", base.trim_end_matches('/')));
    nexa_seo::render_robots(sitemap_url.as_deref())
}

pub fn html_content_type() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
        .expect("el header Content-Type siempre es válido")
}

pub fn javascript_content_type() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"text/javascript; charset=utf-8"[..])
        .expect("el header Content-Type siempre es válido")
}

pub fn css_content_type() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..])
        .expect("el header Content-Type siempre es válido")
}

pub fn xml_content_type() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"application/xml; charset=utf-8"[..])
        .expect("el header Content-Type siempre es válido")
}

pub fn text_content_type() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..])
        .expect("el header Content-Type siempre es válido")
}

pub fn json_content_type() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..])
        .expect("el header Content-Type siempre es válido")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_param_reads_a_value_from_the_query_string() {
        assert_eq!(query_param("/__nexa_dev__/diagnostics?path=/products/x", "path"), Some("/products/x".to_string()));
    }

    #[test]
    fn query_param_is_none_when_the_key_is_missing() {
        assert_eq!(query_param("/foo?other=1", "path"), None);
    }

    #[test]
    fn query_param_is_none_without_a_query_string_at_all() {
        assert_eq!(query_param("/foo", "path"), None);
    }

    #[test]
    fn query_param_finds_a_key_that_is_not_first() {
        assert_eq!(query_param("/foo?a=1&path=/bar&b=2", "path"), Some("/bar".to_string()));
    }

    #[test]
    fn query_param_percent_decodes_the_value() {
        // El bug real que esto fija: `@nexa/devtools` pide
        // `?path=%2F` (la ruta `/` codificada con `encodeURIComponent`)
        // — sin decodificar, esto nunca hacía match contra la ruta `/`
        // (404 silencioso, encontrado corriendo el bundle real de
        // devtools contra un `nexa dev` real).
        assert_eq!(query_param("/x?path=%2F", "path"), Some("/".to_string()));
        assert_eq!(query_param("/x?path=%2Fproducts%2Fiphone-17", "path"), Some("/products/iphone-17".to_string()));
    }

    #[test]
    fn query_param_decodes_a_plus_as_a_space() {
        assert_eq!(query_param("/x?q=hola+mundo", "q"), Some("hola mundo".to_string()));
    }

    #[test]
    fn query_param_leaves_a_malformed_percent_escape_untouched() {
        assert_eq!(query_param("/x?q=100%", "q"), Some("100%".to_string()));
        assert_eq!(query_param("/x?q=100%zz", "q"), Some("100%zz".to_string()));
    }

    #[test]
    fn guesses_javascript_and_css_content_types() {
        // Bug real (Fase 15): un `<script type="module" src="/vendor/x.js">`
        // apuntando a un archivo bajo `public/` (un import map de
        // terceros) se servía como `application/octet-stream` — el
        // navegador rechaza ejecutar un script de módulo así. Encontrado
        // con un navegador real (Chromium vía Playwright) contra un
        // `nexa preview` real.
        assert_eq!(guess_static_content_type("/vendor/nexa-stripe.js").value.as_str(), "text/javascript; charset=utf-8");
        assert_eq!(guess_static_content_type("/x.mjs").value.as_str(), "text/javascript; charset=utf-8");
        assert_eq!(guess_static_content_type("/x.css").value.as_str(), "text/css; charset=utf-8");
    }

    #[test]
    fn falls_back_to_octet_stream_for_an_unknown_extension() {
        assert_eq!(guess_static_content_type("/x.bin").value.as_str(), "application/octet-stream");
    }
}
