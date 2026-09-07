//! Extrae `export const head = { ... }` (Fase 27) — solo tiene efecto en
//! `src/layout.tsx`: favicon y stylesheets que el layout declara para el
//! `<head>` real del documento. Mismo patrón que `schema`: un objeto
//! literal, nunca JS ejecutado.

use oxc_ast::ast::Program;

use nexa_ast::JsonTemplate;

use crate::declaration::{find_named_export, unwrap_object_literal};
use crate::object_literal::from_object_expression;

pub(crate) fn find_head(program: &Program) -> Option<JsonTemplate> {
    let init = find_named_export(program, "head")?;
    let obj = unwrap_object_literal(init)?;
    Some(from_object_expression(obj))
}
