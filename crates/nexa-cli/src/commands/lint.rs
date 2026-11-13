//! `nexa lint` (Fase 23): compila cada página igual que `nexa build`,
//! pero sin escribir nada a `dist/` — solo junta los avisos del SEO
//! Analyzer (`NEXA-SEO-*`, `NEXA-A11Y-*`) y de paquetes (`NEXA-PKG-*`) de
//! *todo* el sitio y falla (código de salida distinto de cero) si
//! encuentra alguno. `nexa build` nunca falla por esto a propósito (un
//! build no debe romperse porque falta un `alt`) — `nexa lint` existe
//! justamente para el caso contrario: un chequeo que un pipeline de CI sí
//! pueda usar para bloquear un merge.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use nexa_router::scan_pages;

use crate::commands::build::{param_sets_for, resolved_pattern};
use crate::pipeline::{compile_page, PageError};

pub fn run() -> Result<()> {
    let pages_dir = Path::new("src/pages");
    let app_enabled = crate::app_mode::is_enabled();
    if !pages_dir.is_dir() && !app_enabled {
        bail!(
            "no se encontró {} — ¿estás dentro de un proyecto Nexa? (usa `nexa create <nombre>`)",
            pages_dir.display()
        );
    }

    let routes = if pages_dir.is_dir() { scan_pages(pages_dir).context("escaneando src/pages")? } else { Vec::new() };
    if routes.is_empty() && !app_enabled {
        bail!("{} no contiene ninguna página .tsx", pages_dir.display());
    }

    let api_base = std::env::var("NEXA_API_URL").unwrap_or_default();
    let no_params = BTreeMap::new();
    let mut total_warnings = 0usize;
    let mut checked = 0usize;

    for route in &routes {
        if route.is_dynamic() {
            let Some(param_sets) = param_sets_for(route, &api_base)? else {
                println!("{}: sin `paths` declarado — se salta (no hay un set de parámetros conocido).", route.pattern);
                continue;
            };

            for params in &param_sets {
                let pattern = resolved_pattern(route, params);
                let page = compile_page(&route.file, &route.pattern, params, &api_base)
                    .map_err(|err| lint_error(&pattern, err))?;
                total_warnings += report(&pattern, &page.seo_warnings, &page.pkg_warnings);
                checked += 1;
            }
            continue;
        }

        let page = compile_page(&route.file, &route.pattern, &no_params, &api_base)
            .map_err(|err| lint_error(&route.pattern, err))?;
        total_warnings += report(&route.pattern, &page.seo_warnings, &page.pkg_warnings);
        checked += 1;
    }

    // Modo App (Fase 60): `nexa build` ya falla ante un error de sintaxis en
    // `src/app/`; `nexa lint` (el chequeo de CI) tiene que fallar igual,
    // sin escribir nada a `dist/`.
    if app_enabled {
        let build = crate::app_mode::compile(&crate::app_mode::app_dir(), &crate::app_mode::runtime_url())
            .context("compilando src/app (Modo App)")?;
        println!("Modo App: {} ruta(s) y {} módulo(s) sin errores.", build.routes.len(), build.modules.len());
        checked += build.routes.len();
    }

    println!();
    if total_warnings == 0 {
        println!("{checked} página(s) revisada(s), sin avisos.");
        return Ok(());
    }

    println!("{checked} página(s) revisada(s), {total_warnings} aviso(s) encontrado(s).");
    bail!("`nexa lint` encontró avisos — corrígelos o revisa si son esperados antes de mergear.");
}

fn report(pattern: &str, seo_warnings: &[nexa_seo::Warning], pkg_warnings: &[crate::pkg_warnings::PackageWarning]) -> usize {
    let count = seo_warnings.len() + pkg_warnings.len();
    if count == 0 {
        return 0;
    }
    println!("{pattern}:");
    for warning in seo_warnings {
        println!("  [{}] {}", warning.code, warning.message);
    }
    for warning in pkg_warnings {
        println!("  [{}] {}", warning.code, warning.message);
    }
    count
}

fn lint_error(pattern: &str, err: PageError) -> anyhow::Error {
    match err {
        PageError::NotFound => anyhow::anyhow!("{pattern}: su `load()` respondió 404"),
        PageError::Other(e) => e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkg_warnings::PackageWarning;
    use nexa_seo::Warning;

    #[test]
    fn report_counts_and_combines_both_warning_kinds() {
        let seo = vec![Warning { code: "NEXA-SEO-001", message: "falta title".to_string() }];
        let pkg = vec![
            PackageWarning { code: "NEXA-PKG-001", message: "usa ui sin declararlo".to_string() },
            PackageWarning { code: "NEXA-PKG-002", message: "usa forms sin declararlo".to_string() },
        ];

        assert_eq!(report("/", &seo, &pkg), 3);
    }

    #[test]
    fn report_returns_zero_and_prints_nothing_extra_when_there_are_no_warnings() {
        assert_eq!(report("/", &[], &[]), 0);
    }
}
