//! `@nexa/devtools` (Fase 14): el lado del servidor — construye el JSON
//! de diagnóstico que sirve `/__nexa_dev__/diagnostics?path=<ruta>`
//! (solo en `nexa dev`, nunca en `build`/`preview`). El panel que lo
//! consume vive en `packages/devtools`.

use nexa_activation::ActivationManifest;
use nexa_seo::Warning;

use crate::pipeline::CompiledPage;
use crate::pkg_warnings::PackageWarning;

pub fn diagnostics_json(pattern: &str, page: &CompiledPage) -> String {
    let counts = page.ir.root.classification_counts();

    let payload = serde_json::json!({
        "pattern": pattern,
        "classification": {
            "static": counts.static_count,
            "dynamic": counts.dynamic_count,
            "interactive": counts.interactive_count,
            "async": counts.async_count,
            "total": counts.total(),
        },
        "initialJsBytes": page.initial_js_bytes,
        "activation": activation_entries(&page.manifest),
        "seoWarnings": seo_warnings(&page.seo_warnings),
        "pkgWarnings": pkg_warnings(&page.pkg_warnings),
    });

    // `unwrap_or_else`, no `expect`: los valores de arriba son todos
    // tipos propios ya validados (nunca un `f64::NAN`/ciclo/lo que sea
    // que haría fallar la serialización) — pero un panel de diagnóstico
    // nunca debe poder tumbar `nexa dev` por un error de serialización
    // inesperado, así que el `{}` de respaldo es deliberado.
    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn activation_entries(manifest: &ActivationManifest) -> serde_json::Value {
    let entries: Vec<serde_json::Value> = manifest
        .entries()
        .map(|(id, entry)| {
            serde_json::json!({
                "id": id,
                "event": entry.event,
                "handler": entry.handler,
                "module": entry.module,
                "strategy": strategy_name(entry.strategy),
            })
        })
        .collect();
    serde_json::Value::Array(entries)
}

fn strategy_name(strategy: nexa_activation::Strategy) -> &'static str {
    use nexa_activation::Strategy;
    match strategy {
        Strategy::Interaction => "interaction",
        Strategy::Visible => "visible",
        Strategy::Idle => "idle",
        Strategy::Load => "load",
        Strategy::Manual => "manual",
    }
}

fn seo_warnings(warnings: &[Warning]) -> serde_json::Value {
    serde_json::Value::Array(
        warnings.iter().map(|w| serde_json::json!({ "code": w.code, "message": w.message })).collect(),
    )
}

fn pkg_warnings(warnings: &[PackageWarning]) -> serde_json::Value {
    serde_json::Value::Array(
        warnings.iter().map(|w| serde_json::json!({ "code": w.code, "message": w.message })).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_activation::{ActivationEntry, Strategy};
    use nexa_ir::{Classification, IrComponent, IrNode, IrNodeKind};
    use std::collections::BTreeSet;

    fn empty_page() -> CompiledPage {
        let mut manifest = ActivationManifest::new();
        manifest.insert(
            3,
            ActivationEntry {
                event: "click".into(),
                handler: "buy".into(),
                module: "/assets/Home-3.js".into(),
                strategy: Strategy::Interaction,
            },
        );

        CompiledPage {
            html: String::new(),
            manifest,
            chunks: vec![],
            ir: IrComponent {
                name: "Home".into(),
                root: IrNode { id: 0, classification: Classification::Static, kind: IrNodeKind::Text("x".into()) },
                dependencies: nexa_ir::DependencyGraph::new(),
            },
            seo_warnings: vec![Warning { code: "NEXA-SEO-001", message: "falta title".into() }],
            ui_used_classes: BTreeSet::new(),
            pkg_warnings: vec![],
            initial_js_bytes: 1234,
        }
    }

    #[test]
    fn produces_valid_json_with_the_expected_top_level_fields() {
        let page = empty_page();
        let json = diagnostics_json("/", &page);
        let value: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");

        assert_eq!(value["pattern"], "/");
        assert_eq!(value["initialJsBytes"], 1234);
        assert_eq!(value["classification"]["static"], 1);
    }

    #[test]
    fn lists_activation_entries_with_their_strategy_name() {
        let page = empty_page();
        let json = diagnostics_json("/", &page);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        let entries = value["activation"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["handler"], "buy");
        assert_eq!(entries[0]["strategy"], "interaction");
    }

    #[test]
    fn lists_seo_warnings() {
        let page = empty_page();
        let json = diagnostics_json("/", &page);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        let warnings = value["seoWarnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0]["code"], "NEXA-SEO-001");
    }
}
