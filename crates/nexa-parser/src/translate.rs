//! Reconoce `t("home.title")`: una llamada a un identificador `t` con un
//! único argumento string literal. Cualquier otra forma (variables como
//! clave, varios argumentos, un callee que no sea `t`...) no se
//! reconoce — igual que el resto del parser, esto nunca ejecuta nada,
//! solo detecta un patrón sintáctico exacto.
//!
//! Dos entradas porque `t(...)` puede aparecer tanto en el cuerpo JSX
//! (`{t("...")}`, tipo `JSXExpression`) como en un valor de `seo`/`schema`
//! (`title: t("...")`, tipo `Expression` normal) — son sintácticamente
//! iguales una vez dentro de la llamada, así que comparten `from_call`.

use oxc_ast::ast::{Argument, CallExpression, Expression, JSXExpression};

pub(crate) fn from_expression(expr: &Expression) -> Option<String> {
    match expr {
        Expression::CallExpression(call) => from_call(call),
        _ => None,
    }
}

pub(crate) fn from_jsx_expression(expr: &JSXExpression) -> Option<String> {
    match expr {
        JSXExpression::CallExpression(call) => from_call(call),
        _ => None,
    }
}

fn from_call(call: &CallExpression) -> Option<String> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name.as_str() != "t" {
        return None;
    }
    if call.arguments.len() != 1 {
        return None;
    }
    match &call.arguments[0] {
        Argument::StringLiteral(s) => Some(s.value.as_str().to_string()),
        _ => None,
    }
}
