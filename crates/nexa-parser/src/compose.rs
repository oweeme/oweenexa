//! Fase 62 — composición de componentes **en tiempo de compilación**.
//!
//! `<Card title={data.name}>hola</Card>` no crea ningún componente en
//! ejecución ni necesita runtime: el parser *expande* la llamada,
//! sustituyendo en el JSX de `Card` cada `props.title` por lo que la
//! llamada pasó (un literal, `data.x`, `params.x`, `t("...")` o una
//! plantilla), y `{props.children}` por los hijos de la llamada. El
//! resultado es exactamente el mismo árbol de `nexa_ast::Node` que si
//! hubieras escrito el HTML a mano — el analyzer, el renderer y la
//! activación no saben que hubo componentes.
//!
//! Sigue siendo análisis, no ejecución: un componente no corre, solo se
//! lee. Por eso valen las mismas reglas del resto de Nexa Core: un prop
//! solo puede ser un valor simbólico (nunca una función arbitraria ni
//! una expresión calculada), y un evento de un componente se recibe
//! desde la página (`onBuy={buy}` -> `onClick={props.onBuy}`).
//!
//! Qué se reconoce como componente (nombre capitalizado):
//! - una `function Nombre(...)` o `const Nombre = (...) =>` del mismo archivo;
//! - un `import Nombre from "./Nombre"` / `import { Nombre } from "../ui"`
//!   relativo — el [`ImportResolver`] lee el archivo (el parser no hace IO).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, ExportDefaultDeclarationKind, Expression, FormalParameters, ImportDeclarationSpecifier,
    JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElement, JSXExpression, Program, PropertyKey,
    Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

use nexa_ast::{Expr, JsonTemplate, Node, Template, TemplatePart, TopLevelConst, Translate};

use crate::error::ParseError;
use crate::handlers::{find_handlers, find_top_level_consts};
use crate::jsx::{convert_child, convert_element};
use crate::template;
use crate::text::expression_container_to_string;
use crate::translate;

/// Cómo el parser lee el archivo de un componente importado. El parser
/// no toca el disco: `nexa-cli` implementa esto con el sistema de
/// archivos real, los tests con un mapa en memoria.
pub trait ImportResolver {
    /// `from_file`: el archivo que hace el `import`. `specifier`: lo que
    /// está entre comillas (`"../components/Card"`). Devuelve el nombre
    /// resuelto del archivo y su contenido.
    fn resolve(&self, from_file: &str, specifier: &str) -> Option<(String, String)>;
}

/// Sin imports: una página que no importa componentes (el caso de siempre).
pub struct NoImports;

impl ImportResolver for NoImports {
    fn resolve(&self, _from_file: &str, _specifier: &str) -> Option<(String, String)> {
        None
    }
}

/// Lo que se acumula mientras se expanden componentes de otros archivos:
/// sus funciones y constantes de nivel superior, que un `onClick` de ese
/// componente puede necesitar en el chunk de activación.
#[derive(Default)]
pub(crate) struct Shared {
    pub handlers: RefCell<BTreeMap<String, String>>,
    pub consts: RefCell<BTreeMap<String, TopLevelConst>>,
}

#[derive(Clone)]
enum ImportKind {
    Default,
    Named(String),
}

#[derive(Clone)]
struct ImportRef {
    specifier: String,
    kind: ImportKind,
}

/// El valor que una llamada le pasa a un prop.
#[derive(Clone, Debug)]
pub(crate) enum PropValue {
    Text(String),
    Expr(Expr),
    Translate(Translate),
    Template(Template),
}

#[derive(Clone, Default)]
struct PropScope {
    /// `props` en `function Card(props)`.
    object: Option<String>,
    /// nombre local destructurado -> nombre real del prop
    /// (`{ title: t }` -> `t` -> `title`).
    locals: BTreeMap<String, String>,
    values: BTreeMap<String, PropValue>,
    children: Vec<Node>,
    /// Nombres tapados por un `<For>` (su parámetro) dentro del cuerpo.
    shadowed: BTreeSet<String>,
}

/// Contexto de conversión de un archivo: en qué archivo estamos, qué
/// componentes puede ver, y (si es el cuerpo de un componente expandido)
/// qué valores tienen sus props.
#[derive(Clone)]
pub(crate) struct Ctx<'r> {
    resolver: &'r dyn ImportResolver,
    pub(crate) shared: &'r Shared,
    file: Rc<str>,
    source: Rc<str>,
    imports: Rc<BTreeMap<String, ImportRef>>,
    scope: Option<Rc<PropScope>>,
    stack: Rc<Vec<String>>,
    /// Parámetros de los `<For>` que envuelven al nodo actual (`item`).
    pub(crate) loop_vars: Rc<Vec<String>>,
}

const MAX_DEPTH: usize = 32;

impl<'r> Ctx<'r> {
    pub(crate) fn root(resolver: &'r dyn ImportResolver, shared: &'r Shared, file: &str, source: &str, program: &Program) -> Self {
        Ctx {
            resolver,
            shared,
            file: file.into(),
            source: source.into(),
            imports: Rc::new(read_imports(program)),
            scope: None,
            stack: Rc::new(Vec::new()),
            loop_vars: Rc::new(Vec::new()),
        }
    }

    /// El cuerpo de un `<For>`: su parámetro tapa a un prop homónimo y es
    /// una raíz válida para un `<For>` anidado.
    pub(crate) fn enter_loop(&self, item_name: &str) -> Ctx<'r> {
        let mut next = self.clone();
        if let Some(scope) = &self.scope {
            let mut s = (**scope).clone();
            s.shadowed.insert(item_name.to_string());
            next.scope = Some(Rc::new(s));
        }
        let mut vars = (*self.loop_vars).clone();
        vars.push(item_name.to_string());
        next.loop_vars = Rc::new(vars);
        next
    }

    pub(crate) fn in_component(&self) -> bool {
        self.scope.is_some()
    }

    /// Si `expr` apunta a un prop del componente que se está expandiendo:
    /// `(nombre del prop, propiedades restantes)`.
    fn prop_ref<'e>(&self, expr: &'e Expr) -> Option<(String, Vec<&'e str>)> {
        let scope = self.scope.as_ref()?;
        let root = expr.root_identifier();
        if scope.shadowed.contains(root) {
            return None;
        }
        let path = expr.property_path();
        if scope.object.as_deref() == Some(root) {
            let (first, rest) = path.split_first()?;
            return Some((first.to_string(), rest.to_vec()));
        }
        scope.locals.get(root).map(|key| (key.clone(), path))
    }

    /// `props.title` -> el valor concreto que pasó la llamada.
    /// `Ok(None)`: no es un prop (sigue siendo una expresión normal).
    /// `Ok(Some(None))`: es un prop que la llamada no pasó.
    pub(crate) fn substitute(&self, expr: &Expr) -> Result<Option<Option<PropValue>>, ParseError> {
        let Some((key, rest)) = self.prop_ref(expr) else { return Ok(None) };
        let scope = self.scope.as_ref().expect("prop_ref implica scope");
        let Some(value) = scope.values.get(&key) else { return Ok(Some(None)) };
        if rest.is_empty() {
            return Ok(Some(Some(value.clone())));
        }
        match value {
            PropValue::Expr(base) => {
                let mut e = base.clone();
                for prop in rest {
                    e = Expr::Member { object: Box::new(e), property: prop.to_string() };
                }
                Ok(Some(Some(PropValue::Expr(e))))
            }
            _ => Err(ParseError::Unsupported(format!(
                "`{}`: el prop `{key}` se pasó como un valor literal/plantilla, pero el componente \
                 accede a una propiedad suya (`.{}`) — pasá `data.x`/`params.x` si necesitás sus campos",
                expr.path(),
                rest.join(".")
            ))),
        }
    }

    pub(crate) fn is_children_ref(&self, expr: &Expr) -> bool {
        matches!(self.prop_ref(expr), Some((key, rest)) if key == "children" && rest.is_empty())
    }

    pub(crate) fn children(&self) -> Vec<Node> {
        self.scope.as_ref().map(|s| s.children.clone()).unwrap_or_default()
    }

    /// Sustituye los props de una plantilla (`href={\`/p/${props.slug}\`}`).
    pub(crate) fn substitute_template(&self, tpl: &Template) -> Result<Template, ParseError> {
        let mut parts = Vec::new();
        for part in &tpl.0 {
            match part {
                TemplatePart::Text(t) => parts.push(TemplatePart::Text(t.clone())),
                TemplatePart::Translate(tr) => parts.push(TemplatePart::Translate(self.substitute_translate(tr)?)),
                TemplatePart::Expr(e) => match self.substitute(e)? {
                    None => parts.push(TemplatePart::Expr(e.clone())),
                    Some(None) => {} // prop no pasado: no aporta nada
                    Some(Some(v)) => parts.extend(value_to_parts(v)),
                },
            }
        }
        Ok(Template(merge_text(parts)))
    }

    pub(crate) fn substitute_translate(&self, tr: &Translate) -> Result<Translate, ParseError> {
        let mut args = Vec::new();
        for (name, expr) in &tr.args {
            match self.substitute(expr)? {
                None => args.push((name.clone(), expr.clone())),
                Some(None) => {}
                Some(Some(PropValue::Expr(e))) => args.push((name.clone(), e)),
                Some(Some(_)) => {
                    return Err(ParseError::Unsupported(format!(
                        "t(\"{}\", {{ {name}: ... }}): un argumento de traducción que viene de un prop debe ser `data.x`/`params.x`",
                        tr.key
                    )))
                }
            }
        }
        Ok(Translate { key: tr.key.clone(), args })
    }

    pub(crate) fn substitute_json(&self, json: &JsonTemplate) -> Result<JsonTemplate, ParseError> {
        Ok(match json {
            JsonTemplate::Expr(e) => match self.substitute(e)? {
                None => json.clone(),
                Some(None) => JsonTemplate::Null,
                Some(Some(PropValue::Expr(e))) => JsonTemplate::Expr(e),
                Some(Some(PropValue::Text(t))) => JsonTemplate::String(t),
                Some(Some(PropValue::Translate(t))) => JsonTemplate::Translate(t),
                Some(Some(PropValue::Template(t))) => JsonTemplate::TextTemplate(t),
            },
            JsonTemplate::Translate(t) => JsonTemplate::Translate(self.substitute_translate(t)?),
            JsonTemplate::TextTemplate(t) => JsonTemplate::TextTemplate(self.substitute_template(t)?),
            JsonTemplate::Array(items) => {
                JsonTemplate::Array(items.iter().map(|i| self.substitute_json(i)).collect::<Result<_, _>>()?)
            }
            JsonTemplate::Object(entries) => JsonTemplate::Object(
                entries.iter().map(|(k, v)| Ok((k.clone(), self.substitute_json(v)?))).collect::<Result<_, ParseError>>()?,
            ),
            other => other.clone(),
        })
    }
}

fn value_to_parts(value: PropValue) -> Vec<TemplatePart> {
    match value {
        PropValue::Text(t) => vec![TemplatePart::Text(t)],
        PropValue::Expr(e) => vec![TemplatePart::Expr(e)],
        PropValue::Translate(t) => vec![TemplatePart::Translate(t)],
        PropValue::Template(t) => t.0,
    }
}

/// Une los `Text` contiguos de una plantilla.
fn merge_text(parts: Vec<TemplatePart>) -> Vec<TemplatePart> {
    let mut out: Vec<TemplatePart> = Vec::new();
    for part in parts {
        if let (TemplatePart::Text(next), Some(TemplatePart::Text(prev))) = (&part, out.last_mut()) {
            prev.push_str(next);
            continue;
        }
        out.push(part);
    }
    out
}

/// Un valor de prop a los nodos que lo representan como hijo.
pub(crate) fn value_to_nodes(value: PropValue) -> Vec<Node> {
    match value {
        PropValue::Text(t) => vec![Node::Text(t)],
        PropValue::Expr(e) => vec![Node::Expression(e)],
        PropValue::Translate(t) => vec![Node::Translate(t)],
        PropValue::Template(t) => t
            .0
            .into_iter()
            .map(|part| match part {
                TemplatePart::Text(t) => Node::Text(t),
                TemplatePart::Expr(e) => Node::Expression(e),
                TemplatePart::Translate(t) => Node::Translate(t),
            })
            .collect(),
    }
}

fn read_imports(program: &Program) -> BTreeMap<String, ImportRef> {
    let mut map = BTreeMap::new();
    for stmt in program.body.iter() {
        let Statement::ImportDeclaration(decl) = stmt else { continue };
        let specifier = decl.source.value.as_str().to_string();
        let Some(specifiers) = &decl.specifiers else { continue };
        for spec in specifiers.iter() {
            match spec {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(d) => {
                    map.insert(d.local.name.as_str().to_string(), ImportRef { specifier: specifier.clone(), kind: ImportKind::Default });
                }
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    map.insert(
                        s.local.name.as_str().to_string(),
                        ImportRef { specifier: specifier.clone(), kind: ImportKind::Named(s.imported.name().as_str().to_string()) },
                    );
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
        }
    }
    map
}

// ---------------------------------------------------------------------
// Encontrar la definición de un componente
// ---------------------------------------------------------------------

struct Callee<'a> {
    params: &'a FormalParameters<'a>,
    root: &'a JSXElement<'a>,
}

fn find_callee<'a>(program: &'a Program<'a>, name: &str, default: bool) -> Option<Callee<'a>> {
    for stmt in program.body.iter() {
        match stmt {
            Statement::FunctionDeclaration(f) if !default && f.id.as_ref().is_some_and(|i| i.name == name) => {
                return callee_from_function(f);
            }
            Statement::VariableDeclaration(v) if !default => {
                if let Some(c) = callee_from_variable(v, name) {
                    return Some(c);
                }
            }
            Statement::ExportDeclaration(export) => {
                if default {
                    continue;
                }
                match &export.declaration {
                    oxc_ast::ast::Declaration::FunctionDeclaration(f) if f.id.as_ref().is_some_and(|i| i.name == name) => {
                        return callee_from_function(f);
                    }
                    oxc_ast::ast::Declaration::VariableDeclaration(v) => {
                        if let Some(c) = callee_from_variable(v, name) {
                            return Some(c);
                        }
                    }
                    _ => {}
                }
            }
            Statement::ExportDefaultDeclaration(export) if default => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(f) = &export.declaration {
                    return callee_from_function(f);
                }
            }
            _ => {}
        }
    }
    None
}

fn callee_from_function<'a>(f: &'a oxc_ast::ast::Function<'a>) -> Option<Callee<'a>> {
    let body = f.body.as_ref()?;
    let root = crate::component::jsx_from_statements(&body.statements)?;
    Some(Callee { params: &f.params, root })
}

fn callee_from_variable<'a>(v: &'a oxc_ast::ast::VariableDeclaration<'a>, name: &str) -> Option<Callee<'a>> {
    for decl in v.declarations.iter() {
        let BindingPattern::BindingIdentifier(id) = &decl.id else { continue };
        if id.name != name {
            continue;
        }
        let Some(Expression::ArrowFunctionExpression(arrow)) = &decl.init else { continue };
        let root = match arrow.get_expression() {
            Some(expr) => crate::component::jsx_from_expression(expr)?,
            None => match &arrow.body {
                oxc_ast::ast::ArrowFunctionBody::FunctionBody(body) => crate::component::jsx_from_statements(&body.statements)?,
                _ => return None,
            },
        };
        return Some(Callee { params: &arrow.params, root });
    }
    None
}

/// `(props)` -> objeto `props`; `({ a, b: c })` -> locales; `()` -> nada.
fn read_params(params: &FormalParameters, component: &str) -> Result<(Option<String>, BTreeMap<String, String>), ParseError> {
    let Some(first) = params.items.first() else { return Ok((None, BTreeMap::new())) };
    if params.items.len() > 1 || params.rest.is_some() {
        return Err(ParseError::Unsupported(format!("`{component}`: un componente recibe un único parámetro (props)")));
    }
    match &first.pattern {
        BindingPattern::BindingIdentifier(id) => Ok((Some(id.name.as_str().to_string()), BTreeMap::new())),
        BindingPattern::ObjectPattern(obj) => {
            if obj.rest.is_some() {
                return Err(ParseError::Unsupported(format!("`{component}`: `...resto` en los props no está soportado")));
            }
            let mut locals = BTreeMap::new();
            for prop in obj.properties.iter() {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str().to_string(),
                    _ => return Err(ParseError::Unsupported(format!("`{component}`: nombre de prop no soportado"))),
                };
                let local = match &prop.value {
                    BindingPattern::BindingIdentifier(id) => id.name.as_str().to_string(),
                    _ => {
                        return Err(ParseError::Unsupported(format!(
                            "`{component}`: los props no admiten valores por defecto ni destructuring anidado (`{key}`)"
                        )))
                    }
                };
                locals.insert(local, key);
            }
            Ok((None, locals))
        }
        _ => Err(ParseError::Unsupported(format!("`{component}`: forma de parámetros no soportada"))),
    }
}

// ---------------------------------------------------------------------
// Expandir una llamada
// ---------------------------------------------------------------------

/// `<Card title="x">hijos</Card>` -> el árbol del cuerpo de `Card` con sus props sustituidos.
pub(crate) fn expand_call(jsx: &JSXElement, name: &str, ctx: &Ctx) -> Result<Node, ParseError> {
    // 1. ¿Dónde está definido?
    let (file, source, def_name, is_default) = locate(name, ctx)?;

    let key = format!("{file}#{def_name}");
    if ctx.stack.contains(&key) || ctx.stack.len() >= MAX_DEPTH {
        return Err(ParseError::Unsupported(format!(
            "el componente `{name}` se incluye a sí mismo (directa o indirectamente) — la composición en Nexa Core no admite recursión"
        )));
    }

    // 2. Los valores que pasa esta llamada (evaluados en el contexto del que llama).
    let mut values = BTreeMap::new();
    for item in jsx.opening_element.attributes.iter() {
        let JSXAttributeItem::Attribute(attr) = item else {
            return Err(ParseError::Unsupported(format!("`<{name} {{...spread}}>`: el spread de props no está soportado")));
        };
        let JSXAttributeName::Identifier(prop) = &attr.name else {
            return Err(ParseError::Unsupported(format!("`<{name}>`: nombre de prop no soportado")));
        };
        let prop = prop.name.as_str().to_string();
        match &attr.value {
            None => {
                values.insert(prop, PropValue::Text(String::new()));
            }
            Some(JSXAttributeValue::StringLiteral(s)) => {
                values.insert(prop, PropValue::Text(s.value.as_str().to_string()));
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(v) = prop_value(&container.expression, container, ctx, name, &prop)? {
                    values.insert(prop, v);
                }
            }
            Some(_) => return Err(ParseError::Unsupported(format!("`<{name} {prop}=...>`: valor de prop no soportado"))),
        }
    }

    // 3. Los hijos de la llamada, también en el contexto del que llama.
    let mut children = Vec::new();
    for child in jsx.children.iter() {
        children.extend(convert_child(child, ctx)?);
    }

    // 4. Leer el archivo del componente y convertir su cuerpo con esos props.
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&file).map_err(|e| ParseError::Syntax(format!("{file}: {e:?}")))?;
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if parsed.diagnostics.has_errors() {
        let msg = parsed.diagnostics.errors().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(ParseError::Syntax(format!("{file}: {msg}")));
    }
    let program = parsed.program;

    let callee = find_callee(&program, &def_name, is_default).ok_or_else(|| {
        ParseError::Unsupported(format!(
            "no se encontró el componente `{def_name}` en {file} (¿es una `function {def_name}` o `const {def_name} = (...) => (<jsx/>)` que devuelve un único elemento JSX?)"
        ))
    })?;
    let (object, locals) = read_params(callee.params, name)?;

    // Las funciones/constantes de ese archivo que un evento suyo pueda usar.
    merge_shared(ctx.shared, find_handlers(&program, &source), find_top_level_consts(&program, &source), &file)?;

    let mut stack = (*ctx.stack).clone();
    stack.push(key);
    let inner = Ctx {
        resolver: ctx.resolver,
        shared: ctx.shared,
        file: file.as_str().into(),
        source: source.as_str().into(),
        imports: Rc::new(read_imports(&program)),
        scope: Some(Rc::new(PropScope { object, locals, values, children, shadowed: BTreeSet::new() })),
        stack: Rc::new(stack),
        loop_vars: ctx.loop_vars.clone(),
    };

    convert_element(callee.root, &inner)
}

fn locate(name: &str, ctx: &Ctx) -> Result<(String, String, String, bool), ParseError> {
    if let Some(import) = ctx.imports.get(name) {
        let (file, source) = ctx.resolver.resolve(&ctx.file, &import.specifier).ok_or_else(|| {
            ParseError::Unsupported(format!(
                "no se pudo leer el componente `{name}` de \"{}\" — solo se importan componentes locales (rutas relativas a un .tsx/.ts)",
                import.specifier
            ))
        })?;
        return Ok(match &import.kind {
            ImportKind::Default => (file, source, name.to_string(), true),
            ImportKind::Named(real) => (file, source, real.clone(), false),
        });
    }
    // Mismo archivo.
    let source = ctx.source.to_string();
    let allocator = Allocator::default();
    let st = SourceType::from_path(&*ctx.file).map_err(|e| ParseError::Syntax(format!("{e:?}")))?;
    let parsed = Parser::new(&allocator, &source, st).parse();
    if find_callee(&parsed.program, name, false).is_some() {
        return Ok((ctx.file.to_string(), source, name.to_string(), false));
    }
    Err(ParseError::Unsupported(format!(
        "el componente `<{name}>` no está definido: no es una función de este archivo ni un `import` local"
    )))
}

fn merge_shared(
    shared: &Shared,
    handlers: BTreeMap<String, String>,
    consts: BTreeMap<String, TopLevelConst>,
    file: &str,
) -> Result<(), ParseError> {
    let mut h = shared.handlers.borrow_mut();
    for (name, code) in handlers {
        match h.get(&name) {
            Some(existing) if *existing != code => {
                return Err(ParseError::Unsupported(format!(
                    "la función `{name}` está definida distinto en dos archivos (uno es {file}) — los eventos de componentes se resuelven por nombre; renombrá una de las dos"
                )))
            }
            _ => {
                h.insert(name, code);
            }
        }
    }
    let mut c = shared.consts.borrow_mut();
    for (name, value) in consts {
        c.entry(name).or_insert(value);
    }
    Ok(())
}

/// El valor de un `prop={...}` en una llamada.
fn prop_value(
    expression: &JSXExpression,
    container: &oxc_ast::ast::JSXExpressionContainer,
    ctx: &Ctx,
    component: &str,
    prop: &str,
) -> Result<Option<PropValue>, ParseError> {
    if let Some(text) = expression_container_to_string(container) {
        return Ok(Some(PropValue::Text(text)));
    }
    if let Some(tr) = translate::from_jsx_expression(expression) {
        return Ok(Some(PropValue::Translate(ctx.substitute_translate(&tr)?)));
    }
    let Some(tpl) = template::from_jsx_text_expression(expression) else {
        return Err(ParseError::Unsupported(format!(
            "`<{component} {prop}={{...}}>`: un prop solo puede ser un literal, `data.x`, `params.x`, `t(\"...\")`, una plantilla de texto o (para un evento) el nombre de una función — nunca una expresión calculada"
        )));
    };
    let tpl = ctx.substitute_template(&tpl)?;
    Ok(match tpl.0.as_slice() {
        [] => None, // era un prop de quien llama que tampoco se pasó
        [TemplatePart::Text(t)] => Some(PropValue::Text(t.clone())),
        [TemplatePart::Expr(e)] => Some(PropValue::Expr(e.clone())),
        [TemplatePart::Translate(t)] => Some(PropValue::Translate(t.clone())),
        _ => Some(PropValue::Template(tpl)),
    })
}
