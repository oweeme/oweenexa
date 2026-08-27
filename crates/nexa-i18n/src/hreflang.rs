use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AlternateLink {
    pub hreflang: String,
    pub href: String,
}

/// Para cada locale disponible, la URL equivalente a la actual pero con
/// `params["locale"]` sustituido por ese locale — conservando el resto de
/// parámetros (ej. `:slug`) igual. Reutiliza la misma sustitución de
/// `:param` que ya usa `nexa-loader` para `load.url`.
pub fn alternate_links(
    route_pattern: &str,
    current_params: &BTreeMap<String, String>,
    locales: &[String],
) -> Vec<AlternateLink> {
    locales
        .iter()
        .filter_map(|locale| {
            let mut params = current_params.clone();
            params.insert("locale".to_string(), locale.clone());
            let href = nexa_loader::resolve_url(route_pattern, &params).ok()?;
            Some(AlternateLink { hreflang: locale.clone(), href })
        })
        .collect()
}
