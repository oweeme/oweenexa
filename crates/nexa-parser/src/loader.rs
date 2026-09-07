//! Extrae `export const load = { url: "..." };` de un archivo de página.
//!
//! Deliberadamente NO se ejecuta nada: `load` tiene que ser un objeto
//! literal estático, no una función. Eso es lo que permite que Rust lo
//! entienda con puro análisis sintáctico, sin un motor de JavaScript
//! embebido (ver docs/FASES-DE-CONSTRUCCION.md, Fase 7).

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, Program};

use nexa_ast::Loader;

use crate::declaration::{find_named_export, unwrap_object_literal};

pub(crate) fn find_loader(program: &Program) -> Option<Loader> {
    find_url_export(program, "load")
}

/// `export const paths = { url: "..." };` (Fase 17) — misma forma
/// sintáctica exacta que `load`, así que reutiliza el mismo parseo; lo
/// único que cambia es qué hace `nexa-cli` con la URL una vez resuelta
/// (pedir un array de sets de parámetros para enumerar, no un único
/// objeto de datos).
pub(crate) fn find_paths(program: &Program) -> Option<Loader> {
    find_url_export(program, "paths")
}

fn find_url_export(program: &Program, name: &str) -> Option<Loader> {
    let init = find_named_export(program, name)?;
    let obj = unwrap_object_literal(init)?;
    let url_template = url_property(obj)?;
    Some(Loader { url_template })
}

fn url_property(obj: &ObjectExpression<'_>) -> Option<String> {
    for prop in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let PropertyKey::StaticIdentifier(key) = &prop.key else {
            continue;
        };
        if key.name.as_str() != "url" {
            continue;
        }
        if let Expression::StringLiteral(s) = &prop.value {
            return Some(s.value.as_str().to_string());
        }
    }

    None
}
