use std::collections::BTreeMap;

use crate::NodeId;

/// Qué nodos del árbol dependen de qué identificador (`product`, `buy`).
///
/// La Fase 4 (reactividad) usará esto para saber, cuando cambia una señal,
/// qué nodos concretos hay que volver a evaluar — sin recorrer el árbol
/// completo.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DependencyGraph {
    edges: BTreeMap<String, Vec<NodeId>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra que `node_id` depende de `identifier`. Lo llama el
    /// analyzer al clasificar cada nodo — el propio grafo no decide qué
    /// depende de qué.
    pub fn add(&mut self, identifier: impl Into<String>, node_id: NodeId) {
        self.edges.entry(identifier.into()).or_default().push(node_id);
    }

    /// Nodos que dependen de `identifier`.
    pub fn dependents_of(&self, identifier: &str) -> &[NodeId] {
        self.edges.get(identifier).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Todos los identificadores de los que depende algún nodo del árbol.
    pub fn identifiers(&self) -> impl Iterator<Item = &str> {
        self.edges.keys().map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}
