//! Los bundles ya compilados de `@nexa/runtime`, `@nexa/router` y
//! `@nexa/forms` (TypeScript -> JS plano, un solo archivo cada uno, sin
//! dependencias externas — ninguno importa nada fuera de sí mismo, así
//! que un bundle de un solo archivo con esbuild basta, sin necesitar un
//! bundler general).
//!
//! Se generan con `npm run build:cli-assets` desde la raíz del repo (ver
//! `assets/`, y `package.json` en la raíz) y se incrustan aquí en tiempo
//! de compilación de `nexa-cli`: así `nexa build`/`nexa preview` pueden
//! escribirlos en `dist/assets/` sin necesitar el repositorio de Nexa
//! disponible en la máquina donde se usa el binario instalado.

pub const NEXA_RUNTIME_JS: &str = include_str!("../assets/nexa-runtime.js");
pub const NEXA_ROUTER_JS: &str = include_str!("../assets/nexa-router.js");
pub const NEXA_FORMS_JS: &str = include_str!("../assets/nexa-forms.js");
/// `@nexa/platform` (Fase 13): igual que `@nexa/forms`, siempre se
/// escribe en `dist/assets/` (barato, es solo un archivo en disco), pero
/// un chunk solo lo importa si de verdad usa `platform.` — ver
/// `nexa-activation::chunk::uses_platform`.
pub const NEXA_PLATFORM_JS: &str = include_str!("../assets/nexa-platform.js");
/// `@nexa/telemetry` (Fase 14): igual que `@nexa/platform`, siempre se
/// escribe en `dist/assets/` (barato), pero el `<script>` de arranque
/// solo lo importa si `nexa.toml` declara `[telemetry] endpoint` — ver
/// `bootstrap::inject`.
pub const NEXA_TELEMETRY_JS: &str = include_str!("../assets/nexa-telemetry.js");
/// `@nexa/islands` (Fase 16): igual que `@nexa/forms`, siempre se
/// escribe en `dist/assets/` (barato), pero el `<script>` de arranque
/// solo lo importa si la página tiene al menos un `data-nexa-island` —
/// ver `bootstrap::inject`.
pub const NEXA_ISLANDS_JS: &str = include_str!("../assets/nexa-islands.js");
/// `@nexa/ui` (comportamiento de Dialog — `ui.openDialog`/`ui.closeDialog`):
/// builtin igual que `platform`, corregido tras descubrir que nunca
/// había estado conectado a un import real utilizable desde una página.
pub const NEXA_UI_JS: &str = include_str!("../assets/nexa-ui.js");
/// `@nexa/dev-client` (Fase 11): solo lo sirve `nexa dev`, nunca `nexa
/// build`/`nexa preview` — ver `commands::dev`.
pub const NEXA_DEV_CLIENT_JS: &str = include_str!("../assets/nexa-dev-client.js");
/// `@nexa/devtools` (Fase 14): el panel de diagnóstico — igual que
/// `@nexa/dev-client`, solo lo sirve `nexa dev`.
pub const NEXA_DEVTOOLS_JS: &str = include_str!("../assets/nexa-devtools.js");

/// Hash corto (8 hex) de contenido — cache-busting real: el nombre de
/// archivo cambia si (y solo si) el contenido cambió. No es una firma
/// criptográfica; mismo algoritmo (FNV-1a) que ya usa `modules::content_hash`
/// para `nexa.lock`, duplicado acá a propósito en vez de crear una
/// dependencia cruzada nueva entre módulos por una decena de líneas.
pub(crate) fn content_short_hash(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (hash ^ (hash >> 32)) as u32)
}

/// Estos archivos están embebidos en el binario (`include_str!` arriba):
/// su contenido es fijo para una compilación dada de `nexa-cli`, así que
/// su hash también lo es — no hace falta recalcularlo en cada request
/// para que sea determinista, pero tampoco hace falta cachearlo: son unos
/// pocos KB, hashearlos de nuevo en cada llamada es barato.
pub fn nexa_runtime_filename() -> String {
    format!("nexa-runtime.{}.js", content_short_hash(NEXA_RUNTIME_JS.as_bytes()))
}
pub fn nexa_router_filename() -> String {
    format!("nexa-router.{}.js", content_short_hash(NEXA_ROUTER_JS.as_bytes()))
}
pub fn nexa_forms_filename() -> String {
    format!("nexa-forms.{}.js", content_short_hash(NEXA_FORMS_JS.as_bytes()))
}
pub fn nexa_platform_filename() -> String {
    format!("nexa-platform.{}.js", content_short_hash(NEXA_PLATFORM_JS.as_bytes()))
}
pub fn nexa_telemetry_filename() -> String {
    format!("nexa-telemetry.{}.js", content_short_hash(NEXA_TELEMETRY_JS.as_bytes()))
}
pub fn nexa_islands_filename() -> String {
    format!("nexa-islands.{}.js", content_short_hash(NEXA_ISLANDS_JS.as_bytes()))
}
pub fn nexa_ui_js_filename() -> String {
    format!("nexa-ui.{}.js", content_short_hash(NEXA_UI_JS.as_bytes()))
}

/// Solo `nexa dev` sirve estos dos (nunca `build`/`preview`) directo desde
/// memoria en cada request — no hace falta un nombre estable entre
/// requests para que el navegador cachee entre visitas de desarrollo.
pub const NEXA_DEV_CLIENT_FILENAME: &str = "nexa-dev-client.js";
pub const NEXA_DEVTOOLS_FILENAME: &str = "nexa-devtools.js";

/// `@nexa/runtime`, `@nexa/router`, `@nexa/forms` y `@nexa/platform`
/// están embebidos en el propio binario: `nexa build` y `nexa preview`
/// siempre pueden servirlos sin depender de que exista ya un
/// `dist/assets/` en disco. `nexa dev` (Fase 11) añade `@nexa/dev-client`
/// encima de esto (ver `commands::dev::serve_framework_asset`).
pub fn framework_asset(path: &str) -> Option<&'static str> {
    let p = path.trim_start_matches('/');
    if p == format!("assets/{}", nexa_runtime_filename()) {
        return Some(NEXA_RUNTIME_JS);
    }
    if p == format!("assets/{}", nexa_router_filename()) {
        return Some(NEXA_ROUTER_JS);
    }
    if p == format!("assets/{}", nexa_forms_filename()) {
        return Some(NEXA_FORMS_JS);
    }
    if p == format!("assets/{}", nexa_platform_filename()) {
        return Some(NEXA_PLATFORM_JS);
    }
    if p == format!("assets/{}", nexa_telemetry_filename()) {
        return Some(NEXA_TELEMETRY_JS);
    }
    if p == format!("assets/{}", nexa_islands_filename()) {
        return Some(NEXA_ISLANDS_JS);
    }
    if p == format!("assets/{}", nexa_ui_js_filename()) {
        return Some(NEXA_UI_JS);
    }
    None
}
