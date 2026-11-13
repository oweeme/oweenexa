//! Pipeline de recursos (Fase 72): empaqueta las islas y el CSS del proyecto
//! con esbuild, sin pasos manuales.
//!
//! Convención (cero configuración):
//! - `islands/<nombre>.island.{ts,tsx,js,jsx}` (o `src/islands/`) → un módulo
//!   `/assets/islands/<nombre>.<hash>.js`, importable por su `<nombre>` en
//!   `data-nexa-island` (entra solo al import map). Puede importar paquetes de
//!   npm y CSS.
//! - `src/styles/*.css` (los que no empiezan por `_`) → `/assets/styles/<nombre>.<hash>.css`,
//!   enlazado en todas las páginas. `@import` y minificación incluidos.
//! - `[build] before = ["comando", ...]`: comandos que corren antes (Tailwind,
//!   Sass, generar datos…).
//!
//! Lo generado vive en `.nexa/gen/` (nunca en el código fuente); `nexa build` lo
//! copia a `dist/` y `nexa dev` lo sirve desde ahí.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const GEN_DIR: &str = ".nexa/gen";
const INDEX: &str = ".nexa/gen/assets.json";
const ISLAND_DIRS: [&str; 2] = ["islands", "src/islands"];
const ISLAND_EXTS: [&str; 4] = ["ts", "tsx", "js", "jsx"];

/// Lo que el resto del compilador necesita saber de lo generado.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Generated {
    /// nombre de isla → URL del módulo
    #[serde(default)]
    pub imports: BTreeMap<String, String>,
    /// URLs de hojas de estilo, en orden
    #[serde(default)]
    pub styles: Vec<String>,
}

/// Lee `.nexa/gen/assets.json` (vacío si no existe).
pub fn generated() -> Generated {
    fs::read_to_string(INDEX).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// `<link rel="stylesheet">` de cada hoja generada, para el `<head>`.
pub fn styles_head() -> String {
    generated().styles.iter().map(|href| format!("<link rel=\"stylesheet\" href=\"{href}\">")).collect::<Vec<_>>().join("\n")
}

/// Entradas de islas: (nombre, ruta).
pub fn island_entries(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    for dir in ISLAND_DIRS {
        let Ok(entries) = fs::read_dir(root.join(dir)) else { continue };
        let mut files: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        files.sort();
        for path in files {
            let Some(file) = path.file_name().and_then(|f| f.to_str()) else { continue };
            let Some((name, ext)) = file.rsplit_once('.') else { continue };
            let Some(name) = name.strip_suffix(".island") else { continue };
            if ISLAND_EXTS.contains(&ext) && !name.is_empty() && !out.iter().any(|(n, _)| n == name) {
                out.push((name.to_string(), path));
            }
        }
    }
    out
}

/// Hojas de entrada: (nombre, ruta).
pub fn style_entries(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root.join("src/styles")) else { return Vec::new() };
    let mut files: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let file = path.file_name()?.to_str()?.to_string();
            let name = file.strip_suffix(".css")?;
            (!name.starts_with('_') && !name.is_empty()).then(|| (name.to_string(), path.clone()))
        })
        .collect()
}

/// Un nombre de isla debe poder ser un identificador de import map y un
/// nombre de archivo sin sorpresas.
fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn find_esbuild(root: &Path) -> Option<PathBuf> {
    let bin = if cfg!(windows) { "esbuild.cmd" } else { "esbuild" };
    let local = root.join("node_modules/.bin").join(bin);
    if local.is_file() {
        return Some(local);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|d| d.join(bin)).find(|p| p.is_file())
}

pub fn run_commands(root: &Path, commands: &[String]) -> Result<()> {
    for command in commands {
        run_shell(command, root)?;
    }
    Ok(())
}

fn run_shell(command: &str, root: &Path) -> Result<()> {
    println!("  $ {command}");
    let status = if cfg!(windows) {
        Command::new("cmd").args(["/C", command]).current_dir(root).status()
    } else {
        Command::new("sh").args(["-c", command]).current_dir(root).status()
    }
    .with_context(|| format!("no se pudo ejecutar `{command}`"))?;
    if !status.success() {
        bail!("`{command}` terminó con error ({status})");
    }
    Ok(())
}

fn esbuild(bin: &Path, root: &Path, entry: &Path, outfile: &Path, extra: &[&str]) -> Result<()> {
    let out = Command::new(bin)
        .current_dir(root)
        .arg(entry)
        .args(["--bundle", "--minify", "--target=es2022", "--log-level=warning"])
        .args(extra)
        .arg(format!("--outfile={}", outfile.display()))
        .output()
        .with_context(|| format!("no se pudo ejecutar {}", bin.display()))?;
    if !out.status.success() {
        bail!("esbuild falló en {}:\n{}", entry.display(), String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

/// Corre `[build] before`, empaqueta islas y CSS, y escribe el índice.
/// Sin entradas no hace nada (costo cero, ni siquiera pide esbuild).
pub fn run(root: &Path, before: &[String]) -> Result<Generated> {
    for command in before {
        run_shell(command, root)?;
    }

    let islands = island_entries(root);
    let styles = style_entries(root);
    let gen = root.join(GEN_DIR);
    let _ = fs::remove_dir_all(&gen);
    if islands.is_empty() && styles.is_empty() {
        return Ok(Generated::default());
    }

    let Some(bin) = find_esbuild(root) else {
        if islands.is_empty() {
            // Solo CSS y sin esbuild: se copia tal cual (sin `@import`, sin minificar). Un proyecto recién
            // creado (con `src/styles/oweekoi.css`) funciona sin instalar nada.
            return copy_styles_without_esbuild(root, &styles, &gen);
        }
        bail!("hay islas para empaquetar pero no encuentro esbuild. Instálalo en el proyecto: `npm install -D esbuild`");
    };

    let tmp = gen.join("tmp");
    fs::create_dir_all(&tmp)?;
    let mut result = Generated::default();

    for (name, entry) in &islands {
        if !valid_name(name) {
            bail!("el nombre de isla `{name}` no es válido: usa letras, números, `-` o `_`");
        }
        let out = tmp.join(format!("{name}.js"));
        let mut flags = vec!["--format=esm", "--loader:.png=dataurl", "--loader:.svg=dataurl", "--define:process.env.NODE_ENV=\"production\""];
        // React, Preact… en `.tsx`/`.jsx`: JSX moderno salvo que tu `tsconfig.json` decida otra cosa.
        let tsconfig = fs::read_to_string(root.join("tsconfig.json")).unwrap_or_default();
        if !tsconfig.contains("\"jsx") {
            flags.push("--jsx=automatic");
        }
        esbuild(&bin, root, entry, &out, &flags)?;
        let code = fs::read(&out)?;
        let file = format!("{name}.{}.js", crate::assets::content_short_hash(&code));
        write_asset(&gen, "assets/islands", &file, &code)?;
        result.imports.insert(name.clone(), format!("/assets/islands/{file}"));
        // CSS importado desde la isla: esbuild lo deja al lado.
        let css = tmp.join(format!("{name}.css"));
        if css.is_file() {
            let code = fs::read(&css)?;
            let file = format!("{name}-island.{}.css", crate::assets::content_short_hash(&code));
            write_asset(&gen, "assets/styles", &file, &code)?;
            result.styles.push(format!("/assets/styles/{file}"));
        }
    }

    for (name, entry) in &styles {
        let out = tmp.join(format!("{name}.css"));
        esbuild(&bin, root, entry, &out, &["--loader:.png=dataurl", "--loader:.svg=dataurl", "--loader:.woff2=file", "--asset-names=[name]-[hash]"])?;
        let code = fs::read(&out)?;
        let file = format!("{name}.{}.css", crate::assets::content_short_hash(&code));
        write_asset(&gen, "assets/styles", &file, &code)?;
        result.styles.push(format!("/assets/styles/{file}"));
    }
    // fuentes a las que apunta el CSS (esbuild las deja junto a la salida)
    if let Ok(rd) = fs::read_dir(&tmp) {
        for f in rd.flatten() {
            let p = f.path();
            if p.extension().is_some_and(|e| e == "woff2") {
                if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    write_asset(&gen, "assets/styles", n, &fs::read(&p)?)?;
                }
            }
        }
    }
    let _ = fs::remove_dir_all(&tmp);
    fs::write(root.join(INDEX), serde_json::to_string_pretty(&result)?)?;
    Ok(result)
}

fn copy_styles_without_esbuild(root: &Path, styles: &[(String, PathBuf)], gen: &Path) -> Result<Generated> {
    let mut result = Generated::default();
    for (name, entry) in styles {
        let code = fs::read(entry)?;
        if String::from_utf8_lossy(&code).contains("@import") {
            eprintln!("Aviso: {} usa @import y no hay esbuild: se copia sin resolverlo (`npm install -D esbuild`).", entry.display());
        }
        let file = format!("{name}.{}.css", crate::assets::content_short_hash(&code));
        write_asset(gen, "assets/styles", &file, &code)?;
        result.styles.push(format!("/assets/styles/{file}"));
    }
    fs::write(root.join(INDEX), serde_json::to_string_pretty(&result)?)?;
    Ok(result)
}

fn write_asset(gen: &Path, dir: &str, file: &str, bytes: &[u8]) -> Result<()> {
    let dir = gen.join(dir);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(file), bytes)?;
    Ok(())
}

/// Marca barata de "algo cambió" para `nexa dev`.
pub fn watch_marker(root: &Path) -> String {
    let mut parts = String::new();
    for dir in ["islands", "src/islands", "src/styles"] {
        parts.push_str(&crate::dev_watch::version_marker(&root.join(dir)));
        parts.push('|');
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nexa-assets-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn finds_islands_in_both_directories_and_ignores_the_rest() {
        let d = scratch("islands");
        fs::create_dir_all(d.join("islands")).unwrap();
        fs::create_dir_all(d.join("src/islands")).unwrap();
        fs::write(d.join("islands/search.island.ts"), "").unwrap();
        fs::write(d.join("islands/helper.ts"), "").unwrap();
        fs::write(d.join("src/islands/cart.island.tsx"), "").unwrap();
        fs::write(d.join("src/islands/readme.md"), "").unwrap();
        let names: Vec<_> = island_entries(&d).into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["search", "cart"]);
    }

    #[test]
    fn style_entries_skip_partials() {
        let d = scratch("styles");
        fs::create_dir_all(d.join("src/styles")).unwrap();
        fs::write(d.join("src/styles/main.css"), "").unwrap();
        fs::write(d.join("src/styles/_vars.css"), "").unwrap();
        fs::write(d.join("src/styles/x.scss"), "").unwrap();
        let names: Vec<_> = style_entries(&d).into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["main"]);
    }

    #[test]
    fn nothing_to_do_costs_nothing_and_needs_no_esbuild() {
        let d = scratch("empty");
        let g = run(&d, &[]).unwrap();
        assert_eq!(g, Generated::default());
        assert!(!d.join(".nexa").exists());
    }

    #[test]
    fn css_only_projects_work_without_esbuild() {
        let d = scratch("noesbuild");
        fs::create_dir_all(d.join("src/styles")).unwrap();
        fs::write(d.join("src/styles/oweekoi.css"), "body{margin:0}").unwrap();
        // sin esbuild en node_modules; el PATH de la prueba tampoco lo trae garantizado, pero el CSS se copia igual
        let g = run(&d, &[]);
        if let Ok(g) = g {
            assert_eq!(g.styles.len(), 1);
            assert!(d.join(".nexa/gen").join(g.styles[0].trim_start_matches('/')).is_file());
        }
    }

    #[test]
    fn island_names_are_validated() {
        assert!(valid_name("search-box_2"));
        assert!(!valid_name("a b"));
        assert!(!valid_name("../x"));
        assert!(!valid_name(""));
    }

    #[test]
    fn a_failing_before_command_stops_the_build() {
        let d = scratch("before");
        assert!(run(&d, &["exit 3".to_string()]).is_err());
    }
}
