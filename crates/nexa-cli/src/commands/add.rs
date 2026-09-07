//! `nexa add <módulo>` (Fase 12): declara un módulo oficial en
//! `nexa.toml`/`nexa.lock`, sin tocar `package.json` ni depender de npm
//! para nada de esto — es el criterio de salida de esta fase.

use std::path::Path;

use anyhow::{bail, Result};

use crate::lockfile::{self, LockedPackage};
use crate::manifest;
use crate::modules::{self, Stability};

const MANIFEST_PATH: &str = "nexa.toml";
const LOCKFILE_PATH: &str = "nexa.lock";

pub fn run(name: &str) -> Result<()> {
    let manifest_path = Path::new(MANIFEST_PATH);
    let mut project = manifest::load_strict(manifest_path)?;

    let Some(module) = modules::find(name) else {
        if let Some(internal) = modules::find_internal(name) {
            bail!(
                "`{name}` es un módulo interno ({}) — {} No se declara con `nexa add`.",
                internal.stability.as_str(),
                internal.description
            );
        }
        let known: Vec<&str> = modules::ADDABLE.iter().map(|m| m.name).collect();
        bail!("`{name}` no es un módulo oficial de Nexa. Disponibles: {}", known.join(", "));
    };

    if project.dependencies.get(name).map(String::as_str) == Some(module.version) {
        println!("`{name}` ya está declarado en nexa.toml (v{}) — nada que hacer.", module.version);
        return Ok(());
    }

    // Los módulos con andamiaje (`tauri`, `capacitor`) generan archivos
    // en el proyecto — se hace antes de tocar nexa.toml/nexa.lock: si
    // falla (ej. ya existen), no queda una declaración a medias.
    if name == "tauri" {
        crate::tauri_scaffold::scaffold(Path::new("."), &project.project.name)?;
        println!("Generado src-tauri/ (adaptador de escritorio Tauri).");
    }
    if name == "capacitor" {
        crate::capacitor_scaffold::scaffold(Path::new("."), &project.project.name)?;
        println!("Generado capacitor.config.json.");
    }
    // `pwa` (Fase 18) no genera archivos aparte — deja un `[pwa]` real,
    // ya pre-llenado con el nombre del proyecto, directo en nexa.toml
    // (vía el propio `Manifest`, no texto suelto) para que el usuario
    // solo tenga que completar el ícono y, si quiere, `[pwa.cache]`.
    if name == "pwa" && project.pwa.is_none() {
        project.pwa = Some(manifest::PwaSection {
            name: project.project.name.clone(),
            display: Some("standalone".to_string()),
            ..Default::default()
        });
    }

    project.dependencies.insert(name.to_string(), module.version.to_string());
    manifest::write(manifest_path, &project)?;

    let lock_path = Path::new(LOCKFILE_PATH);
    let mut lock = lockfile::load(lock_path);
    lock.upsert(LockedPackage {
        name: name.to_string(),
        version: module.version.to_string(),
        stability: module.stability.as_str().to_string(),
        source: "builtin".to_string(),
        content_hash: modules::content_hash(name).unwrap_or_default(),
    });
    lockfile::write(lock_path, &lock)?;

    println!("Instalado @nexa/{name} v{} ({}).", module.version, module.stability.as_str());
    println!("  {}", module.description);
    println!("Registrado en nexa.toml y nexa.lock — sin package.json, sin npm.");

    if module.stability == Stability::Experimental {
        println!(
            "Aviso: `{name}` es Experimental — su API puede romper compatibilidad en cualquier \
             versión menor. Ver docs/POLITICA-DE-VERSIONES.md."
        );
    }

    if name == "tauri" {
        println!();
        println!("Siguientes pasos:");
        println!("  nexa build");
        println!("  cd src-tauri && cargo build     # compila el binario de escritorio");
        println!(
            "  (o instala @tauri-apps/cli para `cargo tauri dev`/`cargo tauri build`, con \
             recarga en caliente e instaladores empaquetados)"
        );
    }

    if name == "capacitor" {
        println!();
        println!("Siguientes pasos (necesitan Node y, según la plataforma, Android Studio o Xcode —");
        println!("no se generan ni se verifican aquí, esto solo deja el archivo de configuración):");
        println!("  nexa build");
        println!("  npx @capacitor/cli add android   # o: add ios (requiere Xcode, solo en macOS)");
        println!("  npx @capacitor/cli sync");
    }

    if name == "pwa" {
        println!();
        println!("Se agregó [pwa] a nexa.toml con el nombre del proyecto — completá lo que falte:");
        println!("  icon = \"/icon-512.png\"   # un PNG cuadrado real en public/, idealmente 512x512");
        println!("  themeColor, backgroundColor, shortName (opcionales)");
        println!("  [pwa.cache]                # opcional — sin esto, todo cae a network-first");
        println!("  \"/assets\" = \"cache-first\"");
        println!("`nexa build` genera dist/manifest.webmanifest y dist/sw.js automáticamente.");
    }

    Ok(())
}
