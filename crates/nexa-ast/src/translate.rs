use crate::Expr;

/// `t("profile.donateTo", { name: data.creatorName })` (Fase 10,
/// interpolación agregada en la Fase 45): la clave y los argumentos
/// nombrados, sin resolver todavía. Cada valor de `args` es la misma
/// `Expr` limitada que el resto de Nexa acepta en cualquier otro lado
/// (`data.x`, `params.x`) — nunca una expresión arbitraria. `args` vacío
/// (el `Vec` por defecto) es exactamente `t("clave")` de siempre.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Translate {
    pub key: String,
    pub args: Vec<(String, Expr)>,
}
