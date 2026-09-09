use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Nexa CLI");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Fase actual: 49 — platform.lifecycle (segundo plugin incremental de @nexa/platform)");
    println!("(Fase 47 fue un documento de diseño, sin cambios de código — ver docs/DISENO-COOKIES-EN-LOAD.md)");
    println!("Ver docs/FASES-DE-CONSTRUCCION.md para el roadmap completo.");
    Ok(())
}
