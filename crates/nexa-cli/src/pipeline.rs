use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use nexa_activation::{ActivationManifest, Chunk, Strategy};
use nexa_ir::IrComponent;
use nexa_loader::LoaderError;
use nexa_renderer::RenderContext;
use nexa_seo::{SeoContext, Warning};

use crate::bootstrap;
use crate::document;
use crate::import_map;
use crate::manifest;
use crate::pkg_warnings::{self, PackageWarning};

const LOCALES_DIR: &str = "src/locales";
const MANIFEST_PATH: &str = "nexa.toml";
/// Locale que se usa para `params.locale`/`t()` en páginas que NO viven
/// bajo `[locale]` (Fase 29). Antes de esta fase, `<html lang>` ya caía a
/// `"es"` en este mismo caso (ver `lang` más abajo) pero `params`/`t()` no
/// tenían ningún fallback — quedaban con la key `"locale"` ausente, lo que
/// producía `href`s sin resolver (atributo completo omitido, ver
/// `nexa-renderer::html::render_attr`) y placeholders inertes
/// `<!--nexa:t(clave)-->` en el cuerpo. Mismo valor que ya estaba
/// hardcodeado para `lang`, ahora reutilizado para que ambos coincidan.
const DEFAULT_LOCALE: &str = "es";

/// El hash "pendiente" que lleva el `href` de `nexa-ui.css` hasta que se
/// conoce el contenido final del sitio completo — ver el comentario junto
/// a `ui_stylesheet_href` en [`compile_page`]. No es un hash real (no
/// tiene por qué parecerlo); solo necesita ser un token que no aparezca
/// por casualidad en ningún hash de verdad (los reales son hex).
pub const UI_CSS_HASH_PLACEHOLDER: &str = "pending";

pub fn ui_stylesheet_href_with_hash(hash: &str) -> String {
    format!("/assets/nexa-ui.{hash}.css")
}

/// El resultado de correr el pipeline completo (parse -> load -> analyze
/// -> render -> SEO -> i18n -> activation -> bootstrap) sobre un único
/// archivo de página.
pub struct CompiledPage {
    pub html: String,
    pub manifest: ActivationManifest,
    pub chunks: Vec<Chunk>,
    pub ir: IrComponent,
    /// Avisos del SEO Analyzer (`NEXA-SEO-*`, `NEXA-A11Y-*`) — nunca
    /// hacen fallar el build, solo se muestran.
    pub seo_warnings: Vec<Warning>,
    /// Clases `nx-*` estáticas que usa esta página. `nexa-cli` acumula
    /// esto entre páginas para decidir qué va en el `nexa-ui.css`
    /// compartido — una sola página no sabe cuánto necesita *todo el
    /// sitio*, así que no ensambla el CSS ella misma.
    pub ui_used_classes: BTreeSet<String>,
    /// Avisos `NEXA-PKG-*` (Fase 12): la página usa un módulo oficial
    /// (`@nexa/ui`, `@nexa/forms`) que `nexa.toml` no declara. Igual que
    /// `seo_warnings`, nunca hacen fallar el build.
    pub pkg_warnings: Vec<PackageWarning>,
    /// Bytes de JS que esta página descarga *sin esperar interacción*
    /// (Fase 14): los assets de framework que el bootstrap siempre/
    /// condicionalmente importa (`@nexa/router` siempre; `@nexa/runtime`
    /// si hay algo interactivo; `@nexa/forms` si hay un formulario) más
    /// los chunks con `Strategy::Load` (se piden de inmediato). Los
    /// chunks `interaction`/`visible`/`idle`/`manual` se difieren a
    /// propósito y no cuentan — es justo la señal que usa el presupuesto
    /// `maxInitialJS`.
    pub initial_js_bytes: u64,
}

/// Lo que puede salir mal al compilar una página, distinguiendo "no
/// existe el dato pedido" (el `nexa preview` de esta fase lo convierte en
/// un 404 real) de cualquier otro fallo.
pub enum PageError {
    NotFound,
    Other(anyhow::Error),
}

impl From<anyhow::Error> for PageError {
    fn from(err: anyhow::Error) -> Self {
        PageError::Other(err)
    }
}

impl std::fmt::Display for PageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageError::NotFound => write!(f, "recurso no encontrado (404)"),
            PageError::Other(err) => write!(f, "{err}"),
        }
    }
}

/// Compila una página `.tsx` de punta a punta, ejecutando su `load()` si
/// lo declara.
///
/// `route_pattern` (ej. `/:locale/products/:slug`) se usa para calcular
/// el `hreflang` de las demás traducciones; `params` son los parámetros
/// ya capturados por `nexa-router` (ej. `{"locale": "es", "slug":
/// "iphone-17"}`) — se usan para resolver `load.url`, para que el
/// componente muestre `{params.slug}`, y para cargar
/// `src/locales/<params.locale>.json` si la página vive bajo `[locale]`.
/// `api_base` es la URL base del backend (`NEXA_API_URL`).
/// `export const paths = { url: "..." }` (Fase 17), si esta página lo
/// declara — lo que `nexa build` necesita saber ANTES de decidir si una
/// ruta dinámica se puede enumerar y pre-renderizar, o si sigue
/// quedando solo para `nexa preview`. Un parseo aparte y barato (no hay
/// forma de pedir "solo el `paths`" sin parsear el archivo primero).
pub fn find_paths_declaration(file: &Path) -> Result<Option<nexa_ast::Loader>> {
    let source = fs::read_to_string(file).with_context(|| format!("leyendo {}", file.display()))?;
    let component = nexa_parser::parse_component(file.to_str().unwrap_or("page.tsx"), &source)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("compilando {}", file.display()))?;
    Ok(component.paths)
}

pub fn compile_page(
    file: &Path,
    route_pattern: &str,
    params: &BTreeMap<String, String>,
    api_base: &str,
) -> Result<CompiledPage, PageError> {
    let source = fs::read_to_string(file).with_context(|| format!("leyendo {}", file.display()))?;

    let component = nexa_parser::parse_component(file.to_str().unwrap_or("page.tsx"), &source)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("compilando {}", file.display()))?;

    let component_name = component.name.clone();

    // Fase 29: si la página no vive bajo `[locale]`, `params` nunca tiene
    // la key `"locale"` (no es que valga "" o "undefined" — la key no
    // existe). Se completa acá, una sola vez, con el mismo default que
    // `<html lang>` ya usaba — así todo lo que sigue (`t()`, `params.locale`
    // en JSX/atributos, hreflang, SEO) ve un locale real en vez de tener
    // que lidiar cada uno por separado con la ausencia.
    let owned_params;
    let params: &BTreeMap<String, String> = if params.contains_key("locale") {
        params
    } else {
        let mut with_default = params.clone();
        with_default.insert("locale".to_string(), DEFAULT_LOCALE.to_string());
        owned_params = with_default;
        &owned_params
    };

    let data = match &component.loader {
        Some(loader) => {
            let value = nexa_loader::load(loader, params, api_base).map_err(|err| match err {
                LoaderError::NotFound => PageError::NotFound,
                other => PageError::Other(anyhow::anyhow!(
                    "cargando datos de {}: {other}",
                    file.display()
                )),
            })?;
            Some(value)
        }
        None => None,
    };

    let locales_dir = Path::new(LOCALES_DIR);
    let translations = params
        .get("locale")
        .and_then(|locale| nexa_i18n::load_locale(locales_dir, locale).ok().flatten());

    let ir = nexa_analyzer::analyze(&component);

    // Fase 45: `t("clave", { name: data.x })` con un `{placeholder}` en
    // el diccionario que no tiene su variable correspondiente (o al
    // revés) es casi siempre un error del desarrollador, no algo que
    // "degradar con gracia" tenga sentido — mejor un build roto y
    // explícito que un `<!--nexa:...-->` o un `{name}` literal en
    // producción que nadie nota hasta que un usuario lo ve. Se valida
    // acá, antes de renderizar, y solo si esta página tiene diccionario
    // (`translations`) — sin locale, `t()` ya degrada de la forma
    // habitual (Fase 10), no hay nada nuevo que validar.
    if let Some(translations) = translations.as_ref() {
        if let Err(message) = validate_translate_calls(&ir.root, translations) {
            return Err(PageError::Other(anyhow::anyhow!(
                "{}: {message}",
                file.display()
            )));
        }
    }

    // Fase 33: la ruta real de esta página, con sus segmentos dinámicos
    // ya resueltos (`/es/products/iphone-17`, no `/:locale/products/:slug`)
    // — mismo `resolve_url` que ya usa `nexa-i18n` para `hreflang`. Nunca
    // bloquea el render si no se puede resolver (no debería pasar nunca
    // en un uso normal, pero un link activo que no aparece es mucho
    // menos grave que un build roto).
    let current_path = nexa_loader::resolve_url(route_pattern, params).ok();

    let render_ctx = RenderContext {
        data: data.as_ref(),
        params,
        translations: translations.as_ref(),
        loop_binding: None,
        current_path: current_path.as_deref(),
    };
    let body = nexa_renderer::render_node(&ir.root, &render_ctx);

    // `src/layout.tsx` (Fase 25), opcional: envuelve el HTML que la
    // página ya renderizó — ver el comentario de `crate::layout` sobre
    // por qué esto no reabre la composición de componentes.
    let mut layout_ui_used_classes = std::collections::BTreeSet::new();
    let mut layout_head_html = String::new();
    let mut layout_island_specifiers = std::collections::BTreeSet::new();
    let body = match crate::layout::find() {
        Some(layout_file) => {
            let rendered = crate::layout::render_for_page(
                &layout_file,
                &body,
                params,
                translations.as_ref(),
                current_path.as_deref(),
            )?;
            layout_ui_used_classes = rendered.ui_used_classes;
            layout_head_html = rendered.head_html;
            layout_island_specifiers = rendered.island_specifiers;
            rendered.html
        }
        None => body,
    };

    let seo_ctx = SeoContext { data: data.as_ref(), params, translations: translations.as_ref() };
    let seo_head = nexa_seo::render_head(component.seo.as_ref(), &seo_ctx);
    let schema_script = nexa_seo::render_schema_script(component.schema.as_ref(), &seo_ctx);
    let seo_warnings = nexa_seo::analyze(&ir.root, component.seo.as_ref());
    // Fase 37: `export const head` no es exclusivo de `src/layout.tsx`
    // — una página individual también puede declararlo (ej. un
    // `google-site-verification` que solo aplica a esa ruta, o un
    // `theme-color` distinto para una sola página). Mismo mecanismo de
    // resolución que ya usaba el layout, invocado una segunda vez acá.
    let page_head_html = nexa_seo::render_layout_head(component.head.as_ref(), &seo_ctx);

    let project_manifest = manifest::load_lenient(Path::new(MANIFEST_PATH));
    let telemetry_endpoint = project_manifest.telemetry.as_ref().and_then(|t| t.endpoint.as_deref());

    let hreflang_head = hreflang_head(route_pattern, params, locales_dir);
    // `[pwa]` (Fase 18): `<link rel="manifest">` + `theme-color`, mismo
    // costo cero que el resto — ausente si el proyecto no declaró `[pwa]`.
    let pwa_head = project_manifest.pwa.as_ref().map(crate::pwa::head_fragment).unwrap_or_default();
    let combined_head = join_head_fragments(&join_head_fragments(&seo_head, &hreflang_head), &pwa_head);
    // `export const head` de src/layout.tsx (Fase 27): a diferencia del
    // resto de lo que devuelve un layout (HTML de <body>, via el splice
    // del slot), esto va al <head> real del documento — un
    // <link rel="icon"> puesto en <body> no lo detectan todos los
    // navegadores de forma confiable.
    let combined_head = join_head_fragments(&combined_head, &layout_head_html);
    // `export const head` de la propia página (Fase 37) — se agrega
    // después del de layout, así una página puede repetir una clave
    // (ej. un `theme-color` distinto) sin que el orden en el HTML
    // final quede ambiguo sobre cuál "gana" para el navegador.
    let combined_head = join_head_fragments(&combined_head, &page_head_html);

    let mut ui_used_classes = nexa_ui::collect_used_classes(&ir.root);
    ui_used_classes.extend(layout_ui_used_classes);
    // El nombre final de `nexa-ui.css` lleva un hash de su contenido
    // (Fase 23) — pero ese contenido es la unión de *todo el sitio*, que
    // recién se conoce después de compilar todas las páginas (ver
    // `commands::build::write_ui_stylesheet`). Por eso el `href` se
    // emite con un hash "pendiente" (`UI_CSS_HASH_PLACEHOLDER`) que quien
    // orqueste el build reemplaza por el real una vez que lo conoce —
    // `nexa dev`/`nexa preview` (una sola página a la vez, ver
    // `page_resolver::resolve_page`) lo conocen de inmediato y no llegan
    // a dejar el placeholder en el HTML que sirven.
    let ui_stylesheet_href = (!ui_used_classes.is_empty()).then(|| ui_stylesheet_href_with_hash(UI_CSS_HASH_PLACEHOLDER));

    // Import map (Fase 15): `platform` (built-in) + lo que el proyecto
    // declare en `[imports]` — solo entra al `<head>` lo que esta
    // página de verdad usa (`import_map::used_by_page`), y solo se le
    // dice a `nexa-activation` qué nombres existen (no a qué URL
    // resuelven — eso es cosa del import map del documento, no del
    // chunk).
    let all_imports = import_map::resolved(&project_manifest);
    let import_names: std::collections::BTreeSet<String> = all_imports.keys().cloned().collect();
    let mut island_specifiers = island_specifiers(&ir.root);
    // Fase 31: un `data-nexa-island` declarado en `src/layout.tsx` (no
    // solo en la página) también necesita entrar al import map y al
    // bootstrap de `nexa-islands.js` — se une acá, antes de que
    // `used_imports`/`has_islands`/`pkg_warnings` lo necesiten.
    island_specifiers.extend(layout_island_specifiers);
    let used_imports = import_map::used_by_page(&all_imports, &component.handlers, &island_specifiers);
    let import_map_script = import_map::script_tag(&used_imports);

    // `params` ya tiene "locale" en este punto (siempre, gracias al
    // fallback de más arriba) — `unwrap_or` queda solo como red de
    // seguridad, nunca debería activarse.
    let lang = params.get("locale").map(String::as_str).unwrap_or(DEFAULT_LOCALE);
    let html = document::assemble(
        &body,
        lang,
        &combined_head,
        schema_script.as_deref(),
        ui_stylesheet_href.as_deref(),
        import_map_script.as_deref(),
    );

    // Bug #27: un handler que usa un `const` de nivel superior no lo
    // bastante simple para copiar dentro de su chunk (una llamada a
    // función, otra variable, algo calculado) hace fallar el build acá
    // mismo, con el mensaje exacto de `nexa-activation` — mejor esto
    // que un `ReferenceError` silencioso en el navegador, el mismo
    // criterio que ya aplica `validate_translate_calls` más arriba.
    let (activation_manifest, chunks) = nexa_activation::build(&ir, &component_name, &component.handlers, &component.consts, &import_names)
        .map_err(|message| PageError::Other(anyhow::anyhow!("{}: {message}", file.display())))?;
    let has_forms = has_forms(&ir.root);
    let has_islands = !island_specifiers.is_empty();
    let has_pwa = project_manifest.pwa.is_some();
    let html = bootstrap::inject(&html, &activation_manifest, has_forms, has_islands, has_pwa, telemetry_endpoint);

    let has_platform = component.handlers.values().any(|source| source.contains("platform."));
    let pkg_warnings = pkg_warnings::check(
        &project_manifest,
        &ui_used_classes,
        has_forms,
        has_platform,
        &island_specifiers,
        &all_imports,
    );

    let initial_js_bytes =
        initial_js_bytes(&activation_manifest, &chunks, has_forms, telemetry_endpoint.is_some());

    Ok(CompiledPage {
        html,
        manifest: activation_manifest,
        chunks,
        ir,
        seo_warnings,
        ui_used_classes,
        pkg_warnings,
        initial_js_bytes,
    })
}

fn initial_js_bytes(
    manifest: &ActivationManifest,
    chunks: &[Chunk],
    has_forms: bool,
    has_telemetry: bool,
) -> u64 {
    let mut total = crate::assets::NEXA_ROUTER_JS.len() as u64;
    if !manifest.is_empty() {
        total += crate::assets::NEXA_RUNTIME_JS.len() as u64;
    }
    if has_forms {
        total += crate::assets::NEXA_FORMS_JS.len() as u64;
    }
    if has_telemetry {
        total += crate::assets::NEXA_TELEMETRY_JS.len() as u64;
    }
    for chunk in chunks {
        if chunk.strategy == Strategy::Load {
            total += chunk.content.len() as u64;
        }
    }
    total
}

/// ¿Hay algún `<form data-nexa-form>` en la página? Si no, `@nexa/forms`
/// no se carga en absoluto — la misma disciplina de costo cero que ya
/// aplica `@nexa/ui` (Fase 9) y la activación (Fase 5).
fn has_forms(root: &nexa_ir::IrNode) -> bool {
    use nexa_ir::IrNodeKind;

    if let IrNodeKind::For { body, .. } = &root.kind {
        return has_forms(body);
    }

    let IrNodeKind::Element { attrs, children, .. } = &root.kind else {
        return false;
    };

    let this_is_a_form = attrs.iter().any(|a| a.name == "data-nexa-form");
    this_is_a_form || children.iter().any(has_forms)
}

/// Los specifiers (`data-nexa-island="..."`) que declara esta página
/// (Fase 16) — se usan tanto para decidir qué entra al import map
/// (`import_map::used_by_page`) como para avisar (`pkg_warnings`) si
/// alguno no está declarado en `nexa.toml [imports]`. `pub(crate)`
/// desde la Fase 31: `layout.rs` la reutiliza tal cual para recoger los
/// specifiers que declare `src/layout.tsx`.
pub(crate) fn island_specifiers(root: &nexa_ir::IrNode) -> BTreeSet<String> {
    use nexa_ir::IrNodeKind;

    let mut out = BTreeSet::new();
    root.walk(&mut |node| {
        if let IrNodeKind::Element { island: Some(island), .. } = &node.kind {
            out.insert(island.specifier.clone());
        }
    });
    out
}

/// `t("clave", { name: data.x })` (Fase 45): la clave del diccionario
/// puede tener `{placeholder}` que no correspondan con `args`, o al
/// revés — cualquiera de los dos casos casi siempre es un error del
/// desarrollador (una variable que se olvidó de pasar, o una que sobra
/// porque cambió el texto y no el código). Solo valida `{t(...)}` del
/// cuerpo JSX (`IrNodeKind::Translate`, alcanzable con `IrNode::walk`);
/// un `t(...)` dentro de `seo`/`schema`/`head` sigue interpolando bien
/// en tiempo de render (mismo `resolve_translation` compartido), pero
/// esta validación de build no lo cubre todavía — ajuste de alcance
/// explícito, documentado en la Fase 45.
fn validate_translate_calls(root: &nexa_ir::IrNode, translations: &serde_json::Value) -> Result<(), String> {
    use nexa_ir::IrNodeKind;

    let mut errors = Vec::new();
    root.walk(&mut |node| {
        let IrNodeKind::Translate(translate) = &node.kind else { return };
        let Some(raw) = lookup_translation_string(translations, &translate.key) else { return };

        let placeholders: BTreeSet<&str> = extract_placeholders(&raw);
        let arg_names: BTreeSet<&str> = translate.args.iter().map(|(name, _)| name.as_str()).collect();

        for missing in placeholders.difference(&arg_names) {
            errors.push(format!(
                "t(\"{}\", ...) usa \"{{{missing}}}\" pero no se pasó esa variable — la clave dice: \"{raw}\"",
                translate.key
            ));
        }
        for extra in arg_names.difference(&placeholders) {
            errors.push(format!(
                "t(\"{}\", {{ {extra}: ... }}) pasa \"{extra}\" pero la clave no tiene ningún \"{{{extra}}}\" — la clave dice: \"{raw}\"",
                translate.key
            ));
        }
    });

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn lookup_translation_string(translations: &serde_json::Value, key: &str) -> Option<String> {
    let mut value = translations;
    for segment in key.split('.') {
        value = value.get(segment)?;
    }
    value.as_str().map(str::to_string)
}

/// Extrae los nombres entre `{}` de un texto (`"Apoyar a {name}"` ->
/// `["name"]`) — sin regex, un escaneo manual alcanza para algo tan
/// simple, y evita sumar una dependencia nueva solo para esto.
fn extract_placeholders(text: &str) -> BTreeSet<&str> {
    let mut names = BTreeSet::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else { break };
        let candidate = &after[..end];
        if !candidate.is_empty() && candidate.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            names.insert(candidate);
        }
        rest = &after[end + 1..];
    }
    names
}

/// `<link rel="alternate" hreflang="...">` por cada locale disponible —
/// vacío si esta ruta no vive bajo `[locale]`, o si no hay ningún
/// `src/locales/*.json`.
fn hreflang_head(route_pattern: &str, params: &BTreeMap<String, String>, locales_dir: &Path) -> String {
    if !route_pattern.contains(":locale") {
        return String::new();
    }

    let available = nexa_i18n::available_locales(locales_dir);
    if available.is_empty() {
        return String::new();
    }

    let links = nexa_i18n::alternate_links(route_pattern, params, &available);
    nexa_seo::render_hreflang_links(&links)
}

fn join_head_fragments(a: &str, b: &str) -> String {
    match (a.is_empty(), b.is_empty()) {
        (true, true) => String::new(),
        (false, true) => a.to_string(),
        (true, false) => b.to_string(),
        (false, false) => format!("{a}\n{b}"),
    }
}

#[cfg(test)]
mod translate_validation_tests {
    use super::*;
    use nexa_ast::{Expr, Translate};
    use nexa_ir::{Classification, IrNode, IrNodeKind};
    use serde_json::json;

    fn translate_node(key: &str, args: Vec<(&str, Expr)>) -> IrNode {
        IrNode {
            id: 0,
            classification: Classification::Dynamic,
            kind: IrNodeKind::Translate(Translate {
                key: key.to_string(),
                args: args.into_iter().map(|(name, expr)| (name.to_string(), expr)).collect(),
            }),
        }
    }

    fn data_dot(name: &str) -> Expr {
        Expr::Member { object: Box::new(Expr::Identifier("data".into())), property: name.into() }
    }

    #[test]
    fn extracts_a_single_placeholder() {
        assert_eq!(extract_placeholders("Apoyar a {name}"), BTreeSet::from(["name"]));
    }

    #[test]
    fn extracts_several_placeholders() {
        assert_eq!(extract_placeholders("{count} de {total}"), BTreeSet::from(["count", "total"]));
    }

    #[test]
    fn a_string_without_placeholders_extracts_nothing() {
        assert!(extract_placeholders("Bienvenido").is_empty());
    }

    #[test]
    fn a_key_and_args_that_match_exactly_is_valid() {
        let root = translate_node("profile.donateTo", vec![("name", data_dot("creatorName"))]);
        let dictionary = json!({ "profile": { "donateTo": "Apoyar a {name}" } });
        assert!(validate_translate_calls(&root, &dictionary).is_ok());
    }

    #[test]
    fn t_without_any_args_on_a_key_without_placeholders_is_valid() {
        let root = translate_node("home.title", vec![]);
        let dictionary = json!({ "home": { "title": "Bienvenido" } });
        assert!(validate_translate_calls(&root, &dictionary).is_ok());
    }

    #[test]
    fn a_placeholder_in_the_dictionary_with_no_matching_arg_is_a_clear_error() {
        let root = translate_node("profile.donateTo", vec![]);
        let dictionary = json!({ "profile": { "donateTo": "Apoyar a {name}" } });
        let err = validate_translate_calls(&root, &dictionary).unwrap_err();
        assert!(err.contains("{name}"), "el error debería mencionar el placeholder sin cubrir: {err}");
        assert!(err.contains("profile.donateTo"));
    }

    #[test]
    fn an_extra_arg_with_no_matching_placeholder_is_a_clear_error() {
        let root = translate_node("home.title", vec![("name", data_dot("x"))]);
        let dictionary = json!({ "home": { "title": "Bienvenido" } });
        let err = validate_translate_calls(&root, &dictionary).unwrap_err();
        assert!(err.contains("name"), "el error debería mencionar el argumento que sobra: {err}");
    }

    #[test]
    fn a_missing_translation_key_is_not_a_validation_error_here() {
        // La clave ausente ya degrada del modo habitual al renderizar
        // (comentario inerte, Fase 10) — no es responsabilidad de esta
        // validación, que solo compara placeholders contra args cuando
        // la clave sí resuelve a un string real.
        let root = translate_node("nope.nope", vec![("name", data_dot("x"))]);
        let dictionary = json!({ "home": { "title": "Bienvenido" } });
        assert!(validate_translate_calls(&root, &dictionary).is_ok());
    }

    #[test]
    fn walks_into_element_children_to_find_translate_nodes() {
        let child = translate_node("profile.donateTo", vec![]);
        let root = IrNode {
            id: 1,
            classification: Classification::Static,
            kind: IrNodeKind::Element {
                tag: "p".into(),
                attrs: vec![],
                events: vec![],
                island: None,
                children: vec![child],
            },
        };
        let dictionary = json!({ "profile": { "donateTo": "Apoyar a {name}" } });
        assert!(validate_translate_calls(&root, &dictionary).is_err());
    }
}
