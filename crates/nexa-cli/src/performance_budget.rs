//! Presupuestos de rendimiento (Fase 14): `[performance]` en `nexa.toml`
//! declara límites opcionales, en bytes. Si el proyecto no declara
//! ninguno, no se comprueba nada — cero fricción por defecto, el mismo
//! criterio que el resto de Nexa. Si se declara y una página lo supera,
//! `nexa build` **termina en error**, no solo avisa: un presupuesto que
//! nunca bloquea no es un presupuesto, es una sugerencia — y para
//! sugerencias ya están los avisos `NEXA-SEO-*`/`NEXA-PKG-*`.

use crate::manifest::PerformanceSection;

/// Lo que hace falta saber de una página ya compilada para comprobar sus
/// presupuestos — deliberadamente más chico que `CompiledPage`: `nexa
/// build` puede quedarse solo con esto por página en vez de guardar cada
/// `CompiledPage` completo hasta el final (el CSS compartido solo se
/// conoce tras compilar *todas* las páginas del sitio).
pub struct PageUsage {
    pub pattern: String,
    pub initial_js_bytes: u64,
    /// `None` si esta página no enlaza `@nexa/ui` en absoluto.
    pub css_bytes: Option<u64>,
    /// La imagen estática más pesada que referencia la página, si
    /// alguna se pudo resolver contra `public/`.
    pub largest_image: Option<(String, u64)>,
}

pub struct Violation {
    pub pattern: String,
    pub budget: &'static str,
    pub limit: u64,
    pub actual: u64,
    /// Ej. la ruta de la imagen que se pasó del límite.
    pub detail: Option<String>,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let extra = self.detail.as_deref().map(|d| format!(" ({d})")).unwrap_or_default();
        write!(
            f,
            "{}: {}{} usa {} bytes, supera el presupuesto {} de {} bytes",
            self.pattern, self.budget, extra, self.actual, self.budget, self.limit
        )
    }
}

pub fn check(budget: &PerformanceSection, pages: &[PageUsage]) -> Vec<Violation> {
    let mut violations = Vec::new();

    for page in pages {
        if let Some(limit) = budget.max_initial_js {
            if page.initial_js_bytes > limit {
                violations.push(Violation {
                    pattern: page.pattern.clone(),
                    budget: "maxInitialJS",
                    limit,
                    actual: page.initial_js_bytes,
                    detail: None,
                });
            }
        }

        if let Some(limit) = budget.max_css {
            if let Some(actual) = page.css_bytes {
                if actual > limit {
                    violations.push(Violation {
                        pattern: page.pattern.clone(),
                        budget: "maxCSS",
                        limit,
                        actual,
                        detail: None,
                    });
                }
            }
        }

        if let Some(limit) = budget.max_image {
            if let Some((src, actual)) = &page.largest_image {
                if *actual > limit {
                    violations.push(Violation {
                        pattern: page.pattern.clone(),
                        budget: "maxImage",
                        limit,
                        actual: *actual,
                        detail: Some(src.clone()),
                    });
                }
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget(max_initial_js: Option<u64>, max_css: Option<u64>, max_image: Option<u64>) -> PerformanceSection {
        PerformanceSection { max_initial_js, max_css, max_image }
    }

    fn page(pattern: &str, initial_js_bytes: u64, css_bytes: Option<u64>, largest_image: Option<(&str, u64)>) -> PageUsage {
        PageUsage {
            pattern: pattern.to_string(),
            initial_js_bytes,
            css_bytes,
            largest_image: largest_image.map(|(src, bytes)| (src.to_string(), bytes)),
        }
    }

    #[test]
    fn no_budget_declared_means_nothing_is_checked() {
        let empty = budget(None, None, None);
        let pages = [page("/", 999_999, Some(999_999), Some(("/huge.png", 999_999)))];

        assert!(check(&empty, &pages).is_empty());
    }

    #[test]
    fn flags_a_page_over_the_initial_js_budget() {
        let b = budget(Some(10_000), None, None);
        let pages = [page("/", 12_000, None, None)];

        let violations = check(&b, &pages);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].budget, "maxInitialJS");
        assert_eq!(violations[0].actual, 12_000);
    }

    #[test]
    fn does_not_flag_a_page_under_budget() {
        let b = budget(Some(10_000), None, None);
        let pages = [page("/", 9_000, None, None)];

        assert!(check(&b, &pages).is_empty());
    }

    #[test]
    fn flags_a_page_over_the_css_budget_only_when_it_actually_links_css() {
        let b = budget(None, Some(5_000), None);
        let with_css = [page("/a", 0, Some(6_000), None)];
        let without_css = [page("/b", 0, None, None)];

        assert_eq!(check(&b, &with_css).len(), 1);
        assert!(check(&b, &without_css).is_empty());
    }

    #[test]
    fn flags_a_page_whose_largest_image_exceeds_the_budget_and_names_it() {
        let b = budget(None, None, Some(100_000));
        let pages = [page("/gallery", 0, None, Some(("/photos/big.jpg", 250_000)))];

        let violations = check(&b, &pages);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].detail.as_deref(), Some("/photos/big.jpg"));
    }

    #[test]
    fn checks_every_page_independently_and_can_report_several_violations() {
        let b = budget(Some(10_000), None, None);
        let pages = [page("/a", 20_000, None, None), page("/b", 5_000, None, None), page("/c", 30_000, None, None)];

        let violations = check(&b, &pages);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern, "/a");
        assert_eq!(violations[1].pattern, "/c");
    }
}
