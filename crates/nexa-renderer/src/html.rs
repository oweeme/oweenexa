//! Construcción de las etiquetas HTML: atributos, marcador de activación,
//! void elements y escape.

use nexa_ast::AttrValue;
use nexa_ir::{Classification, IrNode, IrNodeKind};

use crate::context::{self, RenderContext};

pub(crate) fn render_element(node: &IrNode, ctx: &RenderContext) -> String {
    let IrNodeKind::Element { tag, attrs, island, children, .. } = &node.kind else {
        unreachable!("render_element solo se llama sobre IrNodeKind::Element")
    };

    let attrs_html: String = attrs
        .iter()
        .filter_map(|a| render_attr(a, ctx))
        .collect();

    // El único rastro de interactividad que el HTML necesita es este id:
    // el manifiesto real (evento, módulo, estrategia) vive en
    // `nexa-activation`, no en el HTML.
    let activation_attr = if node.classification == Classification::Interactive {
        format!(" data-nexa=\"{}\"", node.id)
    } else {
        String::new()
    };

    // A diferencia de la activación de eventos, una isla (Fase 16) no
    // tiene un `IslandManifest` paralelo en Rust: el cliente lee el
    // specifier/props/estrategia directamente del DOM, así que sí quedan
    // en el HTML — es una decisión deliberada, no un descuido (ver
    // `docs/FASES-DE-CONSTRUCCION.md`, Fase 16).
    let island_attrs = island.as_ref().map(|i| render_island_attrs(i, ctx)).unwrap_or_default();

    if is_void_element(tag) {
        return format!("<{tag}{attrs_html}{activation_attr}{island_attrs}>");
    }

    let children_html: String = children.iter().map(|child| crate::render_node(child, ctx)).collect();
    format!("<{tag}{attrs_html}{activation_attr}{island_attrs}>{children_html}</{tag}>")
}

fn render_island_attrs(island: &nexa_ast::Island, ctx: &RenderContext) -> String {
    let mut out = format!(" data-nexa-island=\"{}\"", escape_attr(&island.specifier));

    if let Some(props) = &island.props {
        let value = context::resolve_json_template(props, ctx);
        // Los props se serializan siempre en el mismo request que produce
        // el HTML — nunca se ejecuta nada del otro lado del specifier,
        // solo se le pasa este JSON ya resuelto.
        if let Ok(json) = serde_json::to_string(&value) {
            out.push_str(&format!(" data-nexa-props=\"{}\"", escape_attr(&json)));
        }
    }

    let strategy = island.strategy.as_deref().unwrap_or("visible");
    out.push_str(&format!(" data-nexa-strategy=\"{}\"", escape_attr(strategy)));

    out
}

/// `None` = el atributo no aparece en absoluto: un `src=""` roto (por
/// ejemplo, `{data.image}` sin datos) es peor que ningún `src`.
fn render_attr(attr: &nexa_ast::Attr, ctx: &RenderContext) -> Option<String> {
    match &attr.value {
        None => Some(format!(" {}", attr.name)),
        Some(AttrValue::Static(value)) => Some(format!(" {}=\"{}\"", attr.name, escape_attr(value))),
        Some(AttrValue::Dynamic(template)) => {
            let value = context::resolve_template(template, ctx)?;
            Some(format!(" {}=\"{}\"", attr.name, escape_attr(&value)))
        }
    }
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

pub(crate) fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_html(s).replace('"', "&quot;")
}
