//! Avisos `NEXA-PKG-*` (Fase 12): una página puede usar `@nexa/ui`
//! (clases `nx-*`) o `@nexa/forms` (`<form data-nexa-form>`) sin haber
//! corrido `nexa add` — hoy eso sigue funcionando igual (son módulos
//! embebidos, cero-config desde las Fases 9 y 10, y esa disciplina no se
//! reabre aquí), pero ahora `nexa build` avisa que `nexa.toml` no refleja
//! la realidad del proyecto. Nunca bloquea el build — mismo espíritu que
//! los avisos del SEO Analyzer desde la Fase 8.

use std::collections::{BTreeMap, BTreeSet};

use crate::manifest::Manifest;

pub struct PackageWarning {
    pub code: &'static str,
    pub message: String,
}

pub fn check(
    manifest: &Manifest,
    ui_used_classes: &BTreeSet<String>,
    has_forms: bool,
    has_platform: bool,
    island_specifiers: &BTreeSet<String>,
    resolved_imports: &BTreeMap<String, String>,
) -> Vec<PackageWarning> {
    let mut warnings = Vec::new();

    if !ui_used_classes.is_empty() && !manifest.has_dependency("ui") {
        warnings.push(PackageWarning {
            code: "NEXA-PKG-001",
            message: "usa clases `nx-*` de @nexa/ui pero no está declarado en nexa.toml — corre \
                      `nexa add ui`."
                .to_string(),
        });
    }

    if has_forms && !manifest.has_dependency("forms") {
        warnings.push(PackageWarning {
            code: "NEXA-PKG-002",
            message: "usa <form data-nexa-form> de @nexa/forms pero no está declarado en \
                      nexa.toml — corre `nexa add forms`."
                .to_string(),
        });
    }

    if has_platform && !manifest.has_dependency("platform") {
        warnings.push(PackageWarning {
            code: "NEXA-PKG-003",
            message: "un handler usa `platform.` (@nexa/platform) pero no está declarado en \
                      nexa.toml — corre `nexa add platform`."
                .to_string(),
        });
    }

    // Fase 16: `data-nexa-island="x"` con `x` ausente de `[imports]` es un
    // typo casi seguro — a diferencia de PKG-001..003 (un módulo oficial
    // que "ya funciona" sin declarar), aquí no hay fallback posible: sin
    // el nombre en el import map, el navegador nunca podrá resolver el
    // specifier en absoluto.
    for specifier in island_specifiers {
        if !resolved_imports.contains_key(specifier) {
            warnings.push(PackageWarning {
                code: "NEXA-PKG-004",
                message: format!(
                    "usa data-nexa-island=\"{specifier}\" pero \"{specifier}\" no está declarado en \
                     nexa.toml [imports] — el navegador no podrá resolverlo."
                ),
            });
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classes(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn warns_when_ui_classes_are_used_without_declaring_the_dependency() {
        let warnings = check(&Manifest::default(), &classes(&["nx-btn"]), false, false, &BTreeSet::new(), &BTreeMap::new());
        assert!(warnings.iter().any(|w| w.code == "NEXA-PKG-001"));
    }

    #[test]
    fn does_not_warn_once_ui_is_declared() {
        let mut manifest = Manifest::default();
        manifest.dependencies.insert("ui".to_string(), "0.1.0".to_string());

        let warnings = check(&manifest, &classes(&["nx-btn"]), false, false, &BTreeSet::new(), &BTreeMap::new());
        assert!(warnings.is_empty());
    }

    #[test]
    fn warns_when_forms_are_used_without_declaring_the_dependency() {
        let warnings = check(&Manifest::default(), &BTreeSet::new(), true, false, &BTreeSet::new(), &BTreeMap::new());
        assert!(warnings.iter().any(|w| w.code == "NEXA-PKG-002"));
    }

    #[test]
    fn warns_when_platform_is_used_without_declaring_the_dependency() {
        let warnings = check(&Manifest::default(), &BTreeSet::new(), false, true, &BTreeSet::new(), &BTreeMap::new());
        assert!(warnings.iter().any(|w| w.code == "NEXA-PKG-003"));
    }

    #[test]
    fn does_not_warn_once_platform_is_declared() {
        let mut manifest = Manifest::default();
        manifest.dependencies.insert("platform".to_string(), "0.1.0".to_string());

        let warnings = check(&manifest, &BTreeSet::new(), false, true, &BTreeSet::new(), &BTreeMap::new());
        assert!(warnings.is_empty());
    }

    #[test]
    fn a_page_using_none_of_them_gets_no_warnings() {
        let warnings = check(&Manifest::default(), &BTreeSet::new(), false, false, &BTreeSet::new(), &BTreeMap::new());
        assert!(warnings.is_empty());
    }

    fn island_names(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn warns_when_an_island_specifier_is_not_declared_in_the_import_map() {
        let islands = island_names(&["dashboardIsland"]);
        let warnings = check(&Manifest::default(), &BTreeSet::new(), false, false, &islands, &BTreeMap::new());

        assert!(warnings.iter().any(|w| w.code == "NEXA-PKG-004" && w.message.contains("dashboardIsland")));
    }

    #[test]
    fn does_not_warn_once_the_island_specifier_is_in_the_resolved_import_map() {
        let islands = island_names(&["dashboardIsland"]);
        let mut resolved = BTreeMap::new();
        resolved.insert("dashboardIsland".to_string(), "/vendor/dashboard-island.js".to_string());

        let warnings = check(&Manifest::default(), &BTreeSet::new(), false, false, &islands, &resolved);
        assert!(warnings.is_empty());
    }
}
