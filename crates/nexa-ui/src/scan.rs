use std::collections::BTreeSet;

use nexa_ast::AttrValue;
use nexa_ir::{IrNode, IrNodeKind};

use crate::registry;

/// Recoge los tokens de `class="..."` estáticos del árbol que de verdad
/// pertenecen a una familia de `@nexa/ui` (`nx-btn`, `nx-btn-primary`,
/// `nx-card`, ...) — cualquier otra clase del proyecto (`class="hero"`,
/// `class="site-shell"`) se ignora a propósito. Bug real encontrado
/// construyendo layouts compartidos (Fase 25): antes se recogía
/// cualquier clase sin filtrar, así que una página sin ningún uso real
/// de `@nexa/ui` terminaba igual enlazando `nexa-ui.css` (que
/// `build_stylesheet_from_classes` de todos modos generaba vacío/`None`
/// — el `<link>` quedaba apuntando a un archivo que nunca se escribía) y
/// disparando el aviso `NEXA-PKG-001` en falso.
/// `class={expr}` (dinámico) no se puede analizar sin datos reales — se
/// ignora, es una limitación conocida y documentada, no un bug.
pub fn collect_used_classes(root: &IrNode) -> BTreeSet<String> {
    let mut classes = BTreeSet::new();
    walk(root, &mut classes);
    classes.retain(|class| registry::COMPONENTS.iter().any(|(family, _)| registry::matches_family(family, class)));
    classes
}

fn walk(node: &IrNode, classes: &mut BTreeSet<String>) {
    let IrNodeKind::Element { attrs, children, .. } = &node.kind else {
        return;
    };

    for attr in attrs {
        if attr.name != "class" {
            continue;
        }
        if let Some(AttrValue::Static(value)) = &attr.value {
            classes.extend(value.split_whitespace().map(str::to_string));
        }
    }

    for child in children {
        walk(child, classes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_ast::Attr;
    use nexa_ir::Classification;

    fn element_with_class(class: &str) -> IrNode {
        IrNode {
            id: 0,
            classification: Classification::Static,
            kind: IrNodeKind::Element {
                tag: "div".into(),
                attrs: vec![Attr { name: "class".into(), value: Some(AttrValue::Static(class.into())) }],
                events: vec![],
                island: None,
                children: vec![],
            },
        }
    }

    #[test]
    fn ignores_a_class_that_does_not_belong_to_any_nexa_ui_component() {
        let classes = collect_used_classes(&element_with_class("totally-custom-class site-shell"));
        assert!(classes.is_empty());
    }

    #[test]
    fn keeps_a_real_component_class_mixed_with_unrelated_ones() {
        let classes = collect_used_classes(&element_with_class("hero nx-btn nx-btn-primary"));
        assert_eq!(classes.len(), 2);
        assert!(classes.contains("nx-btn"));
        assert!(classes.contains("nx-btn-primary"));
    }
}
