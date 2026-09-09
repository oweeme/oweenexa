//! Construcción de las etiquetas HTML: atributos, marcador de activación,
//! void elements y escape.

use nexa_ast::AttrValue;
use nexa_ir::{Classification, IrNode, IrNodeKind};

use crate::context::{self, RenderContext};

pub(crate) fn render_element(node: &IrNode, ctx: &RenderContext) -> String {
    let IrNodeKind::Element { tag, attrs, island, children, .. } = &node.kind else {
        unreachable!("render_element solo se llama sobre IrNodeKind::Element")
    };

    // `data-nexa-match` (Fase 33) SÍ queda en el HTML final — a
    // diferencia de `data-nexa-strategy` para eventos, el cliente
    // necesita poder leerlo: `packages/router` recalcula qué link está
    // activo después de cada navegación SPA (Fase 32 dejó el layout sin
    // volver a renderizarse en el servidor en cada click, así que el
    // `aria-current` calculado acá al build/request quedaría obsoleto
    // sin ese recálculo del lado del cliente).
    let is_link = tag == "a";
    let attrs_html: String = attrs.iter().filter_map(|a| render_attr(a, ctx)).collect();

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

    // Link activo (Fase 33): resuelto igual que `seo`/`schema` — Rust ya
    // sabe qué ruta está renderizando, así que puede decidir esto sin
    // que el desarrollador escriba ningún condicional (Nexa no los
    // admite en el cuerpo de una página/layout).
    let active_link_attr = if is_link { active_link_attr(attrs, ctx) } else { String::new() };

    if is_void_element(tag) {
        return format!("<{tag}{attrs_html}{activation_attr}{island_attrs}{active_link_attr}>");
    }

    let children_html: String = children.iter().map(|child| crate::render_node(child, ctx)).collect();
    format!("<{tag}{attrs_html}{activation_attr}{island_attrs}{active_link_attr}>{children_html}</{tag}>")
}

/// `aria-current="page"` si el `href` de este `<a>` coincide con la ruta
/// que se está renderizando. Coincidencia exacta por defecto;
/// `data-nexa-match="prefix"` la vuelve por prefijo (con límite de
/// segmento: `/dashboard` no matchea `/dashboard-old`, sí matchea
/// `/dashboard/settings`). `href="/"` en modo prefijo solo matchea `/`
/// exacto — nunca "toda ruta empieza con /", el mismo límite de
/// segmento lo evita sin necesitar un caso especial.
fn active_link_attr(attrs: &[nexa_ast::Attr], ctx: &RenderContext) -> String {
    let Some(current_path) = ctx.current_path else {
        return String::new();
    };
    let Some(href) = attrs.iter().find(|a| a.name == "href").and_then(|a| resolve_attr_string(a, ctx)) else {
        return String::new();
    };
    let match_mode = attrs.iter().find(|a| a.name == "data-nexa-match").and_then(|a| resolve_attr_string(a, ctx));

    let is_active = match match_mode.as_deref() {
        Some("prefix") => is_prefix_match(&href, current_path),
        _ => href == current_path,
    };

    if is_active {
        " aria-current=\"page\"".to_string()
    } else {
        String::new()
    }
}

fn is_prefix_match(href: &str, current_path: &str) -> bool {
    current_path == href || (current_path.starts_with(href) && current_path[href.len()..].starts_with('/'))
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
    if attr.value.is_none() {
        return Some(format!(" {}", attr.name));
    }
    let value = resolve_attr_string(attr, ctx)?;
    Some(format!(" {}=\"{}\"", attr.name, escape_attr(&value)))
}

/// El valor real de un atributo, sin el HTML alrededor — lo que
/// `render_attr` necesita para armar `nombre="valor"`, y lo que el link
/// activo (Fase 33) necesita para comparar `href` contra la ruta actual
/// sin duplicar la resolución de `AttrValue::Dynamic`/`Static`.
fn resolve_attr_string(attr: &nexa_ast::Attr, ctx: &RenderContext) -> Option<String> {
    match &attr.value {
        None => None,
        Some(AttrValue::Static(value)) => Some(value.clone()),
        Some(AttrValue::Dynamic(template)) => context::resolve_template(template, ctx),
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
