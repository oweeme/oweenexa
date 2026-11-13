# e2e — pruebas de punta a punta

Todo con **un solo comando** (Linux, con Chrome y Python 3):

```bash
cargo build --release -p nexa-cli
cd e2e && npm install && cd ..
node e2e/run-all.mjs --nexa target/release/nexa      # 160 comprobaciones, ~90 s
```

Por cada ejemplo, `run-all.mjs` levanta su API, compila, lo sirve con `nexa preview` **y** con el ejecutable
de `nexa bundle`, ejecuta las pruebas en un Chrome real y limpia todos los procesos. Opciones:
`--only smoke,app,phaser,react,i18n,tutorial,blog` y `--keep-going` (no parar en el primer fallo).
`CHROME=/ruta/al/chrome` si no está en `/usr/bin/google-chrome`.

| Suite | Script | Ejemplo | Comprobaciones |
|---|---|---|---|
| Humo (**Windows, macOS, Linux**) | `smoke.mjs <nexa>` | proyecto recién creado | 18 |
| SPA con login por cookie + proxy | `spa-auth.mjs <url>` | `app-demo` | 13 (×2: preview y ejecutable) |
| Core con componentes y `<For>` anidado | `core-components.mjs <url>` | `app-demo` | 10 (×2) |
| Juego Phaser dentro del Modo App | `phaser.mjs <url>` | `phaser-demo` | 7 |
| React + TypeScript puro en islas | `react-islands.mjs <url>` | `react-islands-demo` | 12 |
| Puerta de idioma | `locale-gate.mjs <url>` | `i18n-demo` | 7 (×2) |
| El proyecto del tutorial (SEO, idiomas, isla, panel SPA) | `tutorial.mjs <url>` | `tutorial-blog` | 23 |
| Contenido enriquecido con ataques | `rich-content.mjs <url>` | `blog-store-demo` | 15 |
| Renderizado bajo demanda, caché, i18n, seguridad | `ssr-on-demand.mjs <url>` | `blog-store-demo` | 25 |

Cada script se puede correr suelto contra un servidor que ya tengas levantado (ver el README de cada ejemplo).

## El humo, en cualquier sistema operativo

`smoke.mjs` solo necesita Node y el binario de `nexa` (sin Python ni Chrome): crea un proyecto, lo compila,
lo empaqueta en **un ejecutable**, lo arranca fuera del proyecto y comprueba rutas, SPA, gzip, `ETag`,
caché, cabeceras y path traversal. Es lo más sensible a cada sistema operativo.

```bash
node e2e/smoke.mjs target/release/nexa          # en Windows: target\release\nexa.exe
```
