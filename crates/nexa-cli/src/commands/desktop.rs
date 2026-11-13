//! `nexa build --target desktop` (Fase 63): del proyecto Nexa a un
//! binario de escritorio nativo en un solo paso.
//!
//! 1. `nexa build` con el shell del Modo App apuntando al backend real
//!    (`[desktop.proxy]`, ver `app_mode::shell_html`).
//! 2. `cargo build --release` sobre `src-tauri/` — Tauri embebe `dist/`
//!    dentro del ejecutable (por eso el resultado es un único archivo
//!    que abre una ventana con el webview del sistema operativo).
//!
//! Esto produce el **binario**. Los instaladores (`.deb`/`.msi`/`.dmg`),
//! la firma de código y el actualizador son un paso aparte que hace el
//! CLI de Tauri (`cargo tauri build`) sobre el mismo `src-tauri/`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn build() -> Result<()> {
    let cargo_toml = Path::new("src-tauri/Cargo.toml");
    if !cargo_toml.is_file() {
        bail!(
            "no existe src-tauri/ — corre `nexa add tauri` primero (genera el adaptador de escritorio \
             a partir de la plantilla embebida en el binario de `nexa`)"
        );
    }
    let package = package_name(cargo_toml)?;

    println!("[1/2] nexa build (shell de escritorio: la API apunta al backend real)");
    super::build::run_for(true)?;

    println!();
    println!("[2/2] cargo build --release (src-tauri/) — la primera vez tarda varios minutos");
    let status = Command::new("cargo")
        // `custom-protocol`: embeber `dist/` en el binario. Sin ella Tauri
        // carga `devUrl` (el servidor de `nexa dev`) y la ventana queda
        // en "Could not connect to localhost".
        .args(["build", "--release", "--features", "custom-protocol", "--manifest-path"])
        .arg(cargo_toml)
        .status()
        .context("ejecutando `cargo` — ¿está instalado Rust?")?;
    if !status.success() {
        bail!("`cargo build` falló sobre src-tauri/ (ver la salida de arriba)");
    }

    let binary = release_binary(&package);
    match std::fs::metadata(&binary) {
        Ok(meta) => {
            println!();
            println!("Binario de escritorio: {} ({:.1} MB)", binary.display(), meta.len() as f64 / 1_048_576.0);
            println!("Instaladores y firma: `cargo tauri build` (CLI de Tauri) sobre el mismo src-tauri/.");
        }
        Err(_) => println!("Compilado; no se encontró {} (¿otro target dir?)", binary.display()),
    }
    Ok(())
}

/// El `name` de `[package]` de `src-tauri/Cargo.toml` (es el nombre del binario).
pub fn package_name(cargo_toml: &Path) -> Result<String> {
    let text = std::fs::read_to_string(cargo_toml).with_context(|| format!("leyendo {}", cargo_toml.display()))?;
    let value: toml::Value = toml::from_str(&text).with_context(|| format!("{} no es un Cargo.toml válido", cargo_toml.display()))?;
    value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(str::to_string)
        .context("src-tauri/Cargo.toml no declara [package] name")
}

fn release_binary(package: &str) -> PathBuf {
    let exe = if cfg!(windows) { format!("{package}.exe") } else { package.to_string() };
    Path::new("src-tauri/target/release").join(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_package_name_from_cargo_toml() {
        let dir = std::env::temp_dir().join(format!("nexa-desktop-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("Cargo.toml");
        std::fs::write(&file, "[package]\nname = \"mi-app\"\nversion = \"0.1.0\"\n").unwrap();
        assert_eq!(package_name(&file).unwrap(), "mi-app");
    }
}
