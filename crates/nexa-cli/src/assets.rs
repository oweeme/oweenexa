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
/// `@nexa/dev-client` (Fase 11): solo lo sirve `nexa dev`, nunca `nexa
/// build`/`nexa preview` — ver `commands::dev`.
pub const NEXA_DEV_CLIENT_JS: &str = include_str!("../assets/nexa-dev-client.js");
/// `@nexa/devtools` (Fase 14): el panel de diagnóstico — igual que
/// `@nexa/dev-client`, solo lo sirve `nexa dev`.
pub const NEXA_DEVTOOLS_JS: &str = include_str!("../assets/nexa-devtools.js");

pub const NEXA_RUNTIME_FILENAME: &str = "nexa-runtime.js";
pub const NEXA_ROUTER_FILENAME: &str = "nexa-router.js";
pub const NEXA_FORMS_FILENAME: &str = "nexa-forms.js";
pub const NEXA_PLATFORM_FILENAME: &str = "nexa-platform.js";
pub const NEXA_TELEMETRY_FILENAME: &str = "nexa-telemetry.js";
pub const NEXA_ISLANDS_FILENAME: &str = "nexa-islands.js";
pub const NEXA_DEV_CLIENT_FILENAME: &str = "nexa-dev-client.js";
pub const NEXA_DEVTOOLS_FILENAME: &str = "nexa-devtools.js";

/// `@nexa/runtime`, `@nexa/router`, `@nexa/forms` y `@nexa/platform`
/// están embebidos en el propio binario: `nexa build` y `nexa preview`
/// siempre pueden servirlos sin depender de que exista ya un
/// `dist/assets/` en disco. `nexa dev` (Fase 11) añade `@nexa/dev-client`
/// encima de esto (ver `commands::dev::serve_framework_asset`).
pub fn framework_asset(path: &str) -> Option<&'static str> {
    match path.trim_start_matches('/') {
        p if p == format!("assets/{NEXA_RUNTIME_FILENAME}") => Some(NEXA_RUNTIME_JS),
        p if p == format!("assets/{NEXA_ROUTER_FILENAME}") => Some(NEXA_ROUTER_JS),
        p if p == format!("assets/{NEXA_FORMS_FILENAME}") => Some(NEXA_FORMS_JS),
        p if p == format!("assets/{NEXA_PLATFORM_FILENAME}") => Some(NEXA_PLATFORM_JS),
        p if p == format!("assets/{NEXA_TELEMETRY_FILENAME}") => Some(NEXA_TELEMETRY_JS),
        p if p == format!("assets/{NEXA_ISLANDS_FILENAME}") => Some(NEXA_ISLANDS_JS),
        _ => None,
    }
}
