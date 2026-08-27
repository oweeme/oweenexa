/// Genera `robots.txt`. `sitemap_url` es opcional porque el sitemap solo
/// tiene sentido con una URL base absoluta configurada (ver `nexa-cli`).
pub fn render_robots(sitemap_url: Option<&str>) -> String {
    let mut lines = vec!["User-agent: *".to_string(), "Allow: /".to_string()];

    if let Some(url) = sitemap_url {
        lines.push(String::new());
        lines.push(format!("Sitemap: {url}"));
    }

    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_everything_by_default() {
        let robots = render_robots(None);
        assert!(robots.contains("User-agent: *"));
        assert!(robots.contains("Allow: /"));
        assert!(!robots.contains("Sitemap:"));
    }

    #[test]
    fn references_the_sitemap_when_given() {
        let robots = render_robots(Some("https://example.com/sitemap.xml"));
        assert!(robots.contains("Sitemap: https://example.com/sitemap.xml"));
    }
}
