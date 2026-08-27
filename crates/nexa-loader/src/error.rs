#[derive(Debug)]
pub enum LoaderError {
    /// Falta un parámetro que la plantilla de URL necesita (ej. `:slug`
    /// sin un valor capturado por el router).
    MissingParam(String),
    /// El backend respondió 404: no existe el recurso pedido. Esto es lo
    /// que `nexa-cli` convierte en un 404 real de verdad, en vez de un
    /// 200 con contenido vacío.
    NotFound,
    /// El backend respondió con otro código de error (4xx/5xx).
    Http(u16),
    /// Fallo de red/conexión (backend caído, DNS, timeout...).
    Network(String),
    /// El cuerpo de la respuesta no es JSON válido.
    InvalidJson(String),
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::MissingParam(name) => {
                write!(f, "falta el parámetro `:{name}` en la URL del loader")
            }
            LoaderError::NotFound => write!(f, "el backend respondió 404 (recurso no encontrado)"),
            LoaderError::Http(status) => write!(f, "el backend respondió HTTP {status}"),
            LoaderError::Network(msg) => write!(f, "error de red llamando al backend: {msg}"),
            LoaderError::InvalidJson(msg) => {
                write!(f, "la respuesta del backend no es JSON válido: {msg}")
            }
        }
    }
}

impl std::error::Error for LoaderError {}
