use std::path::Path;

/// Lista los locales disponibles (`es`, `en`...) mirando qué
/// `<locales_dir>/<locale>.json` existen. Así el `hreflang` no necesita
/// que el desarrollador declare la lista en ningún otro sitio — un
/// archivo nuevo ya es un locale nuevo.
pub fn available_locales(locales_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(locales_dir) else {
        return Vec::new();
    };

    let mut locales: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "json" {
                return None;
            }
            path.file_stem()?.to_str().map(str::to_string)
        })
        .collect();

    locales.sort();
    locales
}
