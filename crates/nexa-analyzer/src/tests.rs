use super::analyze;
use nexa_ast::{Component, Element, Event, Expr, Node};
use nexa_ir::{Classification, IrNodeKind};

fn product_name_expr() -> Expr {
    Expr::Member {
        object: Box::new(Expr::Identifier("product".into())),
        property: "name".into(),
    }
}

#[test]
fn classifies_text_as_static() {
    let component = Component {
        name: "Home".into(),
        loader: None,
        paths: None,
        handlers: Default::default(),
        seo: None,
        schema: None,
        head: None,
        root: Node::Text("Hello Nexa".into()),
    };

    let ir = analyze(&component);
    assert_eq!(ir.root.classification, Classification::Static);
}

#[test]
fn classifies_expression_as_dynamic_and_tracks_dependency() {
    let component = Component {
        name: "Home".into(),
        loader: None,
        paths: None,
        handlers: Default::default(),
        seo: None,
        schema: None,
        head: None,
        root: Node::Expression(product_name_expr()),
    };

    let ir = analyze(&component);
    assert_eq!(ir.root.classification, Classification::Dynamic);
    assert_eq!(ir.dependencies.dependents_of("product"), &[ir.root.id]);
}

#[test]
fn classifies_element_with_event_as_interactive() {
    let component = Component {
        name: "Home".into(),
        loader: None,
        paths: None,
        handlers: Default::default(),
        seo: None,
        schema: None,
        head: None,
        root: Node::Element(Element {
            tag: "button".into(),
            attrs: vec![],
            events: vec![Event {
                name: "click".into(),
                handler: Expr::Identifier("buy".into()),
                strategy: None,
            }],
            island: None,
            children: vec![Node::Text("Comprar".into())],
        }),
    };

    let ir = analyze(&component);
    assert_eq!(ir.root.classification, Classification::Interactive);
    assert_eq!(ir.dependencies.dependents_of("buy"), &[ir.root.id]);
}

/// El ejemplo exacto del criterio de salida de la Fase 3
/// (`docs/FASES-DE-CONSTRUCCION.md`): un `<h1>` con una variable y un
/// `<button>` con `onClick` deben clasificarse de forma distinta.
#[test]
fn distinguishes_static_dynamic_and_interactive_siblings() {
    let component = Component {
        name: "ProductPage".into(),
        loader: None,
        paths: None,
        handlers: Default::default(),
        seo: None,
        schema: None,
        head: None,
        root: Node::Element(Element {
            tag: "article".into(),
            attrs: vec![],
            events: vec![],
            island: None,
            children: vec![
                Node::Element(Element {
                    tag: "h1".into(),
                    attrs: vec![],
                    events: vec![],
                    island: None,
                    children: vec![Node::Expression(product_name_expr())],
                }),
                Node::Element(Element {
                    tag: "button".into(),
                    attrs: vec![],
                    events: vec![Event {
                        name: "click".into(),
                        handler: Expr::Identifier("buy".into()),
                        strategy: None,
                    }],
                    island: None,
                    children: vec![Node::Text("Comprar".into())],
                }),
            ],
        }),
    };

    let ir = analyze(&component);
    assert_eq!(ir.root.classification, Classification::Static);

    let IrNodeKind::Element { children, .. } = &ir.root.kind else {
        panic!("expected root element")
    };

    // <h1> en sí es Static: lo dinámico es su hijo, el `{product.name}`.
    assert_eq!(children[0].classification, Classification::Static);
    let IrNodeKind::Element { children: h1_children, .. } = &children[0].kind else {
        panic!("expected h1 element")
    };
    assert_eq!(h1_children[0].classification, Classification::Dynamic);

    // <button onClick={buy}> sí es Interactive.
    assert_eq!(children[1].classification, Classification::Interactive);

    // Nodos: article, h1, {product.name}, button, "Comprar" = 5 en total.
    let counts = ir.root.classification_counts();
    assert_eq!(counts.static_count, 3); // article, h1, "Comprar"
    assert_eq!(counts.dynamic_count, 1); // {product.name}
    assert_eq!(counts.interactive_count, 1); // button
    assert_eq!(counts.total(), 5);

    assert_eq!(ir.dependencies.dependents_of("product"), &[h1_children[0].id]);
    assert_eq!(ir.dependencies.dependents_of("buy"), &[children[1].id]);
}

#[test]
fn classifies_a_for_loop_as_dynamic_and_tracks_its_each_dependency() {
    use nexa_ast::ForLoop;

    let each = Expr::Member { object: Box::new(Expr::Identifier("data".into())), property: "items".into() };
    let item_title = Expr::Member { object: Box::new(Expr::Identifier("item".into())), property: "title".into() };

    let component = Component {
        name: "Articles".into(),
        loader: None,
        paths: None,
        handlers: Default::default(),
        seo: None,
        schema: None,
        head: None,
        root: Node::For(ForLoop {
            each: each.clone(),
            item_name: "item".into(),
            body: Box::new(Node::Expression(item_title)),
        }),
    };

    let ir = analyze(&component);
    assert_eq!(ir.root.classification, Classification::Dynamic);
    assert_eq!(ir.dependencies.dependents_of("data"), &[ir.root.id]);

    let IrNodeKind::For { each: ir_each, item_name, body } = &ir.root.kind else {
        panic!("expected an IrNodeKind::For")
    };
    assert_eq!(ir_each.path(), each.path());
    assert_eq!(item_name, "item");
    // El cuerpo se clasifica una sola vez (es una plantilla, no N copias)
    // y depende de `item`, no de `data` — ambos identificadores quedan
    // en el mismo grafo de dependencias, sin tratamiento especial.
    assert_eq!(body.classification, Classification::Dynamic);
    assert_eq!(ir.dependencies.dependents_of("item"), &[body.id]);
}
