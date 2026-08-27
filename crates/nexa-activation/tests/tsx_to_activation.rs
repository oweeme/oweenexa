//! Prueba de extremo a extremo del criterio de salida de la Fase 5:
//! texto TSX real → parser → analyzer → manifiesto de activación.

#[test]
fn static_page_ships_zero_js() {
    let source = r#"
export default function About() {
    return (
        <main>
            <h1>Sobre nosotros</h1>
            <p>Somos Oweeme.</p>
        </main>
    );
}
"#;

    let component = nexa_parser::parse_component("about.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);
    let (manifest, chunks) = nexa_activation::build(&ir, "About", &component.handlers, &Default::default());

    assert!(manifest.is_empty(), "una página 100% estática no debe generar manifiesto");
    assert!(chunks.is_empty(), "una página 100% estática no debe generar ningún chunk JS");
}

#[test]
fn page_with_one_interactive_button_gets_exactly_one_chunk() {
    let source = r#"
export default function ProductPage() {
    return (
        <article>
            <h1>iPhone 17</h1>
            <p>El texto de la ficha es completamente estático.</p>
            <button onClick={buy}>Comprar</button>
        </article>
    );
}
"#;

    let component = nexa_parser::parse_component("product.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);
    let (manifest, chunks) = nexa_activation::build(&ir, "ProductPage", &component.handlers, &Default::default());

    assert_eq!(manifest.len(), 1, "solo el botón es interactivo");
    assert_eq!(chunks.len(), 1, "solo el botón debe generar su propio chunk JS");

    let html = nexa_renderer::render_html_document(&ir, &nexa_renderer::RenderContext::empty());
    // El <h1> y el <p> estáticos no llevan marcador de activación.
    assert!(html.contains("<h1>iPhone 17</h1>"));
    assert!(!html.contains("<h1 data-nexa"));
    // El botón sí lo lleva, y el id coincide con la única entrada del manifiesto.
    assert!(html.contains("data-nexa=\""));
}

#[test]
fn chunk_contains_the_real_handler_source_when_its_declared_in_the_same_file() {
    let source = r#"
function buy() {
    cart.add(product.id);
}

export default function ProductPage() {
    return (
        <article>
            <h1>iPhone 17</h1>
            <button onClick={buy}>Comprar</button>
        </article>
    );
}
"#;

    let component = nexa_parser::parse_component("product.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);
    let (_manifest, chunks) = nexa_activation::build(&ir, "ProductPage", &component.handlers, &Default::default());

    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("function buy()"));
    assert!(chunks[0].content.contains("cart.add(product.id)"));
}
