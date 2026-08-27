/// La declaración de datos de una página: `export const load = { url:
/// "/api/products/:slug" }`.
///
/// Deliberadamente NO es una función a ejecutar: es un objeto literal
/// estático, así que `nexa-parser` puede extraerlo con análisis sintáctico
/// puro, sin necesitar un motor de JavaScript embebido en el compilador
/// (ver docs/FASES-DE-CONSTRUCCION.md, Fase 7). `:slug` en la URL se
/// sustituye con el parámetro de ruta del mismo nombre (Fase 6).
#[derive(Debug, Clone, PartialEq)]
pub struct Loader {
    pub url_template: String,
}
