//! Resuelve `Expr`/`Template`/`JsonTemplate` contra los datos reales de
//! una página: el mismo par `data`/`params`/`translations` que ya usa
//! `nexa-renderer` (Fases 7 y 10) para resolver `{data.name}` y
//! `{t("...")}` en el cuerpo del documento.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use nexa_ast::{Expr, JsonTemplate, Template, TemplatePart};

pub struct SeoContext<'a> {
    pub data: Option<&'a serde_json::Value>,
    pub params: &'a BTreeMap<String, String>,
    /// El diccionario del locale actual, si la página vive bajo
    /// `[locale]` — ver `nexa-i18n`.
    pub translations: Option<&'a serde_json::Value>,
}

impl SeoContext<'_> {
    /// Sin `load()`, sin parámetros de ruta y sin traducciones (páginas
    /// estáticas de un solo locale, o sin i18n en absoluto).
    pub fn empty() -> Self {
        static NO_PARAMS: OnceLock<BTreeMap<String, String>> = OnceLock::new();
        SeoContext {
            data: None,
            params: NO_PARAMS.get_or_init(BTreeMap::new),
            translations: None,
        }
    }
}

fn resolve_expr(expr: &Expr, ctx: &SeoContext) -> Option<serde_json::Value> {
    match expr.root_identifier() {
        "data" => resolve_path(ctx.data, &expr.property_path()),
        "params" => {
            let mut path = expr.property_path().into_iter();
            let key = path.next()?;
            if path.next().is_some() {
                return None;
            }
            ctx.params.get(key).cloned().map(serde_json::Value::String)
        }
        _ => None,
    }
}

fn resolve_translation(key: &str, ctx: &SeoContext) -> Option<serde_json::Value> {
    let path: Vec<&str> = key.split('.').collect();
    resolve_path(ctx.translations, &path)
}

fn resolve_path(root: Option<&serde_json::Value>, path: &[&str]) -> Option<serde_json::Value> {
    let mut value = root?;
    for segment in path {
        value = value.get(segment)?;
    }
    Some(value.clone())
}

/// Concatena los fragmentos de una plantilla de texto. Si alguna
/// expresión no se puede resolver, la plantilla entera se descarta — un
/// título a medias es peor que uno ausente (el llamador decide el
/// fallback).
pub(crate) fn resolve_template(template: &Template, ctx: &SeoContext) -> Option<String> {
    let mut out = String::new();
    for part in &template.0 {
        match part {
            TemplatePart::Text(text) => out.push_str(text),
            TemplatePart::Expr(expr) => out.push_str(&json_to_text(&resolve_expr(expr, ctx)?)),
            TemplatePart::Translate(key) => out.push_str(&json_to_text(&resolve_translation(key, ctx)?)),
        }
    }
    Some(out)
}

pub(crate) fn resolve_json_template(template: &JsonTemplate, ctx: &SeoContext) -> serde_json::Value {
    match template {
        JsonTemplate::Null => serde_json::Value::Null,
        JsonTemplate::Bool(b) => serde_json::Value::Bool(*b),
        JsonTemplate::Number(n) => json_number(*n),
        JsonTemplate::String(s) => serde_json::Value::String(s.clone()),
        JsonTemplate::Expr(expr) => resolve_expr(expr, ctx).unwrap_or(serde_json::Value::Null),
        JsonTemplate::Translate(key) => resolve_translation(key, ctx).unwrap_or(serde_json::Value::Null),
        JsonTemplate::TextTemplate(tpl) => resolve_template(tpl, ctx)
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null),
        JsonTemplate::Array(items) => {
            serde_json::Value::Array(items.iter().map(|item| resolve_json_template(item, ctx)).collect())
        }
        JsonTemplate::Object(entries) => serde_json::Value::Object(
            entries.iter().map(|(k, v)| (k.clone(), resolve_json_template(v, ctx))).collect(),
        ),
    }
}

/// Un literal numérico escrito en `.tsx` (`price: 999`) llega aquí como
/// `f64` (Fase 8 — `Expression::NumericLiteral` no distingue entero de
/// decimal). `serde_json::Number::from_f64` conserva eso literalmente:
/// `999.0` en vez de `999` en el JSON-LD final. Un entero exacto se
/// serializa como entero — el `999.0` de un precio en `schema.org` es un
/// defecto real, no un detalle cosmético (algunos validadores de rich
/// results son estrictos con el tipo).
fn json_number(n: f64) -> serde_json::Value {
    if n.fract() == 0.0 && n.abs() < i64::MAX as f64 {
        serde_json::Value::Number((n as i64).into())
    } else {
        serde_json::Number::from_f64(n).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)
    }
}

fn json_to_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}
