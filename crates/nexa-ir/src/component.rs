use crate::{DependencyGraph, IrNode};

/// Un componente ya clasificado, listo para que un renderer o un futuro
/// analyzer (SEO, accesibilidad) lo recorra.
#[derive(Debug, Clone, PartialEq)]
pub struct IrComponent {
    pub name: String,
    pub root: IrNode,
    pub dependencies: DependencyGraph,
}
