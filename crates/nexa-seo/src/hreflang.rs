use nexa_i18n::AlternateLink;

/// `<link rel="alternate" hreflang="...">` por cada locale disponible —
/// el "hreflang automático" del criterio de salida de la Fase 10: el
/// desarrollador no escribe estas etiquetas a mano, `nexa-i18n` calcula
/// las URLs y esto solo las serializa.
pub fn render_hreflang_links(links: &[AlternateLink]) -> String {
    links
        .iter()
        .map(|link| {
            format!(
                "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\">",
                escape_attr(&link.hreflang),
                escape_attr(&link.href)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_one_link_per_alternate() {
        let links = vec![
            AlternateLink { hreflang: "es".into(), href: "/es/producto".into() },
            AlternateLink { hreflang: "en".into(), href: "/en/product".into() },
        ];

        let html = render_hreflang_links(&links);
        assert!(html.contains("hreflang=\"es\" href=\"/es/producto\""));
        assert!(html.contains("hreflang=\"en\" href=\"/en/product\""));
    }

    #[test]
    fn empty_list_renders_nothing() {
        assert_eq!(render_hreflang_links(&[]), "");
    }
}
