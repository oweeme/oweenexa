//! `nexa check` (Fase 80): comprueba los tipos de todo el proyecto con TypeScript (`tsc --noEmit`).
//!
//! Nexa no ejecuta TypeScript (compila TSX a HTML y a JS sin comprobar tipos), así que un error de
//! tipos no rompe el build. Esto es el equivalente a `tsc` de cualquier proyecto: usa el
//! `tsconfig.json` y el `nexa-env.d.ts` que dejó `nexa create`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

fn find_tsc(root: &Path) -> Option<PathBuf> {
    let bin = if cfg!(windows) { "tsc.cmd" } else { "tsc" };
    let local = root.join("node_modules/.bin").join(bin);
    if local.is_file() {
        return Some(local);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|d| d.join(bin)).find(|p| p.is_file())
}

pub fn run() -> Result<()> {
    if !Path::new("tsconfig.json").is_file() {
        bail!("no hay tsconfig.json — ¿estás en un proyecto Nexa? (`nexa create` lo genera)");
    }
    let Some(tsc) = find_tsc(Path::new(".")) else {
        bail!("no encuentro TypeScript. Instálalo en el proyecto: `npm install -D typescript`");
    };
    println!("Comprobando tipos ({} --noEmit)…", tsc.file_name().and_then(|n| n.to_str()).unwrap_or("tsc"));
    let status = Command::new(&tsc).args(["--noEmit", "-p", "tsconfig.json"]).status().with_context(|| format!("no se pudo ejecutar {}", tsc.display()))?;
    if !status.success() {
        bail!("hay errores de tipos (arriba)");
    }
    println!("Sin errores de tipos.");
    Ok(())
}
