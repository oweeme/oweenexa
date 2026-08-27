use crate::{Expr, Template};

/// Un valor JSON donde cualquier posición puede ser, en vez de un literal,
/// una referencia simbólica (`data.price`) a resolver contra los datos
/// reales de la página. Es la forma que toma `export const schema = {...}`
/// (JSON-LD / schema.org) antes de que `nexa-seo` la resuelva.
///
/// Sigue la misma disciplina que el resto del proyecto: esto no ejecuta
/// nada, solo describe qué copiar y qué resolver.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonTemplate {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    /// `data.price`, `params.slug`... resuelto en tiempo de render.
    Expr(Expr),
    /// `t("product.name")`: la clave de traducción, sin resolver.
    Translate(String),
    /// `` `https://.../${params.slug}` `` — texto y referencias
    /// mezclados, igual que un campo de `seo`. Se resuelve siempre a un
    /// string JSON.
    TextTemplate(Template),
    Array(Vec<JsonTemplate>),
    /// Se preserva el orden de escritura: importa para JSON-LD legible.
    Object(Vec<(String, JsonTemplate)>),
}
