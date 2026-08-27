//! `nexa add tauri` (Fase 13): genera `src-tauri/` a partir de una
//! plantilla embebida en el propio binario — la misma disciplina que
//! `@nexa/runtime`/`@nexa/router`/`@nexa/forms` (Fases 5-10): el
//! proyecto Nexa no necesita tener este repositorio disponible para
//! generar un adaptador de escritorio real.
//!
//! La plantilla (`assets/tauri/`) no se inventó a mano: sale de un
//! `tauri init --ci` real (Tauri CLI 2.11.4) apuntado a `../dist` — la
//! carpeta que ya produce `nexa build` — con `tauri-plugin-log` quitado
//! (no aporta nada a un adaptador mínimo) y los íconos reducidos a un
//! único PNG placeholder (Tauri exige al menos uno para compilar; los
//! demás tamaños solo hacen falta para empaquetar instaladores reales).
//! Se verificó que compila de verdad con `cargo build` antes de
//! incrustarla.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

const CARGO_TOML_TMPL: &str = include_str!("../assets/tauri/Cargo.toml.tmpl");
const TAURI_CONF_TMPL: &str = include_str!("../assets/tauri/tauri.conf.json.tmpl");
const GITIGNORE_TMPL: &str = include_str!("../assets/tauri/gitignore.tmpl");
const MAIN_RS_TMPL: &str = include_str!("../assets/tauri/src/main.rs");
const LIB_RS: &str = include_str!("../assets/tauri/src/lib.rs");
const BUILD_RS: &str = include_str!("../assets/tauri/build.rs");
const CAPABILITIES_DEFAULT_JSON: &str = include_str!("../assets/tauri/capabilities/default.json");
const ICON_PNG: &[u8] = include_bytes!("../assets/tauri/icons/icon.png");

/// Usado por `modules::content_hash` (Fase 12): un hash de todo el
/// contenido de la plantilla, para que `nexa.lock` pueda notar si
/// cambió entre versiones del binario.
pub fn template_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    for text in [CARGO_TOML_TMPL, TAURI_CONF_TMPL, GITIGNORE_TMPL, MAIN_RS_TMPL, LIB_RS, BUILD_RS, CAPABILITIES_DEFAULT_JSON]
    {
        bytes.extend_from_slice(text.as_bytes());
    }
    bytes.extend_from_slice(ICON_PNG);
    bytes
}

/// Genera `<project_root>/src-tauri/`. Falla si ya existe — igual que
/// `nexa create` con un directorio existente, no se sobrescribe trabajo
/// del usuario en silencio.
pub fn scaffold(project_root: &Path, project_name: &str) -> Result<()> {
    let src_tauri = project_root.join("src-tauri");
    if src_tauri.exists() {
        bail!("{} ya existe — bórralo primero si quieres regenerarlo", src_tauri.display());
    }

    let name = sanitize_package_name(project_name);
    let identifier = format!("com.nexa.{name}");

    fs::create_dir_all(src_tauri.join("src"))?;
    fs::create_dir_all(src_tauri.join("capabilities"))?;
    fs::create_dir_all(src_tauri.join("icons"))?;

    write(&src_tauri.join("Cargo.toml"), &substitute(CARGO_TOML_TMPL, &name, &identifier))?;
    write(&src_tauri.join("tauri.conf.json"), &substitute(TAURI_CONF_TMPL, &name, &identifier))?;
    write(&src_tauri.join(".gitignore"), GITIGNORE_TMPL)?;
    write(&src_tauri.join("build.rs"), BUILD_RS)?;
    write(&src_tauri.join("src/main.rs"), &substitute(MAIN_RS_TMPL, &name, &identifier))?;
    write(&src_tauri.join("src/lib.rs"), LIB_RS)?;
    write(&src_tauri.join("capabilities/default.json"), CAPABILITIES_DEFAULT_JSON)?;
    fs::write(src_tauri.join("icons/icon.png"), ICON_PNG)
        .with_context(|| format!("escribiendo {}", src_tauri.join("icons/icon.png").display()))?;

    Ok(())
}

fn write(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("escribiendo {}", path.display()))
}

fn substitute(template: &str, name: &str, identifier: &str) -> String {
    template
        .replace("{{NAME_UNDERSCORE}}", &name.replace('-', "_"))
        .replace("{{NAME}}", name)
        .replace("{{IDENTIFIER}}", identifier)
}

/// Un nombre de paquete de Cargo válido: minúsculas, dígitos y `-`/`_`
/// solamente, empezando por una letra — el nombre del proyecto
/// (`nexa.toml`) puede traer cualquier otra cosa.
fn sanitize_package_name(name: &str) -> String {
    let mut out: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();

    if out.is_empty() || !out.chars().next().unwrap().is_ascii_alphabetic() {
        out = format!("app-{out}");
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
        let dir = std::env::temp_dir().join(format!("nexa-tauri-scaffold-test-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn sanitizes_names_with_spaces_and_uppercase() {
        assert_eq!(sanitize_package_name("My Cool App"), "my-cool-app");
    }

    #[test]
    fn prefixes_names_that_do_not_start_with_a_letter() {
        assert_eq!(sanitize_package_name("123app"), "app-123app");
    }

    #[test]
    fn scaffold_writes_every_expected_file() {
        let dir = scratch_dir();
        scaffold(&dir, "hello").unwrap();

        for file in [
            "src-tauri/Cargo.toml",
            "src-tauri/tauri.conf.json",
            "src-tauri/.gitignore",
            "src-tauri/build.rs",
            "src-tauri/src/main.rs",
            "src-tauri/src/lib.rs",
            "src-tauri/capabilities/default.json",
            "src-tauri/icons/icon.png",
        ] {
            assert!(dir.join(file).exists(), "falta {file}");
        }
    }

    #[test]
    fn substitutes_the_project_name_into_cargo_toml_and_the_identifier() {
        let dir = scratch_dir();
        scaffold(&dir, "My App").unwrap();

        let cargo_toml = fs::read_to_string(dir.join("src-tauri/Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("name = \"my-app\""));
        assert!(cargo_toml.contains("my_app_lib"));

        let conf = fs::read_to_string(dir.join("src-tauri/tauri.conf.json")).unwrap();
        assert!(conf.contains("\"identifier\": \"com.nexa.my-app\""));
        assert!(conf.contains("\"productName\": \"my-app\""));

        let main_rs = fs::read_to_string(dir.join("src-tauri/src/main.rs")).unwrap();
        assert!(main_rs.contains("my_app_lib::run();"));
    }

    #[test]
    fn refuses_to_overwrite_an_existing_src_tauri() {
        let dir = scratch_dir();
        scaffold(&dir, "hello").unwrap();

        assert!(scaffold(&dir, "hello").is_err());
    }

    #[test]
    fn no_placeholder_survives_substitution() {
        let dir = scratch_dir();
        scaffold(&dir, "hello").unwrap();

        for file in ["src-tauri/Cargo.toml", "src-tauri/tauri.conf.json", "src-tauri/src/main.rs"] {
            let contents = fs::read_to_string(dir.join(file)).unwrap();
            assert!(!contents.contains("{{"), "quedó un placeholder sin sustituir en {file}");
        }
    }
}
