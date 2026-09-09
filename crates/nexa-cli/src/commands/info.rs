use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Nexa CLI");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Fase actual: 56 — Table/Badge/Avatar/Breadcrumbs/Alert/Divider en @nexa/ui");
    println!("(Fase 47 fue un documento de diseño, sin cambios de código — ver docs/DISENO-COOKIES-EN-LOAD.md)");
    println!("(Fase 57 documenta el patrón 'función de @nexa/ui, no isla' para componentes con estado — Tabs/Accordion/Select/sort de tabla, sin código todavía)");
    println!("Ver docs/FASES-DE-CONSTRUCCION.md para el roadmap completo.");
    Ok(())
}
