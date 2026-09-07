//! Genera el `<script>` de arranque que un documento HTML necesita para
//! que Progressive Activation, la navegación tipo SPA y la mejora
//! progresiva de formularios funcionen de verdad en un navegador — el
//! pendiente que había quedado abierto desde la Fase 5 ("el HTML no
//! incluye el `<script>` que cargaría `@nexa/runtime`").

use nexa_activation::ActivationManifest;

use crate::assets;

/// Inserta el script de arranque justo antes de `</body>`.
///
/// El router (navegación SPA) se carga siempre: es lo que hace que un
/// clic interno no recargue la página, y toda la web se beneficia de
/// eso, no solo las páginas con contenido interactivo (ver Fase 6). La
/// activación (`@nexa/runtime`) solo se carga si el manifiesto tiene al
/// menos una entrada (Fase 5), `@nexa/forms` solo si `has_forms` es
/// `true` (Fase 10), y `@nexa/telemetry` (Fase 14) solo si
/// `telemetry_endpoint` es `Some` — que a su vez solo pasa si
/// `nexa.toml` declara `[telemetry] endpoint = "..."`: sin endpoint no
/// hay a dónde enviar nada, así que ni se carga el módulo. `@nexa/islands`
/// (Fase 16) solo si `has_islands` es `true` — misma disciplina de costo
/// cero que `has_forms`. `has_pwa` (Fase 18) registra `/sw.js` — no hace
/// falta un paquete/import propio, es una sola llamada nativa del
/// navegador.
///
/// **Reactivación en navegación SPA (bug real, encontrado con un
/// navegador real):** `initRouter` reemplaza `<body>` vía `innerHTML` —
/// eso nunca ejecuta el `<script>` de la página de destino (es
/// comportamiento estándar del navegador, no algo que se pueda evitar
/// desde `packages/router`). Sin nada más, cualquier página a la que se
/// navega por un link interno queda con el HTML correcto pero CERO JS
/// activado: ni botones, ni formularios, ni islas. Por eso el manifiesto
/// se embebe también como datos inertes (`<script
/// type="application/json" data-nexa-manifest>`, que sí sobrevive el
/// reemplazo, a diferencia de una llamada dentro de un `<script
/// type="module">`), y **toda página** — tenga o no contenido
/// interactivo propio — lleva una función `reactivate(root)` que
/// `initRouter` llama después de cada navegación: relee ese manifiesto
/// del DOM ya reemplazado, y carga (con `import()` dinámico — recién
/// ahí, nunca antes) lo que la página de destino necesite.
pub fn inject(
    html: &str,
    manifest: &ActivationManifest,
    has_forms: bool,
    has_islands: bool,
    has_pwa: bool,
    telemetry_endpoint: Option<&str>,
) -> String {
    let mut prelude = String::new();
    if !manifest.is_empty() {
        let manifest_json = manifest.to_json_pretty().unwrap_or_else(|_| "{}".to_string());
        // Mismo cuidado que `nexa-seo::schema::render_schema_script`:
        // un `</script>` dentro de un valor de texto no debe poder
        // cerrar este tag antes de tiempo.
        let safe_json = manifest_json.replace("</script>", "<\\/script>");
        prelude.push_str(&format!("<script type=\"application/json\" data-nexa-manifest>{safe_json}</script>\n"));
    }

    let mut script = String::from("<script type=\"module\">\n");
    script.push_str(&format!(
        "import {{ initRouter, initPrefetch }} from \"/assets/{}\";\n",
        assets::nexa_router_filename()
    ));

    if !manifest.is_empty() {
        script.push_str(&format!(
            "import {{ initActivation }} from \"/assets/{}\";\n",
            assets::nexa_runtime_filename()
        ));
    }
    if has_forms {
        script.push_str(&format!("import {{ initForms }} from \"/assets/{}\";\n", assets::nexa_forms_filename()));
    }
    if has_islands {
        script.push_str(&format!("import {{ initIslands }} from \"/assets/{}\";\n", assets::nexa_islands_filename()));
    }
    if telemetry_endpoint.is_some() {
        script.push_str(&format!(
            "import {{ initTelemetry }} from \"/assets/{}\";\n",
            assets::nexa_telemetry_filename()
        ));
    }

    script.push_str(&reactivate_fn());

    script.push_str("initRouter({ onNavigate: reactivate });\ninitPrefetch();\n");

    if !manifest.is_empty() {
        // Lee del mismo `<script data-nexa-manifest>` inerte de arriba
        // en vez de embeber el JSON una segunda vez acá — una sola
        // fuente de verdad, y una sola vez por la que hay que
        // preocuparse de escapar `</script>` dentro de un valor.
        script.push_str(
            "disposeActivation = initActivation(JSON.parse(document.querySelector(\"script[data-nexa-manifest]\").textContent));\n",
        );
    }
    if has_forms {
        script.push_str("disposeForms = initForms();\n");
    }
    if has_islands {
        script.push_str("disposeIslands = initIslands();\n");
    }
    if let Some(endpoint) = telemetry_endpoint {
        // `serde_json::to_string` en vez de interpolar el string a mano:
        // así un endpoint con comillas u otros caracteres especiales no
        // rompe (ni escapa fuera de) el `<script>` generado.
        let endpoint_literal = serde_json::to_string(endpoint).unwrap_or_else(|_| "\"\"".to_string());
        script.push_str(&format!("initTelemetry({{ endpoint: {endpoint_literal} }});\n"));
    }
    if has_pwa {
        script.push_str("if (\"serviceWorker\" in navigator) navigator.serviceWorker.register(\"/sw.js\");\n");
    }

    script.push_str("</script>\n");

    insert_before_closing_tag(html, "body", &format!("{prelude}{script}"))
}

/// Se emite igual en toda página, tenga o no contenido interactivo
/// propio: cualquier página puede navegarse *hacia* una que sí lo
/// tenga. Usa `import()` dinámico a propósito — recién se paga el costo
/// de `@nexa/runtime`/`forms`/`islands` si la página de destino de
/// verdad los necesita, nunca antes. Los `import()` de una página que
/// además los cargó estático (arriba) resuelven del propio caché de
/// módulos del navegador — no hay descarga duplicada.
fn reactivate_fn() -> String {
    format!(
        r#"
let disposeActivation = () => {{}};
let disposeForms = () => {{}};
let disposeIslands = () => {{}};

async function reactivate(root) {{
    disposeActivation();
    disposeForms();
    disposeIslands();
    disposeActivation = disposeForms = disposeIslands = () => {{}};

    const manifestEl = root.querySelector("script[data-nexa-manifest]");
    if (manifestEl) {{
        const {{ initActivation }} = await import("/assets/{runtime}");
        disposeActivation = initActivation(JSON.parse(manifestEl.textContent), {{ root }});
    }}
    if (root.querySelector("[data-nexa-form]")) {{
        const {{ initForms }} = await import("/assets/{forms}");
        disposeForms = initForms(root);
    }}
    if (root.querySelector("[data-nexa-island]")) {{
        const {{ initIslands }} = await import("/assets/{islands}");
        disposeIslands = initIslands({{ root }});
    }}
}}
"#,
        runtime = assets::nexa_runtime_filename(),
        forms = assets::nexa_forms_filename(),
        islands = assets::nexa_islands_filename(),
    )
}

/// Inserta `fragment` justo antes de `</{tag}>` (o al final del
/// documento si esa etiqueta no existe). La usa este módulo para
/// `</body>` y `nexa dev` (Fase 11) para `</head>` — el script de
/// auto-reload vive en `<head>` a propósito, para sobrevivir al
/// reemplazo del `<body>` que él mismo dispara.
pub(crate) fn insert_before_closing_tag(html: &str, tag: &str, fragment: &str) -> String {
    let needle = format!("</{tag}>");
    match html.rfind(&needle) {
        Some(pos) => format!("{}{fragment}{}", &html[..pos], &html[pos..]),
        None => format!("{html}{fragment}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_activation::{ActivationEntry, Strategy};

    #[test]
    fn always_loads_the_router_even_without_interactive_nodes() {
        let html = "<html><body><p>hola</p></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, false, None);

        assert!(out.contains(&format!("/assets/{}", assets::nexa_router_filename())));
        // Ni `@nexa/runtime` ni `@nexa/forms` se cargan de forma
        // *estática* (import de nivel superior) para esta página — pero
        // `reactivate()` (Fase 22) sí puede pedirlos con `import()`
        // dinámico si algún día se navega A una página que sí los usa;
        // esa referencia sí aparece en el texto de esa función, a
        // propósito, y no debe confundirse con una carga eager.
        assert!(!out.contains("import { initActivation }"));
        assert!(!out.contains("import { initForms }"));
        assert!(out.contains("initRouter({ onNavigate: reactivate });"));
    }

    #[test]
    fn every_page_defines_reactivate_even_with_no_interactive_content_of_its_own() {
        // Bug real: sin esto, navegar (SPA) DESDE una página estática
        // HACIA una con botones/formularios/islas dejaba esa página de
        // destino con cero JS activado — un <script> insertado vía
        // innerHTML nunca se ejecuta solo.
        let html = "<html><body><p>hola</p></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, false, None);

        assert!(out.contains("async function reactivate(root)"));
        assert!(out.contains("initRouter({ onNavigate: reactivate });"));
    }

    #[test]
    fn embeds_the_manifest_as_inert_json_that_survives_an_innerhtml_swap() {
        let mut manifest = ActivationManifest::new();
        manifest.insert(
            3,
            ActivationEntry {
                event: "click".into(),
                handler: "buy".into(),
                module: "/assets/ProductPage-3.js".into(),
                strategy: Strategy::Interaction,
            },
        );

        let html = "<html><body><button data-nexa=\"3\">Comprar</button></body></html>";
        let out = inject(html, &manifest, false, false, false, None);

        assert!(out.contains("<script type=\"application/json\" data-nexa-manifest>"));
        assert!(out.contains("\"handler\": \"buy\""));
        // El script inerte tiene que ir ANTES del <script type="module">
        // — no hace falta que ninguno se ejecute en un orden particular
        // para funcionar, pero así es más fácil de leer en el HTML real.
        let inert_pos = out.find("data-nexa-manifest").unwrap();
        let module_pos = out.find("<script type=\"module\">").unwrap();
        assert!(inert_pos < module_pos);
    }

    #[test]
    fn escapes_a_closing_script_tag_inside_the_embedded_manifest() {
        let mut manifest = ActivationManifest::new();
        manifest.insert(
            1,
            ActivationEntry {
                event: "click".into(),
                handler: "</script><script>alert(1)".into(),
                module: "/assets/x-1.js".into(),
                strategy: Strategy::Interaction,
            },
        );

        let html = "<html><body><button data-nexa=\"1\"></button></body></html>";
        let out = inject(html, &manifest, false, false, false, None);

        assert!(!out.contains("</script><script>alert(1)"));
    }

    #[test]
    fn loads_the_activation_runtime_when_the_manifest_has_entries() {
        let mut manifest = ActivationManifest::new();
        manifest.insert(
            3,
            ActivationEntry {
                event: "click".into(),
                handler: "buy".into(),
                module: "/assets/ProductPage-3.js".into(),
                strategy: Strategy::Interaction,
            },
        );

        let html = "<html><body><button data-nexa=\"3\">Comprar</button></body></html>";
        let out = inject(html, &manifest, false, false, false, None);

        assert!(out.contains(&format!("/assets/{}", assets::nexa_runtime_filename())));
        assert!(out.contains("initActivation("));
        assert!(out.contains("\"handler\": \"buy\""));
    }

    #[test]
    fn loads_forms_only_when_the_page_has_one() {
        let html = "<html><body><form data-nexa-form></form></body></html>";
        let out = inject(html, &ActivationManifest::new(), true, false, false, None);

        assert!(out.contains(&format!("/assets/{}", assets::nexa_forms_filename())));
        assert!(out.contains("initForms();"));
    }

    #[test]
    fn loads_telemetry_only_when_an_endpoint_is_given() {
        let html = "<html><body></body></html>";

        let without = inject(html, &ActivationManifest::new(), false, false, false, None);
        assert!(!without.contains("nexa-telemetry."));

        let with = inject(html, &ActivationManifest::new(), false, false, false, Some("/api/telemetry"));
        assert!(with.contains(&format!("/assets/{}", assets::nexa_telemetry_filename())));
        assert!(with.contains("initTelemetry({ endpoint: \"/api/telemetry\" });"));
    }

    #[test]
    fn escapes_a_telemetry_endpoint_with_special_characters() {
        let html = "<html><body></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, false, Some("/x\"; alert(1); //"));

        // El endpoint queda como un literal de string JS válido — no
        // rompe (ni escapa) el resto del `<script>`.
        assert!(out.contains("initTelemetry({ endpoint: \"/x\\\"; alert(1); //\" });"));
    }

    #[test]
    fn loads_islands_only_when_the_page_has_one() {
        let html = "<html><body><div data-nexa-island=\"x\"></div></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, true, false, None);

        assert!(out.contains(&format!("/assets/{}", assets::nexa_islands_filename())));
        assert!(out.contains("initIslands();"));
    }

    #[test]
    fn does_not_load_islands_when_the_page_has_none() {
        let html = "<html><body><p>hola</p></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, false, None);

        // No hay import estático ni llamada inicial a initIslands() —
        // `reactivate()` sí puede pedirlo dinámico para una página de
        // destino distinta, eso es intencional (ver el test de arriba).
        assert!(!out.contains("import { initIslands }"));
        assert!(!out.contains("disposeIslands = initIslands();"));
    }

    #[test]
    fn registers_the_service_worker_only_when_the_page_declares_pwa() {
        let html = "<html><body></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, true, None);

        assert!(out.contains("navigator.serviceWorker.register(\"/sw.js\")"));
    }

    #[test]
    fn does_not_register_a_service_worker_without_pwa() {
        let html = "<html><body></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, false, None);

        assert!(!out.contains("serviceWorker"));
    }

    #[test]
    fn inserts_before_the_closing_body_tag() {
        let html = "<html><body><p>hola</p></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, false, None);

        let script_pos = out.find("<script").unwrap();
        let body_close_pos = out.find("</body>").unwrap();
        assert!(script_pos < body_close_pos);
    }

    #[test]
    fn insert_before_closing_tag_targets_the_requested_tag() {
        let html = "<html><head><title>t</title></head><body></body></html>";
        let out = insert_before_closing_tag(html, "head", "<meta>");

        assert!(out.contains("<meta></head>"));
        assert!(!out.contains("<meta></body>"));
    }

    #[test]
    fn insert_before_closing_tag_appends_when_the_tag_is_missing() {
        let out = insert_before_closing_tag("<p>hola</p>", "head", "<meta>");
        assert_eq!(out, "<p>hola</p><meta>");
    }
}
