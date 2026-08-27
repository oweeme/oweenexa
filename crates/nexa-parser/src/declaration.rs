//! Utilidades compartidas por `loader`, `seo` y `schema`: las tres son
//! `export const <nombre> = <objeto literal>`, con la misma azúcar
//! opcional `defineX({...})` encima.

use oxc_ast::ast::{Argument, BindingPattern, Declaration, Expression, ObjectExpression, Program, Statement};

/// Busca `export const <name> = <expr>` entre las declaraciones de nivel
/// superior y devuelve la expresión de inicialización.
pub(crate) fn find_named_export<'p, 'a>(
    program: &'p Program<'a>,
    name: &str,
) -> Option<&'p Expression<'a>> {
    for stmt in program.body.iter() {
        let Statement::ExportDeclaration(export) = stmt else {
            continue;
        };
        let Declaration::VariableDeclaration(var_decl) = &export.declaration else {
            continue;
        };
        for declarator in var_decl.declarations.iter() {
            if !is_named(&declarator.id, name) {
                continue;
            }
            if let Some(init) = &declarator.init {
                return Some(init);
            }
        }
    }
    None
}

fn is_named(pattern: &BindingPattern<'_>, name: &str) -> bool {
    matches!(pattern, BindingPattern::BindingIdentifier(id) if id.name.as_str() == name)
}

/// `defineSEO({...})` / `defineSchema({...})` son azúcar puramente
/// ergonómica: si la expresión es una llamada con un único argumento
/// objeto, se usa ese objeto. Si ya es el objeto directamente (`export
/// const seo = {...}`), se usa tal cual.
pub(crate) fn unwrap_object_literal<'p, 'a>(
    expr: &'p Expression<'a>,
) -> Option<&'p ObjectExpression<'a>> {
    match expr {
        Expression::ObjectExpression(obj) => Some(obj),
        Expression::CallExpression(call) if call.arguments.len() == 1 => match &call.arguments[0] {
            Argument::ObjectExpression(obj) => Some(obj),
            _ => None,
        },
        _ => None,
    }
}
