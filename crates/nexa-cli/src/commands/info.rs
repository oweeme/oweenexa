use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Nexa CLI");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Fase actual: 31 — islas dentro de src/layout.tsx");
    println!("Ver docs/FASES-DE-CONSTRUCCION.md para el roadmap completo.");
    Ok(())
}
