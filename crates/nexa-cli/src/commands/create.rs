use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

pub fn run(name: &str) -> Result<()> {
    let root = PathBuf::from(name);
    if root.exists() {
        bail!("`{}` ya existe", root.display());
    }

    fs::create_dir_all(root.join("src/pages"))
        .with_context(|| format!("creando {}", root.join("src/pages").display()))?;
    fs::create_dir_all(root.join("public"))?;

    fs::write(
        root.join("nexa.toml"),
        format!("[project]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[dependencies]\n"),
    )?;

    fs::write(
        root.join("nexa.config.ts"),
        "export default {\n  app: {\n    name: \"My Nexa App\",\n  },\n};\n",
    )?;

    fs::write(
        root.join("src/pages/index.tsx"),
        "export default function Home() {\n    return (\n        <main>\n            <h1>Hello Nexa</h1>\n            <p>Welcome to Nexa.</p>\n        </main>\n    );\n}\n",
    )?;

    fs::write(root.join(".gitignore"), "dist/\nnode_modules/\n")?;

    println!("Creado proyecto Nexa en {}", root.display());
    println!();
    println!("  cd {name}");
    println!("  nexa build");

    Ok(())
}
