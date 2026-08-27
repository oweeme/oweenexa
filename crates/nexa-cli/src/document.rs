//! Ensambla el documento HTML final de una página: el `<head>` base +
//! lo que resolvió `nexa-seo` (título/meta/OG/Twitter + JSON-LD) + el
//! CSS de `@nexa/ui` que de verdad se usó (Fase 9) + el cuerpo que ya
//! renderizó `nexa-renderer`.
//!
//! Separado de `pipeline.rs` porque es un paso de ensamblado, no de
//! compilación — junta piezas que ya vienen resueltas.

/// `seo_head` puede venir vacío (página sin `seo` declarado, o con
/// campos que no se pudieron resolver): en ese caso se usa un `<title>`
/// por defecto en vez de dejar la página sin ninguno. `ui_stylesheet_href`
/// solo se pasa si `nexa-ui` encontró algún componente usado — una
/// página que no usa `@nexa/ui` no lleva ni el `<link>`. `import_map`
/// (Fase 15) va primero que cualquier otra cosa en `<head>` — un import
/// map debe declararse antes de cualquier `<script type="module">` que
/// dependa de él.
pub fn assemble(
    body: &str,
    seo_head: &str,
    schema_script: Option<&str>,
    ui_stylesheet_href: Option<&str>,
    import_map: Option<&str>,
) -> String {
    let mut head_lines = Vec::new();
    if let Some(script) = import_map {
        head_lines.push(script.to_string());
    }
    head_lines.push("<meta charset=\"UTF-8\">".to_string());
    head_lines.push("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">".to_string());

    if !seo_head.contains("<title>") {
        head_lines.push("<title>Nexa</title>".to_string());
    }
    if !seo_head.is_empty() {
        head_lines.push(seo_head.to_string());
    }
    if let Some(script) = schema_script {
        head_lines.push(script.to_string());
    }
    if let Some(href) = ui_stylesheet_href {
        head_lines.push(format!("<link rel=\"stylesheet\" href=\"{href}\">"));
    }

    format!(
        "<!doctype html>\n<html lang=\"es\">\n<head>\n{}\n</head>\n<body>\n{body}\n</body>\n</html>\n",
        head_lines.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_a_default_title_when_seo_has_none() {
        let html = assemble("<p>hola</p>", "", None, None, None);
        assert!(html.contains("<title>Nexa</title>"));
    }

    #[test]
    fn uses_the_seo_title_instead_of_the_default() {
        let html = assemble("<p>hola</p>", "<title>iPhone 17</title>", None, None, None);
        assert!(html.contains("<title>iPhone 17</title>"));
        assert!(!html.contains("<title>Nexa</title>"));
    }

    #[test]
    fn includes_the_schema_script_when_given() {
        let html = assemble("<p>hola</p>", "", Some("<script type=\"application/ld+json\">{}</script>"), None, None);
        assert!(html.contains("application/ld+json"));
    }

    #[test]
    fn omits_the_ui_stylesheet_link_when_not_given() {
        let html = assemble("<p>hola</p>", "", None, None, None);
        assert!(!html.contains("stylesheet"));
    }

    #[test]
    fn includes_the_ui_stylesheet_link_when_given() {
        let html = assemble("<p>hola</p>", "", None, Some("/assets/nexa-ui.css"), None);
        assert!(html.contains("<link rel=\"stylesheet\" href=\"/assets/nexa-ui.css\">"));
    }

    #[test]
    fn omits_the_import_map_when_not_given() {
        let html = assemble("<p>hola</p>", "", None, None, None);
        assert!(!html.contains("importmap"));
    }

    #[test]
    fn the_import_map_comes_before_anything_else_in_head() {
        let html = assemble(
            "<p>hola</p>",
            "<title>x</title>",
            None,
            None,
            Some("<script type=\"importmap\">{\"imports\":{\"stripe\":\"x\"}}</script>"),
        );

        let head_start = html.find("<head>").unwrap();
        let importmap_pos = html.find("importmap").unwrap();
        let title_pos = html.find("<title>").unwrap();
        assert!(head_start < importmap_pos);
        assert!(importmap_pos < title_pos);
    }
}
