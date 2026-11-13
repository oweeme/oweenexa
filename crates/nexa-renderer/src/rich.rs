//! Contenido enriquecido saneado (Fase 68): `<SafeHtml>` y `<Markdown>`.
//!
//! Todo pasa por `ammonia` (un sanitizador basado en el analizador de HTML5
//! de Servo, que re-serializa el DOM: no depende de expresiones regulares y
//! cierra la clase de ataques de "mutación" que sí rompen a los sanitizadores
//! de texto). Se parte de su lista de etiquetas permitidas por defecto —
//! texto, listas, tablas, enlaces, imágenes, citas, código — y se endurece:
//!
//! - **Fuera**: `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`,
//!   `<form>`, `<input>`, `<svg>`, `<math>`, `<base>`, `<meta>`, `<link>`,
//!   todo atributo `on*`, `style`, `id` (evita DOM clobbering) y `srcdoc`.
//! - **URLs**: solo `http`, `https`, `mailto` y `tel` (o relativas). Un
//!   `javascript:`/`data:` se quita.
//! - **Enlaces**: `rel="noopener noreferrer nofollow ugc"` (contenido de usuarios).
//! - **Imágenes**: `loading="lazy"`.
//! - `class` solo en `<code>`/`<pre>` (resaltado de sintaxis: `language-rust`).

use std::collections::HashSet;
use std::sync::OnceLock;

use ammonia::Builder;
use serde_json::Value;

/// Más que esto no se procesa (se muestra vacío): un post de varios MB no es
/// un post, es un intento de agotar el servidor.
const MAX_INPUT_BYTES: usize = 2 * 1024 * 1024;
/// Anidamiento máximo de etiquetas: protege al analizador y al serializador.
const MAX_DEPTH: usize = 128;

fn cleaner() -> &'static Builder<'static> {
    static CLEANER: OnceLock<Builder<'static>> = OnceLock::new();
    CLEANER.get_or_init(|| {
        let mut b = Builder::default();
        b.url_schemes(HashSet::from(["http", "https", "mailto", "tel"]));
        b.link_rel(Some("noopener noreferrer nofollow ugc"));
        b.add_tag_attributes("code", &["class"]);
        b.add_tag_attributes("pre", &["class"]);
        b.set_tag_attribute_value("img", "loading", "lazy");
        b
    })
}

/// ¿Se abren más etiquetas anidadas que `MAX_DEPTH`? Una aproximación barata
/// (cuenta aperturas menos cierres): no pretende parsear HTML, solo cortar el
/// `<div><div><div>…` de decenas de miles de niveles.
fn too_deep(html: &str) -> bool {
    let bytes = html.as_bytes();
    let (mut depth, mut i) = (0usize, 0usize);
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' {
            match bytes[i + 1] {
                b'/' => depth = depth.saturating_sub(1),
                b'!' | b'?' => {}
                c if c.is_ascii_alphabetic() => {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return true;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    false
}

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let mut out = String::with_capacity(markdown.len() * 3 / 2);
    html::push_html(&mut out, Parser::new_ext(markdown, options));
    out
}

/// El HTML saneado de un valor de los datos. Un valor que no es texto, o que
/// falta, no muestra nada.
pub(crate) fn render_rich(value: Option<Value>, markdown: bool) -> String {
    let Some(Value::String(text)) = value else { return String::new() };
    sanitize(&text, markdown)
}

pub fn sanitize(text: &str, markdown: bool) -> String {
    if text.len() > MAX_INPUT_BYTES || too_deep(text) {
        return String::new();
    }
    if markdown {
        // El HTML que produce el Markdown (y el que el autor mezcle dentro)
        // también pasa por el sanitizador, y la profundidad se mira DESPUÉS de
        // convertir (un `> > > >` de citas anidadas se vuelve `<blockquote>`s).
        let html = markdown_to_html(text);
        if too_deep(&html) {
            return String::new();
        }
        return cleaner().clean(&html).to_string();
    }
    cleaner().clean(text).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(s: &str) -> String {
        sanitize(s, false)
    }
    fn md(s: &str) -> String {
        sanitize(s, true)
    }

    #[test]
    fn rich_formatting_survives() {
        let out = html("<h2>Título</h2><p>Hola <strong>mundo</strong> y <em>más</em>.</p><ul><li>uno</li></ul><blockquote>cita</blockquote><pre><code class=\"language-rust\">fn main(){}</code></pre>");
        for keep in ["<h2>Título</h2>", "<strong>mundo</strong>", "<em>más</em>", "<ul><li>uno</li></ul>", "<blockquote>cita</blockquote>", "<code class=\"language-rust\">"] {
            assert!(out.contains(keep), "{keep} en {out}");
        }
    }

    #[test]
    fn executable_content_is_removed() {
        let attacks = [
            "<script>alert(1)</script>ok",
            "<img src=x onerror=alert(1)>",
            "<a href=\"javascript:alert(1)\">x</a>",
            "<a href=\"  JaVaScRiPt:alert(1)\">x</a>",
            "<a href=\"data:text/html,<script>alert(1)</script>\">x</a>",
            "<iframe src=\"https://evil.com\"></iframe>",
            "<svg onload=alert(1)><script>alert(1)</script></svg>",
            "<math><mtext><script>alert(1)</script></mtext></math>",
            "<form action=\"https://evil.com\"><input name=x></form>",
            "<div style=\"background:url(javascript:alert(1))\">x</div>",
            "<p onclick=\"alert(1)\">x</p>",
            "<base href=\"https://evil.com/\">",
            "<meta http-equiv=\"refresh\" content=\"0;url=https://evil.com\">",
            "<link rel=\"stylesheet\" href=\"https://evil.com/x.css\">",
            "<object data=\"x.swf\"></object><embed src=\"x.swf\">",
            "<noscript><p title=\"</noscript><img src=x onerror=alert(1)>\">",
            "<a href=\"https://ok.com\" onmouseover=\"alert(1)\">x</a>",
            "<img src=\"data:image/svg+xml,<svg onload=alert(1)>\">",
            "<iframe srcdoc=\"<script>alert(1)</script>\"></iframe>",
            "<h1 id=\"cookie\">x</h1>",
        ];
        for attack in attacks {
            let out = html(attack);
            for bad in ["<script", "onerror", "onload", "onclick", "onmouseover", "javascript:", "<iframe", "<svg", "<math", "<form", "<input", "<base", "<meta", "<link", "<object", "<embed", "srcdoc", "style=", "id=", "data:", "evil.com/x.css", "<img src=x"] {
                assert!(!out.to_lowercase().contains(bad), "{attack:?} -> {out:?} contiene {bad}");
            }
        }
    }

    #[test]
    fn links_are_hardened_and_safe_urls_survive() {
        let out = html("<a href=\"https://oweeme.com/blog\">blog</a> <a href=\"/relativa\">r</a> <a href=\"mailto:a@b.com\">m</a> <a href=\"tel:+34600\">t</a>");
        assert!(out.contains("href=\"https://oweeme.com/blog\""), "{out}");
        assert!(out.contains("rel=\"noopener noreferrer nofollow ugc\""), "{out}");
        assert!(out.contains("href=\"/relativa\"") && out.contains("mailto:a@b.com") && out.contains("tel:+34600"), "{out}");
    }

    #[test]
    fn images_keep_https_sources_and_are_lazy() {
        let out = html("<img src=\"https://cdn.x.com/a.png\" alt=\"a\" width=\"10\">");
        assert!(out.contains("src=\"https://cdn.x.com/a.png\"") && out.contains("alt=\"a\"") && out.contains("loading=\"lazy\""), "{out}");
    }

    #[test]
    fn markdown_renders_common_constructs() {
        let out = md("# Título\n\nHola **negrita** y *cursiva* con `código`.\n\n- uno\n- dos\n\n1. a\n2. b\n\n> cita\n\n```rust\nfn main() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n~~tachado~~ y [enlace](https://oweeme.com)");
        for keep in ["<h1>Título</h1>", "<strong>negrita</strong>", "<em>cursiva</em>", "<code>código</code>", "<ul>", "<ol>", "<blockquote>", "<table>", "<del>tachado</del>", "href=\"https://oweeme.com\""] {
            assert!(out.contains(keep), "{keep} en {out}");
        }
        assert!(out.contains("language-rust"), "{out}");
    }

    #[test]
    fn markdown_cannot_smuggle_executable_content() {
        for attack in [
            "[clic](javascript:alert(1))",
            "[clic]( javascript:alert(1))",
            "![x](javascript:alert(1))",
            "<script>alert(1)</script>",
            "texto <img src=x onerror=alert(1)> texto",
            "[a](data:text/html,<script>alert(1)</script>)",
            "<iframe src=\"https://evil.com\"></iframe>",
        ] {
            let out = md(attack).to_lowercase();
            for bad in ["<script", "onerror", "onclick", "javascript:", "<iframe", "data:text"] {
                assert!(!out.contains(bad), "{attack:?} -> {out:?} contiene {bad}");
            }
        }
    }

    #[test]
    fn markdown_that_only_looks_like_a_link_stays_inert_text() {
        // No es un enlace válido: el texto `onclick=...` queda como TEXTO dentro
        // de un `<p>`, sin ninguna etiqueta ni atributo.
        let out = md("[x](https://ok.com \"t\" onclick=\"alert(1)\")");
        assert!(out.starts_with("<p>") && !out.contains("<a"), "{out}");
        assert_eq!(out.matches('<').count(), 2, "solo <p> y </p>: {out}");
    }

    #[test]
    fn only_strings_render_anything() {
        assert_eq!(render_rich(None, false), "");
        assert_eq!(render_rich(Some(serde_json::json!(42)), false), "");
        assert_eq!(render_rich(Some(serde_json::json!({"a": 1})), true), "");
        assert_eq!(render_rich(Some(serde_json::json!("<b>x</b>")), false), "<b>x</b>");
    }

    #[test]
    fn pathological_sizes_and_nesting_render_nothing_instead_of_crashing() {
        let deep = "<div>".repeat(50_000);
        assert_eq!(html(&deep), "");
        let quotes = "> ".repeat(5_000) + "x";
        assert_eq!(md(&quotes), "", "citas anidadas: se vuelven <blockquote> anidados");
        let huge = "a".repeat(MAX_INPUT_BYTES + 1);
        assert_eq!(html(&huge), "");
        // y lo normal, justo debajo del límite de profundidad, sí pasa
        let ok = "<div>".repeat(50) + "x" + &"</div>".repeat(50);
        assert!(html(&ok).contains('x'));
    }

    #[test]
    fn text_is_still_escaped_by_the_html_serializer() {
        assert_eq!(html("1 < 2 & 3 > 2"), "1 &lt; 2 &amp; 3 &gt; 2");
    }
}
