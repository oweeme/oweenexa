use std::collections::BTreeMap;

use super::*;
use nexa_ast::{Attr, AttrValue, Event, Expr, Island, JsonTemplate, Template, TemplatePart};
use nexa_ir::{Classification, DependencyGraph};
use serde_json::json;

fn text(id: usize, value: &str) -> IrNode {
    IrNode {
        id,
        classification: Classification::Static,
        kind: IrNodeKind::Text(value.to_string()),
    }
}

fn expression(id: usize, expr: Expr) -> IrNode {
    IrNode {
        id,
        classification: Classification::Dynamic,
        kind: IrNodeKind::Expression(expr),
    }
}

fn translate(id: usize, key: &str) -> IrNode {
    IrNode {
        id,
        classification: Classification::Dynamic,
        kind: IrNodeKind::Translate(key.to_string()),
    }
}

fn element(
    id: usize,
    classification: Classification,
    tag: &str,
    attrs: Vec<Attr>,
    events: Vec<Event>,
    children: Vec<IrNode>,
) -> IrNode {
    IrNode {
        id,
        classification,
        kind: IrNodeKind::Element {
            tag: tag.to_string(),
            attrs,
            events,
            island: None,
            children,
        },
    }
}

fn island_element(
    id: usize,
    tag: &str,
    island: Island,
    children: Vec<IrNode>,
) -> IrNode {
    IrNode {
        id,
        classification: Classification::Island,
        kind: IrNodeKind::Element {
            tag: tag.to_string(),
            attrs: vec![],
            events: vec![],
            island: Some(island),
            children,
        },
    }
}

fn dotted(root: &str, property: &str) -> Expr {
    Expr::Member {
        object: Box::new(Expr::Identifier(root.into())),
        property: property.into(),
    }
}

fn ctx_with_data(data: &serde_json::Value) -> RenderContext<'_> {
    RenderContext {
        data: Some(data),
        params: empty_params(),
        translations: None,
    }
}

fn ctx_with_params(params: &BTreeMap<String, String>) -> RenderContext<'_> {
    RenderContext { data: None, params, translations: None }
}

fn ctx_with_translations(translations: &serde_json::Value) -> RenderContext<'_> {
    RenderContext { data: None, params: empty_params(), translations: Some(translations) }
}

fn empty_params() -> &'static BTreeMap<String, String> {
    static EMPTY: std::sync::OnceLock<BTreeMap<String, String>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(BTreeMap::new)
}

#[test]
fn renders_nested_static_html() {
    let component = IrComponent {
        name: "Home".into(),
        root: element(
            0,
            Classification::Static,
            "main",
            vec![],
            vec![],
            vec![
                element(1, Classification::Static, "h1", vec![], vec![], vec![text(2, "Hello Nexa")]),
                element(
                    3,
                    Classification::Static,
                    "img",
                    vec![Attr {
                        name: "src".into(),
                        value: Some(AttrValue::Static("/logo.png".into())),
                    }],
                    vec![],
                    vec![],
                ),
            ],
        ),
        dependencies: DependencyGraph::new(),
    };

    let html = render_html_document(&component, &RenderContext::empty());
    assert!(html.contains("<h1>Hello Nexa</h1>"));
    assert!(html.contains("<img src=\"/logo.png\">"));
}

#[test]
fn escapes_dynamic_text() {
    let node = text(0, "<script>alert(1)</script>");
    assert_eq!(
        render_node(&node, &RenderContext::empty()),
        "&lt;script&gt;alert(1)&lt;/script&gt;"
    );
}

#[test]
fn renders_dynamic_expression_as_placeholder_comment_when_theres_no_data() {
    let node = expression(0, dotted("data", "name"));
    assert_eq!(render_node(&node, &RenderContext::empty()), "<!--nexa:data.name-->");
}

#[test]
fn renders_non_data_expressions_as_placeholder_even_with_data_present() {
    // Solo `data.*`/`params.*` se resuelven; cualquier otro identificador
    // (`product`, `count`...) sigue siendo un marcador inerte.
    let node = expression(0, dotted("product", "name"));
    let data = json!({ "name": "iPhone 17" });
    assert_eq!(render_node(&node, &ctx_with_data(&data)), "<!--nexa:product.name-->");
}

#[test]
fn renders_real_data_when_available() {
    let node = expression(0, dotted("data", "name"));
    let data = json!({ "name": "iPhone 17" });
    assert_eq!(render_node(&node, &ctx_with_data(&data)), "iPhone 17");
}

#[test]
fn escapes_real_data_values() {
    let node = expression(0, dotted("data", "name"));
    let data = json!({ "name": "<script>alert(1)</script>" });
    assert_eq!(
        render_node(&node, &ctx_with_data(&data)),
        "&lt;script&gt;alert(1)&lt;/script&gt;"
    );
}

#[test]
fn falls_back_to_placeholder_when_the_property_is_missing() {
    let node = expression(0, dotted("data", "description"));
    let data = json!({ "name": "iPhone 17" }); // sin "description"
    assert_eq!(render_node(&node, &ctx_with_data(&data)), "<!--nexa:data.description-->");
}

#[test]
fn resolves_nested_property_paths() {
    let node = expression(
        0,
        Expr::Member {
            object: Box::new(dotted("data", "price")),
            property: "amount".into(),
        },
    );
    let data = json!({ "price": { "amount": 999, "currency": "USD" } });
    assert_eq!(render_node(&node, &ctx_with_data(&data)), "999");
}

#[test]
fn resolves_route_params() {
    let node = expression(0, dotted("params", "slug"));
    let mut params = BTreeMap::new();
    params.insert("slug".to_string(), "iphone-17".to_string());

    assert_eq!(render_node(&node, &ctx_with_params(&params)), "iphone-17");
}

#[test]
fn falls_back_to_placeholder_when_the_param_is_missing() {
    let node = expression(0, dotted("params", "slug"));
    assert_eq!(render_node(&node, &RenderContext::empty()), "<!--nexa:params.slug-->");
}

#[test]
fn resolves_a_translation_key_against_the_locale_dictionary() {
    let node = translate(0, "home.title");
    let dictionary = json!({ "home": { "title": "Bienvenido" } });

    assert_eq!(render_node(&node, &ctx_with_translations(&dictionary)), "Bienvenido");
}

#[test]
fn falls_back_to_placeholder_when_the_translation_key_is_missing() {
    let node = translate(0, "home.subtitle");
    let dictionary = json!({ "home": { "title": "Bienvenido" } });

    assert_eq!(
        render_node(&node, &ctx_with_translations(&dictionary)),
        "<!--nexa:t(home.subtitle)-->"
    );
}

#[test]
fn falls_back_to_placeholder_when_there_is_no_locale_dictionary_at_all() {
    let node = translate(0, "home.title");
    assert_eq!(render_node(&node, &RenderContext::empty()), "<!--nexa:t(home.title)-->");
}

#[test]
fn escapes_resolved_translations() {
    let node = translate(0, "warning");
    let dictionary = json!({ "warning": "<script>alert(1)</script>" });

    assert_eq!(
        render_node(&node, &ctx_with_translations(&dictionary)),
        "&lt;script&gt;alert(1)&lt;/script&gt;"
    );
}

#[test]
fn renders_interactive_element_with_activation_marker() {
    let node = element(
        7,
        Classification::Interactive,
        "button",
        vec![],
        vec![Event {
            name: "click".into(),
            handler: Expr::Identifier("buy".into()),
            strategy: None,
        }],
        vec![text(8, "Comprar")],
    );
    // El HTML solo lleva el id: el evento/estrategia/módulo real vive en
    // el manifiesto de `nexa-activation`, no en el marcado.
    assert_eq!(
        render_node(&node, &RenderContext::empty()),
        "<button data-nexa=\"7\">Comprar</button>"
    );
}

#[test]
fn static_element_has_no_activation_marker() {
    let node = element(0, Classification::Static, "p", vec![], vec![], vec![text(1, "hola")]);
    assert!(!render_node(&node, &RenderContext::empty()).contains("data-nexa"));
}

#[test]
fn resolves_dynamic_attribute_values_against_data() {
    let node = element(
        0,
        Classification::Static,
        "img",
        vec![
            Attr { name: "src".into(), value: Some(AttrValue::Dynamic(Template::from_expr(dotted("data", "image")))) },
            Attr { name: "alt".into(), value: Some(AttrValue::Dynamic(Template::from_expr(dotted("data", "name")))) },
        ],
        vec![],
        vec![],
    );
    let data = json!({ "image": "/iphone.jpg", "name": "iPhone 17" });

    assert_eq!(
        render_node(&node, &ctx_with_data(&data)),
        "<img src=\"/iphone.jpg\" alt=\"iPhone 17\">"
    );
}

#[test]
fn resolves_a_multi_part_dynamic_attribute_like_a_real_template_literal() {
    // Bug real (Fase 15): antes de esto, `href={`/${params.locale}/x`}`
    // no se podía representar en absoluto (`AttrValue::Dynamic` solo
    // aceptaba una única `Expr`) — el atributo quedaba `value: None` y
    // se renderizaba como `<a href>`, sin ningún valor, en silencio.
    let template = Template(vec![
        TemplatePart::Text("/".into()),
        TemplatePart::Expr(dotted("params", "locale")),
        TemplatePart::Text("/products/".into()),
        TemplatePart::Expr(dotted("params", "slug")),
    ]);
    let node = element(
        0,
        Classification::Static,
        "a",
        vec![Attr { name: "href".into(), value: Some(AttrValue::Dynamic(template)) }],
        vec![],
        vec![],
    );

    let mut params = BTreeMap::new();
    params.insert("locale".to_string(), "es".to_string());
    params.insert("slug".to_string(), "iphone-17".to_string());

    assert_eq!(render_node(&node, &ctx_with_params(&params)), "<a href=\"/es/products/iphone-17\"></a>");
}

#[test]
fn omits_a_dynamic_attribute_entirely_when_it_cannot_be_resolved() {
    // Un `src=""` roto es peor que ningún `src`.
    let node = element(
        0,
        Classification::Static,
        "img",
        vec![Attr { name: "src".into(), value: Some(AttrValue::Dynamic(Template::from_expr(dotted("data", "image")))) }],
        vec![],
        vec![],
    );

    let html = render_node(&node, &RenderContext::empty());
    assert_eq!(html, "<img>");
    assert!(!html.contains("src"));
}

#[test]
fn renders_an_island_mount_point_with_specifier_props_and_default_strategy() {
    // Fase 16: el fallback SSR real (aquí, el texto estático) se sigue
    // renderizando dentro del punto de montaje — la isla no reemplaza
    // nada del lado del servidor, solo añade los atributos de montaje.
    let node = island_element(
        0,
        "div",
        Island {
            specifier: "productFilter".into(),
            props: Some(JsonTemplate::Object(vec![(
                "products".into(),
                JsonTemplate::Expr(dotted("data", "products")),
            )])),
            strategy: None,
        },
        vec![text(1, "iPhone 17")],
    );
    let data = json!({ "products": ["iPhone 17", "Pixel 10"] });

    let html = render_node(&node, &ctx_with_data(&data));
    assert_eq!(
        html,
        "<div data-nexa-island=\"productFilter\" data-nexa-props=\"{&quot;products&quot;:[&quot;iPhone 17&quot;,&quot;Pixel 10&quot;]}\" data-nexa-strategy=\"visible\">iPhone 17</div>"
    );
}

#[test]
fn an_island_with_an_explicit_strategy_keeps_it_in_the_html() {
    let node = island_element(
        0,
        "div",
        Island { specifier: "dashboardIsland".into(), props: None, strategy: Some("load".into()) },
        vec![],
    );

    let html = render_node(&node, &RenderContext::empty());
    assert_eq!(html, "<div data-nexa-island=\"dashboardIsland\" data-nexa-strategy=\"load\"></div>");
}

#[test]
fn an_island_keeps_a_whole_number_prop_as_an_integer_not_a_float() {
    // Bug real (Fase 16, encontrado verificando con datos reales contra
    // el backend PHP del ejemplo): `id: 1` en `data-nexa-props={{...}}`
    // se serializaba como `1.0`, no `1`.
    let node = island_element(
        0,
        "div",
        Island {
            specifier: "dashboardIsland".into(),
            props: Some(JsonTemplate::Object(vec![("id".into(), JsonTemplate::Number(1.0))])),
            strategy: None,
        },
        vec![],
    );

    let html = render_node(&node, &RenderContext::empty());
    assert!(html.contains("&quot;id&quot;:1}"));
    assert!(!html.contains("1.0"));
}

#[test]
fn an_island_is_not_marked_as_an_activation_target() {
    // `data-nexa="<id>"` es la clave del `ActivationManifest` de eventos
    // (Fase 5) — una isla no tiene entrada ahí, así que no debe llevarlo.
    let node = island_element(
        3,
        "div",
        Island { specifier: "x".into(), props: None, strategy: None },
        vec![],
    );

    assert!(!render_node(&node, &RenderContext::empty()).contains("data-nexa=\""));
}
