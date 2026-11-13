//! `nexa create <nombre> [--template core|app]`.
//!
//! Deja un proyecto que compila de inmediato y que el editor entiende:
//! `tsconfig.json` + `nexa-env.d.ts` (tipos solo para autocompletado; el
//! compilador de Nexa no los lee).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

const ENV_DTS: &str = include_str!("../../assets/nexa-env.d.ts");

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "jsx": "preserve",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src", "nexa-env.d.ts"]
}
"#;

const CORE_HOME: &str = "export default function Home() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>Welcome to Nexa.</p>
        </main>
    );
}
";

/// La portada del template `app`: HTML real (SEO) con un enlace a la SPA.
const APP_HOME: &str = "export const seo = { title: \"Mi app\", description: \"Landing con SEO; la aplicación vive en /app\", canonical: \"https://example.com/\" };

export default function Home() {
    return (
        <main>
            <h1>Mi app</h1>
            <p>Esta portada es HTML real, indexable. La aplicación está en la SPA:</p>
            <a href=\"/app\" data-nexa-reload>Abrir la aplicación</a>
        </main>
    );
}
";

const APP_PAGE: &str = "import { state, computed } from \"@nexa/app\";

export default function Counter() {
    const count = state(0);
    const double = computed(() => count.value * 2);
    return (
        <main>
            <h1>Contador</h1>
            <p>{count} × 2 = {double}</p>
            <button onClick={() => count.value++}>Sumar</button>
        </main>
    );
}
";

pub fn run(name: &str, template: &str) -> Result<()> {
    if template != "core" && template != "app" {
        bail!("template desconocido `{template}` — usa `core` (por defecto) o `app` (landing con SEO + SPA)");
    }
    let root = PathBuf::from(name);
    if root.exists() {
        bail!("`{}` ya existe", root.display());
    }

    fs::create_dir_all(root.join("src/pages")).with_context(|| format!("creando {}", root.join("src/pages").display()))?;
    fs::create_dir_all(root.join("public"))?;

    let mut manifest = format!("[project]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
    if template == "app" {
        manifest.push_str("\n[app]\ntitle = \"Mi app\"\n\n# Un backend propio: `nexa dev`/`preview` reenvían /api/* ahí (mismo origen, sin CORS).\n# [proxy]\n# \"/api\" = \"http://127.0.0.1:9100\"\n");
        fs::create_dir_all(root.join("src/app/pages"))?;
        write(&root.join("src/app/pages/index.tsx"), APP_PAGE)?;
    }
    write(&root.join("nexa.toml"), &manifest)?;
    write(&root.join("src/pages/index.tsx"), if template == "app" { APP_HOME } else { CORE_HOME })?;
    write(&root.join("tsconfig.json"), TSCONFIG)?;
    write(&root.join("nexa-env.d.ts"), ENV_DTS)?;
    write(&root.join(".gitignore"), "dist/\nnode_modules/\n.nexa/\n.env.local\n")?;

    println!("Creado proyecto Nexa ({template}) en {}", root.display());
    println!();
    println!("  cd {name}");
    println!("  nexa dev          # http://127.0.0.1:4321");
    println!("  nexa build        # dist/");
    if template == "app" {
        println!("  nexa bundle       # un único ejecutable con su servidor");
    }
    Ok(())
}

fn write(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("escribiendo {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nexa-create-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_core_template_has_editor_support_and_no_dead_config_file() {
        let dir = scratch("core");
        let project = dir.join("hola");
        run(project.to_str().unwrap(), "core").unwrap();
        for f in ["nexa.toml", "tsconfig.json", "nexa-env.d.ts", "src/pages/index.tsx", ".gitignore"] {
            assert!(project.join(f).is_file(), "falta {f}");
        }
        assert!(!project.join("nexa.config.ts").exists(), "nexa.config.ts no lo lee nadie");
        assert!(!project.join("src/app").exists());
    }

    #[test]
    fn the_app_template_adds_a_spa_and_a_landing_that_links_to_it() {
        let dir = scratch("app");
        let project = dir.join("mi-app");
        run(project.to_str().unwrap(), "app").unwrap();
        assert!(project.join("src/app/pages/index.tsx").is_file());
        let toml = fs::read_to_string(project.join("nexa.toml")).unwrap();
        assert!(toml.contains("[app]") && toml.contains("# [proxy]"), "{toml}");
        let home = fs::read_to_string(project.join("src/pages/index.tsx")).unwrap();
        assert!(home.contains("data-nexa-reload"), "el enlace a la SPA debe ser una carga completa");
    }

    #[test]
    fn unknown_templates_and_existing_directories_are_rejected() {
        let dir = scratch("err");
        assert!(run(dir.join("x").to_str().unwrap(), "vue").is_err());
        assert!(run(dir.to_str().unwrap(), "core").is_err(), "el directorio ya existe");
    }
}
