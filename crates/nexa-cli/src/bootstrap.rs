//! Genera el `<script>` de arranque que un documento HTML necesita para
//! que Progressive Activation, la navegación tipo SPA y la mejora
//! progresiva de formularios funcionen de verdad en un navegador — el
//! pendiente que había quedado abierto desde la Fase 5 ("el HTML no
//! incluye el `<script>` que cargaría `@nexa/runtime`").

use nexa_activation::ActivationManifest;

use crate::assets::{
    NEXA_FORMS_FILENAME, NEXA_ISLANDS_FILENAME, NEXA_ROUTER_FILENAME, NEXA_RUNTIME_FILENAME,
    NEXA_TELEMETRY_FILENAME,
};

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
/// cero que `has_forms`.
pub fn inject(
    html: &str,
    manifest: &ActivationManifest,
    has_forms: bool,
    has_islands: bool,
    telemetry_endpoint: Option<&str>,
) -> String {
    let mut script = String::from("<script type=\"module\">\n");
    script.push_str(&format!(
        "import {{ initRouter, initPrefetch }} from \"/assets/{NEXA_ROUTER_FILENAME}\";\n"
    ));

    if !manifest.is_empty() {
        script.push_str(&format!(
            "import {{ initActivation }} from \"/assets/{NEXA_RUNTIME_FILENAME}\";\n"
        ));
    }
    if has_forms {
        script.push_str(&format!("import {{ initForms }} from \"/assets/{NEXA_FORMS_FILENAME}\";\n"));
    }
    if has_islands {
        script.push_str(&format!("import {{ initIslands }} from \"/assets/{NEXA_ISLANDS_FILENAME}\";\n"));
    }
    if telemetry_endpoint.is_some() {
        script.push_str(&format!("import {{ initTelemetry }} from \"/assets/{NEXA_TELEMETRY_FILENAME}\";\n"));
    }

    script.push_str("initRouter();\ninitPrefetch();\n");

    if !manifest.is_empty() {
        let manifest_json = manifest.to_json_pretty().unwrap_or_else(|_| "{}".to_string());
        script.push_str(&format!("initActivation({manifest_json});\n"));
    }
    if has_forms {
        script.push_str("initForms();\n");
    }
    if has_islands {
        script.push_str("initIslands();\n");
    }
    if let Some(endpoint) = telemetry_endpoint {
        // `serde_json::to_string` en vez de interpolar el string a mano:
        // así un endpoint con comillas u otros caracteres especiales no
        // rompe (ni escapa fuera de) el `<script>` generado.
        let endpoint_literal = serde_json::to_string(endpoint).unwrap_or_else(|_| "\"\"".to_string());
        script.push_str(&format!("initTelemetry({{ endpoint: {endpoint_literal} }});\n"));
    }

    script.push_str("</script>\n");

    insert_before_closing_tag(html, "body", &script)
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
        let out = inject(html, &ActivationManifest::new(), false, false, None);

        assert!(out.contains("/assets/nexa-router.js"));
        assert!(!out.contains("/assets/nexa-runtime.js"));
        assert!(!out.contains("/assets/nexa-forms.js"));
        assert!(out.contains("initRouter();"));
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
        let out = inject(html, &manifest, false, false, None);

        assert!(out.contains("/assets/nexa-runtime.js"));
        assert!(out.contains("initActivation("));
        assert!(out.contains("\"handler\": \"buy\""));
    }

    #[test]
    fn loads_forms_only_when_the_page_has_one() {
        let html = "<html><body><form data-nexa-form></form></body></html>";
        let out = inject(html, &ActivationManifest::new(), true, false, None);

        assert!(out.contains("/assets/nexa-forms.js"));
        assert!(out.contains("initForms();"));
    }

    #[test]
    fn loads_telemetry_only_when_an_endpoint_is_given() {
        let html = "<html><body></body></html>";

        let without = inject(html, &ActivationManifest::new(), false, false, None);
        assert!(!without.contains("nexa-telemetry.js"));

        let with = inject(html, &ActivationManifest::new(), false, false, Some("/api/telemetry"));
        assert!(with.contains("/assets/nexa-telemetry.js"));
        assert!(with.contains("initTelemetry({ endpoint: \"/api/telemetry\" });"));
    }

    #[test]
    fn escapes_a_telemetry_endpoint_with_special_characters() {
        let html = "<html><body></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, Some("/x\"; alert(1); //"));

        // El endpoint queda como un literal de string JS válido — no
        // rompe (ni escapa) el resto del `<script>`.
        assert!(out.contains("initTelemetry({ endpoint: \"/x\\\"; alert(1); //\" });"));
    }

    #[test]
    fn loads_islands_only_when_the_page_has_one() {
        let html = "<html><body><div data-nexa-island=\"x\"></div></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, true, None);

        assert!(out.contains("/assets/nexa-islands.js"));
        assert!(out.contains("initIslands();"));
    }

    #[test]
    fn does_not_load_islands_when_the_page_has_none() {
        let html = "<html><body><p>hola</p></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, None);

        assert!(!out.contains("nexa-islands.js"));
        assert!(!out.contains("initIslands();"));
    }

    #[test]
    fn inserts_before_the_closing_body_tag() {
        let html = "<html><body><p>hola</p></body></html>";
        let out = inject(html, &ActivationManifest::new(), false, false, None);

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
