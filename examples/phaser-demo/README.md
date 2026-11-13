# phaser-demo — un juego 2D dentro de Nexa

La portada (`src/pages/`) es HTML real con SEO; el juego (`src/app/pages/index.tsx`)
vive en el **Modo App** y usa [Phaser](https://phaser.io).

```bash
npm install
npm run vendor          # empaqueta Phaser a public/vendor/phaser.js (~1,3 MB, ESM)
nexa dev                # http://127.0.0.1:4321  ->  "Jugar"
```

Qué muestra:

- `[imports] phaser = "/vendor/phaser.js"` en `nexa.toml`: así `import Phaser from "phaser"` se
  resuelve en el navegador (import map), sin bundler propio. Ver `docs/VENDORING.md`.
- El juego se monta con `ref` en un `<div>` y se destruye en `onCleanup` al salir de la página.
- **Juego -> UI**: `update()` escribe una señal de Nexa cada fotograma (`Frames: …`).
- **UI <- juego**: un clic sobre el sprite (`pointerdown`) suma a otra señal (`Clics: …`).

Phaser solo se descarga en la ruta del juego: la portada no lo carga.
