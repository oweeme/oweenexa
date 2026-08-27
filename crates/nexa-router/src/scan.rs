use std::path::Path;

use crate::route::Route;
use crate::segment::Segment;

/// Recorre `pages_dir` (ej. `src/pages`) y construye la tabla de rutas.
///
/// Convención: `index.tsx` mapea a la ruta del directorio que lo
/// contiene; `[slug].tsx` es un segmento dinámico; cualquier otro nombre
/// es un segmento estático literal.
pub fn scan_pages(pages_dir: &Path) -> std::io::Result<Vec<Route>> {
    let mut routes = Vec::new();
    if pages_dir.is_dir() {
        walk(pages_dir, pages_dir, &mut routes)?;
    }
    routes.sort_by(|a, b| a.pattern.cmp(&b.pattern));
    Ok(routes)
}

fn walk(root: &Path, dir: &Path, routes: &mut Vec<Route>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk(root, &path, routes)?;
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("tsx") {
            continue;
        }

        let relative = path.strip_prefix(root).unwrap_or(&path);
        routes.push(route_from_relative_path(relative, &path));
    }
    Ok(())
}

fn route_from_relative_path(relative: &Path, file: &Path) -> Route {
    let mut parts: Vec<String> = relative
        .with_extension("")
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    // `index` como último segmento representa al directorio padre.
    if parts.last().map(String::as_str) == Some("index") {
        parts.pop();
    }

    let segments: Vec<Segment> = parts.iter().map(|p| Segment::parse(p)).collect();
    let pattern = pattern_from_segments(&segments);

    Route {
        pattern,
        file: file.to_path_buf(),
        segments,
    }
}

fn pattern_from_segments(segments: &[Segment]) -> String {
    if segments.is_empty() {
        return "/".to_string();
    }

    let joined = segments
        .iter()
        .map(|s| match s {
            Segment::Static(name) => name.clone(),
            Segment::Dynamic(name) => format!(":{name}"),
        })
        .collect::<Vec<_>>()
        .join("/");

    format!("/{joined}")
}
