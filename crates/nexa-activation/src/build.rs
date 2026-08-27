use std::collections::{BTreeMap, BTreeSet};

use nexa_ir::{Classification, IrComponent, IrNode, IrNodeKind};

use crate::chunk::{self, Chunk};
use crate::manifest::{ActivationEntry, ActivationManifest};
use crate::strategy::Strategy;

/// Recorre el IR ya clasificado y produce el manifiesto de activación más
/// el contenido de cada chunk JS.
///
/// `component_name` se usa solo para nombrar los archivos
/// (`ProductPage-3.js`) de forma legible; no afecta a la clasificación.
/// `handlers` es `nexa_ast::Component::handlers` — el código fuente ya
/// extraído de cada `function nombre() {...}` / `const nombre = () => {}`
/// del archivo, si `nexa-parser` lo encontró. `import_names` (Fase 15)
/// son los identificadores que un chunk puede importar por su nombre
/// pelado (`platform`, o lo que el proyecto declare en `nexa.toml`
/// `[imports]`) — ver `chunk::generate`.
pub fn build(
    component: &IrComponent,
    component_name: &str,
    handlers: &BTreeMap<String, String>,
    import_names: &BTreeSet<String>,
) -> (ActivationManifest, Vec<Chunk>) {
    let mut manifest = ActivationManifest::new();
    let mut chunks = Vec::new();

    collect(&component.root, component_name, handlers, import_names, &mut manifest, &mut chunks);

    (manifest, chunks)
}

fn collect(
    node: &IrNode,
    component_name: &str,
    handlers: &BTreeMap<String, String>,
    import_names: &BTreeSet<String>,
    manifest: &mut ActivationManifest,
    chunks: &mut Vec<Chunk>,
) {
    let IrNodeKind::Element { events, children, .. } = &node.kind else {
        return;
    };

    if node.classification == Classification::Interactive {
        // Un solo evento por nodo en el modelo actual (ver limitaciones de
        // la Fase 3): si un elemento tuviera varios, cada uno necesitaría
        // su propio nodo, no solo su propia entrada de manifiesto.
        if let Some(event) = events.first() {
            let module = format!("/assets/{component_name}-{}.js", node.id);
            let entry = ActivationEntry {
                event: event.name.clone(),
                handler: event.handler.path(),
                module,
                strategy: Strategy::parse(event.strategy.as_deref()),
            };

            let handler_source = handlers.get(&entry.handler).map(String::as_str);
            let filename = format!("{component_name}-{}.js", node.id);
            chunks.push(chunk::generate(filename, &entry, handler_source, import_names));
            manifest.insert(node.id, entry);
        }
    }

    for child in children {
        collect(child, component_name, handlers, import_names, manifest, chunks);
    }
}
