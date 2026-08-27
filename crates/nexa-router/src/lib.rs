//! Fase 6: file-based routing.
//!
//! Convierte la convención de archivos de `src/pages/**/*.tsx` en una
//! tabla de rutas, y resuelve una ruta entrante contra esa tabla. No sabe
//! nada de HTTP ni de renderizado — eso lo usan `nexa-cli` (`nexa build`
//! / `nexa preview`) y, del lado del navegador, `packages/router` (para
//! la navegación tipo SPA).
//!
//! Convención:
//! - `pages/index.tsx` -> `/`
//! - `pages/about.tsx` -> `/about`
//! - `pages/products/index.tsx` -> `/products`
//! - `pages/products/[slug].tsx` -> `/products/:slug`

mod matcher;
mod route;
mod scan;
mod segment;

#[cfg(test)]
mod tests;

pub use matcher::match_route;
pub use route::Route;
pub use scan::scan_pages;
pub use segment::Segment;
