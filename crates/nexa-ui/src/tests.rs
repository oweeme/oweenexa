use nexa_ast::{Attr, AttrValue};
use nexa_ir::{Classification, IrNodeKind};

use super::*;

fn element(tag: &str, attrs: Vec<Attr>, children: Vec<IrNode>) -> IrNode {
    IrNode {
        id: 0,
        classification: Classification::Static,
        kind: IrNodeKind::Element { tag: tag.into(), attrs, events: vec![], island: None, children },
    }
}

fn static_class(value: &str) -> Attr {
    Attr { name: "class".into(), value: Some(AttrValue::Static(value.into())) }
}

#[test]
fn a_page_with_no_nx_classes_ships_no_css_at_all() {
    let root = element("main", vec![], vec![element("h1", vec![], vec![])]);
    assert_eq!(build_stylesheet(&root), None);
}

#[test]
fn using_the_button_includes_tokens_and_button_css_only() {
    let root = element("button", vec![static_class("nx-btn nx-btn-primary")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains("--nx-color-primary"), "debe incluir los design tokens");
    assert!(css.contains(".nx-btn {"));
    assert!(!css.contains(".nx-input {"), "no se usó <input>, no debería incluirse");
    assert!(!css.contains(".nx-card {"));
    assert!(!css.contains(".nx-dialog {"));
}

#[test]
fn using_two_components_includes_both_and_nothing_else() {
    let root = element(
        "div",
        vec![],
        vec![
            element("button", vec![static_class("nx-btn")], vec![]),
            element("input", vec![static_class("nx-input")], vec![]),
        ],
    );
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-btn {"));
    assert!(css.contains(".nx-input {"));
    assert!(!css.contains(".nx-card {"));
    assert!(!css.contains(".nx-dialog {"));
}

#[test]
fn dynamic_class_values_are_not_analyzed() {
    // `class={expr}` no se puede resolver sin datos reales: se ignora,
    // no se incluye CSS a ciegas.
    let root = element(
        "button",
        vec![Attr {
            name: "class".into(),
            value: Some(AttrValue::Dynamic(nexa_ast::Template::from_expr(nexa_ast::Expr::Identifier(
                "someClass".into(),
            )))),
        }],
        vec![],
    );
    assert_eq!(build_stylesheet(&root), None);
}

#[test]
fn an_unrelated_static_class_does_not_pull_in_any_component() {
    let root = element("div", vec![static_class("my-own-class")], vec![]);
    assert_eq!(build_stylesheet(&root), None);
}

#[test]
fn full_source_includes_tokens_and_every_component_regardless_of_usage() {
    let css = full_source();

    assert!(css.contains("--nx-color-primary"));
    assert!(css.contains(".nx-btn {"));
    assert!(css.contains(".nx-input {"));
    assert!(css.contains(".nx-card {"));
    assert!(css.contains(".nx-dialog {"));
}

#[test]
fn full_source_is_deterministic() {
    assert_eq!(full_source(), full_source());
}
