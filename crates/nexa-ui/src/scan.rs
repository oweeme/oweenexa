use std::collections::BTreeSet;

use nexa_ast::AttrValue;
use nexa_ir::{IrNode, IrNodeKind};

/// Recoge todos los tokens de `class="..."` estáticos del árbol.
/// `class={expr}` (dinámico) no se puede analizar sin datos reales — se
/// ignora, es una limitación conocida y documentada, no un bug.
pub fn collect_used_classes(root: &IrNode) -> BTreeSet<String> {
    let mut classes = BTreeSet::new();
    walk(root, &mut classes);
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
