use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Nexa CLI");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Fase actual: 33 — link activo automático (aria-current)");
    println!("Ver docs/FASES-DE-CONSTRUCCION.md para el roadmap completo.");
    Ok(())
}
