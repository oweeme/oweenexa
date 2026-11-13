use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Nexa CLI");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Documentación: https://oweeme.github.io/oweenexa/");
    println!("Código:        https://github.com/oweeme/oweenexa");
    println!();
    println!("Comandos: create · dev · build · preview · bundle · lint · test · add");
    println!("Primer proyecto: nexa create mi-sitio && cd mi-sitio && nexa dev");
    Ok(())
}
