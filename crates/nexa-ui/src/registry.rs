pub(crate) const TOKENS_CSS: &str = include_str!("../assets/tokens.css");

/// `(prefijo de familia, CSS del componente)`. Un componente se incluye
/// si alguna clase usada es exactamente el prefijo, o empieza con
/// `"{prefijo}-"` (así `nx-btn-primary` también activa `nx-btn`).
pub(crate) const COMPONENTS: &[(&str, &str)] = &[
    ("nx-btn", include_str!("../assets/button.css")),
    ("nx-input", include_str!("../assets/input.css")),
    ("nx-card", include_str!("../assets/card.css")),
    ("nx-dialog", include_str!("../assets/dialog.css")),
];

pub(crate) fn matches_family(family: &str, used_class: &str) -> bool {
    used_class == family || used_class.starts_with(&format!("{family}-"))
}
