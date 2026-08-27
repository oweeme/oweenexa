use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::segment::Segment;

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    /// Forma legible del patrón (`/products/:slug`), útil para logs y
    /// para nombrar archivos de salida.
    pub pattern: String,
    /// Archivo `.tsx` que implementa esta ruta.
    pub file: PathBuf,
    pub segments: Vec<Segment>,
}

impl Route {
    pub fn is_dynamic(&self) -> bool {
        self.segments.iter().any(|s| matches!(s, Segment::Dynamic(_)))
    }

    /// Intenta emparejar `path` (ej. `/products/iphone-17`) contra este
    /// patrón. Si coincide, devuelve los parámetros capturados (vacío si
    /// la ruta es completamente estática).
    pub fn matches(&self, path: &str) -> Option<BTreeMap<String, String>> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();

        if parts.len() != self.segments.len() {
            return None;
        }

        let mut params = BTreeMap::new();
        for (segment, part) in self.segments.iter().zip(parts.iter()) {
            match segment {
                Segment::Static(expected) => {
                    if expected != part {
                        return None;
                    }
                }
                Segment::Dynamic(name) => {
                    params.insert(name.clone(), (*part).to_string());
                }
            }
        }

        Some(params)
    }
}
