//! Resuelve una `Expr` dinámica contra lo que hay disponible al renderizar
//! una página: el JSON real que devolvió el `load()` (`data.*`, Fase 7),
//! los parámetros de ruta que capturó `nexa-router` (`params.*`, Fase 6),
//! y el diccionario de traducciones del locale actual (`t("...")`, Fase 10).
//!
//! Convención: solo se resuelve si el identificador raíz es exactamente
//! `data` o `params` — cualquier otra referencia (`product`, `count`...)
//! sigue siendo un marcador inerte, igual que antes de esta fase. Nada se
//! evalúa "por si acaso": solo lo que el propio nombre convencional deja
//! inequívoco. `t("...")` es un caso aparte: no es una `Expr` (no tiene
//! forma de identificador/propiedad), así que se resuelve por separado
//! (ver `resolve_translation`).

use std::collections::BTreeMap;

use nexa_ast::{Expr, JsonTemplate, Template, TemplatePart};

/// Todo lo que el renderer tiene disponible para resolver expresiones
/// dinámicas al renderizar una página concreta.
pub struct RenderContext<'a> {
    pub data: Option<&'a serde_json::Value>,
    pub params: &'a BTreeMap<String, String>,
    /// El diccionario del locale actual (`src/locales/<locale>.json`),
    /// ya cargado por `nexa-i18n` — `None` si la página no vive bajo un
    /// segmento `[locale]`.
    pub translations: Option<&'a serde_json::Value>,
}

impl RenderContext<'_> {
    /// Sin `load()`, sin parámetros de ruta y sin traducciones (páginas
    /// estáticas de un solo locale, o sin i18n en absoluto).
    pub fn empty() -> Self {
        static NO_PARAMS: BTreeMap<String, String> = BTreeMap::new();
        RenderContext {
            data: None,
            params: &NO_PARAMS,
            translations: None,
        }
    }
}

pub(crate) fn resolve(expr: &Expr, ctx: &RenderContext) -> Option<String> {
    match expr.root_identifier() {
        "data" => resolve_by_path(ctx.data, &expr.property_path()),
        "params" => resolve_params(expr, ctx.params),
        _ => None,
    }
}

/// Concatena los fragmentos de un `AttrValue::Dynamic`/valor de atributo
/// (Fase 15): si cualquier fragmento no se puede resolver, la plantilla
/// entera se descarta — mismo criterio que ya usa `nexa-seo` para
/// `seo`/`schema`, aplicado aquí a atributos JSX (`href={`/${a}/${b}`}`,
/// antes de esta fase solo se soportaba un único fragmento).
pub(crate) fn resolve_template(template: &Template, ctx: &RenderContext) -> Option<String> {
    let mut out = String::new();
    for part in &template.0 {
        match part {
            TemplatePart::Text(text) => out.push_str(text),
            TemplatePart::Expr(expr) => out.push_str(&resolve(expr, ctx)?),
            TemplatePart::Translate(key) => out.push_str(&resolve_translation(key, ctx)?),
        }
    }
    Some(out)
}

/// `t("home.title")`: la clave se parte por `.` y se camina el JSON del
/// diccionario, igual que `data.price.amount`.
pub(crate) fn resolve_translation(key: &str, ctx: &RenderContext) -> Option<String> {
    let path: Vec<&str> = key.split('.').collect();
    resolve_by_path(ctx.translations, &path)
}

fn resolve_by_path(root: Option<&serde_json::Value>, path: &[&str]) -> Option<String> {
    let mut value = root?;
    for segment in path {
        value = value.get(segment)?;
    }
    Some(json_value_to_text(value))
}

fn resolve_params(expr: &Expr, params: &BTreeMap<String, String>) -> Option<String> {
    // `params` es plano (viene de segmentos de URL): solo `params.slug`,
    // nunca `params.slug.algo`.
    let mut path = expr.property_path().into_iter();
    let key = path.next()?;
    if path.next().is_some() {
        return None;
    }
    params.get(key).cloned()
}

/// Igual que `resolve`, pero conservando la estructura JSON en vez de
/// convertir a texto — lo que necesitan las props de una isla (Fase 16):
/// `data-nexa-props={{ products: data.products }}` debe llevar el array
/// real, no `"[object Object]"`. Mismo criterio que
/// `nexa-seo/src/resolve.rs::resolve_expr` (se duplica aquí en vez de
/// crear una dependencia entre crates, siguiendo el mismo patrón que ya
/// existe con `resolve_template`).
fn resolve_expr_json(expr: &Expr, ctx: &RenderContext) -> Option<serde_json::Value> {
    match expr.root_identifier() {
        "data" => resolve_json_by_path(ctx.data, &expr.property_path()),
        "params" => resolve_params(expr, ctx.params).map(serde_json::Value::String),
        _ => None,
    }
}

fn resolve_translation_json(key: &str, ctx: &RenderContext) -> Option<serde_json::Value> {
    let path: Vec<&str> = key.split('.').collect();
    resolve_json_by_path(ctx.translations, &path)
}

fn resolve_json_by_path(root: Option<&serde_json::Value>, path: &[&str]) -> Option<serde_json::Value> {
    let mut value = root?;
    for segment in path {
        value = value.get(segment)?;
    }
    Some(value.clone())
}

/// Resuelve `data-nexa-props={{...}}` a JSON real. Sin `Option`: a
/// diferencia de un atributo de texto (donde una referencia rota
/// descarta el atributo entero), una posición que no se pudo resolver
/// se convierte en `null` — el resto del objeto de props sigue siendo
/// útil aunque falte una clave.
pub(crate) fn resolve_json_template(template: &JsonTemplate, ctx: &RenderContext) -> serde_json::Value {
    match template {
        JsonTemplate::Null => serde_json::Value::Null,
        JsonTemplate::Bool(b) => serde_json::Value::Bool(*b),
        JsonTemplate::Number(n) => json_number(*n),
        JsonTemplate::String(s) => serde_json::Value::String(s.clone()),
        JsonTemplate::Expr(expr) => resolve_expr_json(expr, ctx).unwrap_or(serde_json::Value::Null),
        JsonTemplate::Translate(key) => resolve_translation_json(key, ctx).unwrap_or(serde_json::Value::Null),
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

/// Un literal numérico escrito en `.tsx` (`id: 1`) llega aquí como `f64`
/// — un entero exacto se serializa como entero, no como `1.0` (bug real,
/// encontrado con datos reales al verificar la Fase 16: mismo criterio
/// que `nexa-seo/src/resolve.rs::json_number`, duplicado aquí por la
/// misma razón que el resto de este archivo).
fn json_number(n: f64) -> serde_json::Value {
    if n.fract() == 0.0 && n.abs() < i64::MAX as f64 {
        serde_json::Value::Number((n as i64).into())
    } else {
        serde_json::Number::from_f64(n).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)
    }
}

fn json_value_to_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}
