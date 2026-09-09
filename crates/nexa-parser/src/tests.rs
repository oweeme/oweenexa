use super::*;
use nexa_ast::{Expr, JsonTemplate, Node, TemplatePart};

#[test]
fn parses_hello_nexa() {
    let source = r#"
export default function Home() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>Welcome to Nexa.</p>
        </main>
    );
}
"#;
    let component = parse_component("index.tsx", source).expect("should parse");
    assert_eq!(component.name, "Home");

    let Node::Element(root) = &component.root else {
        panic!("expected root element")
    };
    assert_eq!(root.tag, "main");
    assert_eq!(root.children.len(), 2);
}

#[test]
fn parses_attributes_and_void_elements() {
    let source = r#"
export default function Home() {
    return (
        <main>
            <img src="/logo.png" alt="Nexa" />
        </main>
    );
}
"#;
    let component = parse_component("index.tsx", source).expect("should parse");
    let Node::Element(root) = &component.root else {
        panic!("expected root element")
    };
    let Node::Element(img) = &root.children[0] else {
        panic!("expected img element")
    };
    assert_eq!(img.tag, "img");
    assert_eq!(img.attrs.len(), 2);
}

#[test]
fn parses_dynamic_attribute_values() {
    let source = r#"
export default function ProductPage() {
    return (
        <img src={data.image} alt={data.name} />
    );
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let Node::Element(img) = &component.root else {
        panic!("expected img element")
    };

    let src = img.attrs.iter().find(|a| a.name == "src").expect("expected src");
    assert_eq!(single_dynamic_path(&src.value), "data.image");

    let alt = img.attrs.iter().find(|a| a.name == "alt").expect("expected alt");
    assert_eq!(single_dynamic_path(&alt.value), "data.name");
}

#[test]
fn parses_a_multi_part_template_literal_in_an_attribute() {
    // Bug real (Fase 15): antes de esto, `href={\`/${a}/${b}\`}` no se
    // parecía a una única `Expr` (identificador o `a.b`), así que
    // `AttrValue::Dynamic` nunca se construía y el atributo terminaba
    // como `value: None` — indistinguible de un atributo booleano. El
    // HTML final mostraba `<a href>` sin ningún valor, en silencio.
    let source = r#"
export default function Home() {
    return (
        <a href={`/${params.locale}/products/${params.slug}`}>Ver</a>
    );
}
"#;
    let component = parse_component("home.tsx", source).expect("should parse");
    let Node::Element(a) = &component.root else {
        panic!("expected an <a> element")
    };

    let href = a.attrs.iter().find(|attr| attr.name == "href").expect("expected href");
    match &href.value {
        Some(nexa_ast::AttrValue::Dynamic(template)) => {
            // "/" + {locale} + "/products/" + {slug} — sin texto al
            // final, la plantilla termina justo tras `${params.slug}`.
            assert_eq!(template.0.len(), 4, "text/expr/text/expr: {:?}", template.0);
        }
        other => panic!("expected a dynamic (multi-part) href, got {other:?}"),
    }
}

#[test]
fn drops_an_attribute_entirely_when_its_expression_cannot_be_resolved_at_all() {
    // Una expresión que ni siquiera el mecanismo de plantillas reconoce
    // (aquí, una expresión aritmética) debe omitir el atributo por
    // completo — nunca dejarlo como si fuera un atributo booleano real
    // (`<div data-count>` en vez de nada).
    let source = r#"
export default function Counter() {
    return (
        <div data-count={1 + 1}>x</div>
    );
}
"#;
    let component = parse_component("counter.tsx", source).expect("should parse");
    let Node::Element(div) = &component.root else {
        panic!("expected a div element")
    };

    assert!(div.attrs.iter().all(|attr| attr.name != "data-count"), "attrs: {:?}", div.attrs);
}

/// Un atributo dinámico de un solo fragmento (`src={data.image}`, sin
/// texto alrededor): la forma más común, y la que tenían los tests desde
/// antes de que `AttrValue::Dynamic` pasara a `Template` (Fase 15) para
/// poder representar también `href={\`/${a}/${b}\`}`.
fn single_dynamic_path(value: &Option<nexa_ast::AttrValue>) -> String {
    match value {
        Some(nexa_ast::AttrValue::Dynamic(template)) => match template.0.as_slice() {
            [nexa_ast::TemplatePart::Expr(expr)] => expr.path(),
            other => panic!("expected a single Expr part, got {other:?}"),
        },
        other => panic!("expected a dynamic value, got {other:?}"),
    }
}

#[test]
fn parses_dynamic_expressions_and_events() {
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
    let component = parse_component("index.tsx", source).expect("should parse");

    let Node::Element(root) = &component.root else {
        panic!("expected root element")
    };

    let Node::Element(h1) = &root.children[0] else {
        panic!("expected h1 element")
    };
    let Node::Expression(expr) = &h1.children[0] else {
        panic!("expected a dynamic expression, got {:?}", h1.children[0])
    };
    assert_eq!(expr.path(), "product.name");
    assert_eq!(expr.root_identifier(), "product");

    let Node::Element(button) = &root.children[1] else {
        panic!("expected button element")
    };
    assert_eq!(button.events.len(), 1);
    assert_eq!(button.events[0].name, "click");
    assert_eq!(button.events[0].handler, Expr::Identifier("buy".to_string()));
}

#[test]
fn parses_loader_declaration() {
    let source = r#"
export const load = { url: "/api/products/:slug" };

export default function ProductPage() {
    return (
        <article>
            <h1>{data.name}</h1>
        </article>
    );
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let loader = component.loader.expect("expected a loader declaration");
    assert_eq!(loader.url_template, "/api/products/:slug");
}

#[test]
fn page_without_load_export_has_no_loader() {
    let source = r#"
export default function Home() {
    return <main>Hola</main>;
}
"#;
    let component = parse_component("index.tsx", source).expect("should parse");
    assert!(component.loader.is_none());
}

#[test]
fn parses_paths_declaration() {
    let source = r#"
export const paths = { url: "/api/products" };
export const load = { url: "/api/products/:slug" };

export default function ProductPage() {
    return (
        <article>
            <h1>{data.name}</h1>
        </article>
    );
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let paths = component.paths.expect("expected a paths declaration");
    assert_eq!(paths.url_template, "/api/products");
}

#[test]
fn page_without_paths_export_has_no_paths() {
    let source = r#"
export const load = { url: "/api/products/:slug" };

export default function ProductPage() {
    return <article>{data.name}</article>;
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    assert!(component.paths.is_none());
}

#[test]
fn extracts_the_source_of_a_top_level_function_handler() {
    let source = r#"
function buy() {
    console.log("comprando");
}

export default function ProductPage() {
    return (
        <button onClick={buy}>Comprar</button>
    );
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let handler = component.handlers.get("buy").expect("expected a `buy` handler");
    assert!(handler.contains("function buy()"));
    assert!(handler.contains("console.log(\"comprando\")"));
}

#[test]
fn extracts_the_source_of_an_arrow_function_handler() {
    let source = r#"
const buy = () => {
    console.log("comprando");
};

export default function ProductPage() {
    return (
        <button onClick={buy}>Comprar</button>
    );
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let handler = component.handlers.get("buy").expect("expected a `buy` handler");
    assert!(handler.starts_with("const buy ="));
    assert!(handler.contains("console.log(\"comprando\")"));
}

#[test]
fn parses_seo_with_static_and_dynamic_fields() {
    let source = r#"
export const seo = {
    title: `${data.name} | Oweeme`,
    description: "Descripción fija",
    canonical: "/producto/iphone-17",
    openGraph: {
        title: data.name,
        image: "/img/iphone.jpg"
    },
    twitter: {
        card: "summary_large_image"
    }
};

export default function ProductPage() {
    return <article>{data.name}</article>;
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let seo = component.seo.expect("expected an seo declaration");

    let title = seo.title.expect("expected a title");
    assert_eq!(title.0.len(), 2, "\"{{data.name}} | Oweeme\" son dos fragmentos");
    assert!(matches!(&title.0[0], TemplatePart::Expr(e) if e.path() == "data.name"));
    assert!(matches!(&title.0[1], TemplatePart::Text(t) if t == " | Oweeme"));

    assert_eq!(seo.description.unwrap().0, vec![TemplatePart::Text("Descripción fija".into())]);
    assert_eq!(seo.canonical.unwrap().0, vec![TemplatePart::Text("/producto/iphone-17".into())]);
    assert!(matches!(&seo.og_title.unwrap().0[0], TemplatePart::Expr(e) if e.path() == "data.name"));
    assert_eq!(seo.og_image.unwrap().0, vec![TemplatePart::Text("/img/iphone.jpg".into())]);
    assert_eq!(seo.twitter_card.unwrap().0, vec![TemplatePart::Text("summary_large_image".into())]);
}

#[test]
fn parses_seo_wrapped_in_define_seo() {
    let source = r#"
export const seo = defineSEO({
    title: "Título fijo"
});

export default function Home() {
    return <main>Hola</main>;
}
"#;
    let component = parse_component("index.tsx", source).expect("should parse");
    let seo = component.seo.expect("expected an seo declaration");
    assert_eq!(seo.title.unwrap().0, vec![TemplatePart::Text("Título fijo".into())]);
}

#[test]
fn parses_schema_as_nested_json_template() {
    let source = r#"
export const schema = {
    type: "Product",
    name: data.name,
    offers: {
        price: data.price,
        priceCurrency: "USD",
        availability: "https://schema.org/InStock"
    }
};

export default function ProductPage() {
    return <article>{data.name}</article>;
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let schema = component.schema.expect("expected a schema declaration");

    let JsonTemplate::Object(fields) = &schema else {
        panic!("expected a top-level object")
    };

    let get = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);

    assert_eq!(get("type"), Some(&JsonTemplate::String("Product".into())));
    assert!(matches!(get("name"), Some(JsonTemplate::Expr(e)) if e.path() == "data.name"));

    let Some(JsonTemplate::Object(offers)) = get("offers") else {
        panic!("expected a nested `offers` object")
    };
    let get_offer = |key: &str| offers.iter().find(|(k, _)| k == key).map(|(_, v)| v);
    assert!(matches!(get_offer("price"), Some(JsonTemplate::Expr(e)) if e.path() == "data.price"));
    assert_eq!(get_offer("priceCurrency"), Some(&JsonTemplate::String("USD".into())));
}

#[test]
fn page_without_seo_or_schema_has_neither() {
    let source = r#"
export default function Home() {
    return <main>Hola</main>;
}
"#;
    let component = parse_component("index.tsx", source).expect("should parse");
    assert!(component.seo.is_none());
    assert!(component.schema.is_none());
}

// Fase 30 — `<For each={...}>{(item) => (...)}</For>`.

#[test]
fn parses_a_for_loop_over_data() {
    let source = r#"
export default function Articles() {
    return (
        <ul>
            <For each={data.items}>
                {(item) => (
                    <li>
                        <a href={item.slug}>{item.title}</a>
                    </li>
                )}
            </For>
        </ul>
    );
}
"#;
    let component = parse_component("articles.tsx", source).expect("should parse");
    let Node::Element(root) = &component.root else {
        panic!("expected root element")
    };
    let Node::For(for_loop) = &root.children[0] else {
        panic!("expected a For node, got {:?}", root.children[0])
    };

    assert_eq!(for_loop.each.path(), "data.items");
    assert_eq!(for_loop.item_name, "item");

    let Node::Element(li) = for_loop.body.as_ref() else {
        panic!("expected the For body to be a single element")
    };
    assert_eq!(li.tag, "li");
    let Node::Element(a) = &li.children[0] else {
        panic!("expected an anchor inside li")
    };
    let href = a.attrs.iter().find(|attr| attr.name == "href").expect("expected href");
    assert_eq!(single_dynamic_path(&href.value), "item.slug");
    let Node::Expression(title_expr) = &a.children[0] else {
        panic!("expected {{item.title}} as the anchor's text")
    };
    assert_eq!(title_expr.path(), "item.title");
}

#[test]
fn parses_a_for_loop_over_params() {
    let source = r#"
export default function Gallery() {
    return (
        <For each={params.tags}>{(tag) => (<span>{tag}</span>)}</For>
    );
}
"#;
    let component = parse_component("gallery.tsx", source).expect("should parse");
    let Node::For(for_loop) = &component.root else {
        panic!("expected a top-level For node")
    };
    assert_eq!(for_loop.each.path(), "params.tags");
    assert_eq!(for_loop.item_name, "tag");
}

#[test]
fn for_loop_without_each_attribute_is_a_parse_error() {
    let source = r#"
export default function Articles() {
    return <For>{(item) => (<li>{item.title}</li>)}</For>;
}
"#;
    assert!(parse_component("articles.tsx", source).is_err());
}

#[test]
fn for_loop_with_each_on_an_unknown_root_is_a_parse_error() {
    let source = r#"
export default function Articles() {
    return <For each={articles}>{(item) => (<li>{item.title}</li>)}</For>;
}
"#;
    assert!(parse_component("articles.tsx", source).is_err());
}

#[test]
fn for_loop_with_an_extra_attribute_is_a_parse_error() {
    let source = r#"
export default function Articles() {
    return <For each={data.items} class="grid">{(item) => (<li>{item.title}</li>)}</For>;
}
"#;
    assert!(parse_component("articles.tsx", source).is_err());
}

#[test]
fn for_loop_whose_callback_is_not_an_arrow_function_is_a_parse_error() {
    let source = r#"
function renderItem(item) {
    return <li>{item.title}</li>;
}

export default function Articles() {
    return <For each={data.items}>{renderItem}</For>;
}
"#;
    assert!(parse_component("articles.tsx", source).is_err());
}

#[test]
fn for_loop_with_a_block_bodied_callback_is_a_parse_error() {
    let source = r#"
export default function Articles() {
    return (
        <For each={data.items}>
            {(item) => {
                return <li>{item.title}</li>;
            }}
        </For>
    );
}
"#;
    assert!(parse_component("articles.tsx", source).is_err());
}

#[test]
fn for_loop_with_a_destructured_param_is_a_parse_error() {
    let source = r#"
export default function Articles() {
    return <For each={data.items}>{({ title }) => (<li>{title}</li>)}</For>;
}
"#;
    assert!(parse_component("articles.tsx", source).is_err());
}

#[test]
fn for_loop_with_more_than_one_param_is_a_parse_error() {
    let source = r#"
export default function Articles() {
    return <For each={data.items}>{(item, index) => (<li>{item.title}</li>)}</For>;
}
"#;
    assert!(parse_component("articles.tsx", source).is_err());
}

// Fase 36 — el texto pegado a una expresión conserva su espacio.

#[test]
fn text_between_two_expressions_on_the_same_line_keeps_its_edge_spaces() {
    let source = r#"
export default function Product() {
    return <p>{data.name} — ${data.price}</p>;
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let Node::Element(p) = &component.root else {
        panic!("expected root element")
    };
    // name, " — $", price = 3 hijos; el texto del medio conserva el
    // espacio antes del guion Y el que sigue después (pegado al "$").
    assert_eq!(p.children.len(), 3);
    let Node::Text(middle) = &p.children[1] else {
        panic!("expected the middle child to be static text, got {:?}", p.children[1])
    };
    assert_eq!(middle, " — $");
}

// Fase 45 — interpolación en t("clave", { name: data.x }).

#[test]
fn parses_t_with_an_interpolation_argument() {
    let source = r#"
export default function Profile() {
    return <p>{t("profile.donateTo", { name: data.creatorName })}</p>;
}
"#;
    let component = parse_component("profile.tsx", source).expect("should parse");
    let Node::Element(p) = &component.root else {
        panic!("expected root element")
    };
    let Node::Translate(translate) = &p.children[0] else {
        panic!("expected a Translate node, got {:?}", p.children[0])
    };
    assert_eq!(translate.key, "profile.donateTo");
    assert_eq!(translate.args.len(), 1);
    assert_eq!(translate.args[0].0, "name");
    assert_eq!(translate.args[0].1.path(), "data.creatorName");
}

#[test]
fn parses_t_with_several_interpolation_arguments() {
    let source = r#"
export default function Profile() {
    return <p>{t("cart.summary", { count: data.count, slug: params.slug })}</p>;
}
"#;
    let component = parse_component("profile.tsx", source).expect("should parse");
    let Node::Element(p) = &component.root else {
        panic!("expected root element")
    };
    let Node::Translate(translate) = &p.children[0] else {
        panic!("expected a Translate node, got {:?}", p.children[0])
    };
    assert_eq!(translate.args.len(), 2);
    assert_eq!(translate.args[0], ("count".to_string(), Expr::Member { object: Box::new(Expr::Identifier("data".into())), property: "count".into() }));
    assert_eq!(translate.args[1].1.path(), "params.slug");
}

#[test]
fn t_without_a_second_argument_still_parses_with_no_args() {
    let source = r#"
export default function Home() {
    return <h1>{t("home.title")}</h1>;
}
"#;
    let component = parse_component("home.tsx", source).expect("should parse");
    let Node::Element(h1) = &component.root else {
        panic!("expected root element")
    };
    let Node::Translate(translate) = &h1.children[0] else {
        panic!("expected a Translate node, got {:?}", h1.children[0])
    };
    assert_eq!(translate.key, "home.title");
    assert!(translate.args.is_empty());
}

#[test]
fn t_with_a_literal_value_in_the_interpolation_object_is_not_recognized_as_translate() {
    // El segundo argumento solo acepta referencias (`data.x`/`params.x`),
    // igual que cualquier otra posición dinámica de Nexa — un literal
    // (`name: "Ada"`) no es una `Expr`, así que el `t(...)` entero no se
    // reconoce. Una llamada (`CallExpression`) tampoco es un `Expr`
    // genérico válido en ningún otro lado de Nexa, así que el nodo entero
    // se descarta — ni crashea el build, ni inventa contenido.
    let source = r#"
export default function Profile() {
    return <p>{t("profile.donateTo", { name: "Ada" })}</p>;
}
"#;
    let component = parse_component("profile.tsx", source).expect("should parse");
    let Node::Element(p) = &component.root else {
        panic!("expected root element")
    };
    assert!(p.children.is_empty(), "expected the unrecognized t() call to be dropped, got {:?}", p.children);
}

#[test]
fn t_with_more_than_two_arguments_is_not_recognized_as_translate() {
    let source = r#"
export default function Home() {
    return <h1>{t("home.title", { name: data.x }, "extra")}</h1>;
}
"#;
    let component = parse_component("home.tsx", source).expect("should parse");
    let Node::Element(h1) = &component.root else {
        panic!("expected root element")
    };
    assert!(h1.children.is_empty(), "expected the unrecognized t() call to be dropped, got {:?}", h1.children);
}

#[test]
fn t_in_seo_title_supports_interpolation_too() {
    let source = r#"
export const seo = {
    title: t("product.title", { name: data.name }),
    description: "d",
    canonical: "/",
};

export default function Product() {
    return <h1>{data.name}</h1>;
}
"#;
    let component = parse_component("product.tsx", source).expect("should parse");
    let seo = component.seo.expect("expected seo");
    let title = seo.title.expect("expected seo.title");
    assert_eq!(title.0.len(), 1);
    let TemplatePart::Translate(translate) = &title.0[0] else {
        panic!("expected seo.title to be a single Translate part, got {:?}", title.0[0])
    };
    assert_eq!(translate.key, "product.title");
    assert_eq!(translate.args[0].1.path(), "data.name");
}
