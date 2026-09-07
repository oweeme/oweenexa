//! Encuentra el `export default function` de un archivo y extrae el JSX
//! que retorna, delegando la conversión del árbol a [`crate::jsx`].

use oxc_ast::ast::{Expression, ExportDefaultDeclarationKind, Function, Program, Statement};

use nexa_ast::{Component, JsonTemplate, Loader, SeoConfig};

use crate::error::ParseError;
use crate::handlers::find_handlers;
use crate::head::find_head;
use crate::jsx::convert_element;
use crate::loader::{find_loader, find_paths};
use crate::schema::find_schema;
use crate::seo::find_seo;

pub(crate) fn find_component(program: &Program, source: &str) -> Result<Component, ParseError> {
    let loader = find_loader(program);
    let paths = find_paths(program);
    let handlers = find_handlers(program, source);
    let seo = find_seo(program);
    let schema = find_schema(program);
    let head = find_head(program);

    for stmt in program.body.iter() {
        if let Statement::ExportDefaultDeclaration(export) = stmt {
            if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export.declaration {
                return component_from_function(func, loader, paths, handlers, seo, schema, head);
            }
        }
    }
    Err(ParseError::NoDefaultExport)
}

#[allow(clippy::too_many_arguments)]
fn component_from_function(
    func: &Function,
    loader: Option<Loader>,
    paths: Option<Loader>,
    handlers: std::collections::BTreeMap<String, String>,
    seo: Option<SeoConfig>,
    schema: Option<JsonTemplate>,
    head: Option<JsonTemplate>,
) -> Result<Component, ParseError> {
    let name = func
        .id
        .as_ref()
        .map(|id| id.name.as_str().to_string())
        .unwrap_or_else(|| "Component".to_string());

    let body = func.body.as_ref().ok_or(ParseError::NoJsxReturned)?;

    for stmt in body.statements.iter() {
        if let Statement::ReturnStatement(ret_stmt) = stmt {
            let arg = ret_stmt.argument.as_ref().ok_or(ParseError::NoJsxReturned)?;

            return match unwrap_parens(arg) {
                Expression::JSXElement(jsx) => {
                    let root = convert_element(jsx)?;
                    Ok(Component { name, root, loader, paths, handlers, seo, schema, head })
                }
                _ => Err(ParseError::NoJsxReturned),
            };
        }
    }

    Err(ParseError::NoJsxReturned)
}

fn unwrap_parens<'a, 'b>(expr: &'b Expression<'a>) -> &'b Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(inner) => unwrap_parens(&inner.expression),
        other => other,
    }
}
