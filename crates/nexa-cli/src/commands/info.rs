use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Nexa CLI");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Fase actual: 57 — Tabs/Accordion/Dropdown/orden-filtro de Table/buscador de página reactivo en @nexa/ui, sin islas");
    println!("(Fase 47 fue un documento de diseño, sin cambios de código — ver docs/DISENO-COOKIES-EN-LOAD.md)");
    println!("(Fase 56: Table/Badge/Avatar/Breadcrumbs/Alert/Divider/Tooltip/Progress/Skeleton, puramente visuales)");
    println!("Ver docs/FASES-DE-CONSTRUCCION.md para el roadmap completo.");
    Ok(())
}
