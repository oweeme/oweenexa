use nexa_ast::SeoConfig;

use crate::resolve::{resolve_template, SeoContext};

/// Produce las etiquetas de `<head>` que `seo` permite resolver. No
/// inventa valores por defecto — si un campo no se declaró o no se pudo
/// resolver, simplemente no aparece; el llamador decide los fallbacks
/// (ej. un `<title>` genérico) al ensamblar el documento completo.
pub fn render_head(seo: Option<&SeoConfig>, ctx: &SeoContext) -> String {
    let mut tags = Vec::new();

    let field = |get: fn(&SeoConfig) -> Option<&nexa_ast::Template>| {
        seo.and_then(get).and_then(|t| resolve_template(t, ctx))
    };

    if let Some(title) = field(|s| s.title.as_ref()) {
        tags.push(format!("<title>{}</title>", escape_html(&title)));
    }
    if let Some(description) = field(|s| s.description.as_ref()) {
        tags.push(format!(
            "<meta name=\"description\" content=\"{}\">",
            escape_attr(&description)
        ));
    }
    if let Some(canonical) = field(|s| s.canonical.as_ref()) {
        tags.push(format!("<link rel=\"canonical\" href=\"{}\">", escape_attr(&canonical)));
    }
    if let Some(og_title) = field(|s| s.og_title.as_ref()) {
        tags.push(format!("<meta property=\"og:title\" content=\"{}\">", escape_attr(&og_title)));
    }
    if let Some(og_description) = field(|s| s.og_description.as_ref()) {
        tags.push(format!(
            "<meta property=\"og:description\" content=\"{}\">",
            escape_attr(&og_description)
        ));
    }
    if let Some(og_image) = field(|s| s.og_image.as_ref()) {
        tags.push(format!("<meta property=\"og:image\" content=\"{}\">", escape_attr(&og_image)));
    }
    if let Some(twitter_card) = field(|s| s.twitter_card.as_ref()) {
        tags.push(format!(
            "<meta name=\"twitter:card\" content=\"{}\">",
            escape_attr(&twitter_card)
        ));
    }

    tags.join("\n")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_html(s).replace('"', "&quot;")
}
