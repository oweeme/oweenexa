//! Fase 61 — proxy inverso de desarrollo (`[proxy]` en `nexa.toml`).
//!
//! `nexa dev`/`nexa preview` reenvían al backend real las peticiones cuyo
//! camino empieza con un prefijo declarado, y devuelven su respuesta
//! tal cual (estado, cabeceras, cuerpo y TODAS las `Set-Cookie`). Para el
//! navegador la app y la API son el mismo origen: no hay CORS, y una
//! cookie de sesión `HttpOnly` puesta por el backend viaja sola.
//!
//! Es una herramienta de desarrollo/previsualización, no un servidor de
//! producción: ahí lo hace nginx (`nexa add nginx` genera un `location`
//! por entrada de `[proxy]`).

use std::collections::BTreeMap;
use std::io::Read;

use tiny_http::{Header, Request, Response};

/// Cabeceras que no se reenvían: son de un salto (hop-by-hop) o las
/// recalcula el propio cliente/servidor HTTP.
const SKIP_REQUEST: [&str; 8] =
    ["host", "connection", "content-length", "transfer-encoding", "accept-encoding", "upgrade", "keep-alive", "te"];
const SKIP_RESPONSE: [&str; 5] = ["connection", "content-length", "transfer-encoding", "keep-alive", "upgrade"];

/// El prefijo más largo de `proxy` que cubre `path` (`/api` cubre `/api`
/// y `/api/x`, pero no `/apix`), junto con el destino.
pub fn match_target<'a>(proxy: &'a BTreeMap<String, String>, path: &str) -> Option<(&'a str, &'a str)> {
    proxy
        .iter()
        .filter(|(prefix, _)| {
            let p = prefix.trim_end_matches('/');
            path == p || path.strip_prefix(p).is_some_and(|rest| rest.starts_with('/') || rest.starts_with('?'))
        })
        .max_by_key(|(prefix, _)| prefix.len())
        .map(|(prefix, target)| (prefix.as_str(), target.as_str()))
}

/// `http://127.0.0.1:9100` + `/api/users?x=1` -> la URL real de destino.
/// El camino se conserva completo (con su prefijo): el backend ve
/// exactamente `/api/users`, igual que detrás de un `proxy_pass` de nginx
/// sin barra final.
pub fn target_url(target: &str, url: &str) -> String {
    format!("{}{}", target.trim_end_matches('/'), url)
}

/// Reenvía `request` al backend y responde. Consume la petición.
pub fn forward(mut request: Request, target: &str) {
    let url = target_url(target, request.url());
    let method = request.method().as_str().to_string();

    let mut body = Vec::new();
    let _ = request.as_reader().read_to_end(&mut body);

    let client_ip = request.remote_addr().map(|a| a.ip().to_string()).unwrap_or_default();
    let mut builder = ureq::http::Request::builder().method(method.as_str()).uri(&url);
    for header in request.headers() {
        let name = header.field.as_str().as_str().to_ascii_lowercase();
        if SKIP_REQUEST.contains(&name.as_str()) {
            continue;
        }
        builder = builder.header(header.field.as_str().as_str(), header.value.as_str());
    }
    if !client_ip.is_empty() {
        builder = builder.header("X-Forwarded-For", client_ip);
    }

    let outcome = builder
        .body(body)
        .map_err(|e| e.to_string())
        .and_then(|req| agent().run(req).map_err(|e| e.to_string()));

    let response = match outcome {
        Ok(resp) => to_tiny_http(resp),
        Err(message) => bad_gateway(&url, &message),
    };
    let _ = request.respond(response);
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        // Un 404/500 del backend es una respuesta válida que hay que
        // devolver tal cual, no un error del proxy.
        .http_status_as_error(false)
        // Las redirecciones (login -> /dashboard) las sigue el navegador.
        .max_redirects(0)
        .build()
        .into()
}

fn to_tiny_http(resp: ureq::http::Response<ureq::Body>) -> Response<std::io::Cursor<Vec<u8>>> {
    let status = resp.status().as_u16();
    let headers: Vec<(String, Vec<u8>)> = resp
        .headers()
        .iter()
        .filter(|(name, _)| !SKIP_RESPONSE.contains(&name.as_str().to_ascii_lowercase().as_str()))
        .map(|(name, value)| (name.as_str().to_string(), value.as_bytes().to_vec()))
        .collect();
    let mut bytes = Vec::new();
    let _ = resp.into_body().into_reader().read_to_end(&mut bytes);

    let mut out = Response::from_data(bytes).with_status_code(status);
    for (name, value) in headers {
        if let Ok(header) = Header::from_bytes(name.as_bytes(), value.as_slice()) {
            out.add_header(header);
        }
    }
    out
}

fn bad_gateway(url: &str, message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = format!(
        "{{\"error\":\"bad_gateway\",\"message\":\"nexa: no se pudo conectar con el backend ({url}): {}\"}}",
        message.replace('"', "'")
    );
    Response::from_string(body)
        .with_status_code(502)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).expect("header válido"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    #[test]
    fn matches_whole_segments_only() {
        let p = map(&[("/api", "http://b")]);
        assert!(match_target(&p, "/api").is_some());
        assert!(match_target(&p, "/api/users").is_some());
        assert!(match_target(&p, "/api?x=1").is_some());
        assert!(match_target(&p, "/apix").is_none());
        assert!(match_target(&p, "/").is_none());
    }

    #[test]
    fn longest_prefix_wins() {
        let p = map(&[("/api", "http://a"), ("/api/v2", "http://b")]);
        assert_eq!(match_target(&p, "/api/v2/x"), Some(("/api/v2", "http://b")));
        assert_eq!(match_target(&p, "/api/v1/x"), Some(("/api", "http://a")));
    }

    #[test]
    fn target_url_keeps_the_full_path_and_query() {
        assert_eq!(target_url("http://127.0.0.1:9100/", "/api/u?x=1"), "http://127.0.0.1:9100/api/u?x=1");
    }
}
