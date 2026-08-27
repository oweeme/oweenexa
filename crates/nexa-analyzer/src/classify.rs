use nexa_ast::Node;
use nexa_ir::{Classification, DependencyGraph, IrNode, IrNodeKind, NodeId};

pub(crate) fn classify_node(
    node: &Node,
    next_id: &mut NodeId,
    dependencies: &mut DependencyGraph,
) -> IrNode {
    match node {
        Node::Text(text) => IrNode {
            id: alloc_id(next_id),
            classification: Classification::Static,
            kind: IrNodeKind::Text(text.clone()),
        },

        Node::Expression(expr) => {
            let id = alloc_id(next_id);
            dependencies.add(expr.root_identifier(), id);
            IrNode {
                id,
                classification: Classification::Dynamic,
                kind: IrNodeKind::Expression(expr.clone()),
            }
        }

        Node::Translate(key) => {
            let id = alloc_id(next_id);
            dependencies.add("t", id);
            IrNode {
                id,
                classification: Classification::Dynamic,
                kind: IrNodeKind::Translate(key.clone()),
            }
        }

        Node::Element(el) => {
            let id = alloc_id(next_id);

            let children = el
                .children
                .iter()
                .map(|child| classify_node(child, next_id, dependencies))
                .collect();

            for event in &el.events {
                dependencies.add(event.handler.root_identifier(), id);
            }

            // Una isla (Fase 16) es un punto de montaje opaco: el servidor
            // solo renderiza su fallback, y no hay re-render server-driven
            // de sus props tras el mount — por eso no entra al grafo de
            // dependencias, a diferencia de los eventos.
            let classification = if el.island.is_some() {
                Classification::Island
            } else if el.events.is_empty() {
                Classification::Static
            } else {
                Classification::Interactive
            };

            IrNode {
                id,
                classification,
                kind: IrNodeKind::Element {
                    tag: el.tag.clone(),
                    attrs: el.attrs.clone(),
                    events: el.events.clone(),
                    island: el.island.clone(),
                    children,
                },
            }
        }
    }
}

fn alloc_id(next_id: &mut NodeId) -> NodeId {
    let id = *next_id;
    *next_id += 1;
    id
}
