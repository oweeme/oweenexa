mod assets;
mod bootstrap;
mod capacitor_scaffold;
mod commands;
mod copy_dir;
mod devtools;
mod dev_watch;
mod document;
mod image_pipeline;
mod image_scan;
mod import_map;
mod lockfile;
mod manifest;
mod modules;
mod nginx_scaffold;
mod page_resolver;
mod performance_budget;
mod pipeline;
mod pkg_warnings;
mod pwa;
mod serve;
mod tauri_scaffold;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nexa", version, about = "Nexa CLI — HTML-first, backend-agnostic")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Muestra información del toolchain de Nexa.
    Info,
    /// Crea un nuevo proyecto Nexa.
    Create {
        name: String,
    },
    /// Compila todas las páginas de `src/pages/` a `dist/`.
    Build,
    /// Sirve `dist/`; lo que falte (rutas dinámicas) se renderiza al vuelo.
    Preview {
        #[arg(long, default_value_t = 4321)]
        port: u16,
    },
    /// Servidor de desarrollo: recompila cada página en cada petición y
    /// recarga el navegador solo cuando algo bajo `src/` cambió.
    Dev {
        #[arg(long, default_value_t = 4321)]
        port: u16,
    },
    /// Declara un módulo oficial de Nexa (ej. `ui`, `forms`) en
    /// `nexa.toml` y `nexa.lock` — sin tocar `package.json` ni npm.
    Add {
        module: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Info) {
        Command::Info => commands::info::run(),
        Command::Create { name } => commands::create::run(&name),
        Command::Build => commands::build::run(),
        Command::Preview { port } => commands::preview::run(port),
        Command::Dev { port } => commands::dev::run(port),
        Command::Add { module } => commands::add::run(&module),
    }
}
