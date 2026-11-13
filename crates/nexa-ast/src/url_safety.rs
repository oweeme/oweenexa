//! Seguridad de URLs que vienen de **datos** (la API, los parámetros de la
//! ruta): una URL con esquema `javascript:` ejecuta código al hacer clic, y
//! `data:text/html` carga un documento propio. Quien controla un campo de tu
//! API — un comentario, el nombre de un usuario, un "sitio web" — no debe
//! poder poner eso en un `href`.
//!
//! Los valores que escribe el desarrollador a mano (literales en el TSX) no
//! pasan por aquí: son de confianza.

/// ¿`value` empieza con un esquema que ejecuta código o carga un documento
/// activo (`javascript:`, `vbscript:`, `data:`)?
///
/// Imita lo que hacen los navegadores al leer el esquema: ignoran los
/// espacios y caracteres de control del principio y los tabuladores/saltos de
/// línea *dentro* del esquema (`java\tscript:` ES `javascript:`), y no
/// distinguen mayúsculas. Un `:` que aparece después de `/`, `?` o `#` no es un
/// esquema (`/ruta?x=a:b`).
///
/// `allow_data_image`: permite `data:image/...` (para el `src` de una `<img>`).
pub fn has_dangerous_scheme(value: &str, allow_data_image: bool) -> bool {
    let mut chars = value.chars().skip_while(|c| *c <= ' ');
    let mut scheme = String::new();
    loop {
        match chars.next() {
            Some(':') => break,
            Some('/') | Some('?') | Some('#') | None => return false,
            Some('\t') | Some('\n') | Some('\r') => continue,
            Some(c) => {
                scheme.push(c.to_ascii_lowercase());
                if scheme.len() > 32 {
                    return false; // un esquema real es corto
                }
            }
        }
    }
    match scheme.as_str() {
        "javascript" | "vbscript" => true,
        "data" => {
            let rest: String = chars.filter(|c| !matches!(c, '\t' | '\n' | '\r')).take(6).collect::<String>().to_ascii_lowercase();
            !(allow_data_image && rest == "image/")
        }
        _ => false,
    }
}

/// Atributos cuyo valor es una URL que el navegador sigue o carga.
pub fn is_url_attribute(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "href" | "src" | "action" | "formaction" | "poster" | "data" | "cite" | "background" | "ping" | "xlink:href" | "manifest" | "codebase"
    )
}

/// Un valor de **datos** que jamás debe llegar a un atributo: los manejadores
/// de eventos en línea (`onclick="..."`) y `srcdoc` (HTML que se ejecuta).
pub fn is_active_content_attribute(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("on") || n == "srcdoc"
}

/// Solo `http`, `https` o relativa — para `canonical`, `og:image`...
pub fn is_http_or_relative(value: &str) -> bool {
    let trimmed = value.trim_start_matches(|c: char| c <= ' ');
    if trimmed.is_empty() {
        return false;
    }
    let head: String = trimmed.chars().take_while(|c| !matches!(c, '/' | '?' | '#')).filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect();
    match head.split_once(':') {
        None => true, // sin esquema: relativa
        Some((scheme, _)) => scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dangerous_schemes_are_detected_however_they_are_disguised() {
        for evil in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "  javascript:alert(1)",
            "\n\tjavascript:alert(1)",
            "java\tscript:alert(1)",
            "java\nscr\ript:alert(1)",
            "vbscript:msgbox(1)",
            "data:text/html,<script>alert(1)</script>",
            "DATA:text/html;base64,PHNjcmlwdD4=",
            "\u{1}\u{2}javascript:alert(1)",
        ] {
            assert!(has_dangerous_scheme(evil, false), "{evil:?}");
        }
    }

    #[test]
    fn ordinary_urls_are_not_flagged() {
        for fine in [
            "https://example.com/a:b",
            "http://example.com",
            "/blog/post?x=javascript:1",
            "relative/path",
            "#ancla",
            "?q=data:x",
            "mailto:hola@x.com",
            "tel:+34600000000",
            "//cdn.example.com/x.js",
            "",
            "texto sin esquema: solo dos puntos",
        ] {
            assert!(!has_dangerous_scheme(fine, false), "{fine:?}");
        }
    }

    #[test]
    fn data_images_are_allowed_only_when_asked() {
        assert!(has_dangerous_scheme("data:image/png;base64,AAAA", false));
        assert!(!has_dangerous_scheme("data:image/png;base64,AAAA", true));
        assert!(!has_dangerous_scheme("data:IMAGE/svg+xml,<svg/>", true));
        assert!(has_dangerous_scheme("data:text/html,x", true));
    }

    #[test]
    fn url_and_active_content_attribute_names() {
        assert!(is_url_attribute("href") && is_url_attribute("SRC") && is_url_attribute("formaction"));
        assert!(!is_url_attribute("class") && !is_url_attribute("alt"));
        assert!(is_active_content_attribute("onclick") && is_active_content_attribute("onError") && is_active_content_attribute("srcdoc"));
        assert!(!is_active_content_attribute("content") && !is_active_content_attribute("class") && !is_active_content_attribute("href"));
    }

    #[test]
    fn http_or_relative_for_canonical_and_images() {
        for ok in ["https://a.com/x", "HTTP://a.com", "/img/x.png", "img/x.png", "//cdn.com/x", "?q=1"] {
            assert!(is_http_or_relative(ok), "{ok}");
        }
        for bad in ["javascript:alert(1)", "data:text/html,x", "ftp://x", "file:///etc/passwd", "", "  "] {
            assert!(!is_http_or_relative(bad), "{bad}");
        }
    }
}
