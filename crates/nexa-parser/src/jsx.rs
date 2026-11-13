//! Conversión de elementos, atributos, eventos e hijos JSX al AST de Nexa.

use oxc_ast::ast::{
    JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement,
    JSXElementName, JSXExpressionContainer,
};

use nexa_ast::{Attr, AttrValue, Element, Event, Island, Node};

use crate::compose::{self, Ctx, PropValue};
use crate::error::ParseError;
use crate::expr::from_jsx_expression;
use crate::object_literal::from_object_expression;
use crate::template;
use crate::text::{expression_container_to_string, normalize_jsx_text};
use crate::translate;

pub(crate) fn convert_element(jsx: &JSXElement, ctx: &Ctx) -> Result<Node, ParseError> {
    // `<For>` (Fase 30): un nombre capitalizado en JSX siempre llega como
    // `IdentifierReference` (`<Apple/>` referencia un binding de JS), no
    // como `Identifier` (reservado a tags en minúscula, `<div/>`) — por
    // eso se intercepta ANTES del match de abajo, que de otro modo
    // rechazaría cualquier nombre capitalizado como "composición de
    // componentes". `<For>` no es un componente ni un elemento HTML: es
    // un patrón sintáctico reconocido por nombre exacto, igual que
    // `t(...)` se reconoce por ser una llamada a un identificador exacto.
    if let JSXElementName::IdentifierReference(id) = &jsx.opening_element.name {
        if id.name.as_str() == "For" {
            return crate::for_loop::convert_for(jsx, ctx);
        }
        if id.name.as_str() == "Show" {
            return crate::show::convert_show(jsx, ctx);
        }
        // Fase 62: cualquier otro nombre capitalizado es una llamada a un
        // componente, que se expande en tiempo de compilación.
        return compose::expand_call(jsx, id.name.as_str(), ctx);
    }

    let tag = match &jsx.opening_element.name {
        JSXElementName::Identifier(id) => id.name.as_str().to_string(),
        other => {
            return Err(ParseError::Unsupported(format!(
                "`<{other:?}>`: un componente con punto (`<UI.Card>`) no está soportado — importalo por nombre (`import {{ Card }} from ...`)"
            )))
        }
    };

    let mut attrs = Vec::new();
    let mut events = Vec::new();
    let mut strategy_override = None;
    let mut island_specifier = None;
    let mut island_props = None;

    for item in jsx.opening_element.attributes.iter() {
        if let JSXAttributeItem::Attribute(attr) = item {
            if let Some(strategy) = read_strategy_attribute(attr) {
                strategy_override = Some(strategy);
                continue;
            }
            if let Some(specifier) = read_island_attribute(attr) {
                island_specifier = Some(specifier);
                continue;
            }
            if let Some(props) = read_island_props_attribute(attr) {
                island_props = Some(ctx.substitute_json(&props)?);
                continue;
            }
            match convert_event(attr, ctx)? {
                EventOutcome::Event(event) => events.push(event),
                EventOutcome::Omitted => {}
                EventOutcome::NotAnEvent => attrs.extend(convert_attribute(attr, ctx)?),
            }
            // `convert_attribute` devuelve `None` si un `={...}` no se pudo
            // convertir a ningún `AttrValue` (o si es un prop de un
            // componente que la llamada no pasó): el atributo entero se
            // omite, no se guarda como si fuera un atributo booleano
            // (`<a href>` sin valor sería peor que no tener `href`).
        }
        // `{...spread}`: ignorado por ahora (no hay props todavía).
    }

    // `data-nexa-strategy="idle"` no es un atributo HTML: es una pista para
    // el analyzer de activación (Fase 5) sobre cómo activar el/los
    // eventos de este elemento (o, desde la Fase 16, la isla del mismo).
    if let Some(strategy) = &strategy_override {
        for event in &mut events {
            event.strategy = Some(strategy.clone());
        }
    }

    let island = island_specifier.map(|specifier| Island {
        specifier,
        props: island_props,
        strategy: strategy_override,
    });

    let mut children = Vec::new();
    for child in jsx.children.iter() {
        children.extend(convert_child(child, ctx)?);
    }

    Ok(Node::Element(Element {
        tag,
        attrs,
        events,
        island,
        children,
    }))
}

enum EventOutcome {
    Event(Event),
    /// Un evento cuyo prop la llamada no pasó: el elemento queda sin él.
    Omitted,
    NotAnEvent,
}

/// Reconoce atributos tipo `onClick={handler}` (convención JSX: `on` +
/// mayúscula) cuyo valor sea una expresión identificadora, y los convierte
/// en un [`Event`] en vez de en un atributo HTML normal. Dentro de un
/// componente, `onClick={props.onBuy}` toma el handler de la llamada.
fn convert_event(attr: &JSXAttribute, ctx: &Ctx) -> Result<EventOutcome, ParseError> {
    let JSXAttributeName::Identifier(id) = &attr.name else {
        return Ok(EventOutcome::NotAnEvent);
    };
    let Some(event_name) = event_name_from_attr(id.name.as_str()) else {
        return Ok(EventOutcome::NotAnEvent);
    };

    let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
        return Ok(EventOutcome::NotAnEvent);
    };
    let Some(handler) = from_jsx_expression(&container.expression) else {
        return Ok(EventOutcome::NotAnEvent);
    };

    let handler = match ctx.substitute(&handler)? {
        None => handler,
        Some(None) => return Ok(EventOutcome::Omitted),
        Some(Some(PropValue::Expr(e))) => e,
        Some(Some(_)) => {
            return Err(ParseError::Unsupported(format!(
                "`{}`: un evento de un componente debe recibir el nombre de una función de la página (`onBuy={{buy}}`)",
                handler.path()
            )))
        }
    };

    Ok(EventOutcome::Event(Event { name: event_name, handler, strategy: None }))
}

fn event_name_from_attr(name: &str) -> Option<String> {
    let rest = name.strip_prefix("on")?;
    let mut chars = rest.chars();
    let first = chars.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    Some(first.to_ascii_lowercase().to_string() + chars.as_str())
}

/// `data-nexa-strategy="idle"` — anula la estrategia de activación por
/// defecto para los eventos de este elemento (ver `nexa-activation`).
fn read_strategy_attribute(attr: &JSXAttribute) -> Option<String> {
    let JSXAttributeName::Identifier(id) = &attr.name else {
        return None;
    };
    if id.name.as_str() != "data-nexa-strategy" {
        return None;
    }
    match &attr.value {
        Some(JSXAttributeValue::StringLiteral(s)) => Some(s.value.as_str().to_string()),
        _ => None,
    }
}

/// `data-nexa-island="productFilter"` (Fase 16) — marca este elemento
/// como punto de montaje de una isla interactiva. Solo string literal,
/// igual que `data-nexa-strategy`: nada de expresiones dinámicas, porque
/// el specifier tiene que poder resolverse contra `nexa.toml [imports]`
/// en tiempo de compilación.
fn read_island_attribute(attr: &JSXAttribute) -> Option<String> {
    let JSXAttributeName::Identifier(id) = &attr.name else {
        return None;
    };
    if id.name.as_str() != "data-nexa-island" {
        return None;
    }
    match &attr.value {
        Some(JSXAttributeValue::StringLiteral(s)) => Some(s.value.as_str().to_string()),
        _ => None,
    }
}

/// `data-nexa-props={{ products: data.products }}` (Fase 16) — los props
/// iniciales de la isla, resueltos a JSON en tiempo de render con el
/// mismo mecanismo que `seo`/`schema` (`object_literal::from_object_expression`).
/// Nunca se ejecuta nada: solo literales y referencias simbólicas.
fn read_island_props_attribute(attr: &JSXAttribute) -> Option<nexa_ast::JsonTemplate> {
    let JSXAttributeName::Identifier(id) = &attr.name else {
        return None;
    };
    if id.name.as_str() != "data-nexa-props" {
        return None;
    }
    let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
        return None;
    };
    match &container.expression {
        oxc_ast::ast::JSXExpression::ObjectExpression(obj) => Some(from_object_expression(obj)),
        _ => None,
    }
}

/// `None` cuando hubo un `={...}` de verdad pero no se pudo convertir a
/// ningún `AttrValue` — el atributo entero se omite (ver la nota en el
/// llamador), a diferencia de no tener ningún `=` (`<button disabled>`,
/// un atributo booleano real, que sí conserva `value: None` dentro de
/// `Some(Attr)`).
fn convert_attribute(attr: &JSXAttribute, ctx: &Ctx) -> Result<Option<Attr>, ParseError> {
    let name = match &attr.name {
        JSXAttributeName::Identifier(id) => id.name.as_str().to_string(),
        JSXAttributeName::NamespacedName(ns) => {
            format!("{}:{}", ns.namespace.name.as_str(), ns.name.name.as_str())
        }
    };

    let value = match &attr.value {
        None => None,
        Some(JSXAttributeValue::StringLiteral(s)) => {
            Some(AttrValue::Static(s.value.as_str().to_string()))
        }
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            match convert_attribute_expression(container, ctx)? {
                Some(v) => Some(v),
                None => return Ok(None),
            }
        }
        Some(_) => return Ok(None),
    };

    Ok(Some(Attr { name, value }))
}

/// `src={literal}` se pliega a estático; `src={data.image}` o
/// `href={\`/${a}/${b}\`}` (Fase 15: antes solo se soportaba una única
/// referencia, no una plantilla) quedan como una referencia dinámica que
/// el renderer resuelve más tarde (Fase 8) — el mismo tratamiento que
/// reciben los campos de `seo`.
fn convert_attribute_expression(container: &JSXExpressionContainer, ctx: &Ctx) -> Result<Option<AttrValue>, ParseError> {
    if let Some(text) = expression_container_to_string(container) {
        return Ok(Some(AttrValue::Static(text)));
    }
    let Some(tpl) = template::from_jsx_text_expression(&container.expression) else {
        return Ok(None);
    };
    // Fase 62: dentro de un componente, `href={props.url}` toma lo que
    // pasó la llamada. Si nada dinámico queda, el atributo es estático.
    let tpl = ctx.substitute_template(&tpl)?;
    if tpl.0.is_empty() {
        return Ok(None); // un prop que la llamada no pasó: el atributo se omite
    }
    if tpl.0.iter().all(|p| matches!(p, nexa_ast::TemplatePart::Text(_))) {
        let text: String = tpl
            .0
            .iter()
            .filter_map(|p| if let nexa_ast::TemplatePart::Text(t) = p { Some(t.as_str()) } else { None })
            .collect();
        return Ok(Some(AttrValue::Static(text)));
    }
    Ok(Some(AttrValue::Dynamic(tpl)))
}

pub(crate) fn convert_child(child: &JSXChild, ctx: &Ctx) -> Result<Vec<Node>, ParseError> {
    match child {
        JSXChild::Text(text) => Ok(normalize_jsx_text(text.value.as_str()).map(Node::Text).into_iter().collect()),
        JSXChild::Element(el) => Ok(vec![convert_element(el, ctx)?]),
        JSXChild::ExpressionContainer(container) => convert_expression_child(container, ctx),
        JSXChild::Fragment(_) => Err(ParseError::Unsupported(
            "fragmentos (`<>...</>`) como hijos".to_string(),
        )),
        JSXChild::Spread(_) => {
            Err(ParseError::Unsupported("spread children (`{...x}`)".to_string()))
        }
    }
}

/// Un `{expr}` como hijo puede ser un literal (se pliega a texto
/// estático), `t("key")` (Fase 10: `Node::Translate`), o una referencia a
/// una variable (`Node::Expression`, dinámica). Dentro de un componente
/// (Fase 62), `{props.title}` es lo que pasó la llamada y
/// `{props.children}` son sus hijos.
fn convert_expression_child(container: &JSXExpressionContainer, ctx: &Ctx) -> Result<Vec<Node>, ParseError> {
    if let Some(text) = expression_container_to_string(container) {
        return Ok(vec![Node::Text(text)]);
    }
    if let Some(translate) = translate::from_jsx_expression(&container.expression) {
        return Ok(vec![Node::Translate(ctx.substitute_translate(&translate)?)]);
    }
    let Some(expr) = from_jsx_expression(&container.expression) else {
        return Ok(Vec::new());
    };
    if ctx.in_component() {
        if ctx.is_children_ref(&expr) {
            return Ok(ctx.children());
        }
        return Ok(match ctx.substitute(&expr)? {
            None => vec![Node::Expression(expr)],
            Some(None) => Vec::new(),
            Some(Some(value)) => compose::value_to_nodes(value),
        });
    }
    Ok(vec![Node::Expression(expr)])
}
