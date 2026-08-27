use std::collections::BTreeMap;

use serde::Serialize;

use crate::strategy::Strategy;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActivationEntry {
    pub event: String,
    pub handler: String,
    pub module: String,
    pub strategy: Strategy,
}

/// Manifiesto de activación: id de nodo (como texto, para que sea una
/// clave JSON válida) → qué hacer con él. Es lo que `packages/runtime`
/// lee en el navegador para saber qué cargar y cuándo.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ActivationManifest(BTreeMap<String, ActivationEntry>);

impl ActivationManifest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra (o reemplaza) la entrada de `node_id`. Pública porque
    /// construir manifiestos a mano es legítimo fuera de este crate
    /// también (ej. para probar cómo los consume `nexa-cli`), no solo
    /// desde [`crate::build`].
    pub fn insert(&mut self, node_id: usize, entry: ActivationEntry) {
        self.0.insert(node_id.to_string(), entry);
    }

    pub fn get(&self, node_id: usize) -> Option<&ActivationEntry> {
        self.0.get(&node_id.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Todas las entradas, con el id de nodo tal cual quedó serializado
    /// (texto). Usada por `nexa-cli` (Fase 14) para el panel de
    /// DevTools de `nexa dev` — listar qué se activó y con qué
    /// estrategia es justo lo que ese panel promete mostrar.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &ActivationEntry)> {
        self.0.iter().map(|(id, entry)| (id.as_str(), entry))
    }
}
