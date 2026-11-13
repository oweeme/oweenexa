//! Fase 62: la prueba central es que una página con componentes produce
//! EXACTAMENTE el mismo árbol que la misma página escrita a mano — el
//! resto de Nexa no se entera de que hubo composición.

use std::collections::BTreeMap;

use nexa_ast::Node;

use crate::{parse_component, parse_component_with, ImportResolver};

struct MapResolver(BTreeMap<&'static str, &'static str>);

impl ImportResolver for MapResolver {
    fn resolve(&self, _from: &str, specifier: &str) -> Option<(String, String)> {
        self.0.get(specifier).map(|src| (format!("{}.tsx", specifier.trim_start_matches("./")), src.to_string()))
    }
}

fn root_of(source: &str) -> Node {
    parse_component("page.tsx", source).unwrap_or_else(|e| panic!("{e}")).root
}

fn root_with(files: &[(&'static str, &'static str)], source: &str) -> Node {
    let resolver = MapResolver(files.iter().copied().collect());
    parse_component_with("page.tsx", source, &resolver).unwrap_or_else(|e| panic!("{e}")).root
}

fn err_of(source: &str) -> String {
    parse_component("page.tsx", source).unwrap_err().to_string()
}

#[test]
fn a_same_file_component_expands_to_the_hand_written_tree() {
    let composed = root_of(
        r#"
function Card(props) {
    return (<section class="card"><h2>{props.title}</h2>{props.children}</section>);
}
export default function Page() {
    return (<main><Card title="Hola"><p>cuerpo</p></Card></main>);
}
"#,
    );
    let by_hand = root_of(
        r#"export default function Page() {
    return (<main><section class="card"><h2>Hola</h2><p>cuerpo</p></section></main>);
}"#,
    );
    assert_eq!(composed, by_hand);
}

#[test]
fn destructured_props_work_and_a_dynamic_prop_stays_dynamic() {
    let composed = root_of(
        r#"
const Badge = ({ label, href }) => (<a class="badge" href={href}>{label}</a>);
export default function Page() {
    return (<div><Badge label={data.name} href={`/p/${params.slug}`} /></div>);
}
"#,
    );
    let by_hand = root_of(
        r#"export default function Page() {
    return (<div><a class="badge" href={`/p/${params.slug}`}>{data.name}</a></div>);
}"#,
    );
    assert_eq!(composed, by_hand);
}

#[test]
fn a_template_inside_the_component_mixes_text_and_the_prop() {
    let composed = root_of(
        r#"
function Link(props) { return (<a href={`/shop/${props.slug}/details`}>ver</a>); }
export default function Page() {
    return (<div><Link slug={data.slug} /><Link slug="fijo" /></div>);
}
"#,
    );
    let by_hand = root_of(
        r#"export default function Page() {
    return (<div><a href={`/shop/${data.slug}/details`}>ver</a><a href="/shop/fijo/details">ver</a></div>);
}"#,
    );
    assert_eq!(composed, by_hand);
}

#[test]
fn an_imported_component_is_read_through_the_resolver_default_and_named() {
    let files = [
        ("./Card", r#"export default function Card(props) { return (<section>{props.children}</section>); }"#),
        ("./ui", r#"export function Tag({ text }) { return (<span class="tag">{text}</span>); }"#),
    ];
    let composed = root_with(
        &files,
        r#"
import Card from "./Card";
import { Tag } from "./ui";
export default function Page() {
    return (<main><Card><Tag text="nuevo" /></Card></main>);
}
"#,
    );
    let by_hand = root_of(
        r#"export default function Page() {
    return (<main><section><span class="tag">nuevo</span></section></main>);
}"#,
    );
    assert_eq!(composed, by_hand);
}

#[test]
fn components_nest_and_pass_their_own_props_down() {
    let composed = root_of(
        r#"
function Title(props) { return (<h1>{props.text}</h1>); }
function Hero(props) { return (<header><Title text={props.heading} /></header>); }
export default function Page() {
    return (<main><Hero heading={data.title} /></main>);
}
"#,
    );
    let by_hand = root_of(r#"export default function Page() { return (<main><header><h1>{data.title}</h1></header></main>); }"#);
    assert_eq!(composed, by_hand);
}

#[test]
fn a_prop_object_field_extends_the_expression() {
    let composed = root_of(
        r#"
function User(props) { return (<p>{props.user.name}</p>); }
export default function Page() { return (<div><User user={data.owner} /></div>); }
"#,
    );
    let by_hand = root_of(r#"export default function Page() { return (<div><p>{data.owner.name}</p></div>); }"#);
    assert_eq!(composed, by_hand);
}

#[test]
fn an_event_comes_from_the_page_and_the_components_own_functions_are_merged() {
    let resolver = MapResolver(
        [(
            "./Buy",
            r#"
function track() { console.log("buy"); }
export default function Buy(props) { return (<button onClick={props.onBuy} onFocus={track}>comprar</button>); }
"#,
        )]
        .into_iter()
        .collect(),
    );
    let component = parse_component_with(
        "page.tsx",
        r#"
import Buy from "./Buy";
function buy() { alert(1); }
export default function Page() { return (<div><Buy onBuy={buy} /></div>); }
"#,
        &resolver,
    )
    .unwrap();
    let Node::Element(root) = &component.root else { panic!() };
    let Node::Element(button) = &root.children[0] else { panic!() };
    let handlers: Vec<_> = button.events.iter().map(|e| (e.name.as_str(), e.handler.path())).collect();
    assert_eq!(handlers, vec![("click", "buy".to_string()), ("focus", "track".to_string())]);
    assert!(component.handlers.contains_key("buy"));
    assert!(component.handlers.contains_key("track"), "la función del componente debe llegar al chunk");
}

#[test]
fn a_prop_the_call_did_not_pass_is_omitted_not_rendered_empty() {
    let composed = root_of(
        r#"
function Img(props) { return (<img src={props.src} alt={props.alt} />); }
export default function Page() { return (<div><Img src="/a.png" /></div>); }
"#,
    );
    let by_hand = root_of(r#"export default function Page() { return (<div><img src="/a.png" /></div>); }"#);
    assert_eq!(composed, by_hand);
}

#[test]
fn for_inside_a_component_takes_its_list_from_a_prop() {
    let composed = root_of(
        r#"
function List(props) { return (<ul><For each={props.items}>{(item) => (<li>{item.name}</li>)}</For></ul>); }
export default function Page() { return (<main><List items={data.people} /></main>); }
"#,
    );
    let by_hand = root_of(
        r#"export default function Page() { return (<main><ul><For each={data.people}>{(item) => (<li>{item.name}</li>)}</For></ul></main>); }"#,
    );
    assert_eq!(composed, by_hand);
}

#[test]
fn the_for_parameter_shadows_a_prop_of_the_same_name() {
    let composed = root_of(
        r#"
function List({ item, items }) { return (<ul><For each={items}>{(item) => (<li>{item.name}</li>)}</For></ul>); }
export default function Page() { return (<main><List item="IGNORADO" items={data.people} /></main>); }
"#,
    );
    let by_hand = root_of(
        r#"export default function Page() { return (<main><ul><For each={data.people}>{(item) => (<li>{item.name}</li>)}</For></ul></main>); }"#,
    );
    assert_eq!(composed, by_hand);
}

#[test]
fn a_component_can_be_used_inside_a_for_with_the_item_as_a_prop() {
    let composed = root_of(
        r#"
function Row(props) { return (<li>{props.person.name}</li>); }
export default function Page() { return (<ul><For each={data.people}>{(p) => (<Row person={p} />)}</For></ul>); }
"#,
    );
    let by_hand = root_of(
        r#"export default function Page() { return (<ul><For each={data.people}>{(p) => (<li>{p.name}</li>)}</For></ul>); }"#,
    );
    assert_eq!(composed, by_hand);
}

#[test]
fn nested_for_may_iterate_a_field_of_the_outer_item() {
    // Solo el parser: `each={group.members}` ya no se rechaza.
    let root = root_of(
        r#"export default function Page() {
    return (<div><For each={data.groups}>{(group) => (<section><For each={group.members}>{(m) => (<b>{m.name}</b>)}</For></section>)}</For></div>);
}"#,
    );
    let Node::Element(div) = &root else { panic!() };
    assert!(matches!(&div.children[0], Node::For(_)));
}

#[test]
fn errors_are_explicit() {
    assert!(err_of(r#"export default function P() { return (<main><Nope /></main>); }"#).contains("no está definido"));

    assert!(err_of(
        r#"
function A() { return (<div><A /></div>); }
export default function P() { return (<main><A /></main>); }
"#
    )
    .contains("se incluye a sí mismo"));

    assert!(err_of(
        r#"
function C(props) { return (<p>{props.x}</p>); }
export default function P() { return (<main><C x={() => 1} /></main>); }
"#
    )
    .contains("nunca una expresión calculada"));

    assert!(err_of(
        r#"
function C(props) { return (<p>{props.x.y}</p>); }
export default function P() { return (<main><C x="texto" /></main>); }
"#
    )
    .contains("literal/plantilla"));

    assert!(err_of(
        r#"
function C({ a = 1 }) { return (<p>{a}</p>); }
export default function P() { return (<main><C /></main>); }
"#
    )
    .contains("valores por defecto"));
}
