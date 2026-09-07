//! Fase 18: `[pwa]` en `nexa.toml` -> `manifest.webmanifest` + `sw.js`
//! reales, generados por `nexa build` — nada de esto se ejecuta durante
//! la compilación (mismo principio que `load`/`seo`/`schema`): son datos
//! declarativos que este módulo traduce a los dos artefactos reales que
//! un navegador necesita para poder instalar el sitio y funcionar
//! offline.

use std::collections::BTreeMap;

use anyhow::{bail, Result};
use serde_json::json;

use crate::manifest::PwaSection;

const VALID_STRATEGIES: &[&str] = &["cache-first", "network-first", "stale-while-revalidate"];

/// `dist/manifest.webmanifest`. `short_name`/`display` caen a valores
/// razonables si no se declaran (`name` recortado, `"standalone"`).
/// Los íconos se declaran dos veces (192x192 y 512x512) apuntando al
/// mismo archivo si `icon` está presente — los navegadores confían en
/// el campo `sizes` declarado, no verifican el PNG real.
pub fn render_manifest(pwa: &PwaSection) -> String {
    let short_name = pwa.short_name.clone().unwrap_or_else(|| pwa.name.chars().take(12).collect());

    let mut manifest = json!({
        "name": pwa.name,
        "short_name": short_name,
        "start_url": "/",
        "display": pwa.display.clone().unwrap_or_else(|| "standalone".to_string()),
    });

    let obj = manifest.as_object_mut().expect("json!({}) siempre es un objeto");
    if let Some(theme_color) = &pwa.theme_color {
        obj.insert("theme_color".to_string(), json!(theme_color));
    }
    if let Some(background_color) = &pwa.background_color {
        obj.insert("background_color".to_string(), json!(background_color));
    }
    if let Some(icon) = &pwa.icon {
        obj.insert(
            "icons".to_string(),
            json!([
                { "src": icon, "sizes": "192x192", "type": "image/png" },
                { "src": icon, "sizes": "512x512", "type": "image/png" },
            ]),
        );
    }

    serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".to_string())
}

/// `dist/sw.js`: instala, reclama las páginas de inmediato
/// (`skipWaiting`/`clients.claim`, para que la primera visita ya quede
/// controlada sin recargar dos veces), y por cada `fetch` GET elige una
/// de las tres estrategias según qué prefijo de ruta declaró `[pwa.cache]`
/// — el prefijo más largo que matchea gana, así `/api/products` puede
/// tener una regla distinta de `/api` en general. Sin ninguna regla que
/// matchee, cae a `network-first` (intenta la red, si falla usa el
/// cache) — el default más seguro cuando no se declaró nada explícito.
pub fn render_service_worker(cache: &BTreeMap<String, String>) -> Result<String> {
    for (prefix, strategy) in cache {
        if !VALID_STRATEGIES.contains(&strategy.as_str()) {
            bail!(
                "[pwa.cache] \"{prefix}\" = \"{strategy}\" no es una estrategia válida — usa una \
                 de: {}",
                VALID_STRATEGIES.join(", ")
            );
        }
    }

    let mut rules: Vec<(&String, &String)> = cache.iter().collect();
    rules.sort_by_key(|(prefix, _)| std::cmp::Reverse(prefix.len()));

    let rules_js: String = rules
        .iter()
        .map(|(prefix, strategy)| format!("  [{}, {}],", json_string(prefix), json_string(strategy)))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        r#"// Generado por Nexa (Fase 18) a partir de [pwa]/[pwa.cache] en nexa.toml.
const CACHE_NAME = "nexa-pwa-v1";
const RULES = [
{rules_js}
];

function strategyFor(pathname) {{
    for (const [prefix, strategy] of RULES) {{
        if (pathname.startsWith(prefix)) return strategy;
    }}
    return "network-first";
}}

self.addEventListener("install", () => {{
    self.skipWaiting();
}});

self.addEventListener("activate", (event) => {{
    event.waitUntil(self.clients.claim());
}});

self.addEventListener("fetch", (event) => {{
    if (event.request.method !== "GET") return;
    const url = new URL(event.request.url);
    const strategy = strategyFor(url.pathname);

    if (strategy === "cache-first") {{
        event.respondWith(cacheFirst(event.request));
    }} else if (strategy === "stale-while-revalidate") {{
        event.respondWith(staleWhileRevalidate(event.request));
    }} else {{
        event.respondWith(networkFirst(event.request));
    }}
}});

async function cacheFirst(request) {{
    const cached = await caches.match(request);
    if (cached) return cached;
    const response = await fetch(request);
    await putInCache(request, response);
    return response;
}}

async function networkFirst(request) {{
    try {{
        const response = await fetch(request);
        await putInCache(request, response);
        return response;
    }} catch (err) {{
        const cached = await caches.match(request);
        if (cached) return cached;
        throw err;
    }}
}}

async function staleWhileRevalidate(request) {{
    const cached = await caches.match(request);
    const fetchPromise = fetch(request).then((response) => {{
        putInCache(request, response);
        return response;
    }});
    return cached ?? fetchPromise;
}}

async function putInCache(request, response) {{
    if (!response || !response.ok) return;
    const cache = await caches.open(CACHE_NAME);
    await cache.put(request, response.clone());
}}
"#
    ))
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// `<link rel="manifest">` + `<meta name="theme-color">` para el
/// `<head>` de cada página — se ensambla igual que el resto de
/// fragmentos de head (SEO, hreflang), no como un parámetro nuevo de
/// `document::assemble`.
pub fn head_fragment(pwa: &PwaSection) -> String {
    let mut lines = vec!["<link rel=\"manifest\" href=\"/manifest.webmanifest\">".to_string()];
    if let Some(theme_color) = &pwa.theme_color {
        lines.push(format!("<meta name=\"theme-color\" content=\"{theme_color}\">"));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pwa(name: &str) -> PwaSection {
        PwaSection { name: name.to_string(), ..Default::default() }
    }

    #[test]
    fn renders_a_minimal_manifest_with_sane_defaults() {
        let manifest = render_manifest(&pwa("Mi App"));
        assert!(manifest.contains("\"name\": \"Mi App\""));
        assert!(manifest.contains("\"short_name\": \"Mi App\""));
        assert!(manifest.contains("\"display\": \"standalone\""));
        assert!(manifest.contains("\"start_url\": \"/\""));
        assert!(!manifest.contains("\"icons\""));
    }

    #[test]
    fn renders_icons_twice_when_an_icon_is_declared() {
        let mut p = pwa("Mi App");
        p.icon = Some("/icon-512.png".to_string());
        let manifest = render_manifest(&p);
        assert!(manifest.contains("\"192x192\""));
        assert!(manifest.contains("\"512x512\""));
        assert_eq!(manifest.matches("/icon-512.png").count(), 2);
    }

    #[test]
    fn respects_an_explicit_short_name_and_theme_color() {
        let mut p = pwa("Mi App Muy Larga");
        p.short_name = Some("MAML".to_string());
        p.theme_color = Some("#2563eb".to_string());
        let manifest = render_manifest(&p);
        assert!(manifest.contains("\"short_name\": \"MAML\""));
        assert!(manifest.contains("\"theme_color\": \"#2563eb\""));
    }

    #[test]
    fn service_worker_rejects_an_unknown_strategy() {
        let mut cache = BTreeMap::new();
        cache.insert("/api".to_string(), "always-fresh".to_string());
        let err = render_service_worker(&cache).unwrap_err();
        assert!(err.to_string().contains("always-fresh"));
    }

    #[test]
    fn service_worker_matches_the_longest_prefix_first() {
        let mut cache = BTreeMap::new();
        cache.insert("/api".to_string(), "network-first".to_string());
        cache.insert("/api/products".to_string(), "stale-while-revalidate".to_string());

        let sw = render_service_worker(&cache).unwrap();
        let products_pos = sw.find("/api/products").unwrap();
        let api_pos = sw.rfind("[\"/api\"").unwrap();
        assert!(products_pos < api_pos, "el prefijo más específico debe ir primero en RULES");
    }

    #[test]
    fn service_worker_defines_all_three_strategy_functions() {
        let sw = render_service_worker(&BTreeMap::new()).unwrap();
        assert!(sw.contains("async function cacheFirst"));
        assert!(sw.contains("async function networkFirst"));
        assert!(sw.contains("async function staleWhileRevalidate"));
        assert!(sw.contains("self.skipWaiting()"));
        assert!(sw.contains("self.clients.claim()"));
    }

    #[test]
    fn head_fragment_always_includes_the_manifest_link() {
        let fragment = head_fragment(&pwa("Mi App"));
        assert!(fragment.contains("<link rel=\"manifest\" href=\"/manifest.webmanifest\">"));
        assert!(!fragment.contains("theme-color"));
    }

    #[test]
    fn head_fragment_includes_theme_color_when_declared() {
        let mut p = pwa("Mi App");
        p.theme_color = Some("#111827".to_string());
        let fragment = head_fragment(&p);
        assert!(fragment.contains("<meta name=\"theme-color\" content=\"#111827\">"));
    }
}
