//! Extrae `export const seo = { ... }` (o `defineSEO({...})`, azúcar
//! equivalente) de una página.

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, Program};

use nexa_ast::SeoConfig;

use crate::declaration::{find_named_export, unwrap_object_literal};
use crate::template::from_text_expression;

pub(crate) fn find_seo(program: &Program) -> Option<SeoConfig> {
    let init = find_named_export(program, "seo")?;
    let obj = unwrap_object_literal(init)?;

    let mut seo = SeoConfig::default();
    for prop in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        let Some(key) = property_key_name(&prop.key) else {
            continue;
        };

        match key.as_str() {
            "title" => seo.title = from_text_expression(&prop.value),
            "description" => seo.description = from_text_expression(&prop.value),
            "canonical" => seo.canonical = from_text_expression(&prop.value),
            "openGraph" => apply_open_graph(&mut seo, &prop.value),
            "twitter" => apply_twitter(&mut seo, &prop.value),
            _ => {}
        }
    }

    Some(seo)
}

fn apply_open_graph(seo: &mut SeoConfig, value: &Expression) {
    let Expression::ObjectExpression(obj) = value else {
        return;
    };
    for_each_property(obj, |key, value| match key {
        "title" => seo.og_title = from_text_expression(value),
        "description" => seo.og_description = from_text_expression(value),
        "image" => seo.og_image = from_text_expression(value),
        _ => {}
    });
}

fn apply_twitter(seo: &mut SeoConfig, value: &Expression) {
    let Expression::ObjectExpression(obj) = value else {
        return;
    };
    for_each_property(obj, |key, value| {
        if key == "card" {
            seo.twitter_card = from_text_expression(value);
        }
    });
}

fn for_each_property<'a>(obj: &'a ObjectExpression<'a>, mut f: impl FnMut(&str, &Expression<'a>)) {
    for prop in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if let Some(key) = property_key_name(&prop.key) {
            f(&key, &prop.value);
        }
    }
}

fn property_key_name(key: &PropertyKey) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str().to_string()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str().to_string()),
        _ => None,
    }
}
