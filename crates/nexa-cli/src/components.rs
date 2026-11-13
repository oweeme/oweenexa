//! Fase 62: lee del disco los componentes que una página (o el layout)
//! importa con una ruta relativa. `nexa-parser` no hace IO — le pide los
//! archivos a esto, y los expande en tiempo de compilación.

use std::path::{Path, PathBuf};

use nexa_parser::ImportResolver;

pub struct FsResolver;

impl ImportResolver for FsResolver {
    fn resolve(&self, from_file: &str, specifier: &str) -> Option<(String, String)> {
        // Solo rutas relativas: un paquete (`"react"`) nunca es un
        // componente de Nexa Core.
        if !(specifier.starts_with("./") || specifier.starts_with("../")) {
            return None;
        }
        let base = Path::new(from_file).parent().unwrap_or_else(|| Path::new("."));
        let target = base.join(specifier);

        let mut candidates: Vec<PathBuf> = Vec::new();
        if target.extension().is_some_and(|e| e == "tsx" || e == "ts") {
            candidates.push(target.clone());
        }
        for ext in ["tsx", "ts"] {
            let mut with_ext = target.clone().into_os_string();
            with_ext.push(format!(".{ext}"));
            candidates.push(PathBuf::from(with_ext));
        }
        for ext in ["tsx", "ts"] {
            candidates.push(target.join(format!("index.{ext}")));
        }

        candidates.into_iter().find(|p| p.is_file()).and_then(|p| {
            let source = std::fs::read_to_string(&p).ok()?;
            Some((p.to_string_lossy().into_owned(), source))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nexa-components-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("pages")).unwrap();
        fs::create_dir_all(dir.join("components/ui")).unwrap();
        dir
    }

    #[test]
    fn resolves_files_with_or_without_extension_and_index_files() {
        let dir = scratch("resolve");
        fs::write(dir.join("components/Card.tsx"), "card").unwrap();
        fs::write(dir.join("components/ui/index.ts"), "ui").unwrap();
        let from = dir.join("pages/index.tsx");
        let from = from.to_str().unwrap();

        assert_eq!(FsResolver.resolve(from, "../components/Card").unwrap().1, "card");
        assert_eq!(FsResolver.resolve(from, "../components/Card.tsx").unwrap().1, "card");
        assert_eq!(FsResolver.resolve(from, "../components/ui").unwrap().1, "ui");
        assert!(FsResolver.resolve(from, "../components/Nope").is_none());
        assert!(FsResolver.resolve(from, "react").is_none());
    }
}
