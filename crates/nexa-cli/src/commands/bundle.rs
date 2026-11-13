//! `nexa bundle` (Fase 64): el sitio compilado como **un único ejecutable**
//! con su propio servidor de producción. Ver `crate::bundle` (formato) y
//! `crate::prod_server` (qué hace el servidor).
//!
//! El ejecutable es una copia del binario de `nexa` (el de esta máquina, o
//! el que se indique con `--runtime` — p. ej. el de Windows o macOS para
//! empaquetar desde Linux) con el sitio pegado al final.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::bundle::{collect_dist, pack, read_embedded, Config};

pub fn run(out: Option<String>, runtime: Option<String>, no_build: bool, static_only: bool) -> Result<()> {
    if !no_build {
        println!("[1/3] nexa build");
        super::build::run()?;
        println!();
    }

    let dist = Path::new("dist");
    if !dist.is_dir() {
        bail!("no existe dist/ — corre `nexa build` primero (o quita --no-build)");
    }

    let manifest = crate::manifest::load_lenient(Path::new("nexa.toml"));
    let name = project_name(&manifest.project.name);
    // Fase 66: si el sitio tiene rutas dinámicas de Core (`blog/[autor]/[slug]`,
    // `[locale]`, ...), el ejecutable lleva también las fuentes para
    // renderizarlas bajo demanda — un post publicado después del build no
    // exige recompilar. `--static` lo desactiva (solo lo pre-renderizado).
    let has_dynamic_routes = {
        let pages = Path::new("src/pages");
        pages.is_dir() && nexa_router::scan_pages(pages).map(|r| r.iter().any(|route| route.is_dynamic())).unwrap_or(false)
    };
    let dynamic = has_dynamic_routes && !static_only;
    let sources = if dynamic { crate::bundle::collect_sources(Path::new("."))? } else { Vec::new() };

    let (redirects, headers) = manifest.rules()?;
    let config = Config {
        name: name.clone(),
        app_base: crate::app_mode::is_enabled().then(|| crate::app_mode::base_path(&manifest)),
        proxy: manifest.proxy.clone(),
        default_port: 8080,
        dynamic,
        revalidate: manifest.bundle.as_ref().and_then(|b| b.revalidate).unwrap_or(60),
        stale: manifest.bundle.as_ref().and_then(|b| b.stale).unwrap_or(0),
        redirects,
        headers,
    };

    println!("[2/3] empaquetando dist/");
    let files = collect_dist(dist)?;
    let raw_bytes: usize = files.iter().map(|(_, b)| b.len()).sum();
    let payload = pack(&files, &sources, config)?;

    let runtime_path = match runtime {
        Some(p) => PathBuf::from(p),
        None => std::env::current_exe().context("no se pudo localizar el ejecutable de nexa")?,
    };
    if read_embedded(&runtime_path)?.is_some() {
        bail!("{} ya es un paquete de Nexa — usa el binario `nexa` original como --runtime", runtime_path.display());
    }
    let mut bytes = std::fs::read(&runtime_path).with_context(|| format!("leyendo {}", runtime_path.display()))?;
    let runtime_len = bytes.len();
    bytes.extend_from_slice(&payload);

    let out_path = match out {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(format!("{name}-server{}", if cfg!(windows) || runtime_path.extension().is_some_and(|e| e == "exe") { ".exe" } else { "" })),
    };
    if out_path.is_dir() {
        bail!("{} es un directorio — indica un nombre de archivo con --out", out_path.display());
    }
    std::fs::write(&out_path, &bytes).with_context(|| format!("escribiendo {}", out_path.display()))?;
    make_executable(&out_path)?;

    println!("[3/3] listo");
    println!();
    println!("Ejecutable: {}", out_path.display());
    println!(
        "  {:.1} MB  =  {:.1} MB (servidor) + {:.1} MB (sitio: {} archivo(s), {:.1} MB sin comprimir)",
        bytes.len() as f64 / 1_048_576.0,
        runtime_len as f64 / 1_048_576.0,
        payload.len() as f64 / 1_048_576.0,
        files.len(),
        raw_bytes as f64 / 1_048_576.0
    );
    println!("  Es autocontenido: cópialo a cualquier máquina del mismo sistema operativo y ejecútalo.");
    if dynamic {
        println!(
            "  Renderizado bajo demanda: ACTIVO ({} fuente(s) embebida(s)) — las rutas dinámicas no pre-renderizadas se generan",
            sources.len()
        );
        println!("  al recibirlas, pidiendo los datos a la API: ejecútalo con --api-url https://tu-api (o NEXA_API_URL).");
    } else if has_dynamic_routes {
        println!("  Renderizado bajo demanda: desactivado (--static): las rutas dinámicas no pre-renderizadas darán 404.");
    }
    let run_path = if out_path.is_absolute() || out_path.starts_with(".") { out_path.display().to_string() } else { format!("./{}", out_path.display()) };
    println!("  {run_path} --port 8080 --host 0.0.0.0 [--proxy /api=https://api.miapp.com]");
    Ok(())
}

/// Un nombre de archivo seguro a partir del nombre del proyecto.
fn project_name(raw: &str) -> String {
    let cleaned: String = raw.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    if cleaned.trim_matches('-').is_empty() {
        "site".to_string()
    } else {
        cleaned
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::project_name;

    #[test]
    fn project_names_become_safe_file_names() {
        assert_eq!(project_name("mi tienda"), "mi-tienda");
        assert_eq!(project_name("demo_1"), "demo_1");
        assert_eq!(project_name(""), "site");
        assert_eq!(project_name("///"), "site");
    }
}
