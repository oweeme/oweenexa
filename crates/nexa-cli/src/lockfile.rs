//! `nexa.lock` (Fase 12): qué versión (y de qué contenido) quedó resuelta
//! la última vez que `nexa add` corrió. Hoy todos los módulos oficiales
//! viven embebidos en el propio binario de `nexa-cli` — no hay nada que
//! descargar ni que resolver contra un registro remoto —, así que este
//! lockfile no impide nada todavía: es la forma que necesitará un
//! registro real de paquetes de terceros (Fase 15), aplicada desde ya a
//! los módulos oficiales.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Lockfile {
    #[serde(default = "lockfile_format_version")]
    pub version: u32,
    #[serde(rename = "package", default)]
    pub packages: Vec<LockedPackage>,
}

fn lockfile_format_version() -> u32 {
    1
}

impl Default for Lockfile {
    fn default() -> Self {
        Lockfile { version: lockfile_format_version(), packages: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub stability: String,
    /// De dónde vino: hoy siempre `"builtin"` (embebido en el binario de
    /// `nexa-cli`) — el campo existe para cuando la Fase 15 añada
    /// paquetes que sí vengan de un registro real.
    pub source: String,
    pub content_hash: String,
}

impl Lockfile {
    pub fn upsert(&mut self, package: LockedPackage) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.name == package.name) {
            *existing = package;
        } else {
            self.packages.push(package);
        }
        self.packages.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Si `nexa.lock` no existe todavía (primer `nexa add` del proyecto) o
/// está corrupto, se parte de uno vacío — nunca es motivo de error.
pub fn load(path: &Path) -> Lockfile {
    std::fs::read_to_string(path).ok().and_then(|c| toml::from_str(&c).ok()).unwrap_or_default()
}

pub fn write(path: &Path, lockfile: &Lockfile) -> Result<()> {
    let contents = toml::to_string_pretty(lockfile).context("serializando nexa.lock")?;
    std::fs::write(path, contents).with_context(|| format!("escribiendo {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_path() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-lockfile-test-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("nexa.lock")
    }

    fn package(name: &str, version: &str) -> LockedPackage {
        LockedPackage {
            name: name.to_string(),
            version: version.to_string(),
            stability: "stable".to_string(),
            source: "builtin".to_string(),
            content_hash: "fnv1a:0".to_string(),
        }
    }

    #[test]
    fn load_returns_an_empty_lockfile_when_the_file_is_missing() {
        let lock = load(&scratch_path());
        assert_eq!(lock.version, 1);
        assert!(lock.packages.is_empty());
    }

    #[test]
    fn upsert_adds_a_new_package() {
        let mut lock = Lockfile::default();
        lock.upsert(package("ui", "0.1.0"));

        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].name, "ui");
    }

    #[test]
    fn upsert_replaces_an_existing_package_instead_of_duplicating_it() {
        let mut lock = Lockfile::default();
        lock.upsert(package("ui", "0.1.0"));
        lock.upsert(package("ui", "0.2.0"));

        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].version, "0.2.0");
    }

    #[test]
    fn write_then_load_round_trips() {
        let path = scratch_path();
        let mut lock = Lockfile::default();
        lock.upsert(package("forms", "0.1.0"));
        write(&path, &lock).unwrap();

        let reloaded = load(&path);
        assert_eq!(reloaded.packages, lock.packages);
    }
}
