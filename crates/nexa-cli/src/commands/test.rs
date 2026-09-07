//! `nexa test` (Fase 23): orquesta el test runner de JS de un proyecto
//! (típicamente `vitest`, usando `@nexa/test`) — no lo reemplaza, `nexa`
//! no tiene su propio corredor de tests de JS. Lo único que aporta es
//! asegurarse de que `dist/` esté fresco ANTES de correrlo: un chunk
//! generado por Nexa lleva un hash de su contenido en el nombre (Fase
//! 23), así que un test que resuelve su ruta leyendo el `module` de
//! `nexa-manifest.json` (en vez de hardcodear el nombre — ver
//! `docs/REFERENCIA.md`, sección `@nexa/test`) solo encuentra el archivo
//! correcto si el build corrió contra el código fuente actual.

use std::path::Path;
use std::process::Command as ProcessCommand;

use anyhow::{bail, Context, Result};

pub fn run() -> Result<()> {
    println!("Compilando el proyecto antes de correr los tests (dist/ fresco)...");
    crate::commands::build::run()
        .context("`nexa build` falló — no tiene sentido correr tests contra un dist/ inconsistente")?;

    let package_json = Path::new("package.json");
    if !package_json.is_file() {
        bail!(
            "no se encontró package.json — `nexa test` orquesta el test runner de JS del \
             proyecto (ej. vitest, usando @nexa/test), pero no lo reemplaza. Agrega un \
             package.json con un script \"test\" (ver docs/REFERENCIA.md, sección @nexa/test)."
        );
    }

    let content = std::fs::read_to_string(package_json).context("leyendo package.json")?;
    let parsed: serde_json::Value = serde_json::from_str(&content).context("parseando package.json")?;
    let has_test_script = parsed.get("scripts").and_then(|s| s.get("test")).is_some();
    if !has_test_script {
        bail!(
            "package.json no declara un script \"test\" — agrega uno (ej. `\"test\": \"vitest run\"`) \
             para que `nexa test` tenga qué correr."
        );
    }

    println!();
    println!("Corriendo `npm test`...");
    let status = ProcessCommand::new("npm").arg("test").status().context("ejecutando `npm test`")?;
    if !status.success() {
        bail!("`npm test` terminó con errores.");
    }

    Ok(())
}
