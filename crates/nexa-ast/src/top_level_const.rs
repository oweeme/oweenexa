/// `const NOMBRE = <valor>;` de nivel superior en el archivo de una
/// página (Fase 58, bug #27). `nexa-activation` la usa para anteponer
/// la declaración a un chunk de evento que la referencia — un handler
/// extraído a su propio archivo JS no comparte scope de módulo con el
/// resto de la página, así que sin esto un `const` compartido queda
/// sin definir en el chunk (`ReferenceError` en el navegador, en
/// silencio: el build no avisa nada).
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelConst {
    /// Texto fuente exacto de la declaración completa (`const API_BASE
    /// = "https://api.oweeme.com";`), lista para anteponerse tal cual a
    /// un chunk — Rust nunca la ejecuta, solo la copia.
    pub source: String,
    /// `true` si el valor es un literal (string/número/booleano/null),
    /// una plantilla sin interpolación, o un array/objeto compuesto
    /// solo de esos — lo bastante simple para copiarlo dentro de un
    /// chunk sin arrastrar nada más del archivo. `false` para cualquier
    /// otra cosa (una llamada a función, una referencia a otra
    /// variable, una expresión calculada) — un handler que use una
    /// constante no simple hace fallar el build de forma explícita en
    /// vez de generar un chunk roto en silencio.
    pub is_simple: bool,
}
