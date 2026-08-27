/// Un segmento de una ruta, ya interpretado a partir de un nombre de
/// archivo o carpeta (`products` vs `[slug]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Static(String),
    /// El nombre es el del parámetro capturado (`[slug]` -> `"slug"`).
    Dynamic(String),
}

impl Segment {
    pub(crate) fn parse(raw: &str) -> Segment {
        match raw.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            Some(name) => Segment::Dynamic(name.to_string()),
            None => Segment::Static(raw.to_string()),
        }
    }
}
