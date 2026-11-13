//! `<SafeHtml html={data.body} />` y `<Markdown text={data.body} />`
//! (Fase 68): contenido enriquecido que viene de la API (el cuerpo de un
//! post, la descripción de un producto), saneado al renderizar.
//!
//! Nexa escapa *todo* lo que viene de datos, así que `"<p>Hola</p>"` se veía
//! como texto literal. Estos dos nodos son la forma explícita y segura de
//! decir "este valor es HTML/Markdown": el compilador lo deja pasar por un
//! sanitizador con lista de etiquetas permitidas (nada de `<script>`,
//! `<iframe>`, `on*`, `javascript:`...).
//!
//! El valor es la misma `Expr` limitada de siempre (`data.x`, `params.x`, el
//! item de un `<For>`, o un prop del componente que se está expandiendo).
//! No admite hijos: es una hoja.

use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElement};

use nexa_ast::{Expr, Node, RichNode};

use crate::compose::{Ctx, PropValue};
use crate::error::ParseError;
use crate::expr::from_jsx_expression;

pub(crate) fn convert_rich(jsx: &JSXElement, ctx: &Ctx, markdown: bool) -> Result<Node, ParseError> {
    let (component, attr_name) = if markdown { ("Markdown", "text") } else { ("SafeHtml", "html") };

    if jsx.children.iter().any(|c| !matches!(c, oxc_ast::ast::JSXChild::Text(t) if t.value.trim().is_empty())) {
        return Err(ParseError::Unsupported(format!("<{component}> no admite hijos: <{component} {attr_name}={{data.campo}} />")));
    }

    let attrs = &jsx.opening_element.attributes;
    if attrs.len() != 1 {
        return Err(ParseError::Unsupported(format!("<{component}> necesita exactamente un atributo: {attr_name}={{data.campo}}")));
    }
    let JSXAttributeItem::Attribute(attr) = &attrs[0] else {
        return Err(ParseError::Unsupported(format!("<{component}> con spread no soportado")));
    };
    let JSXAttributeName::Identifier(name) = &attr.name else {
        return Err(ParseError::Unsupported(format!("<{component}>: nombre de atributo no soportado")));
    };
    if name.name.as_str() != attr_name {
        return Err(ParseError::Unsupported(format!("<{component}> usa `{attr_name}={{...}}`, no `{}`", name.name.as_str())));
    }
    let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
        return Err(ParseError::Unsupported(format!("<{component} {attr_name}=...> debe ser una expresión: {attr_name}={{data.campo}}")));
    };
    let expr = from_jsx_expression(&container.expression).ok_or_else(|| {
        ParseError::Unsupported(format!(
            "<{component} {attr_name}={{...}}> solo admite `data.x`/`params.x` (o el item de un <For>, o un prop) — nunca un literal ni una expresión"
        ))
    })?;

    // Dentro de un componente: el prop que recibe lo que pasó la llamada.
    let source = match ctx.substitute(&expr)? {
        None => expr,
        Some(Some(PropValue::Expr(e))) => e,
        // un prop que la llamada no pasó: no hay nada que mostrar
        Some(None) => Expr::Identifier("__nexa_missing__".to_string()),
        Some(Some(_)) => {
            return Err(ParseError::Unsupported(format!(
                "el prop usado en <{component}> debe recibir `data.x`/`params.x` — un literal no necesita saneado: escríbelo a mano"
            )))
        }
    };

    Ok(Node::Rich(RichNode { source, markdown }))
}
