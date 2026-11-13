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

/// Codifica un valor de parámetro para ponerlo en una URL (RFC 3986: se
/// dejan tal cual las letras, dígitos y `-._~`; todo lo demás va como
/// `%XX`). Con renderizado bajo demanda (`nexa bundle`) el valor sale de
/// la URL que escribió quien visita: sin esto, `/news/a%3Fadmin%3D1`
/// o `/news/x%2F..%2Fadmin` podrían inyectar query/segmentos en la
/// llamada a *tu* API.
fn encode_param(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
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
        // Un valor `.`/`..` colapsaría o saldría del segmento (`/api/posts/..`).
        if value == "." || value == ".." {
            return Err(LoaderError::InvalidParam(name.to_string()));
        }
        result.push_str(&encode_param(value));
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

    #[test]
    fn param_values_are_percent_encoded_so_a_visitor_cannot_inject_into_the_api_url() {
        let mut params = BTreeMap::new();
        params.insert("slug".to_string(), "a?admin=1&x=#y/../z".to_string());
        let url = resolve_url("/api/posts/:slug", &params).unwrap();
        assert_eq!(url, "/api/posts/a%3Fadmin%3D1%26x%3D%23y%2F..%2Fz");
        assert!(!url.contains('?') && !url.contains('#'));
        assert_eq!(url.matches('/').count(), 3, "el valor no puede agregar segmentos");
    }

    #[test]
    fn unicode_and_spaces_are_encoded_but_plain_slugs_are_untouched() {
        let mut params = BTreeMap::new();
        params.insert("a".to_string(), "iphone-17_pro.v2~x".to_string());
        params.insert("b".to_string(), "año nuevo".to_string());
        assert_eq!(resolve_url("/:a", &params).unwrap(), "/iphone-17_pro.v2~x");
        assert_eq!(resolve_url("/:b", &params).unwrap(), "/a%C3%B1o%20nuevo");
    }

    #[test]
    fn dot_segments_are_rejected() {
        for bad in [".", ".."] {
            let mut params = BTreeMap::new();
            params.insert("slug".to_string(), bad.to_string());
            assert!(matches!(resolve_url("/api/:slug", &params), Err(LoaderError::InvalidParam(n)) if n == "slug"), "{bad}");
        }
    }
}
