//! Fase 62, de punta a punta (parser -> analyzer -> renderer): una página
//! con componentes y `<For>` anidado produce el mismo HTML que la página
//! escrita a mano.

use std::collections::BTreeMap;

use nexa_renderer::{render_node, RenderContext};
use serde_json::json;

fn html(source: &str, data: &serde_json::Value) -> String {
    let component = nexa_parser::parse_component("page.tsx", source).unwrap_or_else(|e| panic!("{e}"));
    let ir = nexa_analyzer::analyze(&component);
    let params = BTreeMap::new();
    let ctx = RenderContext {
        data: Some(data),
        params: &params,
        translations: None,
        loop_binding: None,
        outer_loops: Vec::new(),
        current_path: None,
    };
    render_node(&ir.root, &ctx)
}

fn teams() -> serde_json::Value {
    json!({
        "title": "Equipos",
        "groups": [
            { "name": "Diseño", "members": [{ "name": "Ana", "role": "lead" }, { "name": "Beto", "role": "dev" }] },
            { "name": "Datos", "members": [{ "name": "Carla", "role": "lead" }] },
        ]
    })
}

#[test]
fn nested_for_renders_the_inner_list_of_each_outer_item() {
    let out = html(
        r#"export default function P() {
    return (<div><For each={data.groups}>{(g) => (<section><h2>{g.name}</h2><For each={g.members}>{(m) => (<b>{m.name}</b>)}</For></section>)}</For></div>);
}"#,
        &teams(),
    );
    assert_eq!(
        out,
        "<div><section><h2>Diseño</h2><b>Ana</b><b>Beto</b></section><section><h2>Datos</h2><b>Carla</b></section></div>"
    );
}

#[test]
fn an_inner_loop_variable_shadows_an_outer_one_with_the_same_name() {
    let data = json!({ "rows": [{ "name": "fila", "cells": [{ "name": "celda1" }, { "name": "celda2" }] }] });
    let out = html(
        r#"export default function P() {
    return (<div><For each={data.rows}>{(x) => (<p><i>{x.name}</i><For each={x.cells}>{(x) => (<b>{x.name}</b>)}</For></p>)}</For></div>);
}"#,
        &data,
    );
    assert_eq!(out, "<div><p><i>fila</i><b>celda1</b><b>celda2</b></p></div>");
}

#[test]
fn composed_components_render_exactly_like_the_hand_written_page() {
    let composed = html(
        r#"
function Card(props) { return (<section class="card"><h2>{props.title}</h2>{props.children}</section>); }
function Tag({ text }) { return (<span class="tag">{text}</span>); }
export default function P() {
    return (<main><h1>{data.title}</h1><For each={data.groups}>{(g) => (
        <Card title={g.name}><ul><For each={g.members}>{(m) => (<li><Tag text={m.role} /> {m.name}</li>)}</For></ul></Card>
    )}</For></main>);
}
"#,
        &teams(),
    );
    let by_hand = html(
        r#"export default function P() {
    return (<main><h1>{data.title}</h1><For each={data.groups}>{(g) => (
        <section class="card"><h2>{g.name}</h2><ul><For each={g.members}>{(m) => (<li><span class="tag">{m.role}</span> {m.name}</li>)}</For></ul></section>
    )}</For></main>);
}"#,
        &teams(),
    );
    assert_eq!(composed, by_hand);
    assert!(composed.contains("<h2>Diseño</h2><ul><li><span class=\"tag\">lead</span> Ana</li>"), "{composed}");
}

// ---- <Show> (Fase 62) ----

fn show(data: serde_json::Value) -> String {
    html(
        r#"export default function P() {
    return (<div><Show when={data.flag} fallback={<p>no</p>}><b>si</b></Show></div>);
}"#,
        &data,
    )
}

#[test]
fn show_picks_the_branch_by_json_truthiness() {
    for truthy in [json!(true), json!(1), json!("x"), json!([1]), json!({}), json!({"a": 1})] {
        assert_eq!(show(json!({ "flag": truthy })), "<div><b>si</b></div>", "truthy: {truthy}");
    }
    for falsy in [json!(false), json!(0), json!(""), json!([]), json!(null)] {
        assert_eq!(show(json!({ "flag": falsy })), "<div><p>no</p></div>", "falsy: {falsy}");
    }
    // el dato ni siquiera existe
    assert_eq!(show(json!({})), "<div><p>no</p></div>");
}

#[test]
fn show_without_fallback_renders_nothing_when_false() {
    let out = html(
        r#"export default function P() { return (<div><Show when={data.stock}><b>hay</b></Show></div>); }"#,
        &json!({ "stock": 0 }),
    );
    assert_eq!(out, "<div></div>");
}

#[test]
fn show_works_inside_for_on_each_item_and_inside_a_component() {
    let out = html(
        r#"
function Badge(props) { return (<Show when={props.on}><i>OFERTA</i></Show>); }
export default function P() {
    return (<ul><For each={data.items}>{(it) => (<li>{it.name}<Show when={it.sale}><Badge on={it.sale} /></Show></li>)}</For></ul>);
}"#,
        &json!({ "items": [{ "name": "a", "sale": true }, { "name": "b", "sale": false }] }),
    );
    assert_eq!(out, "<ul><li>a<i>OFERTA</i></li><li>b</li></ul>");
}

#[test]
fn a_show_inside_a_component_that_the_call_did_not_pass_is_false() {
    let out = html(
        r#"
function Note(props) { return (<div><Show when={props.text} fallback={<p>vacío</p>}><p>{props.text}</p></Show></div>); }
export default function P() { return (<main><Note /><Note text={data.msg} /></main>); }"#,
        &json!({ "msg": "hola" }),
    );
    assert_eq!(out, "<main><div><p>vacío</p></div><div><p>hola</p></div></main>");
}

#[test]
fn show_rejects_arbitrary_expressions_with_a_clear_message() {
    let err = nexa_parser::parse_component(
        "page.tsx",
        r#"export default function P() { return (<div><Show when={data.a > 1}><b>x</b></Show></div>); }"#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("nada de comparaciones"), "{err}");
}
