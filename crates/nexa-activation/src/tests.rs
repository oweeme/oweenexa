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

    let (manifest, chunks) = build(&component, "About", &BTreeMap::new(), &no_imports());

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

    let (manifest, chunks) = build(&component, "ProductPage", &BTreeMap::new(), &no_imports());

    assert_eq!(manifest.len(), 1);
    assert_eq!(chunks.len(), 1);

    let entry = manifest.get(3).expect("el botón (id 3) debe estar en el manifiesto");
    assert_eq!(entry.event, "click");
    assert_eq!(entry.handler, "buy");
    assert_eq!(entry.module, "/assets/ProductPage-3.js");
    assert_eq!(entry.strategy, Strategy::Interaction);

    assert_eq!(chunks[0].filename, "ProductPage-3.js");
    assert!(chunks[0].content.contains("addEventListener(\"click\""));
    assert!(chunks[0].content.contains("buy"));
}

#[test]
fn data_nexa_strategy_overrides_the_default() {
    let component = IrComponent {
        name: "Gallery".into(),
        root: button_with_click(0, "expand", Some("visible"), vec![]),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, _chunks) = build(&component, "Gallery", &BTreeMap::new(), &no_imports());
    assert_eq!(manifest.get(0).unwrap().strategy, Strategy::Visible);
}

#[test]
fn manifest_serializes_to_json_keyed_by_node_id() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(5, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let (manifest, _chunks) = build(&component, "ProductPage", &BTreeMap::new(), &no_imports());
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

    let (manifest, _chunks) = build(&component, "ProductPage", &BTreeMap::new(), &no_imports());
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

    let (_manifest, chunks) = build(&component, "ProductPage", &handlers, &no_imports());

    assert!(chunks[0].content.contains("function buy()"));
    assert!(chunks[0].content.contains("cart.add(id)"));
    assert!(!chunks[0].content.contains("no se pudo resolver"));
}

#[test]
fn falls_back_to_a_placeholder_when_the_handler_source_is_unknown() {
    let component = IrComponent {
        name: "ProductPage".into(),
        root: button_with_click(0, "buy", None, vec![]),
        dependencies: DependencyGraph::new(),
    };

    let (_manifest, chunks) = build(&component, "ProductPage", &BTreeMap::new(), &no_imports());

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

    let (_manifest, chunks) = build(&component, "ProductPage", &handlers, &imports(&["platform"]));

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

    let (_manifest, chunks) = build(&component, "ProductPage", &handlers, &imports(&["platform"]));

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

    let (_manifest, chunks) = build(&component, "Checkout", &handlers, &imports(&["platform", "stripe"]));

    assert!(chunks[0].content.contains("import { stripe } from \"stripe\";"));
    assert!(!chunks[0].content.contains("platform"));
}
