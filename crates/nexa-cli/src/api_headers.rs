//! `[api] headers` (Fase 78): token de servidor a servidor para las llamadas de `load`/`paths`.

use std::collections::BTreeMap;

use anyhow::{bail, Result};

/// Sustituye `${VAR}` por el entorno. Una variable que falta es un error claro (mejor que enviar un
/// `Authorization` vacío y recibir un 401 incomprensible).
pub fn interpolate(value: &str, env: &dyn Fn(&str) -> Option<String>) -> Result<String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else { bail!("`{value}`: falta el `}}` de `${{`") };
        let name = &after[..end];
        match env(name) {
            Some(v) => out.push_str(&v),
            None => bail!("[api] headers: la variable de entorno `{name}` no está definida (ponla en el entorno o en `.env`)"),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

pub fn resolve(headers: &BTreeMap<String, String>, env: &dyn Fn(&str) -> Option<String>) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for (name, value) in headers {
        let ok_name = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !ok_name {
            bail!("[api] headers: `{name}` no es un nombre de cabecera válido");
        }
        let value = interpolate(value, env)?;
        if value.contains(['\r', '\n']) {
            bail!("[api] headers: el valor de `{name}` tiene un salto de línea");
        }
        out.push((name.clone(), value));
    }
    Ok(out)
}

/// Lee `[api]` del manifiesto del directorio actual y lo deja listo para el cargador.
pub fn init() -> Result<()> {
    let manifest = crate::manifest::load_lenient(std::path::Path::new("nexa.toml"));
    if let Some(api) = manifest.api.filter(|a| !a.headers.is_empty()) {
        let headers = resolve(&api.headers, &|k| std::env::var(k).ok())?;
        nexa_loader::set_default_headers(headers);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(k: &str) -> Option<String> {
        (k == "TOKEN").then(|| "s3cr3t".to_string())
    }

    #[test]
    fn substitutes_variables_and_fails_on_missing_ones() {
        assert_eq!(interpolate("Bearer ${TOKEN}", &env).unwrap(), "Bearer s3cr3t");
        assert_eq!(interpolate("sin variables", &env).unwrap(), "sin variables");
        assert!(interpolate("Bearer ${NOPE}", &env).is_err());
        assert!(interpolate("${TOKEN", &env).is_err());
    }

    #[test]
    fn header_names_and_values_are_validated() {
        let h: BTreeMap<String, String> = [("Authorization".to_string(), "Bearer ${TOKEN}".to_string())].into();
        assert_eq!(resolve(&h, &env).unwrap(), vec![("Authorization".to_string(), "Bearer s3cr3t".to_string())]);
        let bad: BTreeMap<String, String> = [("X Y".to_string(), "1".to_string())].into();
        assert!(resolve(&bad, &env).is_err());
        let inj: BTreeMap<String, String> = [("X".to_string(), "a\r\nB: c".to_string())].into();
        assert!(resolve(&inj, &env).is_err());
    }
}
