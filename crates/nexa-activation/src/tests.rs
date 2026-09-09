use std::collections::{BTreeMap, BTreeSet};

use super::*;
use nexa_ast::{Event, Expr};
use nexa_ir::{Classification, DependencyGraph, IrComponent, IrNode, IrNodeKind};

fn no_imports() -> BTreeSet<String> {
    BTreeSet::new()
}

fn imports(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

fn static_text(id: usize, value: &str) -> IrNode {
    IrNode {
        id,
        classification: Classification::Static,
        kind: IrNodeKind::Text(value.to_string()),
    }
}

fn button_with_click(id: usize, handler: &str, strategy: Option<&str>, children: Vec<IrNode>) -> IrNode {
    IrNode {
        id,
        classification: Classification::Interactive,
        kind: IrNodeKind::Element {
            tag: "button".into(),
            attrs: vec![],
            events: vec![Event {
                name: "click".into(),
                handler: Expr::Identifier(handler.into()),
                strategy: strategy.map(str::to_string),
            }],
            island: None,
            children,
        },
    }
}

fn static_element(id: usize, tag: &str, children: Vec<IrNode>) -> IrNode {
    IrNode {
        id,
        classification: Classification::Static,
        kind: IrNodeKind::Element {
            tag: tag.into(),
            attrs: vec![],
            events: vec![],
            island: None,
            children,
        },
    }
}

fn for_loop(id: usize, item_name: &str, body: IrNode) -> IrNode {
    IrNode {
        id,
        classification: Classification::Dynamic,
        kind: IrNodeKind::For {
            each: Expr::Member { object: Box::new(Expr::Identifier("data".into())), property: "items".into() },
            item_name: item_name.into(),
            body: Box::new(body),
        },
    }
}

#[test]
fn strategy_parses_known_values_and_falls_back_to_interaction() {
    assert_eq!(Strategy::parse(Some("idle")), Strategy::Idle);
    assert_eq!(Strategy::parse(Some("visible")), Strategy::Visible);
    assert_eq!(Strategy::parse(Some("load")), Strategy::Load);
    assert_eq!(Strategy::parse(Some("manual")), Strategy::Manual);
    assert_eq!(Strategy::parse(Some("algo-inventado")), Strategy::Interaction);
    assert_eq!(Strategy::parse(None), Strategy::Interaction);
}

#[test]
fn static_only_component_produces_no_manifest_entries_and_no_chunks() {
    let component = IrComponent {
        name: "About".into(),
        root: static_element(0, "main", vec![static_text(1, "Nada interactivo aquí")]),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, chunks) = build(&component, "About", &BTreeMap::new(), &BTreeMap::new(), &no_imports()).unwrap();

    assert!(manifest.is_empty(), "una página estática no debe generar manifiesto");
    assert!(chunks.is_empty(), "una página estática no debe generar JS");
}

#[test]
fn interactive_button_gets_one_manifest_entry_and_one_chunk() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: static_element(
            0,
            "article",
            vec![
                static_element(1, "h1", vec![static_text(2, "iPhone 17")]),
                button_with_click(3, "buy", None, vec![static_text(4, "Comprar")]),
            ],
        ),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, chunks) = build(&component, "ProductPage", &BTreeMap::new(), &BTreeMap::new(), &no_imports()).unwrap();

    assert_eq!(manifest.len(), 1);
    assert_eq!(chunks.len(), 1);

    let entry = manifest.get(3).expect("el botón (id 3) debe estar en el manifiesto");
    assert_eq!(entry.event, "click");
    assert_eq!(entry.handler, "buy");
    assert_eq!(entry.strategy, Strategy::Interaction);

    // El nombre de archivo lleva un hash de contenido (cache-busting real,
    // Fase 23) — no es un literal fijo, pero sí un patrón verificable:
    // `<Componente>-<node_id>.<hash de 8 hex>.js`.
    assert!(entry.module.starts_with("/assets/ProductPage-3."));
    assert!(entry.module.ends_with(".js"));
    assert_eq!(entry.module, format!("/assets/{}", chunks[0].filename), "manifest.module y chunk.filename deben coincidir exactamente");

    assert!(chunks[0].filename.starts_with("ProductPage-3."));
    assert!(chunks[0].content.contains("addEventListener(\"click\""));
    assert!(chunks[0].content.contains("buy"));
}

#[test]
fn an_interactive_element_inside_a_for_loop_body_gets_a_manifest_entry() {
    // Fase 30: el cuerpo de un `<For>` se clasifica una sola vez (es una
    // plantilla, no N copias) — un solo `NodeId`/entrada de manifiesto
    // cubre todas las copias que el renderer termine produciendo en
    // tiempo de render. Sin este caso, un `onClick` dentro de un
    // `<For>` quedaría completamente ausente del manifiesto (bug real:
    // `build::collect` hacía early-return en cualquier nodo que no
    // fuera `IrNodeKind::Element`, así que nunca bajaba a `body`).
    let component = IrComponent {
        name: "Catalog".into(),
        root: static_element(
            0,
            "ul",
            vec![for_loop(1, "item", button_with_click(2, "remove", None, vec![static_text(3, "Quitar")]))],
        ),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, chunks) = build(&component, "Catalog", &BTreeMap::new(), &BTreeMap::new(), &no_imports()).unwrap();

    assert_eq!(manifest.len(), 1);
    assert_eq!(chunks.len(), 1);
    let entry = manifest.get(2).expect("el botón dentro del <For> (id 2) debe estar en el manifiesto");
    assert_eq!(entry.handler, "remove");
}

#[test]
fn chunk_filename_changes_when_the_handler_source_changes() {
    let component = || IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(3, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };
    let handlers_with = |source: &str| {
        let mut m = BTreeMap::new();
        m.insert("buy".to_string(), source.to_string());
        m
    };

    let (_, chunks_a) = build(&component(), "ProductPage", &handlers_with("function buy() { cart.add(1); }"), &BTreeMap::new(), &no_imports()).unwrap();
    let (_, chunks_b) = build(&component(), "ProductPage", &handlers_with("function buy() { cart.add(2); }"), &BTreeMap::new(), &no_imports()).unwrap();

    assert_ne!(
        chunks_a[0].filename, chunks_b[0].filename,
        "dos handlers con contenido distinto no deben compartir nombre de archivo"
    );
}

#[test]
fn data_nexa_strategy_overrides_the_default() {
    let component = IrComponent {
        name: "Gallery".into(),
        root: button_with_click(0, "expand", Some("visible"), vec![]),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, _chunks) = build(&component, "Gallery", &BTreeMap::new(), &BTreeMap::new(), &no_imports()).unwrap();
    assert_eq!(manifest.get(0).unwrap().strategy, Strategy::Visible);
}

#[test]
fn manifest_serializes_to_json_keyed_by_node_id() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(5, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, _chunks) = build(&component, "ProductPage", &BTreeMap::new(), &BTreeMap::new(), &no_imports()).unwrap();
    let json = manifest.to_json_pretty().expect("should serialize");

    assert!(json.contains("\"5\""));
    assert!(json.contains("\"event\": \"click\""));
    assert!(json.contains("\"handler\": \"buy\""));
    assert!(json.contains("\"strategy\": \"interaction\""));
}

#[test]
fn entries_iterates_every_manifest_entry() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(5, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, _chunks) = build(&component, "ProductPage", &BTreeMap::new(), &BTreeMap::new(), &no_imports()).unwrap();
    let entries: Vec<_> = manifest.entries().collect();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "5");
    assert_eq!(entries[0].1.handler, "buy");
}

#[test]
fn uses_the_real_handler_source_when_its_known() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(0, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert("buy".to_string(), "function buy() {\n    cart.add(id);\n}".to_string());

    let (_manifest, chunks) = build(&component, "ProductPage", &handlers, &BTreeMap::new(), &no_imports()).unwrap();

    assert!(chunks[0].content.contains("function buy()"));
    assert!(chunks[0].content.contains("cart.add(id)"));
    assert!(!chunks[0].content.contains("no se pudo resolver"));
}

#[test]
fn bug_27_prepends_a_simple_top_level_const_the_handler_actually_uses() {
    let component = IrComponent {
        name: "LoginPage".into(),
        root: button_with_click(0, "handleLogin", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert(
        "handleLogin".to_string(),
        "async function handleLogin() {\n    await fetch(`${API_BASE}/auth/login`);\n}".to_string(),
    );
    let mut consts = BTreeMap::new();
    consts.insert(
        "API_BASE".to_string(),
        nexa_ast::TopLevelConst {
            source: "const API_BASE = \"https://api.oweeme.com\";".to_string(),
            is_simple: true,
        },
    );

    let (_manifest, chunks) = build(&component, "LoginPage", &handlers, &consts, &no_imports()).unwrap();

    assert!(chunks[0].content.contains("const API_BASE = \"https://api.oweeme.com\";"));
    assert!(chunks[0].content.contains("async function handleLogin()"));
}

#[test]
fn bug_27_does_not_prepend_a_const_the_handler_never_mentions() {
    let component = IrComponent {
        name: "LoginPage".into(),
        root: button_with_click(0, "handleLogin", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert("handleLogin".to_string(), "function handleLogin() {\n    console.log(\"hola\");\n}".to_string());
    let mut consts = BTreeMap::new();
    consts.insert(
        "API_BASE".to_string(),
        nexa_ast::TopLevelConst { source: "const API_BASE = \"https://api.oweeme.com\";".to_string(), is_simple: true },
    );

    let (_manifest, chunks) = build(&component, "LoginPage", &handlers, &consts, &no_imports()).unwrap();

    assert!(!chunks[0].content.contains("API_BASE"));
}

#[test]
fn bug_27_does_not_confuse_a_const_with_a_longer_name_that_contains_it() {
    let component = IrComponent {
        name: "LoginPage".into(),
        root: button_with_click(0, "handleLogin", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert(
        "handleLogin".to_string(),
        "function handleLogin() {\n    console.log(MY_API_BASE_URL);\n}".to_string(),
    );
    let mut consts = BTreeMap::new();
    consts.insert(
        "API_BASE".to_string(),
        nexa_ast::TopLevelConst {
            source: "const API_BASE = \"esto no debería aparecer\";".to_string(),
            is_simple: true,
        },
    );

    let (_manifest, chunks) = build(&component, "LoginPage", &handlers, &consts, &no_imports()).unwrap();

    assert!(!chunks[0].content.contains("esto no debería aparecer"));
}

#[test]
fn bug_27_a_const_name_that_only_appears_inside_a_string_or_comment_is_not_a_real_usage() {
    // Falso positivo real, encontrado verificando el fix con un
    // navegador: "console.log(\"...CONFIG\")" contiene el texto
    // "CONFIG" pero el handler nunca usa la constante — antes de
    // enmascarar strings/comentarios, esto rompía el build por algo
    // completamente ajeno al código real.
    let component = IrComponent {
        name: "LoginPage".into(),
        root: button_with_click(0, "handleClick", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert(
        "handleClick".to_string(),
        "function handleClick() {\n    // no toca CONFIG para nada\n    console.log(\"click, sin tocar CONFIG\");\n}"
            .to_string(),
    );
    let mut consts = BTreeMap::new();
    consts.insert(
        "CONFIG".to_string(),
        nexa_ast::TopLevelConst { source: "const CONFIG = buildConfig();".to_string(), is_simple: false },
    );

    let (_manifest, chunks) = build(&component, "LoginPage", &handlers, &consts, &no_imports()).unwrap();

    assert!(chunks[0].content.contains("click, sin tocar CONFIG"), "el string real debe seguir intacto en el chunk");
}

#[test]
fn bug_27_a_real_usage_inside_a_template_literal_interpolation_still_counts() {
    // Lo opuesto del test de arriba: un `${API_BASE}` real adentro de
    // un template literal SÍ tiene que seguir contando como uso real —
    // enmascarar strings no debe enmascarar también las interpolaciones.
    let component = IrComponent {
        name: "LoginPage".into(),
        root: button_with_click(0, "handleLogin", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert(
        "handleLogin".to_string(),
        "async function handleLogin() {\n    await fetch(`${API_BASE}/auth/login`);\n}".to_string(),
    );
    let mut consts = BTreeMap::new();
    consts.insert(
        "API_BASE".to_string(),
        nexa_ast::TopLevelConst { source: "const API_BASE = \"https://api.oweeme.com\";".to_string(), is_simple: true },
    );

    let (_manifest, chunks) = build(&component, "LoginPage", &handlers, &consts, &no_imports()).unwrap();

    assert!(chunks[0].content.contains("const API_BASE ="));
}

#[test]
fn bug_27_fails_explicitly_when_a_handler_uses_a_const_that_is_not_a_simple_literal() {
    let component = IrComponent {
        name: "LoginPage".into(),
        root: button_with_click(0, "handleLogin", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert("handleLogin".to_string(), "function handleLogin() {\n    console.log(CONFIG);\n}".to_string());
    let mut consts = BTreeMap::new();
    consts.insert(
        "CONFIG".to_string(),
        nexa_ast::TopLevelConst { source: "const CONFIG = buildConfig();".to_string(), is_simple: false },
    );

    let error = build(&component, "LoginPage", &handlers, &consts, &no_imports()).unwrap_err();

    assert!(error.contains("handleLogin"));
    assert!(error.contains("CONFIG"));
}

#[test]
fn falls_back_to_a_placeholder_when_the_handler_source_is_unknown() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(0, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let (_manifest, chunks) = build(&component, "ProductPage", &BTreeMap::new(), &BTreeMap::new(), &no_imports()).unwrap();

    assert!(chunks[0].content.contains("no se pudo resolver"));
}

#[test]
fn imports_a_known_identifier_only_when_the_handler_actually_uses_it() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(0, "share", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert(
        "share".to_string(),
        "function share() {\n    platform.share({ title: \"hola\" });\n}".to_string(),
    );

    let (_manifest, chunks) = build(&component, "ProductPage", &handlers, &BTreeMap::new(), &imports(&["platform"])).unwrap();

    // El specifier es el nombre pelado ("platform"), no una ruta — se
    // resuelve vía el import map que declara `nexa-cli` en `<head>`
    // (Fase 15), no una URL cableada a mano en el propio chunk.
    assert!(chunks[0].content.contains("import { platform } from \"platform\";"));
}

#[test]
fn does_not_import_an_identifier_the_handler_does_not_use() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(0, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert("buy".to_string(), "function buy() {\n    cart.add(id);\n}".to_string());

    let (_manifest, chunks) = build(&component, "ProductPage", &handlers, &BTreeMap::new(), &imports(&["platform"])).unwrap();

    assert!(!chunks[0].content.contains("import {"));
}

#[test]
fn generalizes_to_any_declared_identifier_not_just_platform() {
    // Prueba explícita de que esto ya no es un caso especial cableado a
    // mano para `platform` (Fase 13) — un identificador de un paquete
    // de un tercero (Fase 15, ej. `stripe`) funciona exactamente igual,
    // sin que este crate sepa nada sobre quién lo declaró.
    let component = IrComponent {
        name: "Checkout".into(),
        root: button_with_click(0, "pay", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let mut handlers = BTreeMap::new();
    handlers.insert("pay".to_string(), "function pay() {\n    stripe.redirectToCheckout();\n}".to_string());

    let (_manifest, chunks) = build(&component, "Checkout", &handlers, &BTreeMap::new(), &imports(&["platform", "stripe"])).unwrap();

    assert!(chunks[0].content.contains("import { stripe } from \"stripe\";"));
    assert!(!chunks[0].content.contains("platform"));
}
