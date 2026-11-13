//! Encuentra el `export default function` de un archivo y extrae el JSX
//! que retorna, delegando la conversión del árbol a [`crate::jsx`].

use oxc_ast::ast::{Expression, ExportDefaultDeclarationKind, Function, JSXElement, Program, Statement};

use nexa_ast::{Component, JsonTemplate, Loader, SeoConfig, TopLevelConst};

use crate::compose::{Ctx, ImportResolver, Shared};
use crate::error::ParseError;
use crate::handlers::{find_handlers, find_top_level_consts};
use crate::head::find_head;
use crate::jsx::convert_element;
use crate::loader::{find_loader, find_paths};
use crate::schema::find_schema;
use crate::seo::find_seo;

pub(crate) fn find_component(
    program: &Program,
    source: &str,
    file_name: &str,
    resolver: &dyn ImportResolver,
) -> Result<Component, ParseError> {
    let loader = find_loader(program);
    let paths = find_paths(program);
    let handlers = find_handlers(program, source);
    let consts = find_top_level_consts(program, source);
    let seo = find_seo(program);
    let schema = find_schema(program);
    let head = find_head(program);

    let shared = Shared::default();
    let ctx = Ctx::root(resolver, &shared, file_name, source, program);

    for stmt in program.body.iter() {
        if let Statement::ExportDefaultDeclaration(export) = stmt {
            if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export.declaration {
                let mut component = component_from_function(func, &ctx, loader, paths, handlers, consts, seo, schema, head)?;
                // Funciones/constantes de los componentes de otros archivos
                // que usó esta página (sus `onClick` las necesitan). Las de
                // la propia página tienen prioridad.
                for (name, code) in shared.handlers.into_inner() {
                    component.handlers.entry(name).or_insert(code);
                }
                for (name, value) in shared.consts.into_inner() {
                    component.consts.entry(name).or_insert(value);
                }
                return Ok(component);
            }
        }
    }
    Err(ParseError::NoDefaultExport)
}

#[allow(clippy::too_many_arguments)]
fn component_from_function(
    func: &Function,
    ctx: &Ctx,
    loader: Option<Loader>,
    paths: Option<Loader>,
    handlers: std::collections::BTreeMap<String, String>,
    consts: std::collections::BTreeMap<String, TopLevelConst>,
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
    let jsx = jsx_from_statements(&body.statements).ok_or(ParseError::NoJsxReturned)?;
    let root = convert_element(jsx, ctx)?;
    Ok(Component { name, root, loader, paths, handlers, consts, seo, schema, head })
}

/// El único elemento JSX que devuelve un cuerpo de función
/// (`return (<jsx/>);`) — compartido con la expansión de componentes.
pub(crate) fn jsx_from_statements<'a, 'b>(statements: &'b [Statement<'a>]) -> Option<&'b JSXElement<'a>> {
    for stmt in statements {
        if let Statement::ReturnStatement(ret_stmt) = stmt {
            return jsx_from_expression(ret_stmt.argument.as_ref()?);
        }
    }
    None
}

pub(crate) fn jsx_from_expression<'a, 'b>(expr: &'b Expression<'a>) -> Option<&'b JSXElement<'a>> {
    match unwrap_parens(expr) {
        Expression::JSXElement(jsx) => Some(jsx),
        _ => None,
    }
}

fn unwrap_parens<'a, 'b>(expr: &'b Expression<'a>) -> &'b Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(inner) => unwrap_parens(&inner.expression),
        other => other,
    }
}
