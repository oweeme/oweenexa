//! Fase 2: TSX → AST de Nexa.
//!
//! No reimplementamos un parser de TypeScript: usamos `oxc_parser` para
//! obtener el AST de TS/TSX y lo recorremos para producir nuestro propio
//! `nexa_ast::Component`. Esta es una decisión deliberada del roadmap
//! (docs/FASES-DE-CONSTRUCCION.md, Fase 2): reescribir un compilador de
//! TypeScript desde cero sería un proyecto en sí mismo.
//!
//! Limitaciones conocidas de esta fase (intencionales):
//! - Solo se soporta un `export default function Nombre() { return (<jsx/>); }`.
//! - Los hijos JSX solo pueden ser elementos intrínsecos (`<div>`, `<h1>`...),
//!   texto, expresiones literales (`{"texto"}`, `{42}`) o referencias
//!   simples (`{product}`, `{product.name}`) — sin evaluarlas todavía, solo
//!   como dato simbólico para el analyzer (Fase 3, ver `nexa-analyzer`).
//! - Eventos (`onClick={buy}`) se reconocen si el manejador es una
//!   referencia simple. No hay props, condicionales ni composición de
//!   componentes todavía — eso llega en fases posteriores (reactividad,
//!   Progressive Activation, router).
//!
//! Organización interna:
//! - [`error`]: el tipo de error público del crate.
//! - [`component`]: encuentra el `export default function` y su JSX raíz.
//! - [`jsx`]: convierte elementos/atributos/eventos/hijos JSX al AST de Nexa.
//! - [`expr`]: convierte expresiones oxc (`a.b.c`) a `nexa_ast::Expr`.
//! - [`text`]: normaliza texto JSX y expresiones literales simples.
//! - [`loader`]: extrae `export const load = { url: "..." }` (Fase 7).
//! - [`handlers`]: extrae el código fuente de los manejadores de eventos
//!   de nivel superior (Fase 5/7, ver `nexa-activation`).
//! - [`seo`] / [`schema`]: extraen `export const seo = {...}` y `export
//!   const schema = {...}` (Fase 8, ver `nexa-seo`).
//! - [`declaration`] / [`object_literal`] / [`template`]: utilidades
//!   compartidas por `loader`/`seo`/`schema` para leer objetos literales
//!   estáticos y plantillas de texto.
//! - [`translate`]: reconoce `t("home.title")` (Fase 10, ver `nexa-i18n`).

mod component;
mod declaration;
mod error;
mod expr;
mod for_loop;
mod handlers;
mod head;
mod jsx;
mod loader;
mod object_literal;
mod schema;
mod seo;
mod template;
mod text;
mod translate;

#[cfg(test)]
mod tests;

pub use error::ParseError;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use nexa_ast::Component;

/// Parsea el contenido de un archivo `.tsx` y devuelve el primer componente
/// encontrado (su `export default function`).
pub fn parse_component(file_name: &str, source: &str) -> Result<Component, ParseError> {
    let allocator = Allocator::default();
    let source_type =
        SourceType::from_path(file_name).map_err(|e| ParseError::Syntax(format!("{e:?}")))?;

    let ret = Parser::new(&allocator, source, source_type).parse();

    if ret.diagnostics.has_errors() {
        let msg = ret
            .diagnostics
            .errors()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(ParseError::Syntax(msg));
    }

    component::find_component(&ret.program, source)
}
