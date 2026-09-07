use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use nexa_activation::ActivationManifest;
use nexa_ast::Loader;
use nexa_ir::IrComponent;
use nexa_router::{scan_pages, Route, Segment};
use nexa_seo::Warning;

use crate::image_pipeline;
use crate::image_scan;
use crate::performance_budget::{self, PageUsage};
use crate::pipeline::{compile_page, find_paths_declaration, CompiledPage, PageError};

pub fn run() -> Result<()> {
    let pages_dir = Path::new("src/pages");
    if !pages_dir.is_dir() {
        bail!(
            "no se encontró {} — ¿estás dentro de un proyecto Nexa? (usa `nexa create <nombre>`)",
            pages_dir.display()
        );
    }

    let routes = scan_pages(pages_dir).context("escaneando src/pages")?;
    if routes.is_empty() {
        bail!("{} no contiene ninguna página .tsx", pages_dir.display());
    }

    let api_base = std::env::var("NEXA_API_URL").unwrap_or_default();
    let site_url = std::env::var("NEXA_SITE_URL").ok();
    let no_params = BTreeMap::new();

    fs::create_dir_all("dist/assets")?;
    write_framework_assets()?;
    crate::copy_dir::copy_recursive(Path::new("public"), Path::new("dist"))
        .context("copiando public/ a dist/")?;

    let project_manifest = crate::manifest::load_lenient(Path::new("nexa.toml"));
    if let Some(pwa) = &project_manifest.pwa {
        write_pwa_assets(pwa)?;
    }

    let mut built = 0;
    let mut prerendered_routes = 0;
    let mut skipped_dynamic = 0;
    let mut built_patterns = Vec::new();
    let mut ui_classes = BTreeSet::new();
    let mut pending_usage = Vec::new();
    let mut image_cache = image_pipeline::VariantCache::default();
    let public_dir = Path::new("public");
    let dist_root = Path::new("dist");

    for route in &routes {
        if route.is_dynamic() {
            let declared_paths = find_paths_declaration(&route.file)
                .with_context(|| format!("leyendo `paths` de {}", route.pattern))?;

            let Some(paths) = declared_paths else {
                skipped_dynamic += 1;
                continue;
            };

            let param_sets = enumerate_paths(&paths, route, &api_base)?;
            prerendered_routes += 1;

            for params in &param_sets {
                let mut page = compile_page(&route.file, &route.pattern, params, &api_base).map_err(|err| match err {
                    PageError::NotFound => anyhow::anyhow!(
                        "{} con {params:?}: su `load()` respondió 404 para un set de parámetros \
                         que `paths` declaró — revisa que `paths` y `load` estén de acuerdo",
                        route.pattern
                    ),
                    PageError::Other(e) => e,
                })?;
                page.html = image_pipeline::rewrite_images(&page.html, |src| {
                    image_cache.get_or_generate(src, public_dir, dist_root, |err| {
                        eprintln!("Aviso: no se pudo optimizar la imagen {src}: {err:#}");
                    })
                })
                .context("optimizando imágenes")?;

                let dir = output_dir_for_params(route, params);
                write_page_at(&dir, &page)?;
                let resolved_pattern = resolved_pattern(route, params);

                println!(
                    "Compilado {} -> {} ({} bytes)",
                    resolved_pattern,
                    dir.join("index.html").display(),
                    page.html.len()
                );
                print_classification_summary(&page.ir);
                print_activation_summary(&page.manifest, page.chunks.len());
                print_seo_warnings(&page.seo_warnings);
                print_pkg_warnings(&page.pkg_warnings);

                pending_usage.push(PendingPageUsage {
                    pattern: resolved_pattern.clone(),
                    initial_js_bytes: page.initial_js_bytes,
                    links_ui_css: !page.ui_used_classes.is_empty(),
                    largest_image: largest_static_image(&page),
                });

                built_patterns.push(resolved_pattern);
                ui_classes.extend(page.ui_used_classes);
                built += 1;
            }
            continue;
        }

        let mut page = compile_page(&route.file, &route.pattern, &no_params, &api_base).map_err(|err| match err {
            PageError::NotFound => anyhow::anyhow!(
                "{}: su `load()` respondió 404 — no se puede generar un HTML estático para un \
                 dato que no existe",
                route.pattern
            ),
            PageError::Other(e) => e,
        })?;
        page.html = image_pipeline::rewrite_images(&page.html, |src| {
            image_cache.get_or_generate(src, public_dir, dist_root, |err| {
                eprintln!("Aviso: no se pudo optimizar la imagen {src}: {err:#}");
            })
        })
        .context("optimizando imágenes")?;

        write_page(route, &page)?;

        println!(
            "Compilado {} -> {} ({} bytes)",
            route.pattern,
            output_dir(route).join("index.html").display(),
            page.html.len()
        );
        print_classification_summary(&page.ir);
        print_activation_summary(&page.manifest, page.chunks.len());
        print_seo_warnings(&page.seo_warnings);
        print_pkg_warnings(&page.pkg_warnings);

        pending_usage.push(PendingPageUsage {
            pattern: route.pattern.clone(),
            initial_js_bytes: page.initial_js_bytes,
            links_ui_css: !page.ui_used_classes.is_empty(),
            largest_image: largest_static_image(&page),
        });

        built_patterns.push(route.pattern.clone());
        ui_classes.extend(page.ui_used_classes);
        built += 1;
    }

    write_site_files(&built_patterns, site_url.as_deref())?;
    let ui_css_bytes = write_ui_stylesheet(&ui_classes)?;

    println!();
    println!("{built} página(s) estática(s) compilada(s).");
    if prerendered_routes > 0 {
        println!(
            "{prerendered_routes} ruta(s) dinámica(s) pre-renderizada(s) de verdad vía `paths` \
             (Fase 17) — quedaron como HTML estático real en dist/, no dependen de un proceso vivo."
        );
    }
    if image_cache.optimized_count() > 0 {
        println!(
            "{} imagen(es) optimizada(s): variantes AVIF en varios anchos, HTML reescrito a \
             <picture> (Fase 19).",
            image_cache.optimized_count()
        );
    }
    if skipped_dynamic > 0 {
        println!(
            "{skipped_dynamic} ruta(s) dinámica(s) no se pre-renderizaron (no declaran `paths`) \
             — `nexa preview` las renderiza al vuelo bajo demanda."
        );
    }
    if site_url.is_none() {
        println!(
            "Aviso: NEXA_SITE_URL no está definida — dist/sitemap.xml usa rutas relativas, no \
             URLs absolutas (los sitemaps reales las requieren)."
        );
    }

    check_performance_budgets(&pending_usage, ui_css_bytes)?;

    Ok(())
}

/// Lo que hace falta por página para comprobar presupuestos, antes de
/// saber el tamaño final de `nexa-ui.css` (es la unión de *todo el
/// sitio*, solo se conoce tras compilar todas las páginas).
struct PendingPageUsage {
    pattern: String,
    initial_js_bytes: u64,
    links_ui_css: bool,
    largest_image: Option<(String, u64)>,
}

/// La imagen estática más pesada que referencia la página, resuelta
/// contra `public/` (Fase 14) — `None` si no referencia ninguna, o si
/// ninguna de las que referencia existe ahí (ej. viene de una URL
/// externa, que esto no comprueba).
fn largest_static_image(page: &CompiledPage) -> Option<(String, u64)> {
    image_scan::collect_image_srcs(&page.ir.root)
        .into_iter()
        .filter_map(|src| {
            let rest = src.trim_start_matches('/');
            let bytes = fs::metadata(Path::new("public").join(rest)).ok()?.len();
            Some((src, bytes))
        })
        .max_by_key(|(_, bytes)| *bytes)
}

/// Si el proyecto declaró `[performance]` en `nexa.toml`, comprueba cada
/// página compilada contra esos límites — y si alguna se pasa, el build
/// completo termina en error (no solo un aviso: ver el comentario del
/// módulo `performance_budget`).
fn check_performance_budgets(pending: &[PendingPageUsage], ui_css_bytes: Option<u64>) -> Result<()> {
    let project = crate::manifest::load_lenient(Path::new("nexa.toml"));
    let Some(budget) = project.performance else {
        return Ok(());
    };

    let pages: Vec<PageUsage> = pending
        .iter()
        .map(|p| PageUsage {
            pattern: p.pattern.clone(),
            initial_js_bytes: p.initial_js_bytes,
            css_bytes: p.links_ui_css.then_some(ui_css_bytes.unwrap_or(0)),
            largest_image: p.largest_image.clone(),
        })
        .collect();

    let violations = performance_budget::check(&budget, &pages);
    if violations.is_empty() {
        return Ok(());
    }

    println!();
    println!("Presupuesto de rendimiento excedido ({} página(s)):", violations.len());
    for violation in &violations {
        println!("  {violation}");
    }

    bail!(
        "el build no cumple el presupuesto de rendimiento declarado en nexa.toml — súbelo, o \
         reduce lo que envían las páginas listadas arriba"
    );
}

/// Directorio de `dist/` donde vive el `index.html` (y su manifiesto) de
/// esta ruta. `/` es especial: vive directamente en la raíz de `dist/`.
fn output_dir(route: &Route) -> PathBuf {
    if route.pattern == "/" {
        Path::new("dist").to_path_buf()
    } else {
        Path::new("dist").join(route.pattern.trim_start_matches('/'))
    }
}

/// Igual que `output_dir`, pero para una ruta dinámica ya resuelta con
/// un set de parámetros concreto (Fase 17: `paths`) — cada segmento
/// `[dinámico]` se sustituye por su valor real en vez de dejarse como
/// `:nombre` literal (que sería un nombre de carpeta sin sentido).
fn output_dir_for_params(route: &Route, params: &BTreeMap<String, String>) -> PathBuf {
    let mut dir = Path::new("dist").to_path_buf();
    for segment in &route.segments {
        match segment {
            Segment::Static(name) => dir.push(name),
            Segment::Dynamic(name) => dir.push(params.get(name).map(String::as_str).unwrap_or_default()),
        }
    }
    dir
}

/// La forma legible (`/products/iphone-17`, no `/products/:slug`) de un
/// set de parámetros ya resuelto — para los mensajes de `nexa build`, el
/// sitemap y el presupuesto de rendimiento.
fn resolved_pattern(route: &Route, params: &BTreeMap<String, String>) -> String {
    if route.segments.is_empty() {
        return "/".to_string();
    }
    let joined = route
        .segments
        .iter()
        .map(|s| match s {
            Segment::Static(name) => name.clone(),
            Segment::Dynamic(name) => params.get(name).cloned().unwrap_or_default(),
        })
        .collect::<Vec<_>>()
        .join("/");
    format!("/{joined}")
}

/// `export const paths = { url: "..." }` (Fase 17): pide esa URL contra
/// el backend y espera un array JSON de objetos, uno por cada
/// combinación de parámetros a pre-renderizar — ej.
/// `[{"slug":"iphone-17"}, {"slug":"pixel-10"}]`. Cada objeto debe traer
/// un campo string por cada segmento dinámico que la ruta declara
/// (`[locale]/products/[slug].tsx` necesita `locale` y `slug` en cada
/// entrada). Un `paths` mal formado o inalcanzable rompe el build
/// entero — es una declaración, no algo que se pueda ignorar en
/// silencio sin dejar la ruta a medio pre-renderizar.
fn enumerate_paths(paths: &Loader, route: &Route, api_base: &str) -> Result<Vec<BTreeMap<String, String>>> {
    let no_params = BTreeMap::new();
    let value = nexa_loader::load(paths, &no_params, api_base)
        .map_err(|err| anyhow::anyhow!("{}: `paths` falló pidiendo la lista de parámetros: {err}", route.pattern))?;

    let entries = value.as_array().ok_or_else(|| {
        anyhow::anyhow!("{}: `paths` debe responder un array JSON, la respuesta fue: {value}", route.pattern)
    })?;

    let dynamic_names: Vec<&str> = route
        .segments
        .iter()
        .filter_map(|s| match s {
            Segment::Dynamic(name) => Some(name.as_str()),
            Segment::Static(_) => None,
        })
        .collect();

    entries
        .iter()
        .map(|entry| {
            let obj = entry
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("{}: cada entrada de `paths` debe ser un objeto, encontré: {entry}", route.pattern))?;

            let mut params = BTreeMap::new();
            for name in &dynamic_names {
                let value = obj.get(*name).and_then(|v| v.as_str()).ok_or_else(|| {
                    anyhow::anyhow!(
                        "{}: una entrada de `paths` no tiene el campo string `{name}` que la ruta necesita: {entry}",
                        route.pattern
                    )
                })?;
                params.insert((*name).to_string(), value.to_string());
            }
            Ok(params)
        })
        .collect()
}

fn write_page(route: &Route, page: &CompiledPage) -> Result<()> {
    write_page_at(&output_dir(route), page)
}

fn write_page_at(dir: &Path, page: &CompiledPage) -> Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("index.html"), &page.html)?;
    fs::write(
        dir.join("nexa-manifest.json"),
        page.manifest
            .to_json_pretty()
            .context("serializando el manifiesto de activación")?,
    )?;

    for chunk in &page.chunks {
        fs::write(format!("dist/assets/{}", chunk.filename), &chunk.content)?;
    }

    Ok(())
}

/// `sitemap.xml` (con URLs absolutas si hay `NEXA_SITE_URL`) y
/// `robots.txt`, para el conjunto de rutas estáticas ya compiladas.
fn write_site_files(patterns: &[String], site_url: Option<&str>) -> Result<()> {
    let urls: Vec<String> = patterns
        .iter()
        .map(|pattern| match site_url {
            Some(base) => format!("{}{}", base.trim_end_matches('/'), pattern),
            None => pattern.clone(),
        })
        .collect();

    fs::write("dist/sitemap.xml", nexa_seo::render_sitemap(&urls))?;

    let sitemap_url = site_url.map(|base| format!("{}/sitemap.xml", base.trim_end_matches('/')));
    fs::write("dist/robots.txt", nexa_seo::render_robots(sitemap_url.as_deref()))?;

    Ok(())
}

/// `dist/assets/nexa-ui.css`: la unión de lo que usa *todo el sitio* (no
/// solo una página) de `@nexa/ui`. Si ninguna página compilada usó nada,
/// no se escribe ni siquiera el archivo — cero bytes, cero petición.
/// Devuelve su tamaño en bytes (para el presupuesto `maxCSS`, Fase 14).
fn write_ui_stylesheet(used_classes: &BTreeSet<String>) -> Result<Option<u64>> {
    let Some(css) = nexa_ui::build_stylesheet_from_classes(used_classes) else {
        return Ok(None);
    };
    let bytes = css.len() as u64;
    fs::write("dist/assets/nexa-ui.css", css)?;
    println!("Generado dist/assets/nexa-ui.css ({} clase(s) de @nexa/ui en uso).", used_classes.len());
    Ok(Some(bytes))
}

/// Copia `@nexa/runtime`, `@nexa/router`, `@nexa/forms` y
/// `@nexa/platform` (ya compilados, embebidos en el binario — ver
/// `crate::assets`) a `dist/assets/`, para que el `<script>` de arranque
/// que inyecta cada página (y los chunks que usan `platform.`)
/// encuentren algo real.
fn write_framework_assets() -> Result<()> {
    fs::write(
        format!("dist/assets/{}", crate::assets::NEXA_RUNTIME_FILENAME),
        crate::assets::NEXA_RUNTIME_JS,
    )?;
    fs::write(
        format!("dist/assets/{}", crate::assets::NEXA_ROUTER_FILENAME),
        crate::assets::NEXA_ROUTER_JS,
    )?;
    fs::write(
        format!("dist/assets/{}", crate::assets::NEXA_FORMS_FILENAME),
        crate::assets::NEXA_FORMS_JS,
    )?;
    fs::write(
        format!("dist/assets/{}", crate::assets::NEXA_PLATFORM_FILENAME),
        crate::assets::NEXA_PLATFORM_JS,
    )?;
    fs::write(
        format!("dist/assets/{}", crate::assets::NEXA_TELEMETRY_FILENAME),
        crate::assets::NEXA_TELEMETRY_JS,
    )?;
    fs::write(
        format!("dist/assets/{}", crate::assets::NEXA_ISLANDS_FILENAME),
        crate::assets::NEXA_ISLANDS_JS,
    )?;
    fs::write(format!("dist/assets/{}", crate::assets::NEXA_UI_FILENAME), crate::assets::NEXA_UI_JS)?;
    Ok(())
}

/// `dist/manifest.webmanifest` + `dist/sw.js` (Fase 18) — una sola vez
/// para todo el sitio, no por página (a diferencia de `<link
/// rel="manifest">`/el registro del service worker, que sí van en cada
/// página vía `pipeline::compile_page`/`bootstrap::inject`).
fn write_pwa_assets(pwa: &crate::manifest::PwaSection) -> Result<()> {
    fs::write("dist/manifest.webmanifest", crate::pwa::render_manifest(pwa))?;
    let sw = crate::pwa::render_service_worker(&pwa.cache).context("generando dist/sw.js")?;
    fs::write("dist/sw.js", sw)?;
    println!("Generado dist/manifest.webmanifest y dist/sw.js (PWA).");
    Ok(())
}

fn print_classification_summary(ir: &IrComponent) {
    let counts = ir.root.classification_counts();

    println!(
        "  Clasificación: {} nodos — {} static, {} dynamic, {} interactive, {} async, {} island",
        counts.total(),
        counts.static_count,
        counts.dynamic_count,
        counts.interactive_count,
        counts.async_count,
        counts.island_count
    );

    if !ir.dependencies.is_empty() {
        let deps: Vec<&str> = ir.dependencies.identifiers().collect();
        println!("  Dependencias detectadas: {}", deps.join(", "));
    }
}

fn print_activation_summary(manifest: &ActivationManifest, chunk_count: usize) {
    if manifest.is_empty() {
        println!("  Activación: 0 nodos interactivos.");
    } else {
        println!(
            "  Activación: {} nodo(s) interactivo(s) -> {} chunk(s) en dist/assets/",
            manifest.len(),
            chunk_count
        );
    }
}

fn print_seo_warnings(warnings: &[Warning]) {
    for warning in warnings {
        println!("  [{}] {}", warning.code, warning.message);
    }
}

fn print_pkg_warnings(warnings: &[crate::pkg_warnings::PackageWarning]) {
    for warning in warnings {
        println!("  [{}] {}", warning.code, warning.message);
    }
}
