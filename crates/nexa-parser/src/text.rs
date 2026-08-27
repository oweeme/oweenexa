//! Normalización de texto JSX y de expresiones literales simples
//! (`{"texto"}`, `{42}`). Sin evaluación de variables — eso llega con el IR.

use oxc_ast::ast::{JSXExpression, JSXExpressionContainer};

/// Colapsa espacios/indentación de la misma forma que JSX: líneas en blanco
/// se descartan, el resto se une con un único espacio.
pub(crate) fn normalize_jsx_text(raw: &str) -> Option<String> {
    let joined = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

pub(crate) fn expression_container_to_string(container: &JSXExpressionContainer) -> Option<String> {
    match &container.expression {
        JSXExpression::StringLiteral(s) => Some(s.value.as_str().to_string()),
        JSXExpression::NumericLiteral(n) => Some(format_number(n.value)),
        _ => None,
    }
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}
