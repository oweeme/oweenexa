//! `export const head` (Fase 27, `src/layout.tsx`; Fase 37, también en
//! una página individual): favicon, stylesheets y meta tags sueltos que
//! necesitan ir al `<head>` real del documento — un layout normal solo
//! puede aportar HTML de `<body>` (ver `nexa-cli::layout`), así que sin
//! esto un `<link rel="icon">` puesto ahí quedaba atrapado en `<body>`,
//! donde muchos navegadores no lo detectan de forma confiable.

use nexa_ast::JsonTemplate;

use crate::resolve::{resolve_json_template, SeoContext};

/// `{ icon, appleTouchIcon, stylesheets: [...], meta: [{name|property, content}, ...] }`
/// — todos los campos son opcionales; lo que no tenga la forma esperada
/// se ignora en silencio, igual que un campo de `seo` no resoluble
/// (nunca se inventa un valor, nunca rompe el build por esto).
pub fn render_layout_head(head: Option<&JsonTemplate>, ctx: &SeoContext) -> String {
    let Some(head) = head else {
        return String::new();
    };
    let value = resolve_json_template(head, ctx);
    let Some(obj) = value.as_object() else {
        return String::new();
    };

    let mut lines = Vec::new();

    if let Some(icon) = obj.get("icon").and_then(|v| v.as_str()) {
        let type_attr = mime_for(icon).map(|m| format!(" type=\"{m}\"")).unwrap_or_default();
        lines.push(format!("<link rel=\"icon\" href=\"{}\"{type_attr}>", escape_attr(icon)));
    }
    if let Some(apple) = obj.get("appleTouchIcon").and_then(|v| v.as_str()) {
        lines.push(format!("<link rel=\"apple-touch-icon\" href=\"{}\">", escape_attr(apple)));
    }
    if let Some(sheets) = obj.get("stylesheets").and_then(|v| v.as_array()) {
        for sheet in sheets {
            if let Some(href) = sheet.as_str() {
                lines.push(format!("<link rel=\"stylesheet\" href=\"{}\">", escape_attr(href)));
            }
        }
    }
    // `meta` (Fase 37): escape hatch para lo que no tiene un campo fijo
    // propio en `seo{}` — verificación de Search Console, dominio de
    // Facebook, `theme-color` por página, etc. Acepta `name` (la forma
    // común) o `property` (la que usan Open Graph/`theme-color`) — nunca
    // ambos a la vez en la misma entrada; si están los dos, `name` gana.
    if let Some(metas) = obj.get("meta").and_then(|v| v.as_array()) {
        for meta in metas {
            let Some(meta_obj) = meta.as_object() else { continue };
            let Some(content) = meta_obj.get("content").and_then(|v| v.as_str()) else { continue };
            if let Some(name) = meta_obj.get("name").and_then(|v| v.as_str()) {
                lines.push(format!("<meta name=\"{}\" content=\"{}\">", escape_attr(name), escape_attr(content)));
            } else if let Some(property) = meta_obj.get("property").and_then(|v| v.as_str()) {
                lines.push(format!(
                    "<meta property=\"{}\" content=\"{}\">",
                    escape_attr(property),
                    escape_attr(content)
                ));
            }
        }
    }

    lines.join("\n")
}

fn mime_for(path: &str) -> Option<&'static str> {
    if path.ends_with(".svg") {
        Some("image/svg+xml")
    } else if path.ends_with(".png") {
        Some("image/png")
    } else if path.ends_with(".ico") {
        Some("image/x-icon")
    } else {
        None
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_icon_apple_touch_icon_and_stylesheets() {
        let head = JsonTemplate::Object(vec![
            ("icon".into(), JsonTemplate::String("/static/logo.svg".into())),
            ("appleTouchIcon".into(), JsonTemplate::String("/static/favicon-180.png".into())),
            (
                "stylesheets".into(),
                JsonTemplate::Array(vec![JsonTemplate::String("/static/site.css".into())]),
            ),
        ]);

        let out = render_layout_head(Some(&head), &SeoContext::empty());
        assert!(out.contains(r#"<link rel="icon" href="/static/logo.svg" type="image/svg+xml">"#));
        assert!(out.contains(r#"<link rel="apple-touch-icon" href="/static/favicon-180.png">"#));
        assert!(out.contains(r#"<link rel="stylesheet" href="/static/site.css">"#));
    }

    #[test]
    fn is_empty_without_a_declared_head() {
        assert_eq!(render_layout_head(None, &SeoContext::empty()), "");
    }

    #[test]
    fn escapes_a_quote_inside_a_declared_path() {
        let head = JsonTemplate::Object(vec![(
            "icon".into(),
            JsonTemplate::String("/static/\"onload=alert(1).svg".into()),
        )]);
        let out = render_layout_head(Some(&head), &SeoContext::empty());
        assert!(!out.contains("\"onload"));
    }

    fn meta_entry(key: &str, key_value: &str, content: &str) -> JsonTemplate {
        JsonTemplate::Object(vec![
            (key.into(), JsonTemplate::String(key_value.into())),
            ("content".into(), JsonTemplate::String(content.into())),
        ])
    }

    #[test]
    fn renders_a_name_meta_tag() {
        let head = JsonTemplate::Object(vec![(
            "meta".into(),
            JsonTemplate::Array(vec![meta_entry("name", "google-site-verification", "abc123")]),
        )]);
        let out = render_layout_head(Some(&head), &SeoContext::empty());
        assert_eq!(out, r#"<meta name="google-site-verification" content="abc123">"#);
    }

    #[test]
    fn renders_a_property_meta_tag_for_open_graph_style_keys() {
        let head = JsonTemplate::Object(vec![(
            "meta".into(),
            JsonTemplate::Array(vec![meta_entry("property", "theme-color", "#C8102E")]),
        )]);
        let out = render_layout_head(Some(&head), &SeoContext::empty());
        assert_eq!(out, "<meta property=\"theme-color\" content=\"#C8102E\">");
    }

    #[test]
    fn renders_multiple_meta_entries_in_declared_order() {
        let head = JsonTemplate::Object(vec![(
            "meta".into(),
            JsonTemplate::Array(vec![
                meta_entry("name", "google-site-verification", "abc123"),
                meta_entry("property", "theme-color", "#C8102E"),
            ]),
        )]);
        let out = render_layout_head(Some(&head), &SeoContext::empty());
        assert_eq!(
            out,
            "<meta name=\"google-site-verification\" content=\"abc123\">\n<meta property=\"theme-color\" content=\"#C8102E\">"
        );
    }

    #[test]
    fn a_meta_entry_without_content_is_skipped_silently() {
        let head = JsonTemplate::Object(vec![(
            "meta".into(),
            JsonTemplate::Array(vec![JsonTemplate::Object(vec![(
                "name".into(),
                JsonTemplate::String("orphan".into()),
            )])]),
        )]);
        assert_eq!(render_layout_head(Some(&head), &SeoContext::empty()), "");
    }

    #[test]
    fn meta_tags_combine_with_icon_and_stylesheets_in_the_same_head() {
        let head = JsonTemplate::Object(vec![
            ("icon".into(), JsonTemplate::String("/logo.svg".into())),
            ("meta".into(), JsonTemplate::Array(vec![meta_entry("name", "robots", "noindex")])),
        ]);
        let out = render_layout_head(Some(&head), &SeoContext::empty());
        assert!(out.contains(r#"<link rel="icon" href="/logo.svg" type="image/svg+xml">"#));
        assert!(out.contains(r#"<meta name="robots" content="noindex">"#));
    }
}
