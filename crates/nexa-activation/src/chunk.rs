use std::collections::BTreeSet;

use crate::manifest::ActivationEntry;
use crate::strategy::Strategy;

/// Nombre y contenido de un chunk JS.
///
/// Cuando el handler existe en el mismo archivo como `function nombre()`
/// o `const nombre = () => {}`, `nexa-parser` ya extrajo su código fuente
/// exacto (Fase 5/7) y aquí se emite tal cual — Rust nunca lo ejecuta,
/// solo lo copia; quien lo ejecuta es el navegador. Si no se encontró
/// (ej. el handler viene de un import, o de un patrón que todavía no
/// reconocemos), se emite un placeholder explícito en su lugar.
pub struct Chunk {
    pub filename: String,
    pub content: String,
    /// Copiada de la entrada del manifiesto (Fase 14): con qué
    /// estrategia se activa este chunk — `packages/runtime` solo lo
    /// descarga de inmediato si es `Load`; el resto se difiere a
    /// propósito. `nexa-cli` la usa para calcular el "JS inicial" real de
    /// una página al comprobar el presupuesto de rendimiento.
    pub strategy: Strategy,
}

/// `import_names`: identificadores que `nexa-cli` sabe resolver por un
/// import map declarado en `<head>` (Fase 15) — `platform` siempre está
/// (built-in), más lo que el proyecto haya declarado en `nexa.toml`
/// (`[imports]`, ej. `stripe`). Este crate no sabe (ni le hace falta
/// saber) a qué URL resuelve cada uno — solo si el handler usa
/// `<nombre>.` de verdad, y si es así, antepone
/// `import { <nombre> } from "<nombre>";`, confiando en que el import
/// map del documento resuelva ese specifier "pelado". Es el mismo
/// mecanismo que antes solo existía, cableado a mano, para `platform`
/// (Fase 13) — generalizado aquí para que un paquete de un tercero
/// (`stripe`, Fase 15) funcione exactamente igual, sin que este crate
/// necesite saber que existe.
/// Separado de `generate` para que `build::collect` pueda calcular el
/// contenido primero, y solo después decidir el nombre de archivo a
/// partir de su hash de contenido (cache-busting real: dos handlers con
/// contenido distinto nunca comparten nombre de archivo).
pub(crate) fn content_for(entry: &ActivationEntry, handler_source: Option<&str>, import_names: &BTreeSet<String>) -> String {
    match handler_source {
        Some(source) => real_chunk(entry, source, import_names),
        None => placeholder_chunk(entry),
    }
}

pub(crate) fn generate(filename: String, content: String, strategy: Strategy) -> Chunk {
    Chunk { filename, content, strategy }
}

fn real_chunk(entry: &ActivationEntry, handler_source: &str, import_names: &BTreeSet<String>) -> String {
    let mut lines = vec!["// Generado por Nexa: handler extraído tal cual del código fuente de la página.".to_string()];

    // Cada identificador importable se pide solo si el handler de
    // verdad lo usa (`platform.share(...)`, `stripe.load(...)`) — mismo
    // criterio de costo cero que `@nexa/ui`/`@nexa/forms`: un chunk que
    // no lo toca no debe ni pedirlo.
    for name in import_names {
        if uses_identifier(handler_source, name) {
            lines.push(format!("import {{ {name} }} from \"{name}\";"));
        }
    }

    lines.push(handler_source.to_string());
    lines.push(String::new());
    lines.push("export default function activate(el) {".to_string());
    lines.push(format!("    el.addEventListener(\"{}\", {});", entry.event, entry.handler));
    lines.push("}".to_string());
    lines.push(String::new());

    lines.join("\n")
}

/// Un identificador seguido de `.` — la misma clase de detección
/// sintáctica simple que ya usa `nexa-cli` para `nx-*`/`data-nexa-form`
/// (Fases 9/10): no es un analizador de JS real, solo una señal barata y
/// suficiente para este propósito.
fn uses_identifier(handler_source: &str, name: &str) -> bool {
    handler_source.contains(&format!("{name}."))
}

fn placeholder_chunk(entry: &ActivationEntry) -> String {
    let lines = [
        "// Generado por Nexa.".to_string(),
        format!(
            "// No se encontró el código fuente de `{}` en este archivo (¿viene de un",
            entry.handler
        ),
        "// import, o de un patrón que nexa-parser todavía no reconoce?) — placeholder.".to_string(),
        "export default function activate(el) {".to_string(),
        format!("    el.addEventListener(\"{}\", () => {{", entry.event),
        format!(
            "        console.warn(\"[nexa] `{}` no se pudo resolver a código real.\");",
            entry.handler
        ),
        "    });".to_string(),
        "}".to_string(),
        String::new(),
    ];

    lines.join("\n")
}
