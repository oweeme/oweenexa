use nexa_ast::{Attr, Event, Expr, Island, Translate};

use crate::classification::{Classification, ClassificationCounts};

/// Identificador secuencial de un nodo dentro de un árbol IR. Es lo que el
/// dependency graph usa para señalar "este nodo depende de esta variable",
/// y lo que el renderer usa como valor de `data-nexa="<id>"` cuando el
/// nodo es `Interactive`.
pub type NodeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct IrNode {
    pub id: NodeId,
    pub classification: Classification,
    pub kind: IrNodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrNodeKind {
    Element {
        tag: String,
        /// Se conservan del AST porque el renderer sigue necesitándolos
        /// para producir HTML — el IR no descarta la información de
        /// presentación, solo le añade clasificación.
        attrs: Vec<Attr>,
        events: Vec<Event>,
        /// `Some` cuando el elemento es un punto de montaje de isla
        /// (Fase 16) — se conserva tal cual desde el AST, igual que
        /// `attrs`/`events`.
        island: Option<Island>,
        children: Vec<IrNode>,
    },
    Text(String),
    Expression(Expr),
    /// `t("home.title")` (Fase 10, interpolación en la Fase 45): la
    /// clave y los argumentos de traducción, sin resolver.
    Translate(Translate),
    /// `<For each={data.items}>{(item) => (...)}</For>` (Fase 30): `body`
    /// se clasifica una sola vez (es una plantilla, no N copias) — el
    /// renderer es quien produce una copia de su HTML por cada elemento
    /// real del array, en tiempo de render/build.
    For {
        each: Expr,
        item_name: String,
        body: Box<IrNode>,
    },
    /// `<Show when={...}>` (Fase 62): se decide en tiempo de render/build
    /// con los datos reales; solo la rama elegida llega al HTML.
    Show {
        when: Expr,
        then: Vec<IrNode>,
        otherwise: Vec<IrNode>,
    },
    /// Contenido enriquecido saneado (Fase 68): HTML o Markdown de los datos.
    Rich {
        source: Expr,
        markdown: bool,
    },
}

impl IrNode {
    /// Recorre el nodo y todos sus descendientes en preorden.
    pub fn walk(&self, f: &mut impl FnMut(&IrNode)) {
        f(self);
        match &self.kind {
            IrNodeKind::Element { children, .. } => {
                for child in children {
                    child.walk(f);
                }
            }
            IrNodeKind::For { body, .. } => body.walk(f),
            IrNodeKind::Show { then, otherwise, .. } => {
                for child in then.iter().chain(otherwise) {
                    child.walk(f);
                }
            }
            _ => {}
        }
    }

    /// Cuenta cuántos nodos del árbol caen en cada clasificación.
    pub fn classification_counts(&self) -> ClassificationCounts {
        let mut counts = ClassificationCounts::default();
        self.walk(&mut |node| counts.record(node.classification));
        counts
    }
}
