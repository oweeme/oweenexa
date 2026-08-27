//! Copia recursiva de un directorio a otro — usada por `nexa build`
//! (Fase 14) para volcar `public/` en `dist/`. Antes de esta fase nada
//! copiaba `public/`: un `<img src="/logo.png">` con el archivo puesto
//! ahí a mano nunca llegaba a `dist/`, así que se serviría un 404 real en
//! `nexa preview`/producción — un vacío real, no solo teórico, que hacía
//! falta cerrar antes de que el presupuesto `maxImage` (mismo Fase 14)
//! tuviera algo real que medir.

use std::fs;
use std::io;
use std::path::Path;

/// Si `from` no existe, no es un error: no todos los proyectos tienen
/// (o necesitan) `public/`.
pub fn copy_recursive(from: &Path, to: &Path) -> io::Result<()> {
    if !from.exists() {
        return Ok(());
    }

    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_recursive(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-copy-dir-test-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copies_files_and_nested_directories() {
        let root = scratch_dir();
        let from = root.join("public");
        let to = root.join("dist");

        fs::create_dir_all(from.join("images")).unwrap();
        fs::write(from.join("favicon.ico"), b"icon").unwrap();
        fs::write(from.join("images/logo.png"), b"png-bytes").unwrap();

        copy_recursive(&from, &to).unwrap();

        assert_eq!(fs::read(to.join("favicon.ico")).unwrap(), b"icon");
        assert_eq!(fs::read(to.join("images/logo.png")).unwrap(), b"png-bytes");
    }

    #[test]
    fn a_missing_source_directory_is_not_an_error() {
        let root = scratch_dir();
        let from = root.join("does-not-exist");
        let to = root.join("dist");

        assert!(copy_recursive(&from, &to).is_ok());
        assert!(!to.exists());
    }
}
