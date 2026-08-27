//! Conversión de elementos, atributos, eventos e hijos JSX al AST de Nexa.

use oxc_ast::ast::{
    JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement,
    JSXElementName, JSXExpressionContainer,
};

use nexa_ast::{Attr, AttrValue, Element, Event, Island, Node};

use crate::error::ParseError;
use crate::expr::from_jsx_expression;
use crate::object_literal::from_object_expression;
use crate::template;
use crate::text::{expression_container_to_string, normalize_jsx_text};
use crate::translate;

pub(crate) fn convert_element(jsx: &JSXElement) -> Result<Node, ParseError> {
    let tag = match &jsx.opening_element.name {
        JSXElementName::Identifier(id) => id.name.as_str().to_string(),
        other => {
            return Err(ParseError::Unsupported(format!(
                "componentes anidados (`<{other:?}>`) — todavía no hay composición de componentes"
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
                island_props = Some(props);
                continue;
            }
            match convert_event(attr) {
                Some(event) => events.push(event),
                // `convert_attribute` puede devolver `None`: un `={...}`
                // que no se pudo convertir a ningún `AttrValue` (una
                // expresión que ni `template::from_jsx_text_expression`
                // reconoce) se omite entero, no se guarda como si fuera
                // un atributo booleano (`<a href>` sin valor sería peor
                // que no tener `href` en absoluto).
                None => attrs.extend(convert_attribute(attr)),
            }
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
        if let Some(node) = convert_child(child)? {
            children.push(node);
        }
    }

    Ok(Node::Element(Element {
        tag,
        attrs,
        events,
        island,
        children,
    }))
}

/// Reconoce atributos tipo `onClick={handler}` (convención JSX: `on` +
/// mayúscula) cuyo valor sea una expresión identificadora, y los convierte
/// en un [`Event`] en vez de en un atributo HTML normal.
fn convert_event(attr: &JSXAttribute) -> Option<Event> {
    let JSXAttributeName::Identifier(id) = &attr.name else {
        return None;
    };
    let event_name = event_name_from_attr(id.name.as_str())?;

    let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
        return None;
    };
    let handler = from_jsx_expression(&container.expression)?;

    Some(Event {
        name: event_name,
        handler,
        strategy: None,
    })
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
fn convert_attribute(attr: &JSXAttribute) -> Option<Attr> {
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
            Some(convert_attribute_expression(container)?)
        }
        Some(_) => return None,
    };

    Some(Attr { name, value })
}

/// `src={literal}` se pliega a estático; `src={data.image}` o
/// `href={\`/${a}/${b}\`}` (Fase 15: antes solo se soportaba una única
/// referencia, no una plantilla) quedan como una referencia dinámica que
/// el renderer resuelve más tarde (Fase 8) — el mismo tratamiento que
/// reciben los campos de `seo`.
fn convert_attribute_expression(container: &JSXExpressionContainer) -> Option<AttrValue> {
    if let Some(text) = expression_container_to_string(container) {
        return Some(AttrValue::Static(text));
    }
    template::from_jsx_text_expression(&container.expression).map(AttrValue::Dynamic)
}

fn convert_child(child: &JSXChild) -> Result<Option<Node>, ParseError> {
    match child {
        JSXChild::Text(text) => Ok(normalize_jsx_text(text.value.as_str()).map(Node::Text)),
        JSXChild::Element(el) => Ok(Some(convert_element(el)?)),
        JSXChild::ExpressionContainer(container) => Ok(convert_expression_child(container)),
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
/// una variable (`Node::Expression`, dinámica).
fn convert_expression_child(container: &JSXExpressionContainer) -> Option<Node> {
    if let Some(text) = expression_container_to_string(container) {
        return Some(Node::Text(text));
    }
    if let Some(key) = translate::from_jsx_expression(&container.expression) {
        return Some(Node::Translate(key));
    }
    from_jsx_expression(&container.expression).map(Node::Expression)
}
