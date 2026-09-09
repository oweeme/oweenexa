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
fn using_the_drawer_includes_tokens_and_drawer_css_only() {
    let root = element("div", vec![static_class("nx-drawer")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-drawer {"));
    assert!(!css.contains(".nx-dialog {"), "drawer y dialog son familias distintas");
    assert!(!css.contains(".nx-btn {"));
}

#[test]
fn using_a_layout_utility_class_includes_only_its_own_family() {
    let root = element("div", vec![static_class("nx-flex nx-flex-col nx-gap-2")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-flex {"));
    assert!(css.contains(".nx-flex-col {"));
    assert!(css.contains(".nx-gap-2 {"));
    assert!(!css.contains(".nx-grid {"), "grid es una familia distinta de flex/gap");
    assert!(!css.contains(".nx-stack "), "stack es una familia distinta de flex/gap");
    assert!(!css.contains(".nx-btn {"));
}

#[test]
fn using_grid_or_stack_alone_does_not_pull_in_flex_or_gap() {
    let root = element(
        "div",
        vec![],
        vec![
            element("div", vec![static_class("nx-grid nx-grid-cols-3")], vec![]),
            element("div", vec![static_class("nx-stack")], vec![]),
        ],
    );
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-grid {"));
    assert!(css.contains(".nx-grid-cols-3 {"));
    assert!(css.contains(".nx-stack > * + * {"));
    assert!(!css.contains(".nx-flex {"));
    assert!(!css.contains(".nx-gap-1 {"));
}

#[test]
fn using_the_table_includes_tokens_and_table_css_only() {
    let root = element("table", vec![static_class("nx-table nx-table-zebra")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-table {"));
    assert!(css.contains(".nx-table-zebra "));
    assert!(!css.contains(".nx-badge {"));
    assert!(!css.contains(".nx-card {"));
}

#[test]
fn using_the_badge_includes_tokens_and_badge_css_only() {
    let root = element("span", vec![static_class("nx-badge nx-badge-success")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-badge {"));
    assert!(css.contains("--nx-color-success"));
    assert!(!css.contains(".nx-avatar {"));
    assert!(!css.contains(".nx-table {"));
}

#[test]
fn using_the_avatar_includes_tokens_and_avatar_css_only() {
    let root = element("span", vec![static_class("nx-avatar nx-avatar-lg")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-avatar {"));
    assert!(!css.contains(".nx-breadcrumbs {"));
}

#[test]
fn using_breadcrumbs_includes_tokens_and_breadcrumbs_css_only() {
    let root = element("nav", vec![static_class("nx-breadcrumbs")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-breadcrumbs {"));
    assert!(css.contains(".nx-breadcrumbs-link"));
    assert!(!css.contains(".nx-alert {"));
}

#[test]
fn using_the_alert_includes_tokens_and_alert_css_only() {
    let root = element("div", vec![static_class("nx-alert nx-alert-warning")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-alert {"));
    assert!(css.contains("--nx-color-warning"));
    assert!(!css.contains(".nx-divider {"));
}

#[test]
fn using_the_divider_includes_tokens_and_divider_css_only() {
    let root = element("hr", vec![static_class("nx-divider")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-divider {"));
    assert!(!css.contains(".nx-table {"));
}

#[test]
fn using_the_tooltip_includes_tokens_and_tooltip_css_only() {
    let root = element("span", vec![static_class("nx-tooltip")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-tooltip {"));
    assert!(!css.contains(".nx-progress {"));
}

#[test]
fn using_the_progress_bar_includes_tokens_and_progress_css_only() {
    let root = element("div", vec![static_class("nx-progress")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-progress {"));
    assert!(!css.contains(".nx-skeleton {"));
}

#[test]
fn using_the_skeleton_includes_tokens_and_skeleton_css_only() {
    let root = element("div", vec![static_class("nx-skeleton")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-skeleton {"));
    assert!(!css.contains(".nx-tab {"));
}

#[test]
fn using_tabs_includes_tokens_and_tab_css_only() {
    let root = element("div", vec![static_class("nx-tab-group")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-tab-group {"));
    assert!(css.contains(".nx-tab-panel"));
    assert!(!css.contains(".nx-accordion {"));
}

#[test]
fn using_the_accordion_includes_tokens_and_accordion_css_only() {
    let root = element("div", vec![static_class("nx-accordion")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-accordion {"));
    assert!(!css.contains(".nx-dropdown {"));
}

#[test]
fn using_the_dropdown_includes_tokens_and_dropdown_css_only() {
    let root = element("div", vec![static_class("nx-dropdown")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(".nx-dropdown {"));
    assert!(css.contains(".nx-dropdown-menu"));
    assert!(!css.contains(".nx-tooltip {"));
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
    assert!(css.contains(".nx-drawer {"));
    assert!(css.contains(".nx-flex {"));
    assert!(css.contains(".nx-gap-1 {"));
    assert!(css.contains(".nx-grid {"));
    assert!(css.contains(".nx-stack > * + * {"));
    assert!(css.contains(".nx-table {"));
    assert!(css.contains(".nx-badge {"));
    assert!(css.contains(".nx-avatar {"));
    assert!(css.contains(".nx-breadcrumbs {"));
    assert!(css.contains(".nx-alert {"));
    assert!(css.contains(".nx-divider {"));
    assert!(css.contains(".nx-tooltip {"));
    assert!(css.contains(".nx-progress {"));
    assert!(css.contains(".nx-skeleton {"));
    assert!(css.contains(".nx-tab-group {"));
    assert!(css.contains(".nx-accordion {"));
    assert!(css.contains(".nx-dropdown {"));
}

#[test]
fn full_source_includes_the_dark_theme_variant_of_the_tokens() {
    let css = full_source();

    assert!(css.contains("prefers-color-scheme: dark"), "debe respetar el tema del sistema sin JS");
    assert!(css.contains(r#":root[data-theme="dark"]"#), "debe permitir forzar dark con JS");
    assert!(css.contains(r#":root:not([data-theme="light"])"#), "el override manual a light debe ganarle al sistema");
}

#[test]
fn a_page_using_only_the_button_still_ships_the_dark_variant_of_the_tokens() {
    // Los tokens (con su variante dark) se incluyen siempre que se use
    // cualquier componente, no solo si la página "usa dark mode"
    // explícitamente — no hay forma de saber en build time si el
    // visitante tiene `prefers-color-scheme: dark`.
    let root = element("button", vec![static_class("nx-btn")], vec![]);
    let css = build_stylesheet(&root).expect("expected some css");

    assert!(css.contains(r#":root[data-theme="dark"]"#));
}

#[test]
fn full_source_is_deterministic() {
    assert_eq!(full_source(), full_source());
}
