//! Fase 3: representación intermedia (IR) de Nexa.
//!
//! El IR es deliberadamente independiente del HTML — a diferencia de
//! `nexa_ast`, que describe *lo que el desarrollador escribió*, el IR
//! describe *cómo debe tratarse cada nodo*: si es fijo, si depende de
//! datos, o si necesita JavaScript en el navegador.
//!
//! No lo produce nadie más que `nexa-analyzer`. Ver
//! `docs/FASES-DE-CONSTRUCCION.md` (Fase 3) para el criterio de salida.
//!
//! Punto de extensión para fases futuras: el SEO Analyzer (Fase 8) y el
//! Accessibility Analyzer (Fase 9) recorrerán este mismo `IrNode` (con
//! [`IrNode::walk`]) para emitir warnings, sin volver a tocar el parser.

mod classification;
mod component;
mod dependency;
mod node;

pub use classification::{Classification, ClassificationCounts};
pub use component::IrComponent;
pub use dependency::DependencyGraph;
pub use node::{IrNode, IrNodeKind, NodeId};
