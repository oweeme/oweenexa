//! Conversión de expresiones oxc (`identifier`, `a.b.c`) a `nexa_ast::Expr`.
//!
//! Solo identificadores y acceso a propiedades por punto. Cualquier otra
//! expresión (llamadas, condicionales, plantillas...) queda fuera de esta
//! fase: el analyzer necesita saber *de qué variable* depende un nodo, no
//! evaluarla.

use oxc_ast::ast::{Expression, JSXExpression};

use nexa_ast::Expr;

pub(crate) fn from_jsx_expression(expr: &JSXExpression) -> Option<Expr> {
    match expr {
        JSXExpression::Identifier(id) => Some(Expr::Identifier(id.name.as_str().to_string())),
        JSXExpression::StaticMemberExpression(member) => Some(Expr::Member {
            object: Box::new(from_expression(&member.object)?),
            property: member.property.name.as_str().to_string(),
        }),
        _ => None,
    }
}

pub(crate) fn from_expression(expr: &Expression) -> Option<Expr> {
    match expr {
        Expression::Identifier(id) => Some(Expr::Identifier(id.name.as_str().to_string())),
        Expression::StaticMemberExpression(member) => Some(Expr::Member {
            object: Box::new(from_expression(&member.object)?),
            property: member.property.name.as_str().to_string(),
        }),
        Expression::ParenthesizedExpression(inner) => from_expression(&inner.expression),
        _ => None,
    }
}
