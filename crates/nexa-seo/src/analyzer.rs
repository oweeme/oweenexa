use nexa_ast::SeoConfig;
use nexa_ir::{IrNode, IrNodeKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub code: &'static str,
    pub message: String,
}

/// Recorre el mismo IR que ya clasifica `nexa-analyzer` (Fase 3) y avisa
/// de lo que falta — nunca rompe el build, solo informa. Punto de
/// extensión explícito desde la Fase 3: este es exactamente el "SEO
/// Analyzer" que se dejó anotado como pendiente en aquel momento.
pub fn analyze(root: &IrNode, seo: Option<&SeoConfig>) -> Vec<Warning> {
    let mut warnings = Vec::new();
    check_seo_config(seo, &mut warnings);
    check_tree(root, &mut warnings);
    warnings
}

fn check_seo_config(seo: Option<&SeoConfig>, warnings: &mut Vec<Warning>) {
    let has = |get: fn(&SeoConfig) -> bool| seo.is_some_and(get);

    if !has(|s| s.title.is_some()) {
        warnings.push(Warning {
            code: "NEXA-SEO-001",
            message: "falta `title` en `seo` — Google suele ignorar páginas sin título propio"
                .to_string(),
        });
    }
    if !has(|s| s.description.is_some()) {
        warnings.push(Warning {
            code: "NEXA-SEO-002",
            message: "falta `description` en `seo`".to_string(),
        });
    }
    if !has(|s| s.canonical.is_some()) {
        warnings.push(Warning {
            code: "NEXA-SEO-003",
            message: "falta `canonical` en `seo`".to_string(),
        });
    }
}

fn check_tree(node: &IrNode, warnings: &mut Vec<Warning>) {
    let IrNodeKind::Element { tag, attrs, children, .. } = &node.kind else {
        return;
    };

    if tag == "img" && !attrs.iter().any(|a| a.name == "alt") {
        warnings.push(Warning {
            code: "NEXA-A11Y-001",
            message: format!("<img> sin `alt` (nodo #{})", node.id),
        });
    }

    for child in children {
        check_tree(child, warnings);
    }
}
