//! `nexa.toml`: el manifiesto del proyecto (Fase 12). Antes de esta fase
//! existía el archivo (lo escribe `nexa create`) pero nada lo leía — esta
//! es la primera vez que su sección `[dependencies]` significa algo de
//! verdad: qué módulos oficiales (`ui`, `forms`, ...) declaró el proyecto
//! con `nexa add`.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub project: ProjectSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    /// Presupuestos de rendimiento (Fase 14) — ausente por defecto, así
    /// que `nexa build` no comprueba nada a menos que el proyecto lo
    /// pida explícitamente.
    #[serde(default)]
    pub performance: Option<PerformanceSection>,
    /// `@nexa/telemetry` (Fase 14) — ausente por defecto. A propósito no
    /// basta con `nexa add telemetry` (eso solo lo registra en
    /// `[dependencies]`, igual que `ui`/`forms`): el código de
    /// telemetría solo se inyecta en una página si además hay un
    /// `endpoint` declarado aquí. Sin él no hay a dónde enviar nada, así
    /// que no tendría sentido cargarlo — y así ningún proyecto empieza a
    /// mandar datos por accidente solo por tener el módulo declarado.
    #[serde(default)]
    pub telemetry: Option<TelemetrySection>,
    /// Identificadores importables por un tercero (Fase 15) — nombre a
    /// resolver dentro de un handler (`stripe.load(...)`) -> specifier
    /// real (una URL de CDN, o un archivo bajo `public/`). `nexa-cli`
    /// los combina con los suyos propios (ej. `platform`) para: (a)
    /// decirle a `nexa-activation` qué identificadores puede importar un
    /// chunk por su nombre pelado, y (b) declarar el
    /// `<script type="importmap">` real que el navegador necesita para
    /// resolver ese nombre pelado a la URL de verdad.
    #[serde(default)]
    pub imports: BTreeMap<String, String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProjectSection {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
}

/// Límites en bytes, cada uno opcional por separado — declarar solo
/// `maxInitialJS` sin `maxCSS` comprueba solo el primero. Los nombres de
/// campo son exactamente los que usa el documento de arquitectura
/// original (`performance: { maxInitialJS, maxCSS, maxImage }`).
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct PerformanceSection {
    #[serde(rename = "maxInitialJS", skip_serializing_if = "Option::is_none")]
    pub max_initial_js: Option<u64>,
    #[serde(rename = "maxCSS", skip_serializing_if = "Option::is_none")]
    pub max_css: Option<u64>,
    #[serde(rename = "maxImage", skip_serializing_if = "Option::is_none")]
    pub max_image: Option<u64>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct TelemetrySection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

impl Manifest {
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.contains_key(name)
    }
}

/// Para el pipeline de compilación (avisos `NEXA-PKG-*`): si `nexa.toml`
/// no existe o no se puede parsear, se trata como "sin dependencias
/// declaradas" — no es motivo para que una página falle, es solo la
/// señal que activa o no un aviso.
pub fn load_lenient(path: &Path) -> Manifest {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Para `nexa add`: aquí sí debe fallar con un mensaje claro si no hay un
/// `nexa.toml` real (o si está corrupto) — declarar un módulo en un
/// proyecto que no existe no tiene sentido.
pub fn load_strict(path: &Path) -> Result<Manifest> {
    let contents = std::fs::read_to_string(path).with_context(|| {
        format!(
            "no se encontró {} — ¿estás dentro de un proyecto Nexa? (usa `nexa create <nombre>`)",
            path.display()
        )
    })?;
    toml::from_str(&contents).with_context(|| format!("{} no es un nexa.toml válido", path.display()))
}

pub fn write(path: &Path, manifest: &Manifest) -> Result<()> {
    let contents = toml::to_string_pretty(manifest).context("serializando nexa.toml")?;
    std::fs::write(path, contents).with_context(|| format!("escribiendo {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_path(contents: Option<&str>) -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-manifest-test-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("nexa.toml");
        if let Some(contents) = contents {
            std::fs::write(&path, contents).unwrap();
        }
        path
    }

    #[test]
    fn load_lenient_returns_empty_when_the_file_is_missing() {
        let path = scratch_path(None);
        let manifest = load_lenient(&path);
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn load_lenient_reads_declared_dependencies() {
        let path = scratch_path(Some("[project]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nui = \"0.1.0\"\n"));

        let manifest = load_lenient(&path);
        assert!(manifest.has_dependency("ui"));
        assert!(!manifest.has_dependency("forms"));
    }

    #[test]
    fn absent_performance_section_parses_as_none() {
        let path = scratch_path(Some("[project]\nname = \"x\"\nversion = \"0.1.0\"\n"));
        assert!(load_lenient(&path).performance.is_none());
    }

    #[test]
    fn reads_declared_performance_budgets_by_their_original_camel_case_names() {
        let path = scratch_path(Some(
            "[project]\nname = \"x\"\nversion = \"0.1.0\"\n\n[performance]\nmaxInitialJS = 15000\nmaxCSS = 20000\n",
        ));

        let performance = load_lenient(&path).performance.expect("expected a performance section");
        assert_eq!(performance.max_initial_js, Some(15000));
        assert_eq!(performance.max_css, Some(20000));
        assert_eq!(performance.max_image, None);
    }

    #[test]
    fn absent_telemetry_section_parses_as_none() {
        let path = scratch_path(Some("[project]\nname = \"x\"\nversion = \"0.1.0\"\n"));
        assert!(load_lenient(&path).telemetry.is_none());
    }

    #[test]
    fn reads_a_declared_telemetry_endpoint() {
        let path = scratch_path(Some(
            "[project]\nname = \"x\"\nversion = \"0.1.0\"\n\n[telemetry]\nendpoint = \"/api/telemetry\"\n",
        ));

        let telemetry = load_lenient(&path).telemetry.expect("expected a telemetry section");
        assert_eq!(telemetry.endpoint.as_deref(), Some("/api/telemetry"));
    }

    #[test]
    fn absent_imports_section_parses_as_an_empty_map() {
        let path = scratch_path(Some("[project]\nname = \"x\"\nversion = \"0.1.0\"\n"));
        assert!(load_lenient(&path).imports.is_empty());
    }

    #[test]
    fn reads_a_declared_third_party_import() {
        let path = scratch_path(Some(
            "[project]\nname = \"x\"\nversion = \"0.1.0\"\n\n[imports]\nstripe = \"https://cdn.example.com/stripe.js\"\n",
        ));

        let imports = load_lenient(&path).imports;
        assert_eq!(imports.get("stripe").map(String::as_str), Some("https://cdn.example.com/stripe.js"));
    }

    #[test]
    fn load_strict_fails_clearly_when_the_project_does_not_exist() {
        let path = scratch_path(None);
        assert!(load_strict(&path).is_err());
    }

    #[test]
    fn write_then_load_round_trips_dependencies() {
        let path = scratch_path(Some("[project]\nname = \"x\"\nversion = \"0.1.0\"\n"));

        let mut manifest = load_strict(&path).unwrap();
        manifest.dependencies.insert("ui".to_string(), "0.1.0".to_string());
        write(&path, &manifest).unwrap();

        let reloaded = load_strict(&path).unwrap();
        assert_eq!(reloaded.dependencies.get("ui"), Some(&"0.1.0".to_string()));
        assert_eq!(reloaded.project.name, "x");
    }
}
