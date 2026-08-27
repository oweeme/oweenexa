use std::collections::BTreeMap;

use crate::error::LoaderError;

/// Sustituye cada `:nombre` de `template` por el valor correspondiente en
/// `params` (los mismos que captura `nexa-router` al hacer match de una
/// ruta). Falla si la plantilla referencia un parámetro ausente — mejor
/// un error claro en el momento de renderizar que una URL rota.
pub fn resolve_url(template: &str, params: &BTreeMap<String, String>) -> Result<String, LoaderError> {
    template
        .split('/')
        .map(|segment| match segment.strip_prefix(':') {
            Some(name) => params
                .get(name)
                .cloned()
                .ok_or_else(|| LoaderError::MissingParam(name.to_string())),
            None => Ok(segment.to_string()),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("/"))
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
}
