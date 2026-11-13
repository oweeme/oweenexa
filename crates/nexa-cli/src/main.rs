mod assets;
mod bootstrap;
mod bundle;
mod components;
mod capacitor_scaffold;
mod commands;
mod copy_dir;
mod devtools;
mod dev_watch;
mod document;
mod image_pipeline;
mod image_scan;
mod app_mode;
mod import_map;
mod proxy;
mod layout;
mod lockfile;
mod manifest;
mod modules;
mod nginx_scaffold;
mod page_resolver;
mod performance_budget;
mod pipeline;
mod prod_server;
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
    /// Compila todas las páginas de `src/pages/` a `dist/`. Con
    /// `--target desktop` además compila el binario de escritorio
    /// (Tauri) a partir de `src-tauri/`.
    Build {
        /// `desktop`: genera `dist/` para escritorio y compila `src-tauri/`.
        #[arg(long)]
        target: Option<String>,
    },
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
    /// Compila el sitio y lo empaqueta como UN ejecutable con su propio
    /// servidor de producción (compresión, caché, SPA, proxy al backend):
    /// sin Node, sin nginx, sin archivos sueltos.
    Bundle {
        /// Nombre del ejecutable (por defecto `<proyecto>-server`).
        #[arg(long)]
        out: Option<String>,
        /// Binario de `nexa` que se copia como base — para empaquetar para
        /// otro sistema operativo (ej. el `nexa.exe` de Windows).
        #[arg(long)]
        runtime: Option<String>,
        /// No recompilar: usar el `dist/` que ya existe.
        #[arg(long)]
        no_build: bool,
    },
}

fn main() -> Result<()> {
    // Fase 64: si este ejecutable lleva un sitio pegado (`nexa bundle`), no
    // es la CLI de `nexa` sino el servidor de ese sitio.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(payload) = bundle::read_embedded(&exe)? {
            return prod_server::run_embedded(payload, std::env::args().skip(1).collect());
        }
    }

    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Info) {
        Command::Info => commands::info::run(),
        Command::Create { name } => commands::create::run(&name),
        Command::Build { target } => match target.as_deref() {
            None | Some("web") => commands::build::run(),
            Some("desktop") => commands::desktop::build(),
            Some("android") | Some("ios") => anyhow::bail!(
                "`--target {}` todavía no está soportado — para móvil usa `nexa add capacitor` (ver la guía de empaquetado)",
                target.as_deref().unwrap_or_default()
            ),
            Some(other) => anyhow::bail!("target desconocido `{other}` — usa `web` (por defecto) o `desktop`"),
        },
        Command::Preview { port } => commands::preview::run(port),
        Command::Dev { port } => commands::dev::run(port),
        Command::Add { module } => commands::add::run(&module),
        Command::Lint => commands::lint::run(),
        Command::Test => commands::test::run(),
        Command::Bundle { out, runtime, no_build } => commands::bundle::run(out, runtime, no_build),
    }
}
