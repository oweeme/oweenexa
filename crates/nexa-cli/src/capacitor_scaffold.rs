//! `nexa add capacitor` (Fase 13): genera `capacitor.config.json` en la
//! raíz del proyecto — la forma real y mínima que produce `npx
//! @capacitor/cli init <nombre> <id> --web-dir dist` (verificado
//! corriendo ese comando de verdad antes de escribir esta plantilla).
//!
//! A diferencia de `tauri_scaffold`, aquí no hay nada que compilar en
//! este entorno: un proyecto Capacitor real necesita `android/`/`ios/`
//! (generados por `npx cap add android/ios`, que a su vez necesitan el
//! SDK de Android o Xcode) — nada de eso existe ni se puede verificar en
//! este sandbox (no hay Xcode en Linux, y no hay un SDK de Android
//! instalado aquí). Por eso este módulo se limita a lo único que sí se
//! puede generar y verificar sin esas herramientas: el archivo de
//! configuración, más las instrucciones exactas de los pasos que le
//! siguen.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

const TEMPLATE_SHAPE: &str = "{\n  \"appId\": \"com.nexa.{{APP_ID}}\",\n  \"appName\": \"{{APP_NAME}}\",\n  \"webDir\": \"dist\"\n}\n";

/// Usado por `modules::content_hash`: no hay un archivo fijo que
/// embeber (el contenido depende del nombre del proyecto), así que se
/// hashea la *forma* de la plantilla en vez de una instancia concreta.
pub fn template_bytes() -> &'static [u8] {
    TEMPLATE_SHAPE.as_bytes()
}

pub fn scaffold(project_root: &Path, app_name: &str) -> Result<()> {
    let path = project_root.join("capacitor.config.json");
    if path.exists() {
        bail!("{} ya existe — bórralo primero si quieres regenerarlo", path.display());
    }

    fs::write(&path, render(app_name)).with_context(|| format!("escribiendo {}", path.display()))
}

fn render(app_name: &str) -> String {
    let id = format!("com.nexa.{}", sanitize_app_id(app_name));
    format!(
        "{{\n  \"appId\": \"{id}\",\n  \"appName\": \"{app_name}\",\n  \"webDir\": \"dist\"\n}}\n"
    )
}

/// Un `appId` de Capacitor es un identificador de paquete estilo Java:
/// cada segmento debe ser solo `[a-zA-Z][a-zA-Z0-9_]*` — a diferencia de
/// un nombre de paquete de Cargo (`tauri_scaffold::sanitize_package_name`),
/// **el guion no está permitido**. Verificado contra un error real de
/// `npx @capacitor/cli add android` ("Must be in Java package form with
/// no dashes") al probar esto con un proyecto de verdad.
fn sanitize_app_id(name: &str) -> String {
    let mut out: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    if out.is_empty() || !out.chars().next().unwrap().is_ascii_alphabetic() {
        out = format!("app_{out}");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-capacitor-scaffold-test-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scaffold_writes_a_valid_minimal_config() {
        let dir = scratch_dir();
        scaffold(&dir, "hello").unwrap();

        let contents = fs::read_to_string(dir.join("capacitor.config.json")).unwrap();

        assert!(contents.contains("\"appName\": \"hello\""));
        assert!(contents.contains("\"webDir\": \"dist\""));
        assert!(contents.contains("\"appId\": \"com.nexa.hello\""));
    }

    #[test]
    fn refuses_to_overwrite_an_existing_config() {
        let dir = scratch_dir();
        scaffold(&dir, "hello").unwrap();

        assert!(scaffold(&dir, "hello").is_err());
    }

    #[test]
    fn sanitizes_the_app_id_from_a_name_with_spaces() {
        let dir = scratch_dir();
        scaffold(&dir, "My Cool App").unwrap();

        let contents = fs::read_to_string(dir.join("capacitor.config.json")).unwrap();
        assert!(contents.contains("\"appId\": \"com.nexa.my_cool_app\""));
    }

    #[test]
    fn never_puts_a_dash_in_the_app_id_even_though_project_names_commonly_have_one() {
        let dir = scratch_dir();
        scaffold(&dir, "nexa-cap-verify").unwrap();

        let contents = fs::read_to_string(dir.join("capacitor.config.json")).unwrap();
        assert!(contents.contains("\"appId\": \"com.nexa.nexa_cap_verify\""));
        assert!(!contents.contains("appId\": \"com.nexa.nexa-cap-verify\""));
    }
}
