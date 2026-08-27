//! Fase 9: `@nexa/ui` como sistema de diseño separado del core, con
//! *tree-shaking* real: un proyecto que no usa ningún componente de
//! `@nexa/ui` no paga ningún costo de CSS por él — es literalmente el
//! criterio de salida de esta fase.
//!
//! El CSS (design tokens + un archivo por componente) vive en
//! `packages/ui/src/*.css` — ese es el código fuente que el equipo edita.
//! Este crate incrusta una copia (`assets/`, regenerada con `npm run
//! build:ui-assets`) y decide, mirando qué clases `nx-*` aparecen
//! realmente en la página, cuáles de esos archivos incluir.
//!
//! `collect_used_classes` y `build_stylesheet_from_classes` están
//! separadas de `build_stylesheet` a propósito: `nexa-cli` compila varias
//! páginas por build, y el CSS final es la unión de lo que usa *todo el
//! sitio* (un archivo compartido en `dist/assets/`), no solo una página
//! — el llamador necesita poder acumular clases de varias páginas antes
//! de decidir qué CSS ensamblar.
//!
//! Limitación conocida: como no hay composición de componentes todavía
//! (Fase 2), `@nexa/ui` se consume con HTML normal (`<button
//! class="nx-btn nx-btn-primary">`), no con `<Button>`. Por eso el
//! análisis solo puede mirar el atributo `class` — y solo cuando es
//! estático (`class="nx-btn"`, no `class={expr}`): una clase calculada en
//! tiempo de render no se puede saber sin datos reales.

mod registry;
mod scan;

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use nexa_ir::IrNode;

pub use scan::collect_used_classes;

/// Ensambla el CSS de exactamente los componentes cuyas clases aparecen
/// en `used_classes` — nunca más, nunca menos. `None` si el conjunto no
/// toca ningún componente conocido: cero bytes de CSS.
pub fn build_stylesheet_from_classes(used_classes: &BTreeSet<String>) -> Option<String> {
    if used_classes.is_empty() {
        return None;
    }

    let matched: Vec<&str> = registry::COMPONENTS
        .iter()
        .filter(|(family, _)| used_classes.iter().any(|class| registry::matches_family(family, class)))
        .map(|(_, css)| *css)
        .collect();

    if matched.is_empty() {
        return None;
    }

    let mut stylesheet = String::from(registry::TOKENS_CSS);
    for css in matched {
        stylesheet.push('\n');
        stylesheet.push_str(css);
    }

    Some(stylesheet)
}

/// Atajo para una sola página: recoge sus clases y ensambla su CSS.
pub fn build_stylesheet(root: &IrNode) -> Option<String> {
    build_stylesheet_from_classes(&collect_used_classes(root))
}

/// El CSS completo de `@nexa/ui` (tokens + todos los componentes), sin
/// *tree-shaking* — a diferencia de `build_stylesheet_from_classes`, esto
/// no sirve para un sitio real (ahí sí importa enviar solo lo que se usa).
/// Lo usa `nexa-cli` (Fase 12) para calcular el hash de contenido de
/// `nexa.lock`: el módulo en sí, no lo que un proyecto concreto termina
/// consumiendo de él.
pub fn full_source() -> String {
    let mut out = String::from(registry::TOKENS_CSS);
    for (_, css) in registry::COMPONENTS {
        out.push('\n');
        out.push_str(css);
    }
    out
}
