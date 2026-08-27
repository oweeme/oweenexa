//! Prueba de extremo a extremo del criterio de salida de la Fase 8: una
//! página de producto generada por Nexa produce un JSON-LD de Product
//! que cumple los campos que Google exige/recomienda para Rich Results
//! (nombre, imagen, y `offers` con `price`/`priceCurrency`/`availability`).
//!
//! Nota: se valida contra los requisitos documentados de Google para
//! Product structured data (no se envía a la herramienta Rich Results
//! Test real — este entorno no tiene acceso a un navegador/API externa).

use std::collections::BTreeMap;

use serde_json::Value;

#[test]
fn product_page_generates_valid_rich_results_schema() {
    let source = r#"
export const seo = {
    title: `${data.name} | Oweeme`,
    description: data.description,
    canonical: `/producto/${params.slug}`,
    openGraph: {
        title: data.name,
        image: data.image
    }
};

export const schema = {
    type: "Product",
    name: data.name,
    description: data.description,
    image: data.image,
    offers: {
        type: "Offer",
        price: data.price,
        priceCurrency: "USD",
        availability: "https://schema.org/InStock",
        url: `https://oweeme.example/producto/${params.slug}`
    }
};

export default function ProductPage() {
    return (
        <article>
            <h1>{data.name}</h1>
            <img src={data.image} alt={data.name} />
        </article>
    );
}
"#;

    let component = nexa_parser::parse_component("product.tsx", source).expect("should parse");

    let data = serde_json::json!({
        "name": "iPhone 17",
        "description": "El último de Apple",
        "image": "https://oweeme.example/img/iphone17.jpg",
        "price": 999.0
    });
    let mut params = BTreeMap::new();
    params.insert("slug".to_string(), "iphone-17".to_string());

    let ctx = nexa_seo::SeoContext { data: Some(&data), params: &params, translations: None };

    // --- <head> ---
    let head = nexa_seo::render_head(component.seo.as_ref(), &ctx);
    assert!(head.contains("<title>iPhone 17 | Oweeme</title>"));
    assert!(head.contains("El último de Apple"));
    assert!(head.contains("/producto/iphone-17"));
    assert!(head.contains("og:image"));

    // --- JSON-LD ---
    let script = nexa_seo::render_schema_script(component.schema.as_ref(), &ctx)
        .expect("expected a schema script");

    let json_text = script
        .strip_prefix("<script type=\"application/ld+json\">")
        .and_then(|s| s.strip_suffix("</script>"))
        .expect("expected the ld+json script wrapper");
    let value: Value = serde_json::from_str(json_text).expect("schema debe ser JSON válido");

    // Requisitos documentados por Google para "Product" Rich Results:
    // @context/@type, name, y (si se usa `offers`) price + priceCurrency;
    // recomendado: image, description, availability, url.
    assert_eq!(value["@context"], "https://schema.org");
    assert_eq!(value["@type"], "Product");
    assert_eq!(value["name"], "iPhone 17");
    assert_eq!(value["description"], "El último de Apple");
    assert_eq!(value["image"], "https://oweeme.example/img/iphone17.jpg");

    let offers = &value["offers"];
    assert_eq!(offers["@type"], "Offer");
    assert_eq!(offers["price"], 999.0);
    assert_eq!(offers["priceCurrency"], "USD");
    assert_eq!(offers["availability"], "https://schema.org/InStock");
    assert_eq!(offers["url"], "https://oweeme.example/producto/iphone-17");

    // --- SEO Analyzer: todo declarado, cero warnings ---
    let ir = nexa_analyzer::analyze(&component);
    let warnings = nexa_seo::analyze(&ir.root, component.seo.as_ref());
    assert!(warnings.is_empty(), "no debería haber warnings: {warnings:?}");
}

#[test]
fn seo_title_and_schema_name_resolve_a_translate_call() {
    let source = r#"
export const seo = {
    title: t("home.title"),
    canonical: `/${params.locale}`
};

export const schema = {
    type: "WebPage",
    name: t("home.title")
};

export default function Home() {
    return <main><h1>{t("home.title")}</h1></main>;
}
"#;

    let component = nexa_parser::parse_component("home.tsx", source).expect("should parse");

    let translations = serde_json::json!({
        "home": { "title": "Bienvenido a Oweeme" }
    });
    let mut params = BTreeMap::new();
    params.insert("locale".to_string(), "es".to_string());

    let ctx = nexa_seo::SeoContext { data: None, params: &params, translations: Some(&translations) };

    let head = nexa_seo::render_head(component.seo.as_ref(), &ctx);
    assert!(head.contains("<title>Bienvenido a Oweeme</title>"));
    assert!(head.contains("/es"));

    let script = nexa_seo::render_schema_script(component.schema.as_ref(), &ctx)
        .expect("expected a schema script");
    assert!(script.contains("\"name\":\"Bienvenido a Oweeme\""));
}

#[test]
fn seo_title_with_a_missing_translation_key_is_omitted_not_guessed() {
    let source = r#"
export const seo = {
    title: t("home.title")
};

export default function Home() {
    return <main />;
}
"#;

    let component = nexa_parser::parse_component("home.tsx", source).expect("should parse");
    let params = BTreeMap::new();
    let ctx = nexa_seo::SeoContext { data: None, params: &params, translations: None };

    let head = nexa_seo::render_head(component.seo.as_ref(), &ctx);
    assert!(!head.contains("<title>"));
}

#[test]
fn page_missing_seo_and_alt_gets_flagged_by_the_analyzer() {
    let source = r#"
export default function Gallery() {
    return (
        <main>
            <img src="/a.jpg" />
        </main>
    );
}
"#;

    let component = nexa_parser::parse_component("gallery.tsx", source).expect("should parse");
    let ir = nexa_analyzer::analyze(&component);
    let warnings = nexa_seo::analyze(&ir.root, component.seo.as_ref());

    let codes: Vec<&str> = warnings.iter().map(|w| w.code).collect();
    assert!(codes.contains(&"NEXA-A11Y-001"));
    assert!(codes.contains(&"NEXA-SEO-001"));
    assert!(codes.contains(&"NEXA-SEO-002"));
    assert!(codes.contains(&"NEXA-SEO-003"));
}
