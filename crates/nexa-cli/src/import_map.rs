//! El import map real que necesita un navegador para resolver un
//! identificador "pelado" (`import { stripe } from "stripe";`, que
//! `nexa-activation` antepone a un chunk cuando lo detecta) a una URL de
//! verdad. Generaliza lo que hasta la Fase 13 era un caso especial
//! cableado a mano solo para `platform` — ahora es exactamente el mismo
//! mecanismo para un paquete de un tercero declarado en `nexa.toml`
//! (`[imports]`, Fase 15).

use std::collections::{BTreeMap, BTreeSet};

use crate::manifest::Manifest;

/// `platform` (Fase 13) siempre está disponible, resuelto contra el
/// archivo que el propio `nexa-cli` embebe y sirve — no hace falta que
/// el proyecto lo declare en `nexa.toml`.
const BUILTIN_IMPORTS: &[(&str, &str)] = &[("platform", "/assets/nexa-platform.js")];

/// Los builtin de Nexa + lo que el proyecto haya declarado en
/// `[imports]` — si un proyecto reutiliza el nombre `platform`, gana su
/// propia declaración (poco probable, pero mejor que silenciosamente
/// ignorarla).
pub fn resolved(project_manifest: &Manifest) -> BTreeMap<String, String> {
    let mut map: BTreeMap<String, String> =
        BUILTIN_IMPORTS.iter().map(|(name, specifier)| (name.to_string(), specifier.to_string())).collect();
    map.extend(project_manifest.imports.clone());
    map
}

/// Filtra `resolved` a solo los nombres que algún handler de *esta*
/// página de verdad usa (`<nombre>.`), o que alguna isla de la página
/// declara como su `specifier` (Fase 16 — una isla no "usa" el nombre
/// dentro del código fuente de un handler, lo declara directamente en
/// `data-nexa-island`) — mismo criterio de costo cero que el resto de
/// Nexa: una página que no usa `stripe.` no debe declarar `stripe` en su
/// import map, y una página sin ningún import usado no lleva
/// `<script type="importmap">` en absoluto.
pub fn used_by_page(
    resolved: &BTreeMap<String, String>,
    handlers: &BTreeMap<String, String>,
    island_specifiers: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    resolved
        .iter()
        .filter(|(name, _)| {
            handlers.values().any(|source| source.contains(&format!("{name}."))) || island_specifiers.contains(*name)
        })
        .map(|(name, specifier)| (name.clone(), specifier.clone()))
        .collect()
}

/// `<script type="importmap">...</script>`, o `None` si no hay ningún
/// import realmente usado en esta página. Debe ir antes que cualquier
/// `<script type="module">` en el documento — por eso `document::
/// assemble` lo antepone al resto de `<head>`.
pub fn script_tag(used: &BTreeMap<String, String>) -> Option<String> {
    if used.is_empty() {
        return None;
    }

    let json = serde_json::json!({ "imports": used });
    let body = serde_json::to_string(&json).unwrap_or_else(|_| "{\"imports\":{}}".to_string());
    Some(format!("<script type=\"importmap\">{body}</script>"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_always_includes_the_builtin_platform_entry() {
        let map = resolved(&Manifest::default());
        assert_eq!(map.get("platform").map(String::as_str), Some("/assets/nexa-platform.js"));
    }

    #[test]
    fn resolved_merges_project_declared_imports() {
        let mut manifest = Manifest::default();
        manifest.imports.insert("stripe".to_string(), "https://cdn.example.com/stripe.js".to_string());

        let map = resolved(&manifest);
        assert_eq!(map.get("stripe").map(String::as_str), Some("https://cdn.example.com/stripe.js"));
        assert!(map.contains_key("platform"));
    }

    fn handlers(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn used_by_page_only_keeps_names_a_handler_actually_references() {
        let all = resolved(&{
            let mut m = Manifest::default();
            m.imports.insert("stripe".to_string(), "https://cdn.example.com/stripe.js".to_string());
            m
        });

        let used = used_by_page(&all, &handlers(&[("pay", "function pay() { stripe.load(); }")]), &BTreeSet::new());

        assert_eq!(used.len(), 1);
        assert!(used.contains_key("stripe"));
        assert!(!used.contains_key("platform"));
    }

    #[test]
    fn used_by_page_is_empty_when_no_handler_uses_anything_importable() {
        let all = resolved(&Manifest::default());
        let used = used_by_page(&all, &handlers(&[("buy", "function buy() { cart.add(id); }")]), &BTreeSet::new());
        assert!(used.is_empty());
    }

    #[test]
    fn used_by_page_also_keeps_names_an_island_declares_as_its_specifier() {
        // Fase 16: una isla no aparece dentro del *código* de ningún
        // handler — declara su specifier directamente en
        // `data-nexa-island`, así que `used_by_page` necesita una vía de
        // detección aparte de la búsqueda textual `nombre.` en handlers.
        let all = resolved(&{
            let mut m = Manifest::default();
            m.imports.insert("dashboardIsland".to_string(), "/vendor/dashboard-island.js".to_string());
            m
        });

        let islands: BTreeSet<String> = ["dashboardIsland".to_string()].into_iter().collect();
        let used = used_by_page(&all, &BTreeMap::new(), &islands);

        assert_eq!(used.len(), 1);
        assert!(used.contains_key("dashboardIsland"));
    }

    #[test]
    fn script_tag_is_none_for_an_empty_map() {
        assert_eq!(script_tag(&BTreeMap::new()), None);
    }

    #[test]
    fn script_tag_contains_a_valid_import_map() {
        let mut used = BTreeMap::new();
        used.insert("stripe".to_string(), "https://cdn.example.com/stripe.js".to_string());

        let tag = script_tag(&used).expect("expected a script tag");
        assert!(tag.starts_with("<script type=\"importmap\">"));
        assert!(tag.contains("\"stripe\":\"https://cdn.example.com/stripe.js\""));
    }
}
