//! Convierte un valor de texto oxc (literal o plantilla) a
//! `nexa_ast::Template`: una secuencia de texto fijo + referencias
//! simbólicas (`data.name`, `params.slug`) o de traducción (`t("...")`).
//! Dos entradas, una para cada forma en que oxc representa "un valor de
//! texto": `from_text_expression` para un `Expression` normal (campos de
//! `seo`/`schema`), y `from_jsx_text_expression` para un `JSXExpression`
//! (atributos JSX dinámicos, `href={...}` — Fase 15; antes de esa fase
//! solo se soportaba una única referencia simple ahí, no una plantilla
//! con varias partes, así que `href={\`/${a}/${b}\`}` se perdía en
//! silencio: un bug real encontrado construyendo el proyecto de
//! referencia de esa fase).

use oxc_ast::ast::{Expression, JSXExpression, TemplateLiteral};

use nexa_ast::{Template, TemplatePart};

use crate::expr::{from_expression, from_jsx_expression};
use crate::translate;

pub(crate) fn from_text_expression(expr: &Expression) -> Option<Template> {
    match expr {
        Expression::StringLiteral(s) => Some(Template::literal(s.value.as_str())),
        Expression::TemplateLiteral(tpl) => Some(Template(template_literal_to_parts(tpl))),
        Expression::ParenthesizedExpression(inner) => from_text_expression(&inner.expression),
        // `title: data.name` o `title: t("home.title")` (sin plantilla):
        // se tratan como un único fragmento, igual que si estuvieran
        // dentro de `` `${...}` ``.
        other => interpolation_part(other).map(|part| Template(vec![part])),
    }
}

pub(crate) fn from_jsx_text_expression(expr: &JSXExpression) -> Option<Template> {
    match expr {
        JSXExpression::StringLiteral(s) => Some(Template::literal(s.value.as_str())),
        // El literal de plantilla en sí (`quasis`/`expressions`) es
        // exactamente el mismo tipo oxc que en un `Expression` suelto —
        // solo cambia cómo se llegó hasta aquí (JSX vs. un campo de
        // `seo`), así que comparte `template_literal_to_parts`. Lo que
        // va dentro de un `${...}` nunca es un `JSXExpression`, sea cual
        // sea el contenedor exterior — por eso ese helper sigue
        // operando sobre `Expression`.
        JSXExpression::TemplateLiteral(tpl) => Some(Template(template_literal_to_parts(tpl))),
        JSXExpression::ParenthesizedExpression(inner) => from_text_expression(&inner.expression),
        other => jsx_interpolation_part(other).map(|part| Template(vec![part])),
    }
}

fn template_literal_to_parts(tpl: &TemplateLiteral) -> Vec<TemplatePart> {
    let mut parts = Vec::new();
    let mut expr_iter = tpl.expressions.iter();

    for quasi in tpl.quasis.iter() {
        let text = quasi.value.cooked.as_ref().map(|s| s.as_str()).unwrap_or("");
        if !text.is_empty() {
            parts.push(TemplatePart::Text(text.to_string()));
        }

        if !quasi.tail {
            if let Some(expression) = expr_iter.next() {
                if let Some(part) = interpolation_part(expression) {
                    parts.push(part);
                }
            }
        }
    }

    parts
}

fn interpolation_part(expr: &Expression) -> Option<TemplatePart> {
    if let Some(translate) = translate::from_expression(expr) {
        return Some(TemplatePart::Translate(translate));
    }
    from_expression(expr).map(TemplatePart::Expr)
}

fn jsx_interpolation_part(expr: &JSXExpression) -> Option<TemplatePart> {
    if let Some(translate) = translate::from_jsx_expression(expr) {
        return Some(TemplatePart::Translate(translate));
    }
    from_jsx_expression(expr).map(TemplatePart::Expr)
}
