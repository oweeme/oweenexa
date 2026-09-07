//! Fase 8: SEO y datos estructurados como parte del núcleo, no como un
//! módulo opcional.
//!
//! Consume las declaraciones estáticas que ya extrajo `nexa-parser`
//! (`Component::seo`, `Component::schema`) y las resuelve contra los
//! mismos datos que ya tiene el renderer (`data.*` del `load()`,
//! `params.*` de la ruta) para producir:
//! - las etiquetas de `<head>` (title, description, canonical, Open
//!   Graph, Twitter Card) — [`render_head`];
//! - el `<script type="application/ld+json">` de schema.org —
//!   [`render_schema_script`];
//! - `sitemap.xml` / `robots.txt` para el sitio completo — [`sitemap`] /
//!   [`robots`];
//! - `<link rel="alternate" hreflang="...">` a partir de lo que calcula
//!   `nexa-i18n` — [`hreflang`] (Fase 10);
//! - warnings de build si falta algo (`NEXA-SEO-*`, `NEXA-A11Y-*`) —
//!   [`analyze`].
//!
//! Organización:
//! - [`resolve`]: `Template`/`JsonTemplate` + contexto -> valores reales.
//! - [`head`]: `SeoConfig` resuelto -> HTML de `<head>`.
//! - [`schema`]: `JsonTemplate` resuelto -> `<script type="application/ld+json">`.
//! - [`sitemap`] / [`robots`]: generación a nivel de sitio, no de página.
//! - [`hreflang`]: `nexa_i18n::AlternateLink` -> HTML de `<head>`.
//! - [`analyzer`]: warnings — recorre el mismo IR que ya clasifica
//!   `nexa-analyzer` (Fase 3), sin volver a tocar el parser.

mod analyzer;
mod head;
mod hreflang;
mod layout_head;
mod resolve;
mod robots;
mod schema;
mod sitemap;

#[cfg(test)]
mod tests;

pub use analyzer::{analyze, Warning};
pub use head::render_head;
pub use hreflang::render_hreflang_links;
pub use layout_head::render_layout_head;
pub use resolve::SeoContext;
pub use robots::render_robots;
pub use schema::render_schema_script;
pub use sitemap::render_sitemap;
