//! Fase 10: internacionalización de rutas.
//!
//! Convención: un locale es un segmento de ruta dinámico llamado
//! exactamente `locale` (`src/pages/[locale]/...`) — la Fase 6 ya soporta
//! segmentos dinámicos; aquí no hace falta ninguna capacidad nueva de
//! enrutado, solo interpretar ese parámetro concreto.
//!
//! Los textos viven en `src/locales/<locale>.json` (JSON anidado, la
//! misma forma que ya usan `data`/`schema`). `t("home.title")` en el JSX
//! se resuelve contra ese diccionario exactamente igual que `data.name`
//! se resuelve contra el JSON del `load()` — ver `nexa-renderer`, que es
//! quien de verdad hace esa resolución; este crate solo carga el
//! diccionario y calcula qué locales existen y sus URLs equivalentes
//! (`hreflang`).

mod discover;
mod hreflang;
mod locale;

#[cfg(test)]
mod tests;

pub use discover::available_locales;
pub use hreflang::{alternate_links, AlternateLink};
pub use locale::load_locale;
