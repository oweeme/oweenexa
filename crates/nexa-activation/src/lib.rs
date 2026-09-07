//! Fase 5: Progressive Activation.
//!
//! Convierte el IR clasificado (`nexa_ir::IrComponent`) en:
//! - un manifiesto de activación (qué nodo interactivo, qué evento, qué
//!   módulo JS, con qué estrategia), y
//! - el contenido de un chunk JS independiente por nodo interactivo.
//!
//! El HTML ya lleva `data-nexa="<id>"` en cada nodo `Interactive` — lo
//! pone `nexa-renderer`, mirando la misma clasificación. Este crate solo
//! produce el resto: el manifiesto y los chunks, para que el runtime del
//! navegador (`packages/runtime`) sepa qué cargar y cuándo.
//!
//! Organización:
//! - [`strategy`]: `Strategy` — cuándo se activa un nodo.
//! - [`manifest`]: la forma serializable del manifiesto.
//! - [`chunk`]: el contenido (todavía un placeholder) de cada chunk JS.
//! - [`build`]: recorre el IR y junta las dos piezas anteriores.

mod build;
mod chunk;
mod content_hash;
mod manifest;
mod strategy;

#[cfg(test)]
mod tests;

pub use build::build;
pub use chunk::Chunk;
pub use manifest::{ActivationEntry, ActivationManifest};
pub use strategy::Strategy;
