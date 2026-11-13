//! Fase 64 — el servidor de producción que vive dentro de un ejecutable
//! creado con `nexa bundle`.
//!
//! A diferencia de `nexa preview` (un solo hilo, pensado para mirar el
//! sitio mientras se desarrolla), este:
//!
//! - sirve todo **desde memoria** (el sitio viaja dentro del ejecutable);
//! - usa varios hilos;
//! - responde con la versión **gzip** precomprimida si el cliente la acepta;
//! - pone `Cache-Control: immutable` de un año a los archivos con hash de
//!   contenido en el nombre (Fase 23) y `no-cache` + `ETag` al HTML, que
//!   responde `304` si no cambió;
//! - cae al shell de la SPA (Fase 60) en una ruta profunda de `/app/...`
//!   que no es un archivo, y a un `404` real (con `404.html` si existe)
//!   para todo lo demás;
//! - reenvía `[proxy]` al backend (Fase 61), con los mismos cuerpos,
//!   cabeceras y cookies;
//! - añade las mismas cabeceras de seguridad que `nexa preview` y que el
//!   `nginx.conf` generado.
//!
//! `route` es una función pura (petición -> respuesta) para poder probar
//! toda la lógica sin abrir un socket.

use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use tiny_http::{Header, Response, Server};

use crate::bundle::{EntryMeta, Payload};

pub struct Req<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub accept_encoding: &'a str,
    pub if_none_match: &'a str,
}

#[derive(Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

struct Entry {
    meta: EntryMeta,
    content_type: &'static str,
    etag: String,
    cache_control: &'static str,
}

pub struct Site {
    payload: Payload,
    entries: HashMap<String, Entry>,
    app_base: Option<String>,
}

impl Site {
    pub fn new(payload: Payload) -> Site {
        let entries = payload
            .entries
            .iter()
            .map(|meta| {
                let raw = payload.bytes(meta.offset, meta.len);
                let entry = Entry {
                    meta: meta.clone(),
                    content_type: crate::serve::mime_for_path(&meta.path),
                    etag: format!("\"{}\"", crate::assets::content_short_hash(raw)),
                    cache_control: cache_control_for(&meta.path),
                };
                (meta.path.clone(), entry)
            })
            .collect();
        let app_base = payload.config.app_base.clone();
        Site { payload, entries, app_base }
    }

    pub fn file_count(&self) -> usize {
        self.entries.len()
    }
}

/// Los archivos con `.<8 hex>.js|css` en el nombre (los que `nexa build`
/// firma con un hash de su contenido) nunca cambian bajo la misma URL.
fn cache_control_for(path: &str) -> &'static str {
    let file = path.rsplit('/').next().unwrap_or(path);
    let parts: Vec<&str> = file.split('.').collect();
    let hashed = parts.len() >= 3
        && matches!(parts[parts.len() - 1], "js" | "css")
        && parts[parts.len() - 2].len() == 8
        && parts[parts.len() - 2].bytes().all(|b| b.is_ascii_hexdigit());
    if hashed {
        "public, max-age=31536000, immutable"
    } else if file.ends_with(".html") || file == "sw.js" || file == "manifest.webmanifest" {
        // Siempre revalidar (con ETag): un redeploy se ve al instante.
        "no-cache"
    } else {
        "public, max-age=3600"
    }
}

/// `%5B` -> `[`. A diferencia de `serve::percent_decode`, `+` queda como `+`
/// (en una ruta no es un espacio).
fn decode_path(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn security_headers() -> [(&'static str, String); 3] {
    [
        ("X-Content-Type-Options", "nosniff".into()),
        ("X-Frame-Options", "SAMEORIGIN".into()),
        ("Referrer-Policy", "strict-origin-when-cross-origin".into()),
    ]
}

fn text_reply(status: u16, body: &str) -> Reply {
    let mut headers = vec![("Content-Type", "text/plain; charset=utf-8".to_string())];
    headers.extend(security_headers());
    Reply { status, headers, body: body.as_bytes().to_vec() }
}

/// Busca el archivo de una ruta: exacto, `ruta/index.html`, y para la SPA
/// una ruta profunda sin extensión cae a su shell.
fn find<'a>(site: &'a Site, path: &str) -> Option<(&'a Entry, u16)> {
    let rel = path.trim_start_matches('/');
    let trimmed = rel.trim_end_matches('/');

    if !rel.is_empty() && !rel.ends_with('/') {
        if let Some(e) = site.entries.get(rel) {
            return Some((e, 200));
        }
    }
    let index = if trimmed.is_empty() { "index.html".to_string() } else { format!("{trimmed}/index.html") };
    if let Some(e) = site.entries.get(&index) {
        return Some((e, 200));
    }

    if let Some(base) = &site.app_base {
        let last = trimmed.rsplit('/').next().unwrap_or("");
        let inside = path == base || path.strip_prefix(base.as_str()).is_some_and(|r| r.starts_with('/'));
        if inside && !last.contains('.') {
            let shell = format!("{}/200.html", base.trim_start_matches('/'));
            let shell = if site.entries.contains_key(&shell) { shell } else { format!("{}/index.html", base.trim_start_matches('/')) };
            if let Some(e) = site.entries.get(&shell) {
                return Some((e, 200));
            }
        }
    }
    None
}

pub fn route(site: &Site, req: &Req) -> Reply {
    if req.method != "GET" && req.method != "HEAD" {
        let mut r = text_reply(405, "405 — método no permitido");
        r.headers.push(("Allow", "GET, HEAD".to_string()));
        return r;
    }

    let raw_path = req.url.split(['?', '#']).next().unwrap_or("/");
    let Some(path) = decode_path(raw_path) else { return text_reply(400, "400 — ruta inválida") };
    if path.contains('\0') || path.split('/').any(|seg| seg == "..") {
        return text_reply(400, "400 — ruta inválida");
    }

    let Some((entry, status)) = find(site, &path) else {
        return not_found(site);
    };
    entry_reply(site, entry, status, req)
}

fn not_found(site: &Site) -> Reply {
    match site.entries.get("404.html") {
        Some(e) => {
            let mut r = entry_reply(site, e, 404, &Req { method: "GET", url: "/", accept_encoding: "", if_none_match: "" });
            r.status = 404;
            r
        }
        None => text_reply(404, "404 — no encontrado"),
    }
}

fn entry_reply(site: &Site, entry: &Entry, status: u16, req: &Req) -> Reply {
    let wants_gzip = entry.meta.gz_len > 0
        && req.accept_encoding.split(',').any(|e| e.trim().split(';').next().is_some_and(|n| n.eq_ignore_ascii_case("gzip")));

    let etag = if wants_gzip { entry.etag.replacen('"', "\"gz-", 1) } else { entry.etag.clone() };
    let mut headers: Vec<(&'static str, String)> = vec![
        ("Content-Type", entry.content_type.to_string()),
        ("Cache-Control", entry.cache_control.to_string()),
        ("ETag", etag.clone()),
    ];
    if entry.meta.gz_len > 0 {
        headers.push(("Vary", "Accept-Encoding".to_string()));
    }
    headers.extend(security_headers());

    // `If-None-Match` puede traer varios ETag, débiles (`W/`) o `*`.
    let matches = req
        .if_none_match
        .split(',')
        .map(|t| t.trim().trim_start_matches("W/"))
        .any(|t| t == "*" || t == etag);
    if status == 200 && matches {
        return Reply { status: 304, headers, body: Vec::new() };
    }

    let body = if wants_gzip {
        headers.push(("Content-Encoding", "gzip".to_string()));
        site.payload.bytes(entry.meta.gz_offset, entry.meta.gz_len).to_vec()
    } else {
        site.payload.bytes(entry.meta.offset, entry.meta.len).to_vec()
    };
    Reply { status, headers, body }
}

// ---------------------------------------------------------------------
// El servidor: sockets, hilos, argumentos
// ---------------------------------------------------------------------

struct Options {
    host: String,
    port: u16,
    quiet: bool,
    proxy: BTreeMap<String, String>,
}

const HELP: &str = "\
Servidor de producción de un sitio Nexa (creado con `nexa bundle`).

USO:
    <este-ejecutable> [OPCIONES]

OPCIONES:
    --port <N>              Puerto (por defecto: $PORT, o el del proyecto)
    --host <IP>             Interfaz (por defecto 127.0.0.1; usa 0.0.0.0 para
                            aceptar conexiones de otras máquinas)
    --proxy <PREFIJO=URL>   Reenvía las rutas con ese prefijo a un backend
                            (repetible; reemplaza al [proxy] de nexa.toml)
    --quiet                 No imprime una línea por petición
    -h, --help              Esta ayuda
";

fn parse_options(args: &[String], defaults: &crate::bundle::Config) -> Result<Option<Options>> {
    let mut opts = Options {
        host: "127.0.0.1".to_string(),
        port: std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(defaults.default_port),
        quiet: false,
        proxy: defaults.proxy.clone(),
    };
    let mut overridden = false;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().cloned().with_context(|| format!("{name} necesita un valor"));
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                return Ok(None);
            }
            "--port" => opts.port = value("--port")?.parse().context("--port debe ser un número (1-65535)")?,
            "--host" => opts.host = value("--host")?,
            "--quiet" => opts.quiet = true,
            "--proxy" => {
                let v = value("--proxy")?;
                let (prefix, url) = v.split_once('=').context("--proxy espera PREFIJO=URL, ej. /api=https://api.miapp.com")?;
                if !overridden {
                    opts.proxy.clear();
                    overridden = true;
                }
                opts.proxy.insert(prefix.to_string(), url.to_string());
            }
            other => anyhow::bail!("opción desconocida `{other}` (usa --help)"),
        }
    }
    Ok(Some(opts))
}

pub fn run_embedded(payload: Payload, args: Vec<String>) -> Result<()> {
    let Some(opts) = parse_options(&args, &payload.config)? else { return Ok(()) };
    let name = payload.config.name.clone();
    let site = Arc::new(Site::new(payload));

    let address = format!("{}:{}", opts.host, opts.port);
    let server = Arc::new(
        Server::http(&address).map_err(|e| anyhow::anyhow!("{e}")).with_context(|| format!("no se pudo abrir {address}"))?,
    );

    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(2, 16);
    println!("{name}: sirviendo {} archivo(s) desde memoria en http://{address}  ({workers} hilos)", site.file_count());
    for (prefix, target) in &opts.proxy {
        println!("  proxy {prefix} -> {target}");
    }
    if opts.host != "127.0.0.1" && opts.host != "localhost" {
        println!("  atención: escuchando en {} — accesible desde otras máquinas", opts.host);
    }
    println!("Ctrl+C para detener.");

    let proxy = Arc::new(opts.proxy);
    let quiet = opts.quiet;
    let handles: Vec<_> = (0..workers)
        .map(|_| {
            let (server, site, proxy) = (server.clone(), site.clone(), proxy.clone());
            std::thread::spawn(move || {
                while let Ok(request) = server.recv() {
                    handle(&site, &proxy, request, quiet);
                }
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    Ok(())
}

fn header_value(request: &tiny_http::Request, name: &str) -> String {
    request
        .headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
        .unwrap_or_default()
}

fn handle(site: &Site, proxy: &BTreeMap<String, String>, request: tiny_http::Request, quiet: bool) {
    let started = Instant::now();
    let method = request.method().as_str().to_string();
    let url = request.url().to_string();
    let path_only = url.split('?').next().unwrap_or("/").to_string();

    if let Some((_, target)) = crate::proxy::match_target(proxy, &path_only) {
        let target = target.to_string();
        crate::proxy::forward(request, &target);
        if !quiet {
            eprintln!("{method} {url} -> proxy {target} ({} ms)", started.elapsed().as_millis());
        }
        return;
    }

    let accept_encoding = header_value(&request, "Accept-Encoding");
    let if_none_match = header_value(&request, "If-None-Match");
    let reply = route(site, &Req { method: &method, url: &url, accept_encoding: &accept_encoding, if_none_match: &if_none_match });

    let status = reply.status;
    let len = reply.body.len();
    let mut response = Response::new(
        tiny_http::StatusCode(reply.status),
        Vec::new(),
        Cursor::new(reply.body),
        Some(len),
        None,
    );
    for (name, value) in reply.headers {
        if let Ok(h) = Header::from_bytes(name.as_bytes(), value.as_bytes()) {
            response.add_header(h);
        }
    }
    let _ = request.respond(response);
    if !quiet {
        eprintln!("{method} {url} {status} {len}B ({} ms)", started.elapsed().as_millis());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{pack, unpack, Config};

    fn site(app_base: Option<&str>) -> Site {
        let big_html = "<p>hola mundo</p>".repeat(100).into_bytes();
        let files: Vec<(String, Vec<u8>)> = vec![
            ("index.html".into(), big_html.clone()),
            ("about/index.html".into(), b"<h1>about</h1>".to_vec()),
            ("404.html".into(), b"<h1>perdido</h1>".to_vec()),
            ("assets/nexa-runtime.1a2b3c4d.js".into(), "let a=1;".repeat(100).into_bytes()),
            ("assets/logo.png".into(), vec![1, 2, 3, 4]),
            ("assets/app/pages/users/[id].js".into(), b"export default 1".to_vec()),
            ("app/index.html".into(), b"<div id=app></div>".to_vec()),
            ("app/200.html".into(), b"<div id=app>SHELL</div>".to_vec()),
            ("sw.js".into(), b"self.skipWaiting()".to_vec()),
        ];
        let config = Config { name: "t".into(), app_base: app_base.map(String::from), proxy: Default::default(), default_port: 8080 };
        Site::new(unpack(&pack(&files, config).unwrap()).unwrap())
    }

    fn get(site: &Site, url: &str) -> Reply {
        route(site, &Req { method: "GET", url, accept_encoding: "", if_none_match: "" })
    }
    fn header<'a>(r: &'a Reply, name: &str) -> Option<&'a str> {
        r.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    #[test]
    fn serves_the_root_a_directory_index_and_exact_files() {
        let s = site(None);
        assert_eq!(get(&s, "/").status, 200);
        assert_eq!(get(&s, "/about").body, b"<h1>about</h1>");
        assert_eq!(get(&s, "/about/").body, b"<h1>about</h1>");
        assert_eq!(get(&s, "/assets/logo.png").body, vec![1, 2, 3, 4]);
        assert_eq!(header(&get(&s, "/assets/logo.png"), "Content-Type"), Some("image/png"));
        assert_eq!(header(&get(&s, "/index.html?x=1#y"), "Content-Type"), Some("text/html; charset=utf-8"));
    }

    #[test]
    fn percent_encoded_paths_are_decoded() {
        let s = site(None);
        assert_eq!(get(&s, "/assets/app/pages/users/%5Bid%5D.js").body, b"export default 1");
        assert_eq!(get(&s, "/assets/app/pages/users/[id].js").status, 200);
    }

    #[test]
    fn a_missing_page_is_a_real_404_using_404_html_when_present() {
        let r = get(&site(None), "/nope");
        assert_eq!(r.status, 404);
        assert_eq!(r.body, b"<h1>perdido</h1>");
    }

    #[test]
    fn path_traversal_and_bad_encodings_are_rejected() {
        let s = site(None);
        for url in ["/../etc/passwd", "/a/../../x", "/%2e%2e/secret", "/%zz", "/a%00b"] {
            let status = get(&s, url).status;
            assert!(status == 400 || status == 404, "{url} -> {status}");
        }
        assert_eq!(get(&s, "/%2e%2e/secret").status, 400);
        assert_eq!(get(&s, "/../x").status, 400);
    }

    #[test]
    fn only_get_and_head_are_allowed() {
        let s = site(None);
        let r = route(&s, &Req { method: "POST", url: "/", accept_encoding: "", if_none_match: "" });
        assert_eq!(r.status, 405);
        assert_eq!(header(&r, "Allow"), Some("GET, HEAD"));
        assert_eq!(route(&s, &Req { method: "HEAD", url: "/", accept_encoding: "", if_none_match: "" }).status, 200);
    }

    #[test]
    fn spa_deep_links_fall_back_to_the_shell_but_missing_assets_stay_404() {
        let s = site(Some("/app"));
        let deep = get(&s, "/app/users/42");
        assert_eq!((deep.status, deep.body.as_slice()), (200, b"<div id=app>SHELL</div>".as_slice()));
        assert_eq!(get(&s, "/app").status, 200);
        assert_eq!(get(&s, "/app/missing.js").status, 404, "un archivo que falta no debe devolver HTML");
        assert_eq!(get(&s, "/apple/x").status, 404, "`/apple` no pertenece a `/app`");
        // sin Modo App no hay fallback
        assert_eq!(get(&site(None), "/app/users/42").status, 404);
    }

    #[test]
    fn hashed_assets_are_immutable_html_revalidates_the_rest_is_short() {
        let s = site(None);
        assert_eq!(header(&get(&s, "/assets/nexa-runtime.1a2b3c4d.js"), "Cache-Control"), Some("public, max-age=31536000, immutable"));
        assert_eq!(header(&get(&s, "/"), "Cache-Control"), Some("no-cache"));
        assert_eq!(header(&get(&s, "/sw.js"), "Cache-Control"), Some("no-cache"));
        assert_eq!(header(&get(&s, "/assets/logo.png"), "Cache-Control"), Some("public, max-age=3600"));
    }

    #[test]
    fn gzip_is_served_only_when_accepted_and_decodes_to_the_original() {
        use std::io::Read;
        let s = site(None);
        let plain = get(&s, "/");
        assert!(header(&plain, "Content-Encoding").is_none());

        let gz = route(&s, &Req { method: "GET", url: "/", accept_encoding: "br, gzip;q=0.8", if_none_match: "" });
        assert_eq!(header(&gz, "Content-Encoding"), Some("gzip"));
        assert_eq!(header(&gz, "Vary"), Some("Accept-Encoding"));
        assert!(gz.body.len() < plain.body.len());
        let mut decoded = Vec::new();
        flate2::read::GzDecoder::new(gz.body.as_slice()).read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, plain.body);

        // un PNG nunca se comprime, aunque el cliente acepte gzip
        let png = route(&s, &Req { method: "GET", url: "/assets/logo.png", accept_encoding: "gzip", if_none_match: "" });
        assert!(header(&png, "Content-Encoding").is_none());
    }

    #[test]
    fn etag_gives_a_304_with_no_body_and_a_different_etag_per_encoding() {
        let s = site(None);
        let first = get(&s, "/");
        let etag = header(&first, "ETag").unwrap().to_string();
        let again = route(&s, &Req { method: "GET", url: "/", accept_encoding: "", if_none_match: &etag });
        assert_eq!(again.status, 304);
        assert!(again.body.is_empty());

        // débil y en una lista
        let weak = format!("\"otro\", W/{etag}");
        assert_eq!(route(&s, &Req { method: "GET", url: "/", accept_encoding: "", if_none_match: &weak }).status, 304);

        // el ETag de la versión gzip es otro: el del cuerpo sin comprimir no sirve para ella
        let gz = route(&s, &Req { method: "GET", url: "/", accept_encoding: "gzip", if_none_match: &etag });
        assert_eq!(gz.status, 200);
        let gz_etag = header(&gz, "ETag").unwrap().to_string();
        assert_ne!(gz_etag, etag);
        assert_eq!(route(&s, &Req { method: "GET", url: "/", accept_encoding: "gzip", if_none_match: &gz_etag }).status, 304);
    }

    #[test]
    fn every_reply_carries_the_security_headers() {
        let s = site(None);
        for url in ["/", "/nope", "/assets/logo.png"] {
            let r = get(&s, url);
            assert_eq!(header(&r, "X-Content-Type-Options"), Some("nosniff"), "{url}");
            assert_eq!(header(&r, "X-Frame-Options"), Some("SAMEORIGIN"), "{url}");
            assert!(header(&r, "Referrer-Policy").is_some(), "{url}");
        }
    }

    #[test]
    fn options_use_defaults_flags_and_a_proxy_flag_replaces_the_embedded_table() {
        let cfg = Config { name: "t".into(), app_base: None, proxy: [("/api".to_string(), "http://old".to_string())].into(), default_port: 9000 };
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let o = parse_options(&a(&[]), &cfg).unwrap().unwrap();
        assert_eq!((o.host.as_str(), o.proxy.len()), ("127.0.0.1", 1));

        let o = parse_options(&a(&["--port", "1234", "--host", "0.0.0.0", "--proxy", "/v2=https://n.com", "--proxy", "/x=http://y", "--quiet"]), &cfg)
            .unwrap()
            .unwrap();
        assert_eq!((o.port, o.host.as_str(), o.quiet), (1234, "0.0.0.0", true));
        assert_eq!(o.proxy.keys().collect::<Vec<_>>(), vec!["/v2", "/x"], "el --proxy reemplaza la tabla embebida");

        assert!(parse_options(&a(&["--port"]), &cfg).is_err());
        assert!(parse_options(&a(&["--proxy", "sin-igual"]), &cfg).is_err());
        assert!(parse_options(&a(&["--wat"]), &cfg).is_err());
    }
}
