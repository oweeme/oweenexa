//! `export const head` de `src/layout.tsx` (Fase 27): favicon y
//! stylesheets que un layout necesita en el `<head>` real del
//! documento — un layout normal solo puede aportar HTML de `<body>` (ver
//! `nexa-cli::layout`), así que sin esto un `<link rel="icon">` puesto
//! ahí quedaba atrapado en `<body>`, donde muchos navegadores no lo
//! detectan de forma confiable.

use nexa_ast::JsonTemplate;

use crate::resolve::{resolve_json_template, SeoContext};

/// `{ icon: "...", appleTouchIcon: "...", stylesheets: ["...", ...] }`
/// — cualquier campo es opcional; los que no sean string (o array de
/// strings, para `stylesheets`) se ignoran en silencio, igual que un
/// campo de `seo` no resoluble.
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
}
