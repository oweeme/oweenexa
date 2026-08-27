use std::path::Path;

/// Carga `<locales_dir>/<locale>.json`. `Ok(None)` si el archivo no
/// existe — no toda página vive bajo `[locale]`, y no tener traducciones
/// no es un error.
pub fn load_locale(locales_dir: &Path, locale: &str) -> std::io::Result<Option<serde_json::Value>> {
    let path = locales_dir.join(format!("{locale}.json"));
    if !path.exists() {
        return Ok(None);
    }

    let text = std::fs::read_to_string(path)?;
    let value = serde_json::from_str(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Some(value))
}
