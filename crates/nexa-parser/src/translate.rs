//! Reconoce `t("home.title")` y, desde la Fase 45, `t("clave", { name:
//! data.x })`: una llamada a un identificador `t` con un argumento
//! string literal y, opcionalmente, un segundo argumento objeto cuyos
//! valores son la misma `Expr` limitada que el resto de Nexa acepta en
//! cualquier otro lado (`data.x`, `params.x`) — nunca código arbitrario.
//! Cualquier otra forma (variables como clave, un tercer argumento, un
//! valor de interpolación que no sea una referencia simple, un callee
//! que no sea `t`...) no se reconoce — igual que el resto del parser,
//! esto nunca ejecuta nada, solo detecta un patrón sintáctico exacto.
//!
//! Dos entradas porque `t(...)` puede aparecer tanto en el cuerpo JSX
//! (`{t("...")}`, tipo `JSXExpression`) como en un valor de `seo`/`schema`
//! (`title: t("...")`, tipo `Expression` normal) — son sintácticamente
//! iguales una vez dentro de la llamada, así que comparten `from_call`.

use oxc_ast::ast::{Argument, CallExpression, Expression, JSXExpression, ObjectPropertyKind, PropertyKey};

use nexa_ast::Translate;

pub(crate) fn from_expression(expr: &Expression) -> Option<Translate> {
    match expr {
        Expression::CallExpression(call) => from_call(call),
        _ => None,
    }
}

pub(crate) fn from_jsx_expression(expr: &JSXExpression) -> Option<Translate> {
    match expr {
        JSXExpression::CallExpression(call) => from_call(call),
        _ => None,
    }
}

fn from_call(call: &CallExpression) -> Option<Translate> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name.as_str() != "t" {
        return None;
    }
    if call.arguments.is_empty() || call.arguments.len() > 2 {
        return None;
    }
    let key = match &call.arguments[0] {
        Argument::StringLiteral(s) => s.value.as_str().to_string(),
        _ => return None,
    };
    let args = match call.arguments.get(1) {
        None => Vec::new(),
        Some(Argument::ObjectExpression(obj)) => translate_args(obj)?,
        Some(_) => return None,
    };
    Some(Translate { key, args })
}

/// El segundo argumento de `t(...)`: un objeto literal cuyos valores son
/// referencias simples (`data.x`, `params.x`, o un identificador suelto)
/// — el mismo `Expr` que ya acepta cualquier otra posición dinámica de
/// Nexa. Cualquier valor que no sea eso (un literal, una llamada, un
/// objeto anidado...) hace que el segundo argumento entero no se
/// reconozca, y por lo tanto tampoco el `t(...)` — mismo criterio de
/// "todo o nada" que ya usa `from_call` para el resto del patrón.
fn translate_args(obj: &oxc_ast::ast::ObjectExpression) -> Option<Vec<(String, nexa_ast::Expr)>> {
    let mut entries = Vec::new();
    for prop in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            return None; // `{...spread}`: no soportado.
        };
        let name = match &prop.key {
            PropertyKey::StaticIdentifier(id) => id.name.as_str().to_string(),
            PropertyKey::StringLiteral(s) => s.value.as_str().to_string(),
            _ => return None,
        };
        let value = crate::expr::from_expression(&prop.value)?;
        entries.push((name, value));
    }
    Some(entries)
}
