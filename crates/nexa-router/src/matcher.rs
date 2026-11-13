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
                .filter(|r| r.is_dynamic() && !r.has_catch_all())
                .find_map(|r| r.matches(path).map(|params| (r, params)))
        })
        // `[...rest]` es el último recurso: lo que ninguna otra ruta reclama.
        .or_else(|| {
            routes
                .iter()
                .filter(|r| r.has_catch_all())
                .filter_map(|r| r.matches(path).map(|params| (r, params)))
                // el comodín más específico (el de prefijo más largo) gana
                .max_by_key(|(r, _)| r.segments.len())
        })
}
