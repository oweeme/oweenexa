//! El registro de módulos oficiales de Nexa que se pueden declarar con
//! `nexa add <módulo>` (Fase 12). Hoy todos viven embebidos en el propio
//! binario de `nexa-cli` (Fases 9-11) — no hay nada que descargar de una
//! red —, así que este registro es la *forma* de un sistema de paquetes
//! real, no un cliente de un registro remoto (eso es explícitamente la
//! Fase 15, con paquetes de terceros).
//!
//! `@nexa/router` y `@nexa/runtime` no aparecen aquí: son parte del core,
//! siempre activos, no se declaran ni se quitan (ver `bootstrap::inject`).
//! `@nexa/dev-client` tampoco: solo lo usa `nexa dev` internamente,
//! nunca es algo que un proyecto declare (Fase 11).

use crate::assets;

/// Ver `docs/POLITICA-DE-VERSIONES.md` para las reglas completas de cada
/// nivel — en resumen: `Stable` sigue SemVer real (solo rompe en mayor),
/// `Experimental` puede romper en cualquier versión menor (con aviso),
/// `Internal` no tiene garantía de compatibilidad y no se declara a mano.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stability {
    Stable,
    Experimental,
    Internal,
}

impl Stability {
    pub fn as_str(self) -> &'static str {
        match self {
            Stability::Stable => "stable",
            Stability::Experimental => "experimental",
            Stability::Internal => "internal",
        }
    }
}

pub struct ModuleInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub stability: Stability,
    pub description: &'static str,
}

/// Los módulos que de verdad se pueden pedir con `nexa add`. El orden es
/// el que se muestra en los mensajes de error ("disponibles: ...").
pub const ADDABLE: &[ModuleInfo] = &[
    ModuleInfo {
        name: "ui",
        version: "0.1.0",
        stability: Stability::Stable,
        description: "Design tokens + Button/Input/Card/Dialog en CSS, con tree-shaking real.",
    },
    ModuleInfo {
        name: "forms",
        version: "0.1.0",
        stability: Stability::Stable,
        description: "Validación con progressive enhancement sobre la Constraint Validation API nativa.",
    },
    ModuleInfo {
        name: "tauri",
        version: "0.1.0",
        stability: Stability::Experimental,
        description: "Genera src-tauri/ para empaquetar dist/ como app de escritorio nativa.",
    },
    ModuleInfo {
        name: "capacitor",
        version: "0.1.0",
        stability: Stability::Experimental,
        description: "Genera capacitor.config.json para empaquetar dist/ como app móvil (Android/iOS).",
    },
    ModuleInfo {
        name: "platform",
        version: "0.1.0",
        stability: Stability::Experimental,
        description: "isTauri/isCapacitor/isWeb + notifications/storage/share/camera, con un solo código para los tres.",
    },
    ModuleInfo {
        name: "telemetry",
        version: "0.1.0",
        stability: Stability::Experimental,
        description: "Core Web Vitals + errores sin capturar — solo se inyecta si nexa.toml declara [telemetry] endpoint.",
    },
    ModuleInfo {
        name: "pwa",
        version: "0.1.0",
        stability: Stability::Experimental,
        description: "manifest.webmanifest + service worker real generados por nexa build a partir de [pwa] en nexa.toml.",
    },
    ModuleInfo {
        name: "nginx",
        version: "0.1.0",
        stability: Stability::Experimental,
        description: "Genera deploy/nginx.conf para servir dist/ — gzip, cabeceras de seguridad, fallback de rutas dinámicas.",
    },
];

/// Módulos que existen pero que **no** se declaran con `nexa add`: son
/// parte del core, siempre activos (`router`, `runtime`), o son
/// herramientas internas de un solo comando (`dev-client`, solo lo usa
/// `nexa dev`). Sirven para dar un mensaje de error útil cuando alguien
/// intenta `nexa add router` en vez de uno que se equivoca de nombre por
/// completo.
pub const INTERNAL: &[ModuleInfo] = &[
    ModuleInfo {
        name: "router",
        version: "0.1.0",
        stability: Stability::Internal,
        description: "Navegación SPA + prefetch — siempre activo, no se declara.",
    },
    ModuleInfo {
        name: "runtime",
        version: "0.1.0",
        stability: Stability::Internal,
        description: "Progressive Activation — siempre activo, no se declara.",
    },
    ModuleInfo {
        name: "dev-client",
        version: "0.1.0",
        stability: Stability::Internal,
        description: "Auto-reload de `nexa dev` — nunca se declara a mano.",
    },
];

pub fn find(name: &str) -> Option<&'static ModuleInfo> {
    ADDABLE.iter().find(|m| m.name == name)
}

pub fn find_internal(name: &str) -> Option<&'static ModuleInfo> {
    INTERNAL.iter().find(|m| m.name == name)
}

/// Un hash de *contenido* (no una firma criptográfica ni una garantía de
/// integridad frente a manipulación — no hay red de por medio todavía):
/// solo sirve para que `nexa.lock` pueda notar si el contenido embebido
/// de un módulo cambió entre una versión del binario y otra.
pub fn content_hash(name: &str) -> Option<String> {
    match name {
        "ui" => Some(fnv1a_hex(nexa_ui::full_source().as_bytes())),
        "forms" => Some(fnv1a_hex(assets::NEXA_FORMS_JS.as_bytes())),
        "tauri" => Some(fnv1a_hex(&crate::tauri_scaffold::template_bytes())),
        "capacitor" => Some(fnv1a_hex(crate::capacitor_scaffold::template_bytes())),
        "platform" => Some(fnv1a_hex(assets::NEXA_PLATFORM_JS.as_bytes())),
        "telemetry" => Some(fnv1a_hex(assets::NEXA_TELEMETRY_JS.as_bytes())),
        _ => None,
    }
}

fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_known_modules() {
        assert!(find("ui").is_some());
        assert!(find("forms").is_some());
        assert!(find("tauri").is_some());
    }

    #[test]
    fn tauri_is_registered_as_experimental() {
        assert_eq!(find("tauri").unwrap().stability, Stability::Experimental);
    }

    #[test]
    fn platform_is_registered_and_has_a_content_hash() {
        assert!(find("platform").is_some());
        assert!(content_hash("platform").is_some());
    }

    #[test]
    fn does_not_find_unknown_or_internal_modules_as_addable() {
        assert!(find("router").is_none());
        assert!(find("runtime").is_none());
        assert!(find("dev-client").is_none());
        assert!(find("no-existe").is_none());
    }

    #[test]
    fn finds_internal_modules_separately_for_a_better_error_message() {
        assert!(find_internal("router").is_some());
        assert!(find_internal("runtime").is_some());
        assert!(find_internal("dev-client").is_some());
        assert!(find_internal("no-existe").is_none());
        assert!(find_internal("ui").is_none(), "ui es addable, no interno");
    }

    #[test]
    fn content_hash_is_deterministic_and_differs_between_modules() {
        let ui_hash = content_hash("ui").unwrap();
        let forms_hash = content_hash("forms").unwrap();

        assert_eq!(ui_hash, content_hash("ui").unwrap());
        assert_ne!(ui_hash, forms_hash);
    }

    #[test]
    fn content_hash_is_none_for_unknown_modules() {
        assert_eq!(content_hash("no-existe"), None);
    }
}
