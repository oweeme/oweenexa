# CI y pruebas

## Qué se comprueba y dónde

| Comprobación | Comando local | En la CI |
|---|---|---|
| Rust: clippy sin avisos | `cargo clippy --workspace --all-targets -- -D warnings` | `ci.yml` › **rust** (Linux, Windows, macOS) |
| Rust: tests | `cargo test --workspace` | `ci.yml` › **rust** |
| Assets embebidos al día | `npm run build:cli-assets && npm run build:ui-assets && git diff --exit-code -- 'crates/nexa-cli/assets/*.js' 'crates/nexa-ui/assets/*.css'` | `ci.yml` › **assets** |
| JavaScript: tipos y tests | `npm run typecheck --workspaces --if-present && npm test --workspaces --if-present` | `ci.yml` › **js** |
| Humo (crear → compilar → ejecutable → servir) | `node e2e/smoke.mjs target/release/nexa` | `ci.yml` › **smoke** (Linux, Windows, macOS) |
| De punta a punta en Chrome | `node e2e/run-all.mjs --nexa target/release/nexa` | `ci.yml` › **e2e** (Linux) |
| Sitio de documentación: se genera, sin enlaces rotos, y el tutorial coincide con el código | `cd site && npm ci && node check-tutorial.mjs && node build.mjs` | `ci.yml` › **site** y `pages.yml` (publica en GitHub Pages) |
| App de escritorio (Tauri) | `nexa add tauri && nexa build --target desktop` | `desktop.yml` (a mano y cada lunes) |
| Binarios de `nexa` por sistema | `cargo build --release -p nexa-cli` | `release.yml` (al subir una etiqueta `v*`) |

**Por qué hay una comprobación de assets.** El binario de `nexa` *embebe* el JavaScript de `packages/*`
(`include_str!`). Si cambias `packages/` y olvidas `npm run build:cli-assets`, `cargo build` compila con éxito
y `nexa` sirve JavaScript viejo — un error silencioso que ya ocurrió durante el desarrollo. La CI lo detecta.

## Windows y macOS: qué está verificado y qué no

- **Verificado en ejecuciones reales de GitHub Actions** (Windows x64 y macOS): `clippy -D warnings`, los
  **465 tests de Rust** (los mismos que en Linux) y el **humo multiplataforma** (18 comprobaciones: crear,
  compilar, empaquetar en un ejecutable `.exe`/binario, arrancarlo fuera del proyecto y servirlo). Por eso esos
  trabajos **bloquean** igual que Linux.
- **Antes de la primera ejecución** se había comprobado que el código y los tests compilan para
  `x86_64-pc-windows-gnu`, `x86_64-apple-darwin` y `aarch64-apple-darwin` (`cargo check --workspace
  --all-targets --target …`) y se corrigió lo que habría fallado en Windows: `nexa test` ejecutaba `npm` (en
  Windows es `npm.cmd`); el script `build:ui-assets` usaba `cp`; los scripts `vendor` de los ejemplos usaban
  `mkdir -p` y comillas de bash; sin `.gitattributes`, Git en Windows convierte a CRLF y rompe los assets embebidos.
- **App de escritorio (`desktop.yml`, a mano):** **compila en Ubuntu, Windows y macOS** y sube el binario de cada
  uno como artefacto (2,6 a 2,8 MB comprimidos). Las dos primeras ejecuciones destaparon dos defectos reales, ya
  corregidos: la plantilla de Tauri no traía `icons/icon.ico` (Windows lo exige) y el propio flujo cacheaba una
  carpeta que hacía fallar `nexa create` cuando ya había caché. En macOS GitHub a veces no asigna ejecutor por
  escasez (basta reintentar).
- **Sigue sin verificarse:** que esas apps **se abran y funcionen** (solo se comprobó que compilan; la ejecución
  en Linux sí se probó a mano con captura de pantalla), las suites de Chrome (`run-all.mjs`) en Windows y macOS
  (solo corren en Linux), y `nexa bundle --runtime` entre sistemas (en macOS, además, un binario modificado
  necesita `codesign`).

## Empaquetar para otro sistema operativo

`release.yml` publica `nexa` para Linux x64, Windows x64 y macOS (Intel y Apple Silicon). Un binario de esos
sirve de base para `nexa bundle --runtime` desde otro sistema:

```bash
# desde Linux, un servidor para Windows:
nexa bundle --runtime ./nexa-x86_64-pc-windows-msvc/nexa.exe --out mi-sitio.exe
```

(El `--runtime` entre sistemas **no está probado**.)

## Antes de subir cambios

```bash
cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
npm run typecheck --workspaces --if-present && npm test --workspaces --if-present
node e2e/run-all.mjs --nexa target/debug/nexa
```

Se recomienda además pasar [`actionlint`](https://github.com/rhysd/actionlint) por `.github/workflows/`
(aquí los flujos se validaron con un script propio: YAML, referencias `needs`/`matrix`, formato de `uses`).
