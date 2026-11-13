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

// ---- Seguridad: datos de la API NUNCA deben ejecutar código ----

fn render(source: &str, data: &serde_json::Value) -> String {
    html(source, data)
}

#[test]
fn dynamic_urls_with_executable_schemes_are_dropped_but_ordinary_ones_survive() {
    let data = json!({
        "js": "javascript:alert(1)", "js2": "  JaVaScRiPt:alert(1)", "js3": "java\tscript:alert(1)",
        "vb": "vbscript:x", "dh": "data:text/html,<script>alert(1)</script>",
        "ok": "https://example.com/a?b=c:d", "rel": "/blog/post", "mail": "mailto:hola@x.com"
    });
    let out = render(
        r#"export default function P() { return (<div>
            <a id="1" href={data.js}>x</a><a id="2" href={data.js2}>x</a><a id="3" href={data.js3}>x</a>
            <a id="4" href={data.vb}>x</a><a id="5" href={data.dh}>x</a>
            <a id="6" href={data.ok}>x</a><a id="7" href={data.rel}>x</a><a id="8" href={data.mail}>x</a>
            <form action={data.js}></form><iframe src={data.js}></iframe></div>); }"#,
        &data,
    );
    for n in ["1", "2", "3", "4", "5"] {
        assert!(out.contains(&format!("<a id=\"{n}\">x</a>")), "el href peligroso del enlace {n} debe omitirse: {out}");
    }
    assert!(out.contains("<a id=\"6\" href=\"https://example.com/a?b=c:d\">"), "{out}");
    assert!(out.contains("<a id=\"7\" href=\"/blog/post\">") && out.contains("href=\"mailto:hola@x.com\""), "{out}");
    assert!(!out.contains("javascript") && !out.contains("vbscript") && !out.contains("<form action="), "{out}");
    assert!(!out.contains("<iframe src="), "{out}");
}

#[test]
fn a_hand_written_javascript_url_is_trusted_but_the_same_text_from_data_is_not() {
    let out = render(
        r#"export default function P() { return (<div><a id="a" href="javascript:void(0)">x</a><a id="b" href={data.u}>x</a></div>); }"#,
        &json!({ "u": "javascript:void(0)" }),
    );
    assert!(out.contains("<a id=\"a\" href=\"javascript:void(0)\">"), "{out}");
    assert!(out.contains("<a id=\"b\">x</a>"), "{out}");
}

#[test]
fn data_images_are_allowed_only_as_an_image_src() {
    let data = json!({ "img": "data:image/png;base64,AAAA" });
    let out = render(r#"export default function P() { return (<div><img src={data.img} alt="x" /><a href={data.img}>x</a><iframe src={data.img}></iframe></div>); }"#, &data);
    assert!(out.contains("<img src=\"data:image/png;base64,AAAA\""), "{out}");
    assert!(out.contains("<a>x</a>") && !out.contains("<iframe src"), "{out}");
}

#[test]
fn data_can_never_become_an_inline_event_handler_or_srcdoc() {
    let data = json!({ "x": "alert(document.cookie)" });
    let out = render(
        r#"export default function P() { return (<div><b onclick={data.x}>a</b><i onerror={data.x}>b</i><iframe srcdoc={data.x}></iframe><p title={data.x}>c</p></div>); }"#,
        &data,
    );
    assert!(!out.contains("onclick") && !out.contains("onerror") && !out.contains("srcdoc"), "{out}");
    assert!(out.contains("title=\"alert(document.cookie)\""), "un atributo normal sí se conserva (escapado)");
}

#[test]
fn text_and_attribute_values_are_always_escaped() {
    let data = json!({ "evil": "\"><img src=x onerror=alert(1)>", "t": "<script>alert(1)</script>" });
    let out = render(
        r#"export default function P() { return (<div class={data.evil} title={data.evil}><p>{data.t}</p><p>{data.evil}</p></div>); }"#,
        &data,
    );
    assert!(!out.contains("<script>") && !out.contains("<img"), "{out}");
    assert!(out.contains("class=\"&quot;&gt;&lt;img src=x onerror=alert(1)&gt;\""), "{out}");
}

// ---- <SafeHtml> y <Markdown> (Fase 68) ----

#[test]
fn safe_html_renders_rich_content_from_the_api_and_strips_the_attacks() {
    let data = json!({ "body": "<h2>Hola</h2><p>Texto <b>fuerte</b></p><script>alert(1)</script><img src=x onerror=alert(1)><a href=\"javascript:alert(1)\">x</a>" });
    let out = render(r#"export default function P() { return (<article><SafeHtml html={data.body} /></article>); }"#, &data);
    assert!(out.contains("<h2>Hola</h2>") && out.contains("<b>fuerte</b>"), "{out}");
    for bad in ["<script", "onerror", "javascript:"] {
        assert!(!out.contains(bad), "{bad}: {out}");
    }
}

#[test]
fn the_same_html_through_a_plain_expression_is_still_escaped_text() {
    let data = json!({ "body": "<b>x</b>" });
    let out = render(r#"export default function P() { return (<div><p>{data.body}</p><SafeHtml html={data.body} /></div>); }"#, &data);
    assert_eq!(out, "<div><p>&lt;b&gt;x&lt;/b&gt;</p><b>x</b></div>");
}

#[test]
fn markdown_renders_and_is_sanitized() {
    let data = json!({ "md": "# Título\n\nUn [enlace](https://oweeme.com) y [malo](javascript:alert(1)).\n\n<script>alert(1)</script>" });
    let out = render(r#"export default function P() { return (<main><Markdown text={data.md} /></main>); }"#, &data);
    assert!(out.contains("<h1>Título</h1>") && out.contains("href=\"https://oweeme.com\""), "{out}");
    assert!(!out.contains("<script") && !out.contains("javascript:"), "{out}");
}

#[test]
fn rich_content_works_inside_a_for_and_through_a_component_prop() {
    let data = json!({ "posts": [{ "title": "A", "html": "<p>uno <i>i</i></p>" }, { "title": "B", "html": "<p>dos</p><script>x()</script>" }] });
    let out = render(
        r#"
function Body(props) { return (<section><SafeHtml html={props.content} /></section>); }
export default function P() { return (<div><For each={data.posts}>{(p) => (<article><h2>{p.title}</h2><Body content={p.html} /></article>)}</For></div>); }"#,
        &data,
    );
    assert_eq!(out, "<div><article><h2>A</h2><section><p>uno <i>i</i></p></section></article><article><h2>B</h2><section><p>dos</p></section></article></div>");
}

#[test]
fn a_missing_or_non_text_value_renders_nothing() {
    let out = render(
        r#"export default function P() { return (<div><SafeHtml html={data.nope} /><SafeHtml html={data.n} /><Markdown text={data.obj} /></div>); }"#,
        &json!({ "n": 42, "obj": { "a": 1 } }),
    );
    assert_eq!(out, "<div></div>");
}

#[test]
fn rich_nodes_only_accept_a_data_reference_and_nothing_else() {
    let err = |src: &str| nexa_parser::parse_component("page.tsx", src).unwrap_err().to_string();
    assert!(err(r#"export default function P() { return (<div><SafeHtml html="<b>x</b>" /></div>); }"#).contains("debe ser una expresión"));
    assert!(err(r#"export default function P() { return (<div><SafeHtml html={data.a > 1} /></div>); }"#).contains("solo admite"));
    assert!(err(r#"export default function P() { return (<div><SafeHtml>hola</SafeHtml></div>); }"#).contains("no admite hijos"));
    assert!(err(r#"export default function P() { return (<div><SafeHtml text={data.a} /></div>); }"#).contains("usa `html={...}`"));
    assert!(err(r#"export default function P() { return (<div><Markdown html={data.a} /></div>); }"#).contains("usa `text={...}`"));
    assert!(err(r#"export default function P() { return (<div><SafeHtml /></div>); }"#).contains("exactamente un atributo"));
    // un literal pasado por un prop tampoco: no necesita saneado, se escribe a mano
    assert!(err(r#"function B(props) { return (<p><SafeHtml html={props.c} /></p>); }
export default function P() { return (<div><B c="<b>x</b>" /></div>); }"#).contains("un literal no necesita saneado"));
}
