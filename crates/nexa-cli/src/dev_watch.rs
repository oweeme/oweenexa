//! Calcula un "número de versión" barato para `nexa dev`: el `mtime` más
//! reciente entre todos los archivos bajo un directorio (en la práctica,
//! `src/`). No es un watcher real (no hay eventos push) — el cliente en
//! el navegador lo sondea periódicamente (`packages/dev-client`) y
//! compara contra el último valor que vio; si cambió, vuelve a pedir la
//! página actual. Recorrer `src/` en cada sondeo es barato para el
//! tamaño de proyecto que este framework asume, y evita añadir una
//! dependencia de watcher de archivos (`notify`) solo para esta fase.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn version_marker(root: &Path) -> String {
    let mut latest = UNIX_EPOCH;
    visit(root, &mut latest);
    latest.duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos().to_string()
}

fn visit(dir: &Path, latest: &mut SystemTime) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, latest);
        } else if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
            if modified > *latest {
                *latest = modified;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread::sleep;
    use std::time::Duration;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-dev-watch-test-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn changes_when_a_watched_file_is_modified() {
        let dir = scratch_dir();
        fs::write(dir.join("a.tsx"), "one").unwrap();

        let before = version_marker(&dir);
        sleep(Duration::from_millis(10));
        fs::write(dir.join("a.tsx"), "two").unwrap();

        assert_ne!(before, version_marker(&dir));
    }

    #[test]
    fn is_stable_when_nothing_changed() {
        let dir = scratch_dir();
        fs::write(dir.join("a.tsx"), "one").unwrap();

        assert_eq!(version_marker(&dir), version_marker(&dir));
    }

    #[test]
    fn recurses_into_subdirectories() {
        let dir = scratch_dir();
        fs::create_dir_all(dir.join("pages")).unwrap();
        fs::write(dir.join("pages/index.tsx"), "one").unwrap();

        let before = version_marker(&dir);
        sleep(Duration::from_millis(10));
        fs::write(dir.join("pages/index.tsx"), "two").unwrap();

        assert_ne!(before, version_marker(&dir));
    }

    #[test]
    fn a_missing_directory_does_not_panic() {
        let dir = std::env::temp_dir().join("nexa-dev-watch-test-does-not-exist");
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(version_marker(&dir), version_marker(&dir));
    }
}
