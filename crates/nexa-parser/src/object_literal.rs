//! Convierte un literal de objeto/array/valor de oxc a
//! `nexa_ast::JsonTemplate` — usado tanto por `seo` como por `schema`
//! (JSON-LD). Cualquier posición puede ser un literal o una referencia
//! simbólica (`data.price`); nunca se ejecuta nada.

use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey};

use nexa_ast::JsonTemplate;

use crate::expr::from_expression;
use crate::template::from_text_expression;
use crate::translate;

pub(crate) fn from_object_expression(obj: &ObjectExpression) -> JsonTemplate {
    let mut entries = Vec::new();

    for prop in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue; // `{...spread}`: no soportado todavía.
        };
        let Some(key) = property_key_name(&prop.key) else {
            continue;
        };
        entries.push((key, from_expression_value(&prop.value)));
    }

    JsonTemplate::Object(entries)
}

fn from_expression_value(expr: &Expression) -> JsonTemplate {
    match expr {
        Expression::StringLiteral(s) => JsonTemplate::String(s.value.as_str().to_string()),
        Expression::NumericLiteral(n) => JsonTemplate::Number(n.value),
        Expression::BooleanLiteral(b) => JsonTemplate::Bool(b.value),
        Expression::NullLiteral(_) => JsonTemplate::Null,
        Expression::ObjectExpression(obj) => from_object_expression(obj),
        Expression::ArrayExpression(arr) => JsonTemplate::Array(
            arr.elements
                .iter()
                .filter_map(|el| match el {
                    ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => None,
                    other => other.as_expression().map(from_expression_value),
                })
                .collect(),
        ),
        Expression::ParenthesizedExpression(inner) => from_expression_value(&inner.expression),
        Expression::TemplateLiteral(_) => from_text_expression(expr)
            .map(JsonTemplate::TextTemplate)
            .unwrap_or(JsonTemplate::Null),
        Expression::CallExpression(_) => translate::from_expression(expr)
            .map(JsonTemplate::Translate)
            .unwrap_or(JsonTemplate::Null),
        other => from_expression(other).map(JsonTemplate::Expr).unwrap_or(JsonTemplate::Null),
    }
}

fn property_key_name(key: &PropertyKey) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str().to_string()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str().to_string()),
        _ => None,
    }
}
