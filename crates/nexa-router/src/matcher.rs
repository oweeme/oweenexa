use std::collections::BTreeMap;

use crate::route::Route;

/// Encuentra la primera ruta de `routes` que coincide con `path`, junto
/// con los parámetros capturados.
///
/// Las rutas estáticas se prueban antes que las dinámicas: así
/// `/products/featured` (si existiera como archivo estático) nunca es
/// capturado por accidente por `/products/:slug`.
pub fn match_route<'a>(
    routes: &'a [Route],
    path: &str,
) -> Option<(&'a Route, BTreeMap<String, String>)> {
    routes
        .iter()
        .filter(|r| !r.is_dynamic())
        .find_map(|r| r.matches(path).map(|params| (r, params)))
        .or_else(|| {
            routes
                .iter()
                .filter(|r| r.is_dynamic())
                .find_map(|r| r.matches(path).map(|params| (r, params)))
        })
}
