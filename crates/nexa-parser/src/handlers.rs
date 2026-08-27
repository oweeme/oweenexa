//! Extrae, por nombre, el código fuente exacto de los manejadores de
//! eventos declarados en el mismo archivo: `function buy() {...}` o
//! `const buy = () => {...}` de nivel superior.
//!
//! Sigue siendo extracción de texto, no ejecución: Rust nunca interpreta
//! ese código — solo copia el fragmento de fuente correspondiente
//! (usando el `Span` que ya trae el AST) para que `nexa-activation`
//! pueda emitirlo tal cual dentro del chunk JS del evento. Quien
//! finalmente lo ejecuta es el navegador.

use std::collections::BTreeMap;

use oxc_ast::ast::{BindingPattern, Expression, Program, Statement};

pub(crate) fn find_handlers(program: &Program, source: &str) -> BTreeMap<String, String> {
    let mut handlers = BTreeMap::new();

    for stmt in program.body.iter() {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    let name = id.name.as_str().to_string();
                    handlers.insert(name, func.span.source_text(source).to_string());
                }
            }
            Statement::VariableDeclaration(var_decl) => {
                for declarator in var_decl.declarations.iter() {
                    let Some(Expression::ArrowFunctionExpression(_)) = &declarator.init else {
                        continue;
                    };
                    if let Some(name) = binding_identifier_name(&declarator.id) {
                        // Se extrae la declaración completa (`const buy = () =>
                        // {...}`), no solo la función: así el chunk queda con
                        // exactamente el mismo identificador ya en scope.
                        handlers.insert(name, var_decl.span.source_text(source).to_string());
                    }
                }
            }
            _ => {}
        }
    }

    handlers
}

fn binding_identifier_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str().to_string()),
        _ => None,
    }
}
