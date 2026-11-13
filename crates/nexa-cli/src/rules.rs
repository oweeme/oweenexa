//! Redirecciones y cabeceras propias (Fase 75), declaradas en `nexa.toml`:
//!
//! ```toml
//! [redirects]
//! "/blog-viejo" = "/blog"                                  # 301
//! "/docs/v1/*" = { to = "/docs/*", status = 302 }          # `*` recoge el resto
//! "/tienda/:slug" = "/shop/:slug"                          # `:nombre` recoge un segmento
//!
//! [headers."/*"]
//! Content-Security-Policy = "default-src 'self'"
//! [headers."/assets/*"]
//! Cache-Control = "public, max-age=31536000, immutable"
//! ```
//!
//! Los aplican `nexa dev`, `nexa preview` y el ejecutable de `nexa bundle`; `nexa build`
//! escribe además `dist/_redirects` y `dist/_headers` (Netlify, Cloudflare Pages) y
//! páginas con `<meta refresh>` para hosts estáticos que no leen nada de eso.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Redirect {
    pub from: String,
    pub to: String,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeaderRule {
    pub path: String,
    pub headers: BTreeMap<String, String>,
}

/// `"/x" = "/y"` o `"/x" = { to = "/y", status = 308 }`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RedirectTarget {
    To(String),
    Full { to: String, status: Option<u16> },
}

const STATUSES: [u16; 5] = [301, 302, 303, 307, 308];

/// Reglas del manifiesto, ya validadas.
pub fn redirects_from(table: &BTreeMap<String, RedirectTarget>) -> Result<Vec<Redirect>, String> {
    let mut out = Vec::new();
    for (from, target) in table {
        let (to, status) = match target {
            RedirectTarget::To(to) => (to.clone(), 301),
            RedirectTarget::Full { to, status } => (to.clone(), status.unwrap_or(301)),
        };
        if !from.starts_with('/') {
            return Err(format!("[redirects] `{from}`: el origen debe empezar por `/`"));
        }
        if !(to.starts_with('/') || to.starts_with("https://") || to.starts_with("http://")) || to.starts_with("//") {
            return Err(format!("[redirects] `{from}`: el destino `{to}` debe empezar por `/`, `http://` o `https://`"));
        }
        if !STATUSES.contains(&status) {
            return Err(format!("[redirects] `{from}`: el estado {status} no es una redirección (usa 301, 302, 303, 307 o 308)"));
        }
        out.push(Redirect { from: from.clone(), to, status });
    }
    Ok(out)
}

pub fn headers_from(table: &BTreeMap<String, BTreeMap<String, String>>) -> Result<Vec<HeaderRule>, String> {
    let mut out = Vec::new();
    for (path, headers) in table {
        if !path.starts_with('/') {
            return Err(format!("[headers] `{path}`: la ruta debe empezar por `/`"));
        }
        for (name, value) in headers {
            let ok_name = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
            if !ok_name || value.contains(['\r', '\n']) {
                return Err(format!("[headers] `{path}`: la cabecera `{name}` no es válida"));
            }
        }
        out.push(HeaderRule { path: path.clone(), headers: headers.clone() });
    }
    Ok(out)
}

/// Compara una ruta con un patrón: `:nombre` recoge un segmento, `*` final recoge el resto.
/// Devuelve lo capturado (`*` queda bajo la clave `*`).
pub fn matches(pattern: &str, path: &str) -> Option<BTreeMap<String, String>> {
    let pat: Vec<&str> = pattern.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    let mut caps = BTreeMap::new();
    for (i, seg) in pat.iter().enumerate() {
        if *seg == "*" && i == pat.len() - 1 {
            caps.insert("*".to_string(), parts.get(i..).unwrap_or(&[]).join("/"));
            return Some(caps);
        }
        let part = parts.get(i)?;
        if let Some(name) = seg.strip_prefix(':') {
            caps.insert(name.to_string(), (*part).to_string());
        } else if seg != part {
            return None;
        }
    }
    (parts.len() == pat.len()).then_some(caps)
}

fn encode(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~' | b'/') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// A dónde redirigir `path` (ya decodificada), si alguna regla lo pide.
pub fn redirect_for(rules: &[Redirect], path: &str) -> Option<(String, u16)> {
    for rule in rules {
        let Some(caps) = matches(&rule.from, path) else { continue };
        let mut to = rule.to.clone();
        for (name, value) in &caps {
            let token = if name == "*" { "*".to_string() } else { format!(":{name}") };
            to = to.replace(&token, &encode(value));
        }
        // Un valor capturado no puede convertir `/x` en `//evil.com` (redirección abierta).
        if to.starts_with('/') {
            let rest = to.trim_start_matches(['/', '\\']);
            to = format!("/{rest}");
        }
        if to == path {
            continue; // no a sí misma
        }
        return Some((to, rule.status));
    }
    None
}

/// Cabeceras que aplican a `path` (las reglas se acumulan; las posteriores pisan).
pub fn headers_for(rules: &[HeaderRule], path: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for rule in rules {
        if matches(&rule.path, path).is_some() {
            for (k, v) in &rule.headers {
                out.retain(|(n, _)| !n.eq_ignore_ascii_case(k));
                out.push((k.clone(), v.clone()));
            }
        }
    }
    out
}

/// `dist/_redirects` (formato de Netlify y Cloudflare Pages).
pub fn netlify_redirects(rules: &[Redirect]) -> String {
    rules.iter().map(|r| format!("{} {} {}\n", r.from, r.to, r.status)).collect()
}

/// `dist/_headers`.
pub fn netlify_headers(rules: &[HeaderRule]) -> String {
    let mut s = String::new();
    for r in rules {
        s.push_str(&format!("{}\n", r.path));
        for (k, v) in &r.headers {
            s.push_str(&format!("  {k}: {v}\n"));
        }
    }
    s
}

/// Para hosts que solo sirven archivos (GitHub Pages): una página que redirige.
pub fn meta_refresh_page(to: &str) -> String {
    let esc = to.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;");
    format!(
        "<!doctype html>\n<html><head><meta charset=\"UTF-8\"><title>Redirigiendo…</title>\
         <link rel=\"canonical\" href=\"{esc}\"><meta name=\"robots\" content=\"noindex\">\
         <meta http-equiv=\"refresh\" content=\"0; url={esc}\"></head>\
         <body><a href=\"{esc}\">{esc}</a></body></html>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(from: &str, to: &str, status: u16) -> Redirect {
        Redirect { from: from.into(), to: to.into(), status }
    }

    #[test]
    fn exact_params_and_splat() {
        let rules = [r("/viejo", "/nuevo", 301), r("/tienda/:slug", "/shop/:slug", 302), r("/docs/v1/*", "/docs/*", 308)];
        assert_eq!(redirect_for(&rules, "/viejo"), Some(("/nuevo".into(), 301)));
        assert_eq!(redirect_for(&rules, "/tienda/zapato"), Some(("/shop/zapato".into(), 302)));
        assert_eq!(redirect_for(&rules, "/docs/v1/a/b"), Some(("/docs/a/b".into(), 308)));
        assert_eq!(redirect_for(&rules, "/docs/v1"), Some(("/docs/".into(), 308)));
        assert_eq!(redirect_for(&rules, "/otra"), None);
        assert_eq!(redirect_for(&rules, "/viejo/x"), None);
    }

    #[test]
    fn a_captured_value_cannot_make_an_open_redirect() {
        let rules = [r("/go/*", "/*", 302), r("/x/:a", "/:a", 302)];
        // "/go//evil.com" → el resto es "evil.com" (los vacíos se descartan) → "/evil.com"
        assert_eq!(redirect_for(&rules, "/go//evil.com").unwrap().0, "/evil.com");
        let to = redirect_for(&rules, "/x/%2Fevil.com").unwrap().0;
        assert!(to.starts_with('/') && !to.starts_with("//"), "{to}");
        assert!(redirect_for(&rules, "/go/a\r\nSet-Cookie: x").unwrap().0.contains("%0D%0A"));
    }

    #[test]
    fn validation_rejects_bad_rules() {
        let t = |to: &str, st: Option<u16>| -> BTreeMap<String, RedirectTarget> {
            [("/a".to_string(), RedirectTarget::Full { to: to.into(), status: st })].into()
        };
        assert!(redirects_from(&t("/b", None)).is_ok());
        assert!(redirects_from(&t("https://x.com/b", Some(308))).is_ok());
        assert!(redirects_from(&t("//evil.com", None)).is_err());
        assert!(redirects_from(&t("javascript:alert(1)", None)).is_err());
        assert!(redirects_from(&t("/b", Some(200))).is_err());
        let bad: BTreeMap<_, _> = [("a".to_string(), RedirectTarget::To("/b".into()))].into();
        assert!(redirects_from(&bad).is_err());
    }

    #[test]
    fn headers_accumulate_by_path_and_reject_injection() {
        let table: BTreeMap<String, BTreeMap<String, String>> = [
            ("/*".to_string(), [("X-A".to_string(), "1".to_string())].into()),
            ("/api/*".to_string(), [("x-a".to_string(), "2".to_string()), ("X-B".to_string(), "b".to_string())].into()),
        ]
        .into();
        let rules = headers_from(&table).unwrap();
        assert_eq!(headers_for(&rules, "/"), vec![("X-A".to_string(), "1".to_string())]);
        let h = headers_for(&rules, "/api/x");
        assert!(h.contains(&("x-a".to_string(), "2".to_string())) && h.len() == 2);
        let evil: BTreeMap<String, BTreeMap<String, String>> =
            [("/*".to_string(), [("X-A".to_string(), "1\r\nSet-Cookie: a=b".to_string())].into())].into();
        assert!(headers_from(&evil).is_err());
    }

    #[test]
    fn static_host_outputs() {
        assert_eq!(netlify_redirects(&[r("/a", "/b", 301)]), "/a /b 301\n");
        assert!(meta_refresh_page("/b?x=1&y=\"2\"").contains("url=/b?x=1&amp;y=&quot;2&quot;"));
    }
}
