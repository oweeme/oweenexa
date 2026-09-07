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
mod layout;
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
    /// Compila cada página sin escribir `dist/` y falla si el SEO
    /// Analyzer o los avisos de paquetes encontraron algo — a diferencia
    /// de `nexa build`, que nunca falla por esto.
    Lint,
    /// Compila el proyecto (`dist/` fresco, con los nombres de archivo
    /// con hash de la Fase 23) y corre el test runner de JS declarado en
    /// `package.json` (`npm test`) — no reemplaza a `vitest`/etc., solo
    /// se asegura de que corra contra un build consistente.
    Test,
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
        Command::Lint => commands::lint::run(),
        Command::Test => commands::test::run(),
    }
}
