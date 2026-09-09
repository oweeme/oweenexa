use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Nexa CLI");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Fase actual: 34 — rutas dinámicas sin paths en producción, de verdad");
    println!("Ver docs/FASES-DE-CONSTRUCCION.md para el roadmap completo.");
    Ok(())
}
