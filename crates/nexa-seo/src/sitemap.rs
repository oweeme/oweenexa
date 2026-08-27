/// Genera `sitemap.xml` para el sitio completo. Es responsabilidad del
/// llamador (`nexa-cli`) juntar las URLs absolutas de todas las rutas
/// estáticas — esto solo sabe convertir esa lista en el XML correcto.
pub fn render_sitemap(urls: &[String]) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for url in urls {
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", escape_xml(url)));
        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");
    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_one_url_entry_per_url() {
        let xml = render_sitemap(&["https://example.com/".into(), "https://example.com/about".into()]);
        assert!(xml.contains("<loc>https://example.com/</loc>"));
        assert!(xml.contains("<loc>https://example.com/about</loc>"));
        assert_eq!(xml.matches("<url>").count(), 2);
    }

    #[test]
    fn escapes_special_characters_in_urls() {
        let xml = render_sitemap(&["https://example.com/?a=1&b=2".into()]);
        assert!(xml.contains("&amp;"));
        assert!(!xml.contains("a=1&b"));
    }
}
