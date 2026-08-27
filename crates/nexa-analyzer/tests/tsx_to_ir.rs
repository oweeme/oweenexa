//! Prueba de extremo a extremo del criterio de salida de la Fase 3:
//! texto TSX real → parser → analyzer → IR clasificado.

use nexa_ir::Classification;

#[test]
fn tsx_with_variable_and_button_is_classified_correctly() {
    let source = r#"
export default function ProductPage() {
    return (
        <article>
            <h1>{product.name}</h1>
            <button onClick={buy}>Comprar</button>
        </article>
    );
}
"#;

    let component = nexa_parser::parse_component("product.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);

    let counts = ir.root.classification_counts();
    assert_eq!(counts.dynamic_count, 1, "el {{product.name}} debe ser Dynamic");
    assert_eq!(counts.interactive_count, 1, "el botón con onClick debe ser Interactive");
    assert!(counts.static_count >= 2, "article, h1 y el texto son Static");

    assert!(!ir.dependencies.dependents_of("product").is_empty());
    assert!(!ir.dependencies.dependents_of("buy").is_empty());

    assert_eq!(ir.root.classification, Classification::Static);
}
