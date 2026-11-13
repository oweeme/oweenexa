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
//! - **renderiza bajo demanda** (Fase 66) las rutas dinámicas de Core que
//!   no están pre-renderizadas — un post publicado después del build, un
//!   producto nuevo — con el mismo pipeline de `nexa preview`, pidiéndole
//!   los datos a la API, y guarda el resultado en una **caché con tiempo de
//!   revalidación** (ISR): un solo render por ruta y ventana, aunque lleguen
//!   mil visitas a la vez; si la API cae, sigue sirviendo la última copia.
//!
//! `route` es una función pura (petición -> respuesta) para poder probar
//! toda la lógica sin abrir un socket.

use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    /// Renderizado bajo demanda (Fase 66), si el paquete lleva fuentes.
    dynamic: Option<Dynamic>,
    /// Donde el renderizado deja los archivos que genera (chunks, CSS).
    workdir: Option<PathBuf>,
    /// Nombres de cabecera propios (`[headers]`) con vida `'static`: se crean una vez al arrancar.
    header_names: HashMap<String, &'static str>,
    /// Con un token, `POST /__nexa/purge` vacía la caché de páginas renderizadas bajo demanda.
    purge_token: Option<String>,
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
        let header_names = payload
            .config
            .headers
            .iter()
            .flat_map(|r| r.headers.keys())
            .map(|k| (k.clone(), &*Box::leak(k.clone().into_boxed_str())))
            .collect();
        Site { payload, entries, app_base, dynamic: None, workdir: None, header_names, purge_token: None }
    }

    /// Activa el renderizado bajo demanda de las rutas no pre-renderizadas.
    pub fn with_dynamic(mut self, dynamic: Dynamic, workdir: Option<PathBuf>) -> Site {
        self.dynamic = Some(dynamic);
        self.workdir = workdir;
        self
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
    let mut reply = route_inner(site, req);
    // `[headers]` (Fase 75): las del proyecto pisan las de serie.
    let path = req.url.split(['?', '#']).next().and_then(decode_path).unwrap_or_default();
    for (name, value) in crate::rules::headers_for(&site.payload.config.headers, &path) {
        if let Some(name) = site.header_names.get(&name) {
            reply.headers.retain(|(n, _)| !n.eq_ignore_ascii_case(name));
            reply.headers.push((name, value));
        }
    }
    reply
}

fn route_inner(site: &Site, req: &Req) -> Reply {
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

    // `[redirects]` (Fase 75)
    if let Some((to, status)) = crate::rules::redirect_for(&site.payload.config.redirects, &path) {
        let mut r = text_reply(status, "");
        r.headers.push(("Location", to));
        return r;
    }

    let Some((entry, status)) = find(site, &path) else {
        // Los archivos que generó un render (chunks de eventos, CSS de una
        // página dinámica) no están en el paquete: viven en el directorio de
        // trabajo.
        if let Some(rest) = path.strip_prefix("/assets/") {
            return workdir_asset(site, rest, req).unwrap_or_else(|| not_found(site));
        }
        if let Some(dynamic) = &site.dynamic {
            if looks_like_a_page(&path) {
                return dynamic_reply(site, dynamic, &path, req);
            }
        }
        return not_found(site);
    };
    entry_reply(site, entry, status, req)
}

/// Solo una ruta que parece una página (sin extensión de archivo, de tamaño
/// razonable) se intenta renderizar: un bot pidiendo `/wp-login.php` o
/// `/.env` no debe costarle una llamada a tu API.
fn looks_like_a_page(path: &str) -> bool {
    let last = path.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    !last.contains('.') && path.len() <= 512 && path.split('/').count() <= 16
}

fn workdir_asset(site: &Site, rest: &str, req: &Req) -> Option<Reply> {
    let workdir = site.workdir.as_ref()?;
    if rest.is_empty() || Path::new(rest).components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    let file = workdir.join("dist/assets").join(rest);
    let bytes = std::fs::read(&file).ok()?;
    let path = format!("assets/{rest}");
    let etag = format!("\"{}\"", crate::assets::content_short_hash(&bytes));
    Some(body_reply(
        crate::serve::mime_for_path(&path),
        cache_control_for(&path),
        &etag,
        &bytes,
        None,
        200,
        &[],
        req,
    ))
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
    let raw = site.payload.bytes(entry.meta.offset, entry.meta.len);
    let gz = (entry.meta.gz_len > 0).then(|| site.payload.bytes(entry.meta.gz_offset, entry.meta.gz_len));
    body_reply(entry.content_type, entry.cache_control, &entry.etag, raw, gz, status, &[], req)
}

/// Arma la respuesta de un cuerpo con su versión gzip opcional: negocia la
/// codificación, calcula el `ETag` (distinto por codificación) y contesta
/// `304` si el cliente ya lo tiene. Lo comparten los archivos del paquete
/// y las páginas renderizadas bajo demanda.
#[allow(clippy::too_many_arguments)]
fn body_reply(
    content_type: &str,
    cache_control: &str,
    etag_base: &str,
    raw: &[u8],
    gz: Option<&[u8]>,
    status: u16,
    extra: &[(&'static str, String)],
    req: &Req,
) -> Reply {
    let wants_gzip = gz.is_some()
        && req.accept_encoding.split(',').any(|e| e.trim().split(';').next().is_some_and(|n| n.eq_ignore_ascii_case("gzip")));

    let etag = if wants_gzip { etag_base.replacen('"', "\"gz-", 1) } else { etag_base.to_string() };
    let mut headers: Vec<(&'static str, String)> = vec![
        ("Content-Type", content_type.to_string()),
        ("Cache-Control", cache_control.to_string()),
        ("ETag", etag.clone()),
    ];
    if gz.is_some() {
        headers.push(("Vary", "Accept-Encoding".to_string()));
    }
    headers.extend(extra.iter().cloned());
    headers.extend(security_headers());

    // `If-None-Match` puede traer varios ETag, débiles (`W/`) o `*`.
    let matches = req.if_none_match.split(',').map(|t| t.trim().trim_start_matches("W/")).any(|t| t == "*" || t == etag);
    if status == 200 && matches {
        return Reply { status: 304, headers, body: Vec::new() };
    }

    let body = if wants_gzip {
        headers.push(("Content-Encoding", "gzip".to_string()));
        gz.unwrap_or(raw).to_vec()
    } else {
        raw.to_vec()
    };
    Reply { status, headers, body }
}

// ---------------------------------------------------------------------
// Renderizado bajo demanda con caché (ISR) — Fase 66
// ---------------------------------------------------------------------

pub enum Rendered {
    Html(String),
    /// La ruta no existe, o la API respondió 404 para esos datos.
    NotFound,
    /// Falló la API o el render. El mensaje es solo para el log del
    /// servidor: nunca se le muestra a quien visita (puede traer la URL
    /// interna de tu API).
    Failed(String),
}

pub trait Renderer: Send + Sync {
    fn render(&self, path: &str) -> Rendered;
}

struct Cached {
    status: u16,
    raw: Vec<u8>,
    gz: Option<Vec<u8>>,
    etag: String,
    stored: Instant,
}

impl Cached {
    fn page(html: String, now: Instant) -> Cached {
        let raw = html.into_bytes();
        let gz = crate::bundle::gzip(&raw).ok().filter(|g| g.len() < raw.len());
        let etag = format!("\"{}\"", crate::assets::content_short_hash(&raw));
        Cached { status: 200, raw, gz, etag, stored: now }
    }
    fn missing(now: Instant) -> Cached {
        Cached { status: 404, raw: Vec::new(), gz: None, etag: String::new(), stored: now }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    /// Renderizada ahora mismo (y guardada).
    Miss,
    /// Reutilizada dentro de la ventana de revalidación.
    Hit,
    /// La API falló: se sirve la última copia, aunque esté vencida.
    Stale,
    /// Vencida pero dentro de `stale-while-revalidate`: se sirve ya y se renueva en segundo plano.
    Revalidating,
}

impl CacheState {
    fn header(self) -> &'static str {
        match self {
            CacheState::Miss => "miss",
            CacheState::Hit => "hit",
            CacheState::Stale => "stale",
            CacheState::Revalidating => "revalidating",
        }
    }
}

pub enum DynOutcome {
    Page(Arc<CachedPage>, CacheState),
    Missing,
    Unavailable,
}

/// Una página renderizada, lista para responder.
pub struct CachedPage {
    raw: Vec<u8>,
    gz: Option<Vec<u8>>,
    etag: String,
}

const MAX_CACHED_PAGES: usize = 2000;
const STRIPES: usize = 64;
/// Un 404 se recuerda poco: un post que "todavía no existe" debe aparecer
/// pronto, pero no a costa de una llamada a la API por cada visita de un bot.
const NEGATIVE_TTL: Duration = Duration::from_secs(10);

/// La caché de páginas renderizadas bajo demanda (cuesta clonar: es un `Arc`).
pub struct Dynamic(Arc<DynInner>);

struct DynInner {
    renderer: Box<dyn Renderer>,
    ttl: Duration,
    /// `stale-while-revalidate`: pasada la ventana `ttl`, durante estos segundos se sirve la copia vieja
    /// AL INSTANTE y se renueva en segundo plano. 0 = desactivado (se renueva esperando).
    stale: Duration,
    cache: Mutex<HashMap<String, Arc<Cached>>>,
    /// Un cerrojo por "franja" de rutas: si mil visitas piden a la vez un
    /// post vencido, una sola lo renderiza y el resto espera ese resultado
    /// en vez de golpear tu API mil veces.
    stripes: Vec<Mutex<()>>,
    /// Rutas que se están renovando en segundo plano (una sola vez cada una).
    refreshing: Mutex<std::collections::HashSet<String>>,
}

impl Dynamic {
    pub fn new(renderer: Box<dyn Renderer>, revalidate_secs: u64) -> Dynamic {
        Dynamic(Arc::new(DynInner {
            renderer,
            ttl: Duration::from_secs(revalidate_secs),
            stale: Duration::ZERO,
            cache: Mutex::new(HashMap::new()),
            stripes: (0..STRIPES).map(|_| Mutex::new(())).collect(),
            refreshing: Mutex::new(std::collections::HashSet::new()),
        }))
    }

    /// Activa `stale-while-revalidate` durante `secs` segundos después de vencer cada página.
    pub fn with_stale(self, secs: u64) -> Dynamic {
        let inner = Arc::try_unwrap(self.0).unwrap_or_else(|_| panic!("with_stale se usa al construir, antes de compartir"));
        Dynamic(Arc::new(DynInner { stale: Duration::from_secs(secs), ..inner }))
    }

    pub fn get(&self, path: &str, now: Instant) -> DynOutcome {
        DynInner::get(&self.0, path, now)
    }

    /// Borra de la caché una ruta (o todas con `None`): lo siguiente que se pida se renderiza de nuevo.
    pub fn purge(&self, path: Option<&str>) -> usize {
        let mut cache = self.0.cache.lock().unwrap_or_else(|e| e.into_inner());
        match path {
            Some(p) => usize::from(cache.remove(p).is_some()),
            None => {
                let n = cache.len();
                cache.clear();
                n
            }
        }
    }
}

impl DynInner {
    fn lookup(&self, path: &str, now: Instant) -> Option<Arc<Cached>> {
        self.cache.lock().unwrap_or_else(|e| e.into_inner()).get(path).cloned().filter(|c| {
            let ttl = if c.status == 200 { self.ttl } else { self.ttl.min(NEGATIVE_TTL) };
            !ttl.is_zero() && now.saturating_duration_since(c.stored) < ttl
        })
    }

    /// Una página vencida pero todavía dentro de la ventana de `stale-while-revalidate`.
    fn lookup_stale(&self, path: &str, now: Instant) -> Option<Arc<Cached>> {
        if self.stale.is_zero() || self.ttl.is_zero() {
            return None;
        }
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(path)
            .cloned()
            .filter(|c| c.status == 200 && now.saturating_duration_since(c.stored) < self.ttl + self.stale)
    }

    fn outcome(c: &Arc<Cached>, state: CacheState) -> DynOutcome {
        if c.status == 200 {
            DynOutcome::Page(Arc::new(CachedPage { raw: c.raw.clone(), gz: c.gz.clone(), etag: c.etag.clone() }), state)
        } else {
            DynOutcome::Missing
        }
    }

    fn store(&self, path: &str, entry: Arc<Cached>) {
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if cache.len() >= MAX_CACHED_PAGES && !cache.contains_key(path) {
            if let Some(oldest) = cache.iter().min_by_key(|(_, c)| c.stored).map(|(k, _)| k.clone()) {
                cache.remove(&oldest);
            }
        }
        cache.insert(path.to_string(), entry);
    }

    fn stripe(path: &str) -> usize {
        path.bytes().fold(0usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize)) % STRIPES
    }

    /// Renueva `path` en un hilo aparte (si no hay ya otro haciéndolo). Si la API falla, la copia vieja se queda.
    fn refresh_in_background(this: &Arc<DynInner>, path: &str) {
        if !this.refreshing.lock().unwrap_or_else(|e| e.into_inner()).insert(path.to_string()) {
            return;
        }
        let (inner, path) = (this.clone(), path.to_string());
        std::thread::spawn(move || {
            {
                let _guard = inner.stripes[Self::stripe(&path)].lock().unwrap_or_else(|e| e.into_inner());
                match inner.renderer.render(&path) {
                    Rendered::Html(html) => inner.store(&path, Arc::new(Cached::page(html, Instant::now()))),
                    Rendered::NotFound => inner.store(&path, Arc::new(Cached::missing(Instant::now()))),
                    Rendered::Failed(message) => eprintln!("render {path} (en segundo plano): {message}"),
                }
            }
            inner.refreshing.lock().unwrap_or_else(|e| e.into_inner()).remove(&path);
        });
    }

    fn get(this: &Arc<DynInner>, path: &str, now: Instant) -> DynOutcome {
        if let Some(c) = this.lookup(path, now) {
            return Self::outcome(&c, CacheState::Hit);
        }
        if let Some(c) = this.lookup_stale(path, now) {
            Self::refresh_in_background(this, path);
            return Self::outcome(&c, CacheState::Revalidating);
        }

        let _guard = this.stripes[Self::stripe(path)].lock().unwrap_or_else(|e| e.into_inner());
        // Quien esperaba el cerrojo encuentra ahora lo que renderizó el anterior.
        if let Some(c) = this.lookup(path, now) {
            return Self::outcome(&c, CacheState::Hit);
        }

        match this.renderer.render(path) {
            Rendered::Html(html) => {
                let entry = Arc::new(Cached::page(html, now));
                this.store(path, entry.clone());
                Self::outcome(&entry, CacheState::Miss)
            }
            Rendered::NotFound => {
                this.store(path, Arc::new(Cached::missing(now)));
                DynOutcome::Missing
            }
            Rendered::Failed(message) => {
                eprintln!("render {path}: {message}");
                // Más vale una página algo vieja que un error: si la API
                // cae, el sitio sigue en pie con lo último que sirvió.
                let previous = this.cache.lock().unwrap_or_else(|e| e.into_inner()).get(path).cloned();
                match previous {
                    Some(old) if old.status == 200 => Self::outcome(&old, CacheState::Stale),
                    _ => DynOutcome::Unavailable,
                }
            }
        }
    }
}

fn dynamic_reply(site: &Site, dynamic: &Dynamic, path: &str, req: &Req) -> Reply {
    match dynamic.get(path, Instant::now()) {
        DynOutcome::Page(page, state) => body_reply(
            "text/html; charset=utf-8",
            // El navegador revalida siempre (barato: 304); la ventana de
            // `--revalidate` es la del servidor frente a tu API.
            "no-cache",
            &page.etag,
            &page.raw,
            page.gz.as_deref(),
            200,
            &[("X-Nexa-Cache", state.header().to_string())],
            req,
        ),
        DynOutcome::Missing => not_found(site),
        DynOutcome::Unavailable => {
            let mut r = text_reply(502, "502 — no se pudo generar la página ahora; vuelve a intentarlo en unos segundos");
            r.headers.push(("Retry-After", "5".to_string()));
            r
        }
    }
}

/// El renderizador real: el mismo pipeline de `nexa preview` (parsear,
/// expandir componentes, pedir `load()` a la API, renderizar, layout, i18n).
pub struct PipelineRenderer {
    pub api_base: String,
}

impl Renderer for PipelineRenderer {
    fn render(&self, path: &str) -> Rendered {
        use crate::page_resolver::{resolve_page, ResolveError};
        match resolve_page(Path::new("src/pages"), path, &self.api_base, false) {
            Ok(html) => Rendered::Html(html),
            Err(ResolveError::NotFound) => Rendered::NotFound,
            Err(ResolveError::Failed(message)) => Rendered::Failed(message),
        }
    }
}

/// Extrae las fuentes del paquete a un directorio de trabajo **privado**
/// (el pipeline lee `src/`, `nexa.toml` y escribe `dist/assets/` relativos
/// al directorio actual) y se coloca ahí.
pub fn prepare_workdir(payload: &Payload) -> Result<PathBuf> {
    let mut seed = String::new();
    for s in &payload.sources {
        seed.push_str(&s.path);
        seed.push_str(&crate::assets::content_short_hash(payload.bytes(s.offset, s.len)));
    }
    let hash = crate::assets::content_short_hash(seed.as_bytes());
    let name: String = payload.config.name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    // `XDG_RUNTIME_DIR` ya es privado del usuario; si no, el temporal del sistema.
    let base = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).filter(|p| p.is_dir()).unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("nexa-{name}-{hash}"));

    std::fs::create_dir_all(&dir).with_context(|| format!("creando {}", dir.display()))?;
    secure_private_dir(&dir, &base)?;

    for source in &payload.sources {
        let rel = Path::new(&source.path);
        // Defensa en profundidad: un paquete manipulado no puede escribir fuera.
        if rel.components().any(|c| !matches!(c, Component::Normal(_)))
            || !(source.path.starts_with("src/") || source.path == "nexa.toml" || source.path == ".nexa/gen/assets.json" || source.path == "nexa.lock")
        {
            anyhow::bail!("el paquete trae una fuente con una ruta no permitida: {}", source.path);
        }
        let target = dir.join(rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, payload.bytes(source.offset, source.len))?;
    }
    std::fs::create_dir_all(dir.join("dist/assets"))?;
    Ok(dir)
}

#[cfg(unix)]
fn secure_private_dir(dir: &Path, base: &Path) -> Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    // El uid propio, sin libc: el de un archivo que acabamos de crear.
    let probe = base.join(format!(".nexa-probe-{}", std::process::id()));
    std::fs::write(&probe, b"")?;
    let me = std::fs::metadata(&probe)?.uid();
    let _ = std::fs::remove_file(&probe);
    let meta = std::fs::metadata(dir)?;
    if meta.uid() != me {
        anyhow::bail!("{} pertenece a otro usuario — no se usa como directorio de trabajo", dir.display());
    }
    if meta.permissions().mode() & 0o077 != 0 {
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn secure_private_dir(_dir: &Path, _base: &Path) -> Result<()> {
    Ok(())
}

// ---------------------------------------------------------------------
// El servidor: sockets, hilos, argumentos
// ---------------------------------------------------------------------

struct Options {
    host: String,
    port: u16,
    quiet: bool,
    proxy: BTreeMap<String, String>,
    revalidate: u64,
    stale: u64,
    api_url: String,
    purge_token: Option<String>,
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
    --api-url <URL>         API de la que las páginas dinámicas piden sus datos
                            (por defecto $NEXA_API_URL) — solo si el sitio tiene
                            rutas dinámicas
    --revalidate <SEG>      Segundos que se reutiliza una página renderizada
                            bajo demanda antes de volver a pedirle los datos a
                            la API (por defecto $NEXA_REVALIDATE o 60; 0 = nunca)
    --stale <SEG>           stale-while-revalidate: tras vencer, durante estos
                            segundos se sirve la copia vieja al instante y se
                            renueva en segundo plano (por defecto $NEXA_STALE o 0)
    --purge-token <TOKEN>   Activa `POST /__nexa/purge[?path=/ruta]` con la cabecera
                            `Authorization: Bearer <TOKEN>` (por defecto
                            $NEXA_PURGE_TOKEN; mínimo 8 caracteres): vacía la caché
    --quiet                 No imprime una línea por petición
    -h, --help              Esta ayuda
";

fn parse_options(args: &[String], defaults: &crate::bundle::Config) -> Result<Option<Options>> {
    let mut opts = Options {
        host: "127.0.0.1".to_string(),
        port: std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(defaults.default_port),
        quiet: false,
        proxy: defaults.proxy.clone(),
        revalidate: std::env::var("NEXA_REVALIDATE").ok().and_then(|v| v.parse().ok()).unwrap_or(defaults.revalidate),
        stale: std::env::var("NEXA_STALE").ok().and_then(|v| v.parse().ok()).unwrap_or(defaults.stale),
        api_url: std::env::var("NEXA_API_URL").unwrap_or_default(),
        purge_token: std::env::var("NEXA_PURGE_TOKEN").ok().filter(|t| !t.is_empty()),
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
            "--revalidate" => opts.revalidate = value("--revalidate")?.parse().context("--revalidate debe ser un número de segundos")?,
            "--stale" => opts.stale = value("--stale")?.parse().context("--stale debe ser un número de segundos")?,
            "--purge-token" => opts.purge_token = Some(value("--purge-token")?).filter(|t| !t.is_empty()),
            "--api-url" => opts.api_url = value("--api-url")?.trim_end_matches('/').to_string(),
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
    let dynamic_enabled = payload.config.dynamic && !payload.sources.is_empty();

    // El pipeline lee `src/`, `nexa.toml` y escribe `dist/assets/` relativos
    // al directorio actual: se extraen a un directorio privado y se entra.
    let workdir = if dynamic_enabled {
        let workdir = prepare_workdir(&payload)?;
        std::env::set_current_dir(&workdir).with_context(|| format!("entrando a {}", workdir.display()))?;
        // `[api] headers` (Fase 78): el token sale del entorno del servidor, nunca del paquete.
        crate::api_headers::init()?;
        Some(workdir)
    } else {
        None
    };
    let mut site = Site::new(payload);
    if let Some(workdir) = workdir {
        site = site.with_dynamic(Dynamic::new(Box::new(PipelineRenderer { api_base: opts.api_url.clone() }), opts.revalidate).with_stale(opts.stale), Some(workdir));
    }
    site.purge_token = opts.purge_token.clone().filter(|t| t.len() >= 8);
    if opts.purge_token.as_ref().is_some_and(|t| t.len() < 8) {
        eprintln!("atención: el token de purga debe tener al menos 8 caracteres: la purga queda desactivada");
    }
    let site = Arc::new(site);

    let address = format!("{}:{}", opts.host, opts.port);
    let server = Arc::new(
        Server::http(&address).map_err(|e| anyhow::anyhow!("{e}")).with_context(|| format!("no se pudo abrir {address}"))?,
    );

    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(2, 16);
    println!("{name}: sirviendo {} archivo(s) desde memoria en http://{address}  ({workers} hilos)", site.file_count());
    for (prefix, target) in &opts.proxy {
        println!("  proxy {prefix} -> {target}");
    }
    if dynamic_enabled {
        let api = if opts.api_url.is_empty() { "(sin definir)".to_string() } else { opts.api_url.clone() };
        println!("  renderizado bajo demanda: activo — API {api}, revalida cada {} s", opts.revalidate);
        if opts.api_url.is_empty() {
            println!("  atención: define --api-url (o $NEXA_API_URL) o las páginas dinámicas con `load()` fallarán");
        }
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

const PURGE_PATH: &str = "/__nexa/purge";

/// `POST /__nexa/purge[?path=/blog/mi-post]` con `Authorization: Bearer <token>`: vacía la caché de esa ruta
/// (o de todas). Sin token configurado no existe (404): no se revela.
fn purge(site: &Site, method: &str, url: &str, authorization: &str) -> Reply {
    let (Some(token), Some(dynamic)) = (site.purge_token.as_deref(), site.dynamic.as_ref()) else {
        return text_reply(404, "404 — no encontrado");
    };
    if method != "POST" {
        let mut r = text_reply(405, "405 — usa POST");
        r.headers.push(("Allow", "POST".to_string()));
        return r;
    }
    let given = authorization.strip_prefix("Bearer ").unwrap_or("");
    if !constant_time_eq(given.as_bytes(), token.as_bytes()) {
        return text_reply(401, "401 — token incorrecto");
    }
    let path = crate::serve::query_param(url, "path");
    let purged = dynamic.purge(path.as_deref());
    let mut r = text_reply(200, &format!("{{\"purged\":{purged}}}"));
    r.headers.retain(|(n, _)| *n != "Content-Type");
    r.headers.insert(0, ("Content-Type", "application/json".to_string()));
    r
}

/// Comparación en tiempo constante (sin salir al primer byte distinto).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    diff == 0
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

    if path_only == PURGE_PATH {
        let auth = header_value(&request, "Authorization");
        let reply = purge(site, &method, &url, &auth);
        let mut response = Response::from_data(reply.body).with_status_code(reply.status);
        for (name, value) in reply.headers {
            if let Ok(h) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                response.add_header(h);
            }
        }
        let _ = request.respond(response);
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
        let config = Config { name: "t".into(), app_base: app_base.map(String::from), proxy: Default::default(), default_port: 8080, dynamic: false, revalidate: 60, stale: 0, redirects: vec![], headers: vec![] };
        Site::new(unpack(&pack(&files, &[], config).unwrap()).unwrap())
    }

    fn get(site: &Site, url: &str) -> Reply {
        route(site, &Req { method: "GET", url, accept_encoding: "", if_none_match: "" })
    }
    fn header<'a>(r: &'a Reply, name: &str) -> Option<&'a str> {
        r.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    fn site_with_rules(redirects: Vec<crate::rules::Redirect>, headers: Vec<crate::rules::HeaderRule>) -> Site {
        let files: Vec<(String, Vec<u8>)> = vec![("index.html".into(), b"<h1>hi</h1>".to_vec()), ("about/index.html".into(), b"<h1>about</h1>".to_vec())];
        let config = Config { name: "t".into(), app_base: None, proxy: Default::default(), default_port: 8080, dynamic: false, revalidate: 60, stale: 0, redirects, headers };
        Site::new(unpack(&pack(&files, &[], config).unwrap()).unwrap())
    }

    #[test]
    fn redirects_answer_with_location_and_status() {
        let s = site_with_rules(
            vec![crate::rules::Redirect { from: "/viejo".into(), to: "/about".into(), status: 308 }, crate::rules::Redirect { from: "/go/*".into(), to: "/*".into(), status: 302 }],
            vec![],
        );
        let r = get(&s, "/viejo?x=1");
        assert_eq!((r.status, header(&r, "Location")), (308, Some("/about")));
        let r = get(&s, "/go//evil.com");
        assert_eq!((r.status, header(&r, "Location")), (302, Some("/evil.com")), "nunca `//evil.com`");
        assert_eq!(get(&s, "/about").status, 200);
    }

    #[test]
    fn custom_headers_apply_by_path_and_override_the_defaults() {
        let rules = crate::rules::headers_from(
            &[
                ("/*".to_string(), [("Content-Security-Policy".to_string(), "default-src 'self'".to_string()), ("X-Frame-Options".to_string(), "DENY".to_string())].into()),
                ("/about".to_string(), [("X-Extra".to_string(), "1".to_string())].into()),
            ]
            .into(),
        )
        .unwrap();
        let s = site_with_rules(vec![], rules);
        let home = get(&s, "/");
        assert_eq!(header(&home, "Content-Security-Policy"), Some("default-src 'self'"));
        assert_eq!(header(&home, "X-Frame-Options"), Some("DENY"));
        assert_eq!(home.headers.iter().filter(|(n, _)| n.eq_ignore_ascii_case("x-frame-options")).count(), 1);
        assert_eq!(header(&home, "X-Extra"), None);
        assert_eq!(header(&get(&s, "/about"), "X-Extra"), Some("1"));
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
        let cfg = Config { name: "t".into(), app_base: None, proxy: [("/api".to_string(), "http://old".to_string())].into(), default_port: 9000, dynamic: false, revalidate: 60, stale: 0, redirects: vec![], headers: vec![] };
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

    // ---- Renderizado bajo demanda (Fase 66) ----

    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    enum Mode {
        Html(&'static str),
        NotFound,
        Fail,
    }

    #[derive(Clone)]
    struct Fake {
        calls: Arc<AtomicUsize>,
        mode: Arc<Mutex<Mode>>,
        delay_ms: u64,
    }

    impl Fake {
        fn new(mode: Mode) -> Fake {
            Fake { calls: Arc::new(AtomicUsize::new(0)), mode: Arc::new(Mutex::new(mode)), delay_ms: 0 }
        }
        fn set(&self, mode: Mode) {
            *self.mode.lock().unwrap() = mode;
        }
        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl Renderer for Fake {
        fn render(&self, _path: &str) -> Rendered {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(self.delay_ms));
            }
            match &*self.mode.lock().unwrap() {
                Mode::Html(h) => Rendered::Html(h.to_string()),
                Mode::NotFound => Rendered::NotFound,
                Mode::Fail => Rendered::Failed("la API no responde (http://interna:9100)".into()),
            }
        }
    }

    fn dynamic_site(fake: &Fake, ttl: u64) -> Site {
        site(None).with_dynamic(Dynamic::new(Box::new(fake.clone()), ttl), None)
    }

    fn wait_for(cond: impl Fn() -> bool) {
        for _ in 0..200 {
            if cond() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("la condición no se cumplió a tiempo");
    }

    #[test]
    fn stale_while_revalidate_serves_the_old_copy_instantly_and_renews_in_the_background() {
        let fake = Fake::new(Mode::Html("<h1>v1</h1>"));
        let d = Dynamic::new(Box::new(fake.clone()), 60).with_stale(300);
        let t0 = Instant::now();
        assert!(matches!(d.get("/blog/a/b", t0), DynOutcome::Page(_, CacheState::Miss)));

        // la API publica una versión nueva
        fake.set(Mode::Html("<h1>v2</h1>"));
        // vencida pero dentro de la ventana: responde YA con la vieja, sin esperar a la API
        match d.get("/blog/a/b", t0 + Duration::from_secs(61)) {
            DynOutcome::Page(page, CacheState::Revalidating) => assert_eq!(page.raw, b"<h1>v1</h1>"),
            _ => panic!("debía servir la copia vieja"),
        }
        // …mientras la renueva en segundo plano (una sola vez aunque lleguen más visitas)
        d.get("/blog/a/b", t0 + Duration::from_secs(62));
        wait_for(|| fake.calls() >= 2);
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(fake.calls(), 2, "una sola renovación por ruta");
        match d.get("/blog/a/b", Instant::now() + Duration::from_secs(1)) {
            DynOutcome::Page(page, CacheState::Hit) => assert_eq!(page.raw, b"<h1>v2</h1>", "la siguiente visita ya ve la nueva"),
            _ => panic!("debía ser un acierto con la copia nueva"),
        }
    }

    #[test]
    fn past_the_stale_window_the_visit_waits_for_a_fresh_render_and_a_dead_api_keeps_the_old_copy() {
        let fake = Fake::new(Mode::Html("<h1>v1</h1>"));
        let d = Dynamic::new(Box::new(fake.clone()), 60).with_stale(100);
        let t0 = Instant::now();
        d.get("/p", t0);
        fake.set(Mode::Html("<h1>v2</h1>"));
        assert!(matches!(d.get("/p", t0 + Duration::from_secs(200)), DynOutcome::Page(_, CacheState::Miss)), "fuera de la ventana se espera");
        // si la API cae durante la renovación en segundo plano, la copia vieja sigue siendo la que se sirve
        let fake2 = Fake::new(Mode::Html("<h1>ok</h1>"));
        let d2 = Dynamic::new(Box::new(fake2.clone()), 60).with_stale(300);
        d2.get("/q", t0);
        fake2.set(Mode::Fail);
        assert!(matches!(d2.get("/q", t0 + Duration::from_secs(70)), DynOutcome::Page(_, CacheState::Revalidating)));
        wait_for(|| fake2.calls() >= 2);
        std::thread::sleep(Duration::from_millis(50));
        assert!(matches!(d2.get("/q", t0 + Duration::from_secs(71)), DynOutcome::Page(_, CacheState::Revalidating | CacheState::Hit)));
    }

    #[test]
    fn purge_removes_one_path_or_everything_and_needs_the_right_token() {
        let fake = Fake::new(Mode::Html("<h1>x</h1>"));
        let mut s = dynamic_site(&fake, 600);
        s.purge_token = Some("token-largo-123".to_string());
        let t0 = Instant::now();
        let d = s.dynamic.as_ref().unwrap();
        d.get("/a", t0);
        d.get("/b", t0);
        assert_eq!(fake.calls(), 2);

        assert_eq!(purge(&s, "POST", "/__nexa/purge?path=/a", "Bearer incorrecto").status, 401);
        assert_eq!(purge(&s, "POST", "/__nexa/purge?path=/a", "").status, 401);
        assert_eq!(purge(&s, "GET", "/__nexa/purge", "Bearer token-largo-123").status, 405);
        assert_eq!(fake.calls(), 2, "sin token válido no se purga nada");

        let r = purge(&s, "POST", "/__nexa/purge?path=/a", "Bearer token-largo-123");
        assert_eq!((r.status, r.body.as_slice()), (200, &b"{\"purged\":1}"[..]));
        assert!(matches!(d.get("/a", t0), DynOutcome::Page(_, CacheState::Miss)));
        assert!(matches!(d.get("/b", t0), DynOutcome::Page(_, CacheState::Hit)));
        assert_eq!(purge(&s, "POST", "/__nexa/purge", "Bearer token-largo-123").body, b"{\"purged\":2}");
        assert!(matches!(d.get("/b", t0), DynOutcome::Page(_, CacheState::Miss)));
    }

    #[test]
    fn purge_does_not_exist_without_a_token() {
        let fake = Fake::new(Mode::Html("x"));
        let s = dynamic_site(&fake, 60);
        assert_eq!(purge(&s, "POST", "/__nexa/purge", "Bearer lo-que-sea").status, 404);
        assert!(constant_time_eq(b"abc", b"abc") && !constant_time_eq(b"abc", b"abd") && !constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn a_rendered_page_is_cached_for_the_revalidation_window_then_rendered_again() {
        let fake = Fake::new(Mode::Html("<h1>post</h1>"));
        let d = Dynamic::new(Box::new(fake.clone()), 60);
        let t0 = Instant::now();

        assert!(matches!(d.get("/blog/a/b", t0), DynOutcome::Page(_, CacheState::Miss)));
        for secs in [1, 30, 59] {
            assert!(matches!(d.get("/blog/a/b", t0 + Duration::from_secs(secs)), DynOutcome::Page(_, CacheState::Hit)));
        }
        assert_eq!(fake.calls(), 1, "dentro de la ventana, la API no se vuelve a llamar");

        assert!(matches!(d.get("/blog/a/b", t0 + Duration::from_secs(61)), DynOutcome::Page(_, CacheState::Miss)));
        assert_eq!(fake.calls(), 2);
        // otra ruta = otra entrada
        d.get("/blog/a/c", t0);
        assert_eq!(fake.calls(), 3);
    }

    #[test]
    fn revalidate_zero_never_reuses_a_page() {
        let fake = Fake::new(Mode::Html("x"));
        let d = Dynamic::new(Box::new(fake.clone()), 0);
        let t0 = Instant::now();
        for _ in 0..3 {
            d.get("/p/q", t0);
        }
        assert_eq!(fake.calls(), 3);
    }

    #[test]
    fn when_the_api_fails_the_last_copy_is_served_stale_instead_of_an_error() {
        let fake = Fake::new(Mode::Html("<h1>v1</h1>"));
        let d = Dynamic::new(Box::new(fake.clone()), 10);
        let t0 = Instant::now();
        d.get("/blog/a/b", t0);

        fake.set(Mode::Fail);
        let later = t0 + Duration::from_secs(3600);
        match d.get("/blog/a/b", later) {
            DynOutcome::Page(page, CacheState::Stale) => assert_eq!(page.raw, b"<h1>v1</h1>"),
            _ => panic!("debe servir la copia vieja"),
        }
        // y sin ninguna copia previa, no hay nada que servir
        assert!(matches!(d.get("/blog/never/seen", later), DynOutcome::Unavailable));
    }

    #[test]
    fn a_missing_page_is_remembered_only_briefly_so_a_new_post_appears_soon() {
        let fake = Fake::new(Mode::NotFound);
        let d = Dynamic::new(Box::new(fake.clone()), 600);
        let t0 = Instant::now();
        assert!(matches!(d.get("/blog/a/nuevo", t0), DynOutcome::Missing));
        assert!(matches!(d.get("/blog/a/nuevo", t0 + Duration::from_secs(5)), DynOutcome::Missing));
        assert_eq!(fake.calls(), 1, "un bot insistiendo no golpea la API");

        fake.set(Mode::Html("<h1>ya existe</h1>"));
        assert!(matches!(d.get("/blog/a/nuevo", t0 + Duration::from_secs(11)), DynOutcome::Page(_, CacheState::Miss)), "a los 10 s el 404 caduca aunque revalidate sea 600");
    }

    #[test]
    fn a_burst_of_simultaneous_requests_renders_once_not_once_per_visitor() {
        let mut fake = Fake::new(Mode::Html("<h1>popular</h1>"));
        fake.delay_ms = 80; // una API lenta: todos llegan mientras el primero espera
        let d = Arc::new(Dynamic::new(Box::new(fake.clone()), 60));
        let t0 = Instant::now();

        let handles: Vec<_> = (0..24)
            .map(|_| {
                let d = d.clone();
                std::thread::spawn(move || matches!(d.get("/blog/a/popular", t0), DynOutcome::Page(..)))
            })
            .collect();
        assert!(handles.into_iter().all(|h| h.join().unwrap()));
        assert_eq!(fake.calls(), 1, "24 visitas simultáneas = 1 llamada a la API");
    }

    #[test]
    fn dynamic_routes_are_rendered_through_route_and_report_the_cache_state() {
        let fake = Fake::new(Mode::Html("<h1>hola</h1>".repeat(40).leak()));
        let s = dynamic_site(&fake, 60);

        let first = get(&s, "/blog/hector/mi-post");
        assert_eq!(first.status, 200);
        assert_eq!(header(&first, "X-Nexa-Cache"), Some("miss"));
        assert_eq!(header(&first, "Content-Type"), Some("text/html; charset=utf-8"));
        assert_eq!(header(&get(&s, "/blog/hector/mi-post"), "X-Nexa-Cache"), Some("hit"));
        assert_eq!(fake.calls(), 1);

        // gzip y ETag también para páginas dinámicas
        let gz = route(&s, &Req { method: "GET", url: "/blog/hector/mi-post", accept_encoding: "gzip", if_none_match: "" });
        assert_eq!(header(&gz, "Content-Encoding"), Some("gzip"));
        let etag = header(&first, "ETag").unwrap().to_string();
        let again = route(&s, &Req { method: "GET", url: "/blog/hector/mi-post", accept_encoding: "", if_none_match: &etag });
        assert_eq!(again.status, 304);
    }

    #[test]
    fn prerendered_files_win_over_dynamic_rendering() {
        let fake = Fake::new(Mode::Html("DINAMICO"));
        let s = dynamic_site(&fake, 60);
        assert_eq!(get(&s, "/about").body, b"<h1>about</h1>");
        assert_eq!(fake.calls(), 0, "lo pre-renderizado nunca llama a la API");
    }

    #[test]
    fn things_that_do_not_look_like_pages_never_cost_an_api_call() {
        let fake = Fake::new(Mode::Html("x"));
        let s = dynamic_site(&fake, 60);
        for url in ["/wp-login.php", "/.env", "/favicon.ico", "/robots.txt", "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r"] {
            assert_eq!(get(&s, url).status, 404, "{url}");
        }
        assert_eq!(fake.calls(), 0);
    }

    #[test]
    fn a_failure_with_no_previous_copy_is_a_502_that_leaks_no_internal_details() {
        let fake = Fake::new(Mode::Fail);
        let s = dynamic_site(&fake, 60);
        let r = get(&s, "/blog/a/b");
        assert_eq!(r.status, 502);
        assert_eq!(header(&r, "Retry-After"), Some("5"));
        let body = String::from_utf8_lossy(&r.body);
        assert!(!body.contains("interna") && !body.contains("9100"), "{body}");
    }

    #[test]
    fn a_page_that_the_api_says_does_not_exist_is_a_real_404_with_the_sites_404_html() {
        let fake = Fake::new(Mode::NotFound);
        let s = dynamic_site(&fake, 60);
        let r = get(&s, "/blog/a/no-existe");
        assert_eq!(r.status, 404);
        assert_eq!(r.body, b"<h1>perdido</h1>");
    }

    #[test]
    fn assets_generated_by_a_render_are_served_from_the_workdir_and_cannot_escape_it() {
        let dir = std::env::temp_dir().join(format!("nexa-prod-workdir-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("dist/assets")).unwrap();
        std::fs::write(dir.join("dist/assets/Post-3.1a2b3c4d.js"), "export default 1").unwrap();
        std::fs::write(dir.join("secreto.txt"), "no").unwrap();
        let s = site(None).with_dynamic(Dynamic::new(Box::new(Fake::new(Mode::NotFound)), 60), Some(dir));

        let ok = get(&s, "/assets/Post-3.1a2b3c4d.js");
        assert_eq!(ok.status, 200);
        assert_eq!(header(&ok, "Cache-Control"), Some("public, max-age=31536000, immutable"));
        assert_eq!(get(&s, "/assets/falta.js").status, 404);
        for evil in ["/assets/../secreto.txt", "/assets/%2e%2e/secreto.txt", "/assets/a/../../secreto.txt"] {
            assert_ne!(get(&s, evil).status, 200, "{evil}");
        }
        assert!(workdir_asset(&s, "../secreto.txt", &Req { method: "GET", url: "/", accept_encoding: "", if_none_match: "" }).is_none());
    }

    fn payload_with_sources(sources: Vec<(&str, &str)>) -> Payload {
        let sources: Vec<(String, Vec<u8>)> = sources.into_iter().map(|(p, c)| (p.to_string(), c.as_bytes().to_vec())).collect();
        let config = Config { name: "mi sitio!".into(), app_base: None, proxy: Default::default(), default_port: 8080, dynamic: true, revalidate: 60, stale: 0, redirects: vec![], headers: vec![] };
        unpack(&pack(&[], &sources, config).unwrap()).unwrap()
    }

    #[test]
    fn prepare_workdir_extracts_the_sources_into_a_private_directory() {
        let payload = payload_with_sources(vec![("src/pages/blog/[a]/[b].tsx", "export default 1"), ("nexa.toml", "[project]")]);
        let dir = prepare_workdir(&payload).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("src/pages/blog/[a]/[b].tsx")).unwrap(), "export default 1");
        assert!(dir.join("dist/assets").is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&dir).unwrap().permissions().mode() & 0o077, 0, "solo el dueño puede entrar");
        }
    }

    #[test]
    fn a_tampered_package_cannot_write_outside_the_workdir() {
        for evil in ["../evil.txt", "src/../../evil.txt", "/etc/passwd", "otra/cosa.tsx"] {
            let payload = payload_with_sources(vec![(evil, "x")]);
            assert!(prepare_workdir(&payload).is_err(), "{evil}");
        }
    }

    #[test]
    fn options_accept_revalidate_and_api_url() {
        let cfg = Config { name: "t".into(), app_base: None, proxy: Default::default(), default_port: 9000, dynamic: true, revalidate: 60, stale: 0, redirects: vec![], headers: vec![] };
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let o = parse_options(&a(&["--revalidate", "5", "--api-url", "https://api.miapp.com/"]), &cfg).unwrap().unwrap();
        assert_eq!((o.revalidate, o.api_url.as_str()), (5, "https://api.miapp.com"));
        assert!(parse_options(&a(&["--revalidate", "abc"]), &cfg).is_err());
    }
}
