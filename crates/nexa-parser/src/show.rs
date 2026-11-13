//! `<Show when={data.stock} fallback={<p>Agotado</p>}>...</Show>`
//! (Fase 62): un condicional que resuelve el compilador contra los datos
//! reales de la página — no un `if` ni una expresión arbitraria. Igual
//! que `<For>`, es un patrón sintáctico reconocido por nombre exacto.
//!
//! `when` es la misma `Expr` limitada de siempre (`data.x`, `params.x`,
//! el item de un `<For>`, o —dentro de un componente— un prop que recibe
//! una de esas). `fallback` es un único elemento JSX opcional.

use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElement, JSXExpression};

use nexa_ast::{Expr, Node, ShowNode};

use crate::compose::{Ctx, PropValue};
use crate::error::ParseError;
use crate::expr::from_jsx_expression;
use crate::jsx::{convert_child, convert_element};

pub(crate) fn convert_show(jsx: &JSXElement, ctx: &Ctx) -> Result<Node, ParseError> {
    let mut when: Option<Expr> = None;
    let mut otherwise: Vec<Node> = Vec::new();

    for item in jsx.opening_element.attributes.iter() {
        let JSXAttributeItem::Attribute(attr) = item else {
            return Err(ParseError::Unsupported("<Show> con spread attribute no soportado".to_string()));
        };
        let JSXAttributeName::Identifier(name) = &attr.name else {
            return Err(ParseError::Unsupported("<Show> con nombre de atributo no soportado".to_string()));
        };
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return Err(ParseError::Unsupported(format!("<Show {}=...> debe ser una expresión: {{...}}", name.name.as_str())));
        };

        match name.name.as_str() {
            "when" => {
                let expr = from_jsx_expression(&container.expression).ok_or_else(|| {
                    ParseError::Unsupported(
                        "<Show when={...}> solo admite `data.x`/`params.x` (o el item de un <For>) — nada de comparaciones ni expresiones; calculá el valor en el backend".to_string(),
                    )
                })?;
                when = Some(match ctx.substitute(&expr)? {
                    None => expr,
                    Some(Some(PropValue::Expr(e))) => e,
                    // un prop que la llamada no pasó: siempre falso
                    Some(None) => Expr::Identifier("__nexa_missing__".to_string()),
                    Some(Some(_)) => {
                        return Err(ParseError::Unsupported(
                            "el prop usado en `when` debe recibir `data.x`/`params.x`".to_string(),
                        ))
                    }
                });
            }
            "fallback" => {
                otherwise = match &container.expression {
                    JSXExpression::JSXElement(el) => vec![convert_element(el, ctx)?],
                    _ => {
                        return Err(ParseError::Unsupported(
                            "<Show fallback={...}> debe ser un elemento JSX: fallback={<p>...</p>}".to_string(),
                        ))
                    }
                };
            }
            other => {
                return Err(ParseError::Unsupported(format!("<Show> solo admite `when` y `fallback`, no `{other}`")));
            }
        }
    }

    let when = when.ok_or_else(|| ParseError::Unsupported("<Show> necesita `when={...}`".to_string()))?;

    let mut then = Vec::new();
    for child in jsx.children.iter() {
        then.extend(convert_child(child, ctx)?);
    }

    Ok(Node::Show(ShowNode { when, then, otherwise }))
}
