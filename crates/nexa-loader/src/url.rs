use std::collections::BTreeMap;

use crate::error::LoaderError;

/// Sustituye cada `:nombre` de `template` por el valor correspondiente en
/// `params` (los mismos que captura `nexa-router` al hacer match de una
/// ruta). Falla si la plantilla referencia un parámetro ausente — mejor
/// un error claro en el momento de renderizar que una URL rota.
///
/// Bug #24: la primera versión solo reemplazaba un `:nombre` cuando era un
/// segmento *completo* entre `/` (`segment.strip_prefix(':')`) — un
/// `:locale` pegado a un query string (`/api/x?lang=:locale`) queda dentro
/// del mismo segmento que `x?lang=...`, que no empieza con `:`, así que
/// nunca se tocaba. `substitute_in_segment` busca `:nombre` en cualquier
/// posición del segmento, no solo al principio.
pub fn resolve_url(template: &str, params: &BTreeMap<String, String>) -> Result<String, LoaderError> {
    template
        .split('/')
        .map(|segment| substitute_in_segment(segment, params))
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("/"))
}

/// Reemplaza cada `:nombre` dentro de `segment` (uno de los pedazos entre
/// `/` de la plantilla completa) por su valor en `params`. Un `:` sin
/// ningún caracter de identificador después (al final del segmento, o
/// seguido de algo que no es letra/dígito/`_`) se deja tal cual — no todo
/// `:` en una URL es un parámetro de Nexa (ej. `https://`).
fn substitute_in_segment(segment: &str, params: &BTreeMap<String, String>) -> Result<String, LoaderError> {
    let mut result = String::with_capacity(segment.len());
    let mut rest = segment;

    while let Some(colon_at) = rest.find(':') {
        result.push_str(&rest[..colon_at]);
        let after_colon = &rest[colon_at + 1..];
        let name_len = after_colon.find(|c: char| !c.is_ascii_alphanumeric() && c != '_').unwrap_or(after_colon.len());
        let name = &after_colon[..name_len];

        if name.is_empty() {
            result.push(':');
            rest = after_colon;
            continue;
        }

        let value = params.get(name).ok_or_else(|| LoaderError::MissingParam(name.to_string()))?;
        result.push_str(value);
        rest = &after_colon[name_len..];
    }

    result.push_str(rest);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_a_single_param() {
        let mut params = BTreeMap::new();
        params.insert("slug".to_string(), "iphone-17".to_string());

        let url = resolve_url("/api/products/:slug", &params).unwrap();
        assert_eq!(url, "/api/products/iphone-17");
    }

    #[test]
    fn passes_through_templates_without_params() {
        let params = BTreeMap::new();
        let url = resolve_url("/api/products", &params).unwrap();
        assert_eq!(url, "/api/products");
    }

    #[test]
    fn substitutes_multiple_params() {
        let mut params = BTreeMap::new();
        params.insert("category".to_string(), "phones".to_string());
        params.insert("slug".to_string(), "iphone-17".to_string());

        let url = resolve_url("/api/:category/:slug", &params).unwrap();
        assert_eq!(url, "/api/phones/iphone-17");
    }

    #[test]
    fn missing_param_is_an_error() {
        let params = BTreeMap::new();
        let err = resolve_url("/api/products/:slug", &params).unwrap_err();
        assert!(matches!(err, LoaderError::MissingParam(name) if name == "slug"));
    }

    #[test]
    fn substitutes_a_param_inside_a_query_string() {
        // Bug #24: `?lang=:locale` no es un segmento completo entre `/` —
        // antes del fix, esto pasaba de largo sin reemplazar nada.
        let mut params = BTreeMap::new();
        params.insert("locale".to_string(), "es".to_string());

        let url = resolve_url("/api/projects?type=service&lang=:locale", &params).unwrap();
        assert_eq!(url, "/api/projects?type=service&lang=es");
    }

    #[test]
    fn substitutes_multiple_params_in_the_same_segment() {
        let mut params = BTreeMap::new();
        params.insert("locale".to_string(), "es".to_string());
        params.insert("page".to_string(), "2".to_string());

        let url = resolve_url("/api/products?lang=:locale&page=:page", &params).unwrap();
        assert_eq!(url, "/api/products?lang=es&page=2");
    }

    #[test]
    fn a_lone_colon_with_no_identifier_after_it_is_left_as_is() {
        let params = BTreeMap::new();
        let url = resolve_url("https://api.miapp.com/products", &params).unwrap();
        assert_eq!(url, "https://api.miapp.com/products");
    }

    #[test]
    fn missing_param_inside_a_query_string_is_still_an_error() {
        let params = BTreeMap::new();
        let err = resolve_url("/api/projects?lang=:locale", &params).unwrap_err();
        assert!(matches!(err, LoaderError::MissingParam(name) if name == "locale"));
    }
}
