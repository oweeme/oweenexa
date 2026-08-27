use crate::Template;

/// `export const seo = { title: ..., description: ..., ... }`.
///
/// Cada campo es opcional: una página puede declarar solo lo que le
/// importa, y `nexa-seo` decide los valores por defecto / qué omitir.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SeoConfig {
    pub title: Option<Template>,
    pub description: Option<Template>,
    pub canonical: Option<Template>,
    pub og_title: Option<Template>,
    pub og_description: Option<Template>,
    pub og_image: Option<Template>,
    pub twitter_card: Option<Template>,
}
