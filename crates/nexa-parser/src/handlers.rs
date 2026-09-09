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

use oxc_ast::ast::{
    ArrayExpressionElement, BindingPattern, Expression, ObjectPropertyKind, Program, Statement,
};

use nexa_ast::TopLevelConst;

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

/// `const NOMBRE = <valor>;` de nivel superior, que NO sea uno de los
/// handlers de arriba (Fase 58, bug #27) — un `const buy = () => {}` ya
/// quedó capturado en `find_handlers`; acá interesa el resto: una URL
/// compartida, un límite de paginación, cualquier valor que un handler
/// pueda referenciar sin ser, él mismo, una función.
pub(crate) fn find_top_level_consts(program: &Program, source: &str) -> BTreeMap<String, TopLevelConst> {
    let mut consts = BTreeMap::new();

    for stmt in program.body.iter() {
        let Statement::VariableDeclaration(var_decl) = stmt else { continue };

        // Toda la declaración es "un handler", no una constante — ya la
        // tiene `find_handlers`. Evita clasificar la misma línea dos
        // veces si mezclara un `const buy = () => {}, X = 1;` (raro,
        // pero mejor no arrastrar X sola sin su handler hermano).
        let is_handler_statement = var_decl
            .declarations
            .iter()
            .any(|d| matches!(d.init, Some(Expression::ArrowFunctionExpression(_))));
        if is_handler_statement {
            continue;
        }

        for declarator in var_decl.declarations.iter() {
            let Some(init) = &declarator.init else { continue };
            let Some(name) = binding_identifier_name(&declarator.id) else { continue };

            consts.insert(
                name,
                TopLevelConst {
                    source: var_decl.span.source_text(source).to_string(),
                    is_simple: is_simple_literal(init),
                },
            );
        }
    }

    consts
}

/// Literal / plantilla sin interpolación / array u objeto compuesto
/// solo de esos — lo único que se puede copiar dentro de un chunk sin
/// arrastrar nada más del archivo (ni ejecutar nada). Cualquier otra
/// cosa (una llamada, otra variable, una expresión calculada) es "no
/// simple": si un handler la usa, el build falla explícito en vez de
/// generar un chunk con un identificador sin definir.
fn is_simple_literal(expr: &Expression) -> bool {
    match expr {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_) => true,
        Expression::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        Expression::ParenthesizedExpression(inner) => is_simple_literal(&inner.expression),
        Expression::ArrayExpression(arr) => arr.elements.iter().all(|el| match el {
            ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => false,
            other => other.as_expression().is_some_and(is_simple_literal),
        }),
        Expression::ObjectExpression(obj) => obj.properties.iter().all(|prop| match prop {
            ObjectPropertyKind::ObjectProperty(p) => !p.computed && is_simple_literal(&p.value),
            _ => false,
        }),
        _ => false,
    }
}
