//! `nexa add docker` (Fase 76): un `Dockerfile` listo para el ejecutable de `nexa bundle`.
//!
//! El ejecutable ya incluye servidor, caché, proxy y renderizado bajo demanda: la imagen solo lo
//! copia. Se compila en Linux (`nexa bundle`) y luego `docker build`.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

fn sanitize(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    if s.is_empty() { "app".into() } else { s }
}

fn dockerfile(name: &str) -> String {
    format!(
        r#"# Generado por `nexa add docker`.
#
#   nexa bundle --out {name}-server      # en Linux: compila el sitio y lo empaqueta
#   docker build -t {name} .
#   docker run -p 8080:8080 -e NEXA_API_URL=https://tu-api {name}
#
# El ejecutable es UN archivo con todo (páginas, servidor, caché, proxy): la imagen solo lo copia.
# Debe compilarse en una distribución con una glibc igual o más antigua que la de esta imagen
# (si falla con `GLIBC_x.y not found`, usa una imagen más nueva, p. ej. `ubuntu:24.04`).
FROM debian:stable-slim

RUN useradd --system --no-create-home nexa
COPY {name}-server /usr/local/bin/{name}-server
USER nexa

ENV NEXA_API_URL=""
EXPOSE 8080
CMD ["/usr/local/bin/{name}-server", "--host", "0.0.0.0", "--port", "8080"]
"#
    )
}

const DOCKERIGNORE: &str = "# Generado por `nexa add docker`\n*\n!*-server\n";

pub fn scaffold(project_root: &Path, project_name: &str) -> Result<()> {
    let name = sanitize(project_name);
    let file = project_root.join("Dockerfile");
    if file.exists() {
        bail!("{} ya existe — bórralo primero si quieres regenerarlo", file.display());
    }
    fs::write(&file, dockerfile(&name)).with_context(|| format!("escribiendo {}", file.display()))?;
    let ignore = project_root.join(".dockerignore");
    if !ignore.exists() {
        fs::write(&ignore, DOCKERIGNORE)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dockerfile_names_the_executable_and_listens_on_all_interfaces() {
        let d = dockerfile("mi-blog");
        assert!(d.contains("COPY mi-blog-server /usr/local/bin/mi-blog-server"));
        assert!(d.contains("\"--host\", \"0.0.0.0\""));
        assert!(!d.contains("root"), "no corre como root");
    }

    #[test]
    fn names_are_sanitized_and_existing_files_are_not_overwritten() {
        assert_eq!(sanitize("mi sitio!"), "mi-sitio-");
        let dir = std::env::temp_dir().join(format!("nexa-docker-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        scaffold(&dir, "x").unwrap();
        assert!(dir.join(".dockerignore").is_file());
        assert!(scaffold(&dir, "x").is_err());
    }
}
