//! `<For each={data.items}>{(item) => (...)}</For>` (Fase 30): iteración
//! literal y segura, reconocida como un patrón sintáctico fijo — nunca un
//! `.map()` ni código arbitrario del desarrollador. `each` solo admite
//! `data.x`/`params.x` (mismo `Expr` limitado de siempre); el callback debe
//! ser un arrow function de cuerpo conciso con un único parámetro sin
//! destructuring, cuyo cuerpo es un único elemento JSX.

use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Expression, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue, JSXChild, JSXElement, JSXExpression,
};

use nexa_ast::{Expr, ForLoop, Node};

use crate::error::ParseError;
use crate::expr::from_jsx_expression;
use crate::jsx::convert_element;
use crate::text::normalize_jsx_text;

pub(crate) fn convert_for(jsx: &JSXElement) -> Result<Node, ParseError> {
    let each = find_each_attribute(jsx)?;

    let mut real_children = jsx.children.iter().filter(|child| !is_insignificant_whitespace(child));

    let only_child = real_children
        .next()
        .ok_or_else(|| ParseError::Unsupported("<For> necesita un hijo: {(item) => (...)}".to_string()))?;
    if real_children.next().is_some() {
        return Err(ParseError::Unsupported(
            "<For> admite un único hijo — solo {(item) => (...)}, sin hermanos".to_string(),
        ));
    }

    let JSXChild::ExpressionContainer(container) = only_child else {
        return Err(ParseError::Unsupported(
            "el hijo de <For> debe ser {(item) => (...)}".to_string(),
        ));
    };

    let JSXExpression::ArrowFunctionExpression(arrow) = &container.expression else {
        return Err(ParseError::Unsupported(
            "el hijo de <For> debe ser un arrow function: (item) => (...)".to_string(),
        ));
    };

    let item_name = single_identifier_param(arrow)?;

    let body_expr = arrow.get_expression().ok_or_else(|| {
        ParseError::Unsupported(
            "el callback de <For> debe tener cuerpo conciso: (item) => (<jsx/>) — sin `{ return ... }`"
                .to_string(),
        )
    })?;

    let jsx_body = unwrap_parens_to_jsx(body_expr)?;
    let body = convert_element(jsx_body)?;

    Ok(Node::For(ForLoop { each, item_name, body: Box::new(body) }))
}

/// `each={data.items}`: el único atributo que `<For>` admite — mismo `Expr`
/// limitado que el resto de Nexa (`data.x`/`params.x`), nunca una
/// expresión arbitraria ni un identificador suelto sin raíz reconocida.
fn find_each_attribute(jsx: &JSXElement) -> Result<Expr, ParseError> {
    let attrs = &jsx.opening_element.attributes;
    if attrs.len() != 1 {
        return Err(ParseError::Unsupported(
            "<For> solo admite el atributo `each` (nada de clases, estrategia, etc.)".to_string(),
        ));
    }

    let JSXAttributeItem::Attribute(attr) = &attrs[0] else {
        return Err(ParseError::Unsupported("<For> con spread attribute no soportado".to_string()));
    };
    let JSXAttributeName::Identifier(id) = &attr.name else {
        return Err(ParseError::Unsupported("<For> con nombre de atributo no soportado".to_string()));
    };
    if id.name.as_str() != "each" {
        return Err(ParseError::Unsupported(format!(
            "<For> solo admite el atributo `each`, no `{}`",
            id.name.as_str()
        )));
    }

    let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
        return Err(ParseError::Unsupported(
            "each={...} debe ser una expresión, ej. each={data.items}".to_string(),
        ));
    };
    let expr = from_jsx_expression(&container.expression).ok_or_else(|| {
        ParseError::Unsupported("each debe ser `data.x` o `params.x` — nada de expresiones arbitrarias".to_string())
    })?;
    if !matches!(expr.root_identifier(), "data" | "params") {
        return Err(ParseError::Unsupported(
            "each debe tener `data` o `params` como raíz, ej. `data.items`".to_string(),
        ));
    }
    Ok(expr)
}

/// `(item) => ...`: exactamente un parámetro, sin destructuring ni default.
fn single_identifier_param(arrow: &ArrowFunctionExpression) -> Result<String, ParseError> {
    if arrow.params.items.len() != 1 || arrow.params.rest.is_some() {
        return Err(ParseError::Unsupported(
            "el callback de <For> necesita exactamente un parámetro: (item) => (...)".to_string(),
        ));
    }
    let param = &arrow.params.items[0];
    if param.initializer.is_some() {
        return Err(ParseError::Unsupported(
            "el parámetro de <For> no admite un valor por defecto".to_string(),
        ));
    }
    match &param.pattern {
        BindingPattern::BindingIdentifier(id) => Ok(id.name.as_str().to_string()),
        _ => Err(ParseError::Unsupported(
            "el parámetro de <For> no admite destructuring — usá (item) => ... y accedé a item.x adentro"
                .to_string(),
        )),
    }
}

fn unwrap_parens_to_jsx<'a, 'b>(expr: &'b Expression<'a>) -> Result<&'b JSXElement<'a>, ParseError> {
    match expr {
        Expression::ParenthesizedExpression(inner) => unwrap_parens_to_jsx(&inner.expression),
        Expression::JSXElement(jsx) => Ok(jsx),
        _ => Err(ParseError::Unsupported(
            "el callback de <For> debe devolver un único elemento JSX".to_string(),
        )),
    }
}

fn is_insignificant_whitespace(child: &JSXChild) -> bool {
    matches!(child, JSXChild::Text(text) if normalize_jsx_text(text.value.as_str()).is_none())
}
