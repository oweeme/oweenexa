use std::collections::BTreeMap;

use serde_json::json;

use nexa_ast::{Expr, JsonTemplate, SeoConfig, Template, TemplatePart};
use nexa_ir::{Classification, IrNode, IrNodeKind};

use super::*;

fn data_dot(property: &str) -> Expr {
    Expr::Member {
        object: Box::new(Expr::Identifier("data".into())),
        property: property.into(),
    }
}

fn ctx_with_data(data: &serde_json::Value) -> SeoContext<'_> {
    SeoContext { data: Some(data), params: empty_params(), translations: None }
}

fn empty_params() -> &'static BTreeMap<String, String> {
    static EMPTY: std::sync::OnceLock<BTreeMap<String, String>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(BTreeMap::new)
}

#[test]
fn render_head_includes_static_and_resolved_fields() {
    let seo = SeoConfig {
        title: Some(Template(vec![
            TemplatePart::Expr(data_dot("name")),
            TemplatePart::Text(" | Oweeme".into()),
        ])),
        description: Some(Template::literal("Descripción fija")),
        canonical: Some(Template::literal("/producto/iphone-17")),
        ..Default::default()
    };
    let data = json!({ "name": "iPhone 17" });

    let head = render_head(Some(&seo), &ctx_with_data(&data));

    assert!(head.contains("<title>iPhone 17 | Oweeme</title>"));
    assert!(head.contains("<meta name=\"description\" content=\"Descripción fija\">"));
    assert!(head.contains("<link rel=\"canonical\" href=\"/producto/iphone-17\">"));
}

#[test]
fn render_head_escapes_resolved_values() {
    let seo = SeoConfig {
        title: Some(Template::literal("<script>alert(1)</script>")),
        ..Default::default()
    };

    let head = render_head(Some(&seo), &SeoContext::empty());
    assert!(!head.contains("<script>alert"));
    assert!(head.contains("&lt;script&gt;"));
}

#[test]
fn render_head_omits_unresolved_fields_instead_of_guessing() {
    // `title` depende de `data.name`, pero no hay datos: no debe inventarse nada.
    let seo = SeoConfig {
        title: Some(Template(vec![TemplatePart::Expr(data_dot("name"))])),
        ..Default::default()
    };

    let head = render_head(Some(&seo), &SeoContext::empty());
    assert!(!head.contains("<title>"));
}

#[test]
fn render_head_with_no_seo_declaration_is_empty() {
    assert_eq!(render_head(None, &SeoContext::empty()), "");
}

#[test]
fn render_schema_script_adds_context_and_renames_type() {
    let schema = JsonTemplate::Object(vec![
        ("type".into(), JsonTemplate::String("Product".into())),
        ("name".into(), JsonTemplate::Expr(data_dot("name"))),
        (
            "offers".into(),
            JsonTemplate::Object(vec![
                ("price".into(), JsonTemplate::Expr(data_dot("price"))),
                ("priceCurrency".into(), JsonTemplate::String("USD".into())),
            ]),
        ),
    ]);
    let data = json!({ "name": "iPhone 17", "price": 999 });

    let script = render_schema_script(Some(&schema), &ctx_with_data(&data)).unwrap();

    assert!(script.starts_with("<script type=\"application/ld+json\">"));
    assert!(script.contains("\"@context\":\"https://schema.org\""));
    assert!(script.contains("\"@type\":\"Product\""));
    assert!(!script.contains("\"type\":\"Product\""));
    assert!(script.contains("\"name\":\"iPhone 17\""));
    assert!(script.contains("\"price\":999"), "el precio debe seguir siendo un número JSON");
}

#[test]
fn render_schema_script_respects_an_explicit_at_type() {
    let schema = JsonTemplate::Object(vec![
        ("@type".into(), JsonTemplate::String("Article".into())),
        ("type".into(), JsonTemplate::String("esto no debería sobrescribir @type".into())),
    ]);

    let script = render_schema_script(Some(&schema), &SeoContext::empty()).unwrap();
    assert!(script.contains("\"@type\":\"Article\""));
}

#[test]
fn render_schema_script_is_none_without_a_declaration() {
    assert_eq!(render_schema_script(None, &SeoContext::empty()), None);
}

#[test]
fn render_schema_script_keeps_a_whole_number_literal_as_an_integer() {
    // Bug real (Fase 16, encontrado verificando con datos reales): un
    // literal entero escrito en `.tsx` (`price: 999`) llega como `f64`
    // — sin el arreglo, `serde_json::Number::from_f64` lo serializaba
    // como `999.0`, no `999`.
    let schema = JsonTemplate::Object(vec![("price".into(), JsonTemplate::Number(999.0))]);
    let script = render_schema_script(Some(&schema), &SeoContext::empty()).unwrap();

    assert!(script.contains("\"price\":999"));
    assert!(!script.contains("999.0"));
}

fn img_without_alt(id: usize) -> IrNode {
    IrNode {
        id,
        classification: Classification::Static,
        kind: IrNodeKind::Element {
            tag: "img".into(),
            attrs: vec![],
            events: vec![],
            island: None,
            children: vec![],
        },
    }
}

fn img_with_alt(id: usize) -> IrNode {
    IrNode {
        id,
        classification: Classification::Static,
        kind: IrNodeKind::Element {
            tag: "img".into(),
            attrs: vec![nexa_ast::Attr {
                name: "alt".into(),
                value: Some(nexa_ast::AttrValue::Static("una descripción".into())),
            }],
            events: vec![],
            island: None,
            children: vec![],
        },
    }
}

#[test]
fn analyze_warns_about_missing_alt_and_missing_seo_fields() {
    let warnings = analyze(&img_without_alt(0), None);

    assert!(warnings.iter().any(|w| w.code == "NEXA-A11Y-001"));
    assert!(warnings.iter().any(|w| w.code == "NEXA-SEO-001"));
    assert!(warnings.iter().any(|w| w.code == "NEXA-SEO-002"));
    assert!(warnings.iter().any(|w| w.code == "NEXA-SEO-003"));
}

#[test]
fn analyze_is_clean_when_everything_is_declared() {
    let seo = SeoConfig {
        title: Some(Template::literal("t")),
        description: Some(Template::literal("d")),
        canonical: Some(Template::literal("/c")),
        ..Default::default()
    };

    let warnings = analyze(&img_with_alt(0), Some(&seo));
    assert!(warnings.is_empty());
}
