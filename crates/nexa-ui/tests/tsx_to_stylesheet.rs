//! Prueba de extremo a extremo del criterio de salida de la Fase 9: un
//! proyecto que no importa `@nexa/ui` no paga ningún costo de CSS por
//! ella, y uno que usa un componente concreto solo paga por ese.

#[test]
fn a_plain_page_ships_zero_nexa_ui_css() {
    let source = r#"
export default function Home() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>Sin ningún componente de @nexa/ui.</p>
        </main>
    );
}
"#;
    let component = nexa_parser::parse_component("index.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);

    assert_eq!(nexa_ui::build_stylesheet(&ir.root), None);
}

#[test]
fn a_page_using_the_button_ships_only_tokens_and_button_css() {
    let source = r#"
export default function ProductPage() {
    return (
        <article>
            <h1>iPhone 17</h1>
            <button class="nx-btn nx-btn-primary" onClick={buy}>Comprar</button>
        </article>
    );
}
"#;
    let component = nexa_parser::parse_component("product.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);

    let css = nexa_ui::build_stylesheet(&ir.root).expect("expected some css");
    assert!(css.contains("--nx-color-primary"));
    assert!(css.contains(".nx-btn {"));
    assert!(!css.contains(".nx-dialog {"));
    assert!(!css.contains(".nx-card {"));
    assert!(!css.contains(".nx-input {"));
}

#[test]
fn a_page_using_a_table_and_badges_ships_only_those_two_families() {
    let source = r#"
export default function Orders() {
    return (
        <div class="nx-table-wrap">
            <table class="nx-table nx-table-zebra">
                <thead><tr><th>Pedido</th><th>Estado</th></tr></thead>
                <tbody>
                    <tr><td>#1024</td><td><span class="nx-badge nx-badge-success">Pagado</span></td></tr>
                    <tr><td>#1025</td><td><span class="nx-badge nx-badge-warning">Pendiente</span></td></tr>
                </tbody>
            </table>
        </div>
    );
}
"#;
    let component = nexa_parser::parse_component("orders.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);

    let css = nexa_ui::build_stylesheet(&ir.root).expect("expected some css");
    assert!(css.contains(".nx-table {"));
    assert!(css.contains(".nx-badge {"));
    assert!(!css.contains(".nx-card {"));
    assert!(!css.contains(".nx-dialog {"));
    assert!(!css.contains(".nx-avatar {"));
}
