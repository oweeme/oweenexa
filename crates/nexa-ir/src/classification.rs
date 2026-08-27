/// Clasificación de un nodo, tal como la define la especificación de Nexa
/// ("Tres estados de un nodo" + "Async Components" en
/// `docs/Arquitectura SEO Completo Framework.md`):
///
/// - `Static`: HTML fijo. No depende de datos ni de JavaScript.
/// - `Dynamic`: depende de datos (`{product.name}`), pero se resuelve en
///   el renderer — no necesita JavaScript en el navegador.
/// - `Interactive`: tiene al menos un manejador de eventos (`onClick`) —
///   necesita runtime en el navegador (Progressive Activation, Fase 5).
/// - `Async`: reservado para contenido que se carga de forma diferida
///   (Fase 7+). El analyzer todavía no produce esta variante.
/// - `Island` (Fase 16): punto de montaje de una isla interactiva
///   (`data-nexa-island`) — el servidor solo renderiza el fallback; todo
///   lo interactivo se monta en el cliente vía un módulo externo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Static,
    Dynamic,
    Interactive,
    Async,
    Island,
}

/// Conteo de nodos por clasificación de un árbol completo. Es la base de
/// lo que en la Fase 14 será el reporte de DevTools ("42 KB de HTML, 8 KB
/// de JS activado").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClassificationCounts {
    pub static_count: usize,
    pub dynamic_count: usize,
    pub interactive_count: usize,
    pub async_count: usize,
    pub island_count: usize,
}

impl ClassificationCounts {
    pub(crate) fn record(&mut self, classification: Classification) {
        match classification {
            Classification::Static => self.static_count += 1,
            Classification::Dynamic => self.dynamic_count += 1,
            Classification::Interactive => self.interactive_count += 1,
            Classification::Async => self.async_count += 1,
            Classification::Island => self.island_count += 1,
        }
    }

    pub fn total(&self) -> usize {
        self.static_count
            + self.dynamic_count
            + self.interactive_count
            + self.async_count
            + self.island_count
    }
}
