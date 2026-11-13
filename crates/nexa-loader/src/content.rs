//! Contenido local (Fase 73): `load = "content:/posts"` lee archivos de
//! `src/content/` en vez de llamar a una API. Un blog o documentación sin backend.
//!
//! - `content:/posts`        → array con todas las entradas de `src/content/posts/`
//! - `content:/posts/:slug`  → una entrada (404 si no existe)
//! - `content:/site`         → el JSON de `src/content/site.json` tal cual
//! - `content:/posts?perPage=5&page=2` → solo esa página de la lista (paginación)
//! - `content:/posts?perPage=5&pages`  → `[{"n":"1"},{"n":"2"},…]`: las páginas que existen, para `paths`
//!
//! Cada `.md` aporta su cabecera (`---` … `---`, un YAML sencillo) más `slug` (el
//! nombre del archivo) y `body` (el Markdown, para `<Markdown>`). Cada `.json`
//! con un objeto aporta sus campos más `slug`. Las entradas con `draft: true`
//! no existen. Orden: `date` descendente si todas la tienen; si no, por `slug`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::error::LoaderError;

pub const SCHEME: &str = "content:";
const ROOT: &str = "src/content";
const MAX_FILE: u64 = 5 * 1024 * 1024;

pub fn is_content(template: &str) -> bool {
    template.starts_with(SCHEME)
}

/// `resolved` es la plantilla ya sustituida (`content:/posts/hola`).
pub fn load(resolved: &str) -> Result<Value, LoaderError> {
    load_from(Path::new(ROOT), resolved)
}

pub fn load_from(root: &Path, resolved: &str) -> Result<Value, LoaderError> {
    let rest = resolved.strip_prefix(SCHEME).unwrap_or(resolved);
    // `resolve_url` codifica `/` como `%2F` (un comodín `[...rest]` llega como `a%2Fb`).
    let decoded = rest.replace("%2F", "/").replace("%2f", "/");
    let path = decoded.split(['?', '#']).next().unwrap_or("").trim_matches('/');
    let mut parts = Vec::new();
    for part in path.split('/').filter(|p| !p.is_empty()) {
        // `resolve_url` ya codificó el valor: un `%2F` o `..` no es un nombre de archivo válido.
        if part == "." || part == ".." || !part.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
            return Err(LoaderError::NotFound);
        }
        parts.push(part);
    }
    let target: PathBuf = parts.iter().fold(root.to_path_buf(), |p, part| p.join(part));
    let query = rest.split('#').next().unwrap_or("").split_once('?').map(|(_, q)| q).unwrap_or("");

    if target.is_dir() && query_has(query, "perPage") {
        return paginate(collection(&target)?, query);
    }
    if target.is_dir() {
        // `docs/guia/` con un `index.md` es la PÁGINA `guia` (no una lista), salvo la raíz de la colección.
        if parts.len() >= 2 {
            for ext in ["md", "json"] {
                let index = target.join(format!("index.{ext}"));
                if index.is_file() {
                    return entry(&index)?.ok_or(LoaderError::NotFound);
                }
            }
        }
        return collection(&target);
    }
    for ext in ["md", "json"] {
        let file = target.with_extension_appended(ext);
        if file.is_file() {
            return entry(&file)?.ok_or(LoaderError::NotFound);
        }
    }
    Err(LoaderError::NotFound)
}

fn query_has(query: &str, key: &str) -> bool {
    query.split('&').any(|kv| kv.split('=').next() == Some(key))
}

fn query_num(query: &str, key: &str) -> Option<usize> {
    query.split('&').find_map(|kv| kv.strip_prefix(key)?.strip_prefix('=')?.parse().ok())
}

/// `?perPage=N&page=K` → la página K (desde 1); `?perPage=N&pages` → la lista de páginas.
fn paginate(all: Value, query: &str) -> Result<Value, LoaderError> {
    let items = all.as_array().cloned().unwrap_or_default();
    let per = query_num(query, "perPage").filter(|n| *n > 0 && *n <= 1000).ok_or_else(|| LoaderError::InvalidParam("perPage".into()))?;
    let pages = items.len().div_ceil(per).max(1);
    if query_has(query, "pages") {
        return Ok(Value::Array((1..=pages).map(|n| serde_json::json!({ "n": n.to_string() })).collect()));
    }
    let page = query_num(query, "page").unwrap_or(1);
    if page == 0 || page > pages {
        return Err(LoaderError::NotFound);
    }
    Ok(Value::Array(items.into_iter().skip((page - 1) * per).take(per).collect()))
}

trait AppendExt {
    fn with_extension_appended(&self, ext: &str) -> PathBuf;
}
impl AppendExt for PathBuf {
    fn with_extension_appended(&self, ext: &str) -> PathBuf {
        let mut s = self.clone().into_os_string();
        s.push(".");
        s.push(ext);
        PathBuf::from(s)
    }
}

fn collection(dir: &Path) -> Result<Value, LoaderError> {
    let mut files = Vec::new();
    walk(dir, &mut files, 0)?;
    files.sort();
    let mut items = Vec::new();
    for file in files {
        if let Some(Value::Object(mut o)) = entry(&file)? {
            // `path`: la ruta dentro de la colección, sin extensión (`guia/instalar`):
            // lo que un `[...path]` necesita para enumerar páginas anidadas.
            let rel = file.strip_prefix(dir).unwrap_or(&file).with_extension("");
            let mut parts: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
            // `guia/index.md` es la página `guia` (la raíz de la colección conserva `index`)
            if parts.len() > 1 && parts.last().is_some_and(|l| l == "index") {
                parts.pop();
            }
            let rel = parts.join("/");
            o.entry("path").or_insert(Value::String(rel));
            items.push(o);
        }
    }
    let all_dated = !items.is_empty() && items.iter().all(|o| o.get("date").is_some_and(|d| d.is_string()));
    if all_dated {
        items.sort_by(|a, b| b["date"].as_str().cmp(&a["date"].as_str()));
    }
    Ok(Value::Array(items.into_iter().map(Value::Object).collect()))
}

/// Recorre subcarpetas (hasta 8 niveles) reuniendo los `.md` y `.json`.
fn walk(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<(), LoaderError> {
    if depth > 8 {
        return Ok(());
    }
    for e in fs::read_dir(dir).map_err(|e| LoaderError::Network(e.to_string()))?.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out, depth + 1)?;
        } else if p.is_file() && matches!(p.extension().and_then(|e| e.to_str()), Some("md") | Some("json")) {
            out.push(p);
        }
    }
    Ok(())
}

/// `Ok(None)`: es un borrador.
fn entry(file: &Path) -> Result<Option<Value>, LoaderError> {
    let meta = fs::metadata(file).map_err(|e| LoaderError::Network(e.to_string()))?;
    if meta.len() > MAX_FILE {
        return Err(LoaderError::InvalidJson(format!("{} es demasiado grande", file.display())));
    }
    let text = fs::read_to_string(file).map_err(|e| LoaderError::Network(format!("{}: {e}", file.display())))?;
    let slug = file.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let mut object = if file.extension().is_some_and(|e| e == "json") {
        match serde_json::from_str::<Value>(&text).map_err(|e| LoaderError::InvalidJson(format!("{}: {e}", file.display())))? {
            Value::Object(o) => o,
            other => return Ok(Some(other)), // un JSON que no es un objeto se devuelve tal cual
        }
    } else {
        let (front, body) = split_front_matter(&text);
        let mut o = parse_front_matter(front);
        o.insert("body".into(), Value::String(body.trim_start_matches(['\r', '\n']).to_string()));
        o
    };
    if object.get("draft") == Some(&Value::Bool(true)) {
        return Ok(None);
    }
    object.entry("slug").or_insert(Value::String(slug));
    Ok(Some(Value::Object(object)))
}

/// (cabecera, cuerpo)
fn split_front_matter(text: &str) -> (&str, &str) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(rest) = text.strip_prefix("---") else { return ("", text) };
    let rest = rest.trim_start_matches([' ', '\t']);
    let Some(rest) = rest.strip_prefix("\r\n").or_else(|| rest.strip_prefix('\n')) else { return ("", text) };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            return (&rest[..offset], &rest[offset + line.len()..]);
        }
        offset += line.len();
    }
    ("", text)
}

fn parse_front_matter(front: &str) -> Map<String, Value> {
    let mut map = Map::new();
    let mut current_list: Option<String> = None;
    for raw in front.lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let (Some(key), Some(item)) = (&current_list, line.trim_start().strip_prefix("- ")) {
            if let Some(Value::Array(a)) = map.get_mut(key) {
                a.push(scalar(item.trim()));
            }
            continue;
        }
        current_list = None;
        let Some((key, value)) = line.split_once(':') else { continue };
        let key = key.trim().to_string();
        let value = value.trim();
        if key.is_empty() || key.contains(' ') {
            continue;
        }
        if value.is_empty() {
            map.insert(key.clone(), Value::Array(Vec::new()));
            current_list = Some(key);
        } else if let Some(inner) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
            map.insert(key, Value::Array(split_list(inner).iter().map(|s| scalar(s)).collect()));
        } else {
            map.insert(key, scalar(value));
        }
    }
    map
}

fn split_list(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in inner.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => { quote = None; cur.push(c); }
            (None, '"' | '\'') => { quote = Some(c); cur.push(c); }
            (None, ',') => { out.push(cur.trim().to_string()); cur.clear(); }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn scalar(s: &str) -> Value {
    let s = s.trim();
    for q in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(q) && s.ends_with(q) {
            return Value::String(s[1..s.len() - 1].to_string());
        }
    }
    match s {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" | "~" => Value::Null,
        _ => {
            if let Ok(n) = s.parse::<i64>() {
                Value::from(n)
            } else if s.contains('.') && s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-') {
                s.parse::<f64>().ok().and_then(serde_json::Number::from_f64).map(Value::Number).unwrap_or_else(|| Value::String(s.to_string()))
            } else {
                Value::String(s.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn site(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nexa-content-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("posts")).unwrap();
        fs::write(d.join("posts/hola.md"), "---\ntitle: \"Hola: mundo\"\ndate: 2025-02-01\ntags: [a, \"b, c\"]\nprice: 9.5\nfeatured: true\n---\n# Hola\n\ntexto\n").unwrap();
        fs::write(d.join("posts/viejo.md"), "---\ntitle: Viejo\ndate: 2024-01-01\ntags:\n  - x\n  - y\n---\ncuerpo").unwrap();
        fs::write(d.join("posts/borrador.md"), "---\ntitle: B\ndate: 2026-01-01\ndraft: true\n---\nx").unwrap();
        fs::write(d.join("posts/sin-cabecera.json"), "{\"title\":\"J\",\"date\":\"2023-05-05\"}").unwrap();
        fs::write(d.join("site.json"), "{\"name\":\"Mi sitio\",\"links\":[1,2]}").unwrap();
        d
    }

    #[test]
    fn a_collection_lists_entries_newest_first_without_drafts() {
        let d = site("list");
        let v = load_from(&d, "content:/posts").unwrap();
        let slugs: Vec<_> = v.as_array().unwrap().iter().map(|o| o["slug"].as_str().unwrap()).collect();
        assert_eq!(slugs, vec!["hola", "viejo", "sin-cabecera"]);
    }

    #[test]
    fn front_matter_types_and_body() {
        let d = site("one");
        let v = load_from(&d, "content:/posts/hola").unwrap();
        assert_eq!(v["title"], "Hola: mundo");
        assert_eq!(v["tags"], json!(["a", "b, c"]));
        assert_eq!(v["price"], json!(9.5));
        assert_eq!(v["featured"], json!(true));
        assert_eq!(v["date"], "2025-02-01");
        assert_eq!(v["body"], "# Hola\n\ntexto\n");
        let v = load_from(&d, "content:/posts/viejo").unwrap();
        assert_eq!(v["tags"], json!(["x", "y"]));
    }

    #[test]
    fn nested_collections_expose_the_relative_path() {
        let d = site("nested");
        fs::create_dir_all(d.join("docs/guia")).unwrap();
        fs::write(d.join("docs/index.md"), "---\ntitle: Inicio\n---\nhola").unwrap();
        fs::write(d.join("docs/guia/instalar.md"), "---\ntitle: Instalar\n---\npaso 1").unwrap();
        let v = load_from(&d, "content:/docs").unwrap();
        let paths: Vec<_> = v.as_array().unwrap().iter().map(|o| o["path"].as_str().unwrap()).collect();
        assert_eq!(paths, vec!["guia/instalar", "index"]);
        // y se pide por ese camino, con `/` o `%2F` (así llega un `[...path]`)
        assert_eq!(load_from(&d, "content:/docs/guia/instalar").unwrap()["title"], "Instalar");
        fs::write(d.join("docs/guia/index.md"), "---\ntitle: La guía\n---\nx").unwrap();
        assert_eq!(load_from(&d, "content:/docs/guia").unwrap()["title"], "La guía", "una carpeta con index.md es una página");
        assert!(load_from(&d, "content:/docs").unwrap().is_array(), "la raíz de la colección es la lista");
        assert_eq!(load_from(&d, "content:/docs/guia%2Finstalar").unwrap()["title"], "Instalar");
    }

    #[test]
    fn pagination_slices_the_list_and_enumerates_the_pages() {
        let d = site("pages");
        let slugs = |q: &str| -> Vec<String> { load_from(&d, q).unwrap().as_array().unwrap().iter().map(|o| o["slug"].as_str().unwrap().to_string()).collect() };
        assert_eq!(slugs("content:/posts?perPage=2&page=1"), vec!["hola", "viejo"]);
        assert_eq!(slugs("content:/posts?perPage=2&page=2"), vec!["sin-cabecera"]);
        assert_eq!(load_from(&d, "content:/posts?perPage=2&pages").unwrap(), json!([{"n":"1"},{"n":"2"}]));
        assert!(matches!(load_from(&d, "content:/posts?perPage=2&page=3"), Err(LoaderError::NotFound)));
        assert!(matches!(load_from(&d, "content:/posts?perPage=2&page=0"), Err(LoaderError::NotFound)));
        assert!(load_from(&d, "content:/posts?perPage=0&page=1").is_err());
    }

    #[test]
    fn json_files_are_entries_or_plain_data() {
        let d = site("json");
        assert_eq!(load_from(&d, "content:/posts/sin-cabecera").unwrap()["slug"], "sin-cabecera");
        assert_eq!(load_from(&d, "content:/site").unwrap()["name"], "Mi sitio");
    }

    #[test]
    fn missing_drafts_and_unsafe_names_are_not_found() {
        let d = site("nf");
        // `a%2Fb` es `posts/a/b`: una carpeta que no existe
        assert!(matches!(load_from(&site("nf2"), "content:/posts/a%2Fb"), Err(LoaderError::NotFound)));
        for p in ["content:/posts/nada", "content:/posts/borrador", "content:/posts/..", "content:/../x", "content:/posts/%2E%2E"] {
            assert!(matches!(load_from(&d, p), Err(LoaderError::NotFound)), "{p}");
        }
    }
}
