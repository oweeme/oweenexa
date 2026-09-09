//! Layouts compartidos (Fase 25): `src/layout.tsx`, opcional — si existe,
//! envuelve el HTML de *toda* página del proyecto (header/footer/nav
//! reutilizables), sin reabrir la composición de componentes rechazada
//! desde la Fase 2.
//!
//! **Por qué esto no es composición de componentes:** un layout es un
//! archivo `.tsx` más, compilado con el mismo parser/analyzer/renderer
//! que cualquier página — Rust nunca aprende a pasar props ni a anidar
//! un componente dentro de otro. Lo único nuevo es un splice de texto a
//! nivel de HTML ya renderizado: se busca el único elemento marcado
//! `data-nexa-slot` (debe estar vacío) y se inserta ahí, tal cual, el
//! HTML que ya renderizó la página. Ni el layout ni la página saben el
//! uno del otro en ningún punto anterior a ese splice.
//!
//! **Alcance deliberado de la Fase 25:** el layout no puede tener eventos
//! interactivos propios (`onClick`, etc.) — sus ids de nodo viven en un
//! `IrComponent` separado del de la página, así que mezclarlos en el
//! mismo manifiesto de activación colisionaría (ambos empiezan a contar
//! desde 0). Las clases `nx-*` del layout SÍ se cuentan: se unen a las
//! de la página para decidir qué entra en `nexa-ui.css`.
//!
//! **Islas sí, desde la Fase 31:** a diferencia de un evento, una isla
//! (`data-nexa-island`) no tiene entrada en el manifiesto de
//! activación — el cliente lee specifier/props/estrategia directamente
//! del DOM (Fase 16), así que el problema de ids colisionados nunca
//! aplicó. Lo único que hacía falta era que el pipeline recogiera los
//! specifiers de islas *del layout* además de los de la página, para
//! que el import map en `<head>` y el bootstrap de `nexa-islands.js`
//! los tuvieran en cuenta — ver `island_specifiers` en `pipeline.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use nexa_ir::{Classification, IrNode, IrNodeKind};
use nexa_renderer::RenderContext;

pub const SLOT_ATTR: &str = "data-nexa-slot";

pub fn find() -> Option<PathBuf> {
    let path = Path::new("src/layout.tsx");
    path.is_file().then(|| path.to_path_buf())
}

pub struct RenderedLayout {
    pub html: String,
    pub ui_used_classes: std::collections::BTreeSet<String>,
    /// HTML real de `<head>` (Fase 27): `<link rel="icon">`,
    /// `<link rel="apple-touch-icon">`, `<link rel="stylesheet">` — lo
    /// que `export const head` del layout haya declarado, ya resuelto.
    /// Vacío si el layout no declara `head`. A diferencia de `html`
    /// (que termina dentro de `<body>`, por el splice del slot), esto
    /// el llamador lo mezcla en el `<head>` de verdad del documento.
    pub head_html: String,
    /// `data-nexa-island="..."` que el layout mismo declara (Fase 31) —
    /// el llamador los une a los de la página para el import map y el
    /// bootstrap de islas, exactamente igual que ya hace con
    /// `ui_used_classes`.
    pub island_specifiers: std::collections::BTreeSet<String>,
}

/// Compila `layout_file` (parse -> analyze -> render, igual que una
/// página) y devuelve su HTML con `body` ya insertado en el
/// `data-nexa-slot`, más las clases `nx-*` que el layout mismo usa y el
/// HTML de `<head>` que haya declarado (`export const head`).
pub fn render_for_page(
    layout_file: &Path,
    body: &str,
    params: &BTreeMap<String, String>,
    translations: Option<&serde_json::Value>,
) -> Result<RenderedLayout> {
    let source = fs::read_to_string(layout_file).with_context(|| format!("leyendo {}", layout_file.display()))?;
    let component = nexa_parser::parse_component(layout_file.to_str().unwrap_or("layout.tsx"), &source)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("compilando {}", layout_file.display()))?;

    let ir = nexa_analyzer::analyze(&component);
    validate(&ir.root)?;

    let render_ctx = RenderContext { data: None, params, translations, loop_binding: None };
    let layout_html = nexa_renderer::render_node(&ir.root, &render_ctx);
    let html = splice_slot(&layout_html, body)?;
    let ui_used_classes = nexa_ui::collect_used_classes(&ir.root);

    let seo_ctx = nexa_seo::SeoContext { data: None, params, translations };
    let head_html = nexa_seo::render_layout_head(component.head.as_ref(), &seo_ctx);
    let island_specifiers = crate::pipeline::island_specifiers(&ir.root);

    Ok(RenderedLayout { html, ui_used_classes, head_html, island_specifiers })
}

/// Exactamente un elemento `data-nexa-slot`, vacío, y ningún evento
/// interactivo en todo el layout — las islas sí están permitidas desde
/// la Fase 31, ver el comentario del módulo.
fn validate(root: &IrNode) -> Result<()> {
    let mut slots = 0usize;
    let mut non_empty_slot = false;
    let mut interactive = false;

    root.walk(&mut |node| {
        if node.classification == Classification::Interactive {
            interactive = true;
        }
        if let IrNodeKind::Element { attrs, children, .. } = &node.kind {
            if attrs.iter().any(|a| a.name == SLOT_ATTR) {
                slots += 1;
                if !children.is_empty() {
                    non_empty_slot = true;
                }
            }
        }
    });

    if interactive {
        bail!(
            "src/layout.tsx tiene un evento interactivo (onClick/etc.) — no soportado \
             (los ids de nodo del layout y los de cada página compartirían el mismo manifiesto \
             de activación). Una isla (`data-nexa-island`) sí está permitida — no usa ids de \
             manifiesto."
        );
    }
    if slots != 1 {
        bail!("src/layout.tsx debe tener exactamente un elemento con `{SLOT_ATTR}` (tiene {slots})");
    }
    if non_empty_slot {
        bail!("el elemento `{SLOT_ATTR}` de src/layout.tsx debe estar vacío — ahí se inserta el HTML de cada página");
    }
    Ok(())
}

fn splice_slot(layout_html: &str, body: &str) -> Result<String> {
    let marker_pos = layout_html
        .find(SLOT_ATTR)
        .ok_or_else(|| anyhow::anyhow!("src/layout.tsx no tiene ningún elemento con `{SLOT_ATTR}`"))?;
    let open_end = layout_html[marker_pos..]
        .find('>')
        .map(|i| marker_pos + i + 1)
        .ok_or_else(|| anyhow::anyhow!("no se pudo encontrar el cierre de la etiqueta con `{SLOT_ATTR}`"))?;

    Ok(format!("{}{}{}", &layout_html[..open_end], body, &layout_html[open_end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splice_slot_inserts_the_body_inside_the_marked_element() {
        let layout = r#"<html><body><header>Nav</header><div data-nexa-slot></div><footer>Pie</footer></body></html>"#;
        let out = splice_slot(layout, "<p>hola</p>").unwrap();
        assert!(out.contains("<div data-nexa-slot><p>hola</p></div>"));
    }

    #[test]
    fn splice_slot_fails_with_a_clear_message_when_there_is_no_slot() {
        let layout = "<html><body><header>Nav</header></body></html>";
        let err = splice_slot(layout, "<p>hola</p>").unwrap_err();
        assert!(err.to_string().contains("data-nexa-slot"));
    }

    fn ir_root_for(source: &str) -> IrNode {
        let component = nexa_parser::parse_component("layout.tsx", source).expect("should parse");
        nexa_analyzer::analyze(&component).root
    }

    #[test]
    fn validate_accepts_an_island_inside_the_layout() {
        // Fase 31: a diferencia de un `onClick`, una isla no tiene
        // entrada en el manifiesto de activación — no hay ids que
        // puedan colisionar con los de la página.
        let source = r#"
export default function Layout() {
    return (
        <div class="shell">
            <div data-nexa-island="siteHeader" data-nexa-strategy="load">
                <header>Nav estático</header>
            </div>
            <div data-nexa-slot></div>
        </div>
    );
}
"#;
        assert!(validate(&ir_root_for(source)).is_ok());
    }

    #[test]
    fn validate_still_rejects_a_direct_onclick_in_the_layout() {
        let source = r#"
function toggle() {}

export default function Layout() {
    return (
        <div class="shell">
            <button onClick={toggle}>Menu</button>
            <div data-nexa-slot></div>
        </div>
    );
}
"#;
        let err = validate(&ir_root_for(source)).unwrap_err();
        assert!(err.to_string().contains("evento interactivo"));
    }

    #[test]
    fn validate_still_rejects_an_onclick_inside_an_islands_fallback() {
        // El fallback de una isla se renderiza server-side como
        // cualquier otro nodo — un `onClick` ahí sí tendría el mismo
        // problema de ids colisionados que en cualquier otro lugar del
        // layout, así que sigue sin estar permitido.
        let source = r#"
function toggle() {}

export default function Layout() {
    return (
        <div class="shell">
            <div data-nexa-island="siteHeader">
                <button onClick={toggle}>Menu</button>
            </div>
            <div data-nexa-slot></div>
        </div>
    );
}
"#;
        assert!(validate(&ir_root_for(source)).is_err());
    }
}
