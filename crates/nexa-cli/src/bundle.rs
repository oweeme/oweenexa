//! Fase 64 — `nexa bundle`: el sitio entero como **un solo ejecutable**.
//!
//! El ejecutable resultante es una copia del propio binario de `nexa` con
//! `dist/` (comprimido) y la configuración pegados al final. Al arrancar,
//! `nexa` mira si lleva esa carga (los últimos bytes del archivo son una
//! marca fija): si la lleva, no actúa como CLI sino como el servidor de
//! producción de ese sitio (`prod_server`). Sin Rust, sin Node, sin nginx y
//! sin ningún archivo suelto en la máquina de destino.
//!
//! Disposición del archivo:
//!
//! ```text
//! [ ejecutable de nexa ][ blobs ][ cabecera JSON ][ cola de 24 bytes ]
//! cola = cabecera_len (u64 LE) | blobs_len (u64 LE) | "NEXABNDL"
//! ```
//!
//! Cada archivo de `dist/` se guarda tal cual y, si es texto y comprimir
//! ayuda, también en gzip — el servidor elige según `Accept-Encoding`, sin
//! comprimir nada en cada petición.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

pub const MAGIC: &[u8; 8] = b"NEXABNDL";
const TRAILER_LEN: u64 = 24;
/// v2 (Fase 66): el paquete puede llevar también las fuentes del sitio
/// (`sources`) para renderizar rutas dinámicas bajo demanda. Un paquete v1
/// (sin fuentes) se sigue leyendo.
const FORMAT_VERSION: u32 = 2;

/// Lo que el servidor necesita saber del sitio, más allá de sus archivos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub name: String,
    /// Prefijo de la SPA del Modo App (`/app`), si el proyecto la tiene —
    /// toda ruta profunda bajo él que no sea un archivo cae a su shell.
    pub app_base: Option<String>,
    /// `[proxy]` de `nexa.toml`: prefijo -> backend. Se puede reemplazar al
    /// arrancar con `--proxy /api=https://...`.
    pub proxy: BTreeMap<String, String>,
    pub default_port: u16,
    /// Fase 66: el paquete lleva las fuentes y renderiza bajo demanda las
    /// rutas dinámicas de Core que no están pre-renderizadas (un post
    /// publicado después del build, un producto nuevo...).
    #[serde(default)]
    pub dynamic: bool,
    /// Segundos que se reutiliza una página renderizada bajo demanda antes
    /// de volver a pedirle los datos a la API (0 = nunca cachear).
    #[serde(default = "default_revalidate")]
    pub revalidate: u64,
    /// `[redirects]` y `[headers]` de `nexa.toml` (Fase 75).
    #[serde(default)]
    pub redirects: Vec<crate::rules::Redirect>,
    #[serde(default)]
    pub headers: Vec<crate::rules::HeaderRule>,
}

fn default_revalidate() -> u64 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntryMeta {
    /// Ruta relativa a `dist/`, con `/` (`assets/app.1a2b3c4d.js`).
    pub path: String,
    pub offset: u64,
    pub len: u64,
    /// 0/0 si no hay versión gzip.
    pub gz_offset: u64,
    pub gz_len: u64,
}

#[derive(Serialize, Deserialize)]
struct Header {
    version: u32,
    config: Config,
    entries: Vec<EntryMeta>,
    /// Fuentes del sitio para el renderizado bajo demanda. **Nunca se
    /// sirven por HTTP**: solo el servidor las extrae a su directorio de
    /// trabajo.
    #[serde(default)]
    sources: Vec<EntryMeta>,
}

/// El sitio ya cargado en memoria.
#[derive(Debug)]
pub struct Payload {
    pub config: Config,
    pub entries: Vec<EntryMeta>,
    pub sources: Vec<EntryMeta>,
    pub blobs: Vec<u8>,
}

impl Payload {
    pub fn bytes(&self, offset: u64, len: u64) -> &[u8] {
        &self.blobs[offset as usize..(offset + len) as usize]
    }
}

/// Extensiones de texto: las que vale la pena guardar también en gzip.
fn is_compressible(path: &str) -> bool {
    matches!(
        Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or(""),
        "html" | "css" | "js" | "mjs" | "json" | "svg" | "xml" | "txt" | "webmanifest" | "map"
    )
}

pub(crate) fn gzip(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}

/// Arma la carga (blobs + cabecera + cola) a partir de los archivos del
/// sitio y, opcionalmente, las fuentes para renderizar bajo demanda.
pub fn pack(files: &[(String, Vec<u8>)], sources: &[(String, Vec<u8>)], config: Config) -> Result<Vec<u8>> {
    let mut blobs: Vec<u8> = Vec::new();
    let mut entries = Vec::with_capacity(files.len());

    for (path, bytes) in files {
        let offset = blobs.len() as u64;
        blobs.extend_from_slice(bytes);

        let (mut gz_offset, mut gz_len) = (0, 0);
        if is_compressible(path) && bytes.len() >= 256 {
            let compressed = gzip(bytes)?;
            // Si no gana al menos un 5%, no vale la pena servirlo comprimido.
            if (compressed.len() as f64) < bytes.len() as f64 * 0.95 {
                gz_offset = blobs.len() as u64;
                gz_len = compressed.len() as u64;
                blobs.extend_from_slice(&compressed);
            }
        }
        entries.push(EntryMeta { path: path.clone(), offset, len: bytes.len() as u64, gz_offset, gz_len });
    }

    // Las fuentes van tal cual (son poco texto y no se sirven nunca).
    let mut source_entries = Vec::with_capacity(sources.len());
    for (path, bytes) in sources {
        let offset = blobs.len() as u64;
        blobs.extend_from_slice(bytes);
        source_entries.push(EntryMeta { path: path.clone(), offset, len: bytes.len() as u64, gz_offset: 0, gz_len: 0 });
    }

    let header = serde_json::to_vec(&Header { version: FORMAT_VERSION, config, entries, sources: source_entries })?;

    let mut out = blobs;
    let blobs_len = out.len() as u64;
    out.extend_from_slice(&header);
    out.extend_from_slice(&(header.len() as u64).to_le_bytes());
    out.extend_from_slice(&blobs_len.to_le_bytes());
    out.extend_from_slice(MAGIC);
    Ok(out)
}

/// Interpreta una carga completa (lo que devuelve `pack`).
pub fn unpack(payload: &[u8]) -> Result<Payload> {
    let len = payload.len() as u64;
    if len < TRAILER_LEN || &payload[(len - 8) as usize..] != MAGIC {
        bail!("no es un paquete de Nexa (falta la marca NEXABNDL)");
    }
    let header_len = u64::from_le_bytes(payload[(len - 24) as usize..(len - 16) as usize].try_into()?);
    let blobs_len = u64::from_le_bytes(payload[(len - 16) as usize..(len - 8) as usize].try_into()?);
    if header_len.saturating_add(blobs_len).saturating_add(TRAILER_LEN) > len {
        bail!("paquete de Nexa corrupto (longitudes inconsistentes)");
    }
    let header_start = (len - TRAILER_LEN - header_len) as usize;
    let header: Header = serde_json::from_slice(&payload[header_start..(len - TRAILER_LEN) as usize])
        .context("la cabecera del paquete no es JSON válido")?;
    if header.version == 0 || header.version > FORMAT_VERSION {
        bail!("el paquete usa el formato v{} y este `nexa` entiende hasta v{FORMAT_VERSION}", header.version);
    }
    let blobs_start = header_start - blobs_len as usize;
    Ok(Payload {
        config: header.config,
        entries: header.entries,
        sources: header.sources,
        blobs: payload[blobs_start..header_start].to_vec(),
    })
}

/// ¿El ejecutable `exe` lleva un sitio pegado? Lee solo los últimos 24
/// bytes para decidirlo: costo despreciable en cada arranque de `nexa`.
pub fn read_embedded(exe: &Path) -> Result<Option<Payload>> {
    let mut file = File::open(exe).with_context(|| format!("abriendo {}", exe.display()))?;
    let total = file.metadata()?.len();
    if total < TRAILER_LEN {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(total - TRAILER_LEN))?;
    let mut trailer = [0u8; TRAILER_LEN as usize];
    file.read_exact(&mut trailer)?;
    if &trailer[16..] != MAGIC {
        return Ok(None);
    }
    let header_len = u64::from_le_bytes(trailer[0..8].try_into()?);
    let blobs_len = u64::from_le_bytes(trailer[8..16].try_into()?);
    let section = header_len.saturating_add(blobs_len).saturating_add(TRAILER_LEN);
    if section > total {
        bail!("el paquete embebido está corrupto (longitudes inconsistentes)");
    }
    file.seek(SeekFrom::Start(total - section))?;
    let mut bytes = vec![0u8; section as usize];
    file.read_exact(&mut bytes)?;
    unpack(&bytes).map(Some)
}

/// Las fuentes que hacen falta para renderizar bajo demanda: `src/**`
/// (menos `src/app`, que ya viaja compilado en `dist/`), `nexa.toml` y
/// `nexa.lock`.
pub fn collect_sources(project_root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<std::result::Result<_, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let rel = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
            if rel == "src/app" {
                continue;
            }
            if path.is_dir() {
                walk(&path, root, out)?;
            } else {
                out.push((rel, fs::read(&path).with_context(|| format!("leyendo {}", path.display()))?));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    let src = project_root.join("src");
    if src.is_dir() {
        walk(&src, project_root, &mut out)?;
    }
    // El índice de islas/estilos empaquetados (Fase 72): el render bajo demanda lo necesita.
    for name in [".nexa/gen/assets.json", "nexa.toml", "nexa.lock"] {
        let file = project_root.join(name);
        if file.is_file() {
            out.push((name.to_string(), fs::read(&file)?));
        }
    }
    Ok(out)
}

/// Todos los archivos de `dist/`, con rutas relativas y `/` como separador.
pub fn collect_dist(dist: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<std::result::Result<_, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out)?;
            } else {
                let rel = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
                out.push((rel, fs::read(&path).with_context(|| format!("leyendo {}", path.display()))?));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dist, dist, &mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config { name: "demo".into(), app_base: Some("/app".into()), proxy: [("/api".to_string(), "http://x".to_string())].into(), default_port: 8080, dynamic: false, revalidate: 60, redirects: vec![], headers: vec![] }
    }

    fn files() -> Vec<(String, Vec<u8>)> {
        vec![
            ("index.html".into(), "<html>".repeat(200).into_bytes()),
            ("logo.png".into(), vec![0x89, 0x50, 0x4e, 0x47, 1, 2, 3]),
            ("assets/a.js".into(), b"x".to_vec()),
        ]
    }

    #[test]
    fn pack_then_unpack_round_trips_every_file() {
        let payload = unpack(&pack(&files(), &[], config()).unwrap()).unwrap();
        assert_eq!(payload.config, config());
        assert_eq!(payload.entries.len(), 3);
        for (meta, (path, bytes)) in payload.entries.iter().zip(files()) {
            assert_eq!(meta.path, path);
            assert_eq!(payload.bytes(meta.offset, meta.len), bytes.as_slice());
        }
    }

    #[test]
    fn only_compressible_files_that_benefit_get_a_gzip_copy_and_it_decodes_back() {
        let payload = unpack(&pack(&files(), &[], config()).unwrap()).unwrap();
        let by = |p: &str| payload.entries.iter().find(|e| e.path == p).unwrap().clone();

        let html = by("index.html");
        assert!(html.gz_len > 0 && html.gz_len < html.len, "el HTML repetitivo debe comprimirse");
        let mut decoded = Vec::new();
        flate2::read::GzDecoder::new(payload.bytes(html.gz_offset, html.gz_len)).read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, payload.bytes(html.offset, html.len));

        assert_eq!(by("logo.png").gz_len, 0, "un PNG no es texto");
        assert_eq!(by("assets/a.js").gz_len, 0, "un archivo diminuto no se comprime");
    }

    #[test]
    fn a_payload_appended_to_any_executable_is_found_by_read_embedded() {
        let dir = std::env::temp_dir().join(format!("nexa-bundle-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("fake-exe");

        let mut bytes = vec![0xAB; 5000]; // "el ejecutable"
        bytes.extend(pack(&files(), &[], config()).unwrap());
        fs::write(&exe, &bytes).unwrap();
        let found = read_embedded(&exe).unwrap().expect("debe detectar la carga");
        assert_eq!(found.config.name, "demo");
        assert_eq!(found.entries.len(), 3);

        // un ejecutable sin carga (el `nexa` normal) no se confunde
        let plain = dir.join("plain");
        fs::write(&plain, vec![0xAB; 5000]).unwrap();
        assert!(read_embedded(&plain).unwrap().is_none());
        let tiny = dir.join("tiny");
        fs::write(&tiny, b"x").unwrap();
        assert!(read_embedded(&tiny).unwrap().is_none());
    }

    #[test]
    fn sources_travel_in_the_payload_and_are_not_mixed_with_the_public_files() {
        let sources = vec![("src/pages/blog/[a]/[b].tsx".to_string(), b"export default 1".to_vec()), ("nexa.toml".to_string(), b"[project]".to_vec())];
        let payload = unpack(&pack(&files(), &sources, Config { dynamic: true, ..config() }).unwrap()).unwrap();
        assert!(payload.config.dynamic);
        assert_eq!(payload.entries.len(), 3, "las fuentes no son archivos públicos");
        assert_eq!(payload.sources.len(), 2);
        assert_eq!(payload.bytes(payload.sources[0].offset, payload.sources[0].len), b"export default 1");
        assert!(payload.entries.iter().all(|e| !e.path.starts_with("src/")));
    }

    #[test]
    fn collect_sources_skips_the_compiled_app_and_includes_the_manifest() {
        let dir = std::env::temp_dir().join(format!("nexa-sources-test-{}", std::process::id()));
        fs::create_dir_all(dir.join("src/pages")).unwrap();
        fs::create_dir_all(dir.join("src/app/pages")).unwrap();
        fs::create_dir_all(dir.join("src/locales")).unwrap();
        fs::write(dir.join("src/pages/index.tsx"), "x").unwrap();
        fs::write(dir.join("src/app/pages/index.tsx"), "y").unwrap();
        fs::write(dir.join("src/locales/es.json"), "{}").unwrap();
        fs::write(dir.join("nexa.toml"), "[project]").unwrap();
        let names: Vec<_> = collect_sources(&dir).unwrap().into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["src/locales/es.json", "src/pages/index.tsx", "nexa.toml"]);
    }

    #[test]
    fn corrupt_lengths_are_an_error_not_a_panic() {
        let mut p = pack(&files(), &[], config()).unwrap();
        let n = p.len();
        p[n - 24..n - 16].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(unpack(&p).is_err());
        assert!(unpack(b"corto").is_err());
    }
}
