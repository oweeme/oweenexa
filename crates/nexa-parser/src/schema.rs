//! Extrae `export const schema = { ... }` (o `defineSchema({...})`) de
//! una página — la declaración de JSON-LD / schema.org.

use oxc_ast::ast::Program;

use nexa_ast::JsonTemplate;

use crate::declaration::{find_named_export, unwrap_object_literal};
use crate::object_literal::from_object_expression;

pub(crate) fn find_schema(program: &Program) -> Option<JsonTemplate> {
    let init = find_named_export(program, "schema")?;
    let obj = unwrap_object_literal(init)?;
    Some(from_object_expression(obj))
}
