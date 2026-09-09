//! Estructura interna de un componente Nexa (Fase 2: solo HTML estático).
//!
//! Este AST es deliberadamente independiente de `oxc` — `nexa-parser` lo
//! construye a partir del AST de TypeScript/TSX, y `nexa-renderer` lo
//! consume para producir HTML. Ningún otro crate debería depender del AST
//! de `oxc` directamente.

mod component;
mod expr;
mod json_template;
mod loader;
mod node;
mod seo;
mod template;
mod translate;

pub use component::Component;
pub use expr::Expr;
pub use json_template::JsonTemplate;
pub use loader::Loader;
pub use node::{Attr, AttrValue, Element, Event, ForLoop, Island, Node};
pub use seo::SeoConfig;
pub use template::{Template, TemplatePart};
pub use translate::Translate;
