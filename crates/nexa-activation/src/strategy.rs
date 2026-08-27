use serde::Serialize;

/// Cuándo se activa un nodo interactivo (ver "Estrategias de activación",
/// docs/Arquitectura SEO Completo Framework.md, Paso 4 — Runtime).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Strategy {
    /// Se activa en la primera interacción real del usuario (click,
    /// input...). Es el valor por defecto: no carga JS hasta que hace
    /// falta.
    Interaction,
    /// Se activa cuando el elemento entra en el viewport.
    Visible,
    /// Se activa cuando el navegador está inactivo (`requestIdleCallback`).
    Idle,
    /// Se activa inmediatamente al cargar la página.
    Load,
    /// No se activa sola: algo más debe pedirlo explícitamente.
    Manual,
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy::Interaction
    }
}

impl Strategy {
    /// Interpreta el valor crudo de `data-nexa-strategy`. Un valor
    /// desconocido o ausente cae al valor por defecto en vez de romper el
    /// build — todavía no existe infraestructura de diagnósticos (eso
    /// llega con el SEO/A11y Analyzer) para avisar de esto como warning.
    pub fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("interaction") => Strategy::Interaction,
            Some("visible") => Strategy::Visible,
            Some("idle") => Strategy::Idle,
            Some("load") => Strategy::Load,
            Some("manual") => Strategy::Manual,
            _ => Strategy::default(),
        }
    }
}
