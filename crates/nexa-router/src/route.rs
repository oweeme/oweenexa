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
        self.segments.iter().any(|s| matches!(s, Segment::Dynamic(_) | Segment::CatchAll(_)))
    }

    pub fn has_catch_all(&self) -> bool {
        self.segments.iter().any(|s| matches!(s, Segment::CatchAll(_)))
    }

    /// Intenta emparejar `path` (ej. `/products/iphone-17`) contra este
    /// patrón. Si coincide, devuelve los parámetros capturados (vacío si
    /// la ruta es completamente estática).
    pub fn matches(&self, path: &str) -> Option<BTreeMap<String, String>> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();

        let catch_all = self.has_catch_all();
        if (!catch_all && parts.len() != self.segments.len())
            || (catch_all && parts.len() < self.segments.len())
        {
            return None;
        }

        let mut params = BTreeMap::new();
        for (i, segment) in self.segments.iter().enumerate() {
            let part = &parts[i];
            match segment {
                Segment::CatchAll(name) => {
                    // solo vale como último segmento
                    if i != self.segments.len() - 1 {
                        return None;
                    }
                    params.insert(name.clone(), parts[i..].join("/"));
                }
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
