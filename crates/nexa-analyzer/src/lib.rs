//! Fase 3: clasifica el árbol de Nexa (`nexa_ast::Component`) en un IR
//! (`nexa_ir::IrComponent`), marcando cada nodo como `Static`, `Dynamic` o
//! `Interactive`, y construyendo el dependency graph.
//!
//! Reglas de clasificación (ver [`classify`]):
//! - Texto → siempre `Static`.
//! - `{expr}` → siempre `Dynamic`, y registra una dependencia sobre el
//!   identificador raíz de `expr` (`product.name` depende de `product`).
//! - Un elemento con al menos un evento (`onClick`) → `Interactive`, y
//!   registra una dependencia sobre el manejador (`buy`).
//! - Un elemento sin eventos → `Static`, independientemente de sus hijos:
//!   la clasificación es propia de cada nodo, no se propaga hacia arriba.

mod classify;

#[cfg(test)]
mod tests;

use nexa_ast::Component;
use nexa_ir::{DependencyGraph, IrComponent};

pub fn analyze(component: &Component) -> IrComponent {
    let mut dependencies = DependencyGraph::new();
    let mut next_id = 0;

    let root = classify::classify_node(&component.root, &mut next_id, &mut dependencies);

    IrComponent {
        name: component.name.clone(),
        root,
        dependencies,
    }
}
