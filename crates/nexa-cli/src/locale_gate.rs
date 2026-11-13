//! La portada `/` de un sitio multi-idioma (Fase 67).
//!
//! Con páginas solo bajo `src/pages/[locale]/...` y ninguna `src/pages/index.tsx`,
//! `https://oweeme.com/` no existía: un 404 en la puerta de entrada del sitio.
//! Esto genera una **puerta de idioma**: una página mínima que lleva al
//! visitante a su idioma (el del navegador si el sitio lo tiene, si no el
//! idioma por defecto) y, para quien no ejecuta JavaScript, enlaces a cada
//! idioma, `canonical` y `hreflang="x-default"` hacia el idioma por defecto.
//!
//! Solo se genera si el proyecto NO tiene su propia `src/pages/index.tsx`:
//! una portada escrita a mano siempre gana.

use std::path::Path;

use nexa_router::Route;

const LOCALES_DIR: &str = "src/locales";

/// El idioma por defecto del proyecto: `[i18n] default`, `es` si no se declara,
/// y si ese no existe entre los idiomas disponibles, el primero (orden alfabético).
pub fn default_locale(available: &[String]) -> String {
    let wanted = crate::manifest::load_lenient(Path::new("nexa.toml")).i18n.and_then(|i| i.default).unwrap_or_else(|| "es".to_string());
    if available.contains(&wanted) {
        wanted
    } else {
        available.first().cloned().unwrap_or(wanted)
    }
}

/// ¿Hace falta una puerta de idioma? Sí si hay rutas bajo `[locale]` y nadie
/// sirve la raíz `/`. Devuelve su HTML.
pub fn for_project(routes: &[Route]) -> Option<String> {
    let has_root = routes.iter().any(|r| r.pattern == "/");
    let has_locale_routes = routes.iter().any(|r| r.pattern == "/:locale" || r.pattern.starts_with("/:locale/"));
    if has_root || !has_locale_routes {
        return None;
    }
    let available: Vec<String> = nexa_i18n::available_locales(Path::new(LOCALES_DIR)).into_iter().filter(|l| crate::pipeline::is_safe_locale(l)).collect();
    if available.is_empty() {
        return None;
    }
    let default = default_locale(&available);
    Some(crate::csp::reapply(&render(&default, &available)))
}

/// El HTML de la puerta. `default` y `locales` ya son códigos seguros
/// (`is_safe_locale`: letras, dígitos, `-` y `_`), así que no necesitan escape.
pub fn render(default: &str, locales: &[String]) -> String {
    let list_js = locales.iter().map(|l| format!("\"{}\"", l.to_ascii_lowercase())).collect::<Vec<_>>().join(",");
    let alternates: String = locales
        .iter()
        .map(|l| format!("<link rel=\"alternate\" hreflang=\"{l}\" href=\"/{l}\">\n"))
        .collect();
    let links: String = locales.iter().map(|l| format!("<li><a href=\"/{l}\" hreflang=\"{l}\">{l}</a></li>")).collect();
    format!(
        "<!doctype html>\n<html lang=\"{default}\">\n<head>\n<meta charset=\"UTF-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{default}</title>\n<link rel=\"canonical\" href=\"/{default}\">\n\
         <link rel=\"alternate\" hreflang=\"x-default\" href=\"/{default}\">\n{alternates}\
         <script>(function(){{var s=[{list_js}],p=navigator.languages||[navigator.language||\"\"];\
         for(var i=0;i<p.length;i++){{var c=String(p[i]).toLowerCase(),b=c.split(\"-\")[0];\
         var m=s.indexOf(c)>=0?c:(s.indexOf(b)>=0?b:null);\
         if(m){{location.replace(\"/\"+m+location.search+location.hash);return}}}}\
         location.replace(\"/{default}\"+location.search+location.hash)}})()</script>\n\
         <noscript><meta http-equiv=\"refresh\" content=\"0; url=/{default}\"></noscript>\n\
         </head>\n<body>\n<ul>{links}</ul>\n</body>\n</html>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gate_links_every_language_and_points_x_default_at_the_default() {
        let html = render("es", &["de".into(), "en".into(), "es".into()]);
        for l in ["de", "en", "es"] {
            assert!(html.contains(&format!("<a href=\"/{l}\" hreflang=\"{l}\">")), "{html}");
            assert!(html.contains(&format!("<link rel=\"alternate\" hreflang=\"{l}\" href=\"/{l}\">")), "{html}");
        }
        assert!(html.contains("rel=\"alternate\" hreflang=\"x-default\" href=\"/es\""), "{html}");
        assert!(html.contains("<link rel=\"canonical\" href=\"/es\">") && html.contains("<html lang=\"es\">"));
        assert!(html.contains("var s=[\"de\",\"en\",\"es\"]"), "el script elige entre los idiomas que el sitio tiene: {html}");
        assert!(html.contains("<noscript><meta http-equiv=\"refresh\" content=\"0; url=/es\"></noscript>"), "sin JS se va al idioma por defecto: {html}");
    }

    #[test]
    fn a_project_with_its_own_root_page_or_without_locale_routes_gets_no_gate() {
        let route = |p: &str| Route { pattern: p.to_string(), file: std::path::PathBuf::from("x.tsx"), segments: vec![] };
        assert!(for_project(&[route("/"), route("/:locale")]).is_none(), "una portada propia siempre gana");
        assert!(for_project(&[route("/about")]).is_none(), "sin [locale] no hay nada que decidir");
    }
}
