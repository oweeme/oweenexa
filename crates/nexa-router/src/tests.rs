use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use super::*;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Crea un directorio temporal único por test (los tests de Rust corren
/// en paralelo, así que no pueden compartir uno solo) y escribe en él los
/// archivos `.tsx` vacíos que describen `paths`.
fn pages_dir(paths: &[&str]) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("nexa-router-test-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    for path in paths {
        let file = dir.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "export default function Page() { return <div />; }").unwrap();
    }

    dir
}

fn patterns(routes: &[Route]) -> Vec<&str> {
    routes.iter().map(|r| r.pattern.as_str()).collect()
}

#[test]
fn scans_index_and_static_pages() {
    let dir = pages_dir(&["index.tsx", "about.tsx"]);
    let routes = scan_pages(&dir).unwrap();
    assert_eq!(patterns(&routes), vec!["/", "/about"]);
}

#[test]
fn nested_index_maps_to_the_parent_directory() {
    let dir = pages_dir(&["products/index.tsx"]);
    let routes = scan_pages(&dir).unwrap();
    assert_eq!(patterns(&routes), vec!["/products"]);
}

#[test]
fn bracket_segment_becomes_dynamic() {
    let dir = pages_dir(&["products/[slug].tsx"]);
    let routes = scan_pages(&dir).unwrap();
    assert_eq!(patterns(&routes), vec!["/products/:slug"]);
    assert!(routes[0].is_dynamic());
}

#[test]
fn missing_pages_dir_yields_no_routes() {
    let dir = std::env::temp_dir().join("nexa-router-test-does-not-exist");
    let routes = scan_pages(&dir).unwrap();
    assert!(routes.is_empty());
}

#[test]
fn match_route_captures_dynamic_param() {
    let dir = pages_dir(&["products/[slug].tsx"]);
    let routes = scan_pages(&dir).unwrap();

    let (route, params) = match_route(&routes, "/products/iphone-17").expect("should match");
    assert_eq!(route.pattern, "/products/:slug");
    assert_eq!(params.get("slug").map(String::as_str), Some("iphone-17"));
}

#[test]
fn match_route_returns_none_when_nothing_matches() {
    let dir = pages_dir(&["about.tsx"]);
    let routes = scan_pages(&dir).unwrap();
    assert!(match_route(&routes, "/no-existe").is_none());
}

#[test]
fn match_route_prefers_static_over_dynamic_at_the_same_shape() {
    let dir = pages_dir(&["products/featured.tsx", "products/[slug].tsx"]);
    let routes = scan_pages(&dir).unwrap();

    let (route, params) = match_route(&routes, "/products/featured").expect("should match");
    assert_eq!(route.pattern, "/products/featured");
    assert!(params.is_empty());

    let (route, params) = match_route(&routes, "/products/iphone-17").expect("should match");
    assert_eq!(route.pattern, "/products/:slug");
    assert_eq!(params.get("slug").map(String::as_str), Some("iphone-17"));
}

#[test]
fn root_route_matches_only_the_empty_path() {
    let dir = pages_dir(&["index.tsx", "about.tsx"]);
    let routes = scan_pages(&dir).unwrap();

    assert!(match_route(&routes, "/").is_some());
    assert!(match_route(&routes, "/about").is_some());
    assert!(match_route(&routes, "/about/extra").is_none());
}

#[test]
fn catch_all_captures_the_rest_and_is_the_last_resort() {
    let dir = pages_dir(&["docs/index.tsx", "docs/intro.tsx", "docs/[section]/index.tsx", "docs/[...rest].tsx", "[...all].tsx"]);
    let routes = scan_pages(&dir).unwrap();
    assert!(patterns(&routes).contains(&"/docs/:rest*"));
    let at = |p: &str| match_route(&routes, p).map(|(r, params)| (r.pattern.clone(), params));
    // una ruta exacta o de un solo segmento dinámico gana al comodín
    assert_eq!(at("/docs/intro").unwrap().0, "/docs/intro");
    assert_eq!(at("/docs/guias").unwrap().0, "/docs/:section");
    // el comodín recoge uno o más segmentos
    let (pattern, params) = at("/docs/guias/avanzado/x").unwrap();
    assert_eq!(pattern, "/docs/:rest*");
    assert_eq!(params["rest"], "guias/avanzado/x");
    // `/docs` exacto es su index, no el comodín (que pide al menos un segmento)
    assert_eq!(at("/docs").unwrap().0, "/docs");
    // el comodín de la raíz recoge cualquier cosa
    assert_eq!(at("/otra/cosa").unwrap().1["all"], "otra/cosa");
}

#[test]
fn a_catch_all_needs_at_least_one_segment() {
    let dir = pages_dir(&["wiki/[...path].tsx"]);
    let routes = scan_pages(&dir).unwrap();
    assert!(match_route(&routes, "/wiki").is_none());
    assert_eq!(match_route(&routes, "/wiki/a").unwrap().1["path"], "a");
}

#[test]
fn files_starting_with_an_underscore_are_not_pages() {
    let dir = pages_dir(&["index.tsx", "_layout.tsx", "docs/_layout.tsx", "docs/_helper.tsx", "docs/intro.tsx"]);
    assert_eq!(patterns(&scan_pages(&dir).unwrap()), vec!["/", "/docs/intro"]);
}
