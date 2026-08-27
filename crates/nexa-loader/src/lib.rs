//! Fase 7: ejecuta el `load` declarado por una página.
//!
//! No hay ningún motor de JavaScript aquí: `nexa-parser` ya extrajo
//! `load` como una plantilla de URL estática (`nexa_ast::Loader`). Este
//! crate solo (1) sustituye los parámetros de ruta en esa plantilla, y
//! (2) hace la petición HTTP real contra el backend — el "backend de
//! negocio" al que se refiere toda la especificación de Nexa (PHP, Go,
//! Python, Rust...). Nexa nunca ejecuta el JS del desarrollador para
//! obtener datos; solo sabe pedirlos por HTTP.

mod error;
mod url;

pub use error::LoaderError;
pub use url::resolve_url;

use std::collections::BTreeMap;

use nexa_ast::Loader;

/// Resuelve la URL de `loader` (sustituyendo `:params`) y hace un GET
/// real contra `api_base` + esa ruta. Devuelve el JSON ya parseado, o un
/// error que distingue "no existe" (404) del resto de fallos — es lo que
/// permite a `nexa-cli` responder con un 404 real en vez de un 200 vacío.
pub fn load(
    loader: &Loader,
    params: &BTreeMap<String, String>,
    api_base: &str,
) -> Result<serde_json::Value, LoaderError> {
    let path = resolve_url(&loader.url_template, params)?;
    let full_url = format!("{api_base}{path}");

    ureq::get(&full_url)
        .call()
        .map_err(classify_error)?
        .body_mut()
        .read_json::<serde_json::Value>()
        .map_err(|e| LoaderError::InvalidJson(e.to_string()))
}

fn classify_error(err: ureq::Error) -> LoaderError {
    match err {
        ureq::Error::StatusCode(404) => LoaderError::NotFound,
        ureq::Error::StatusCode(code) => LoaderError::Http(code),
        other => LoaderError::Network(other.to_string()),
    }
}
