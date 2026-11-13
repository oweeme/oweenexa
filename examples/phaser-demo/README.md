# phaser-demo — un juego 2D dentro de Nexa

La portada (`src/pages/`) es HTML real con SEO; el juego (`src/app/pages/index.tsx`)
vive en el **Modo App** y usa [Phaser](https://phaser.io).

```bash
npm install
# (nada más: Nexa empaqueta Phaser solo, ~1,3 MB, a partir de islands/phaser.island.js)
nexa dev                # http://127.0.0.1:4321  ->  "Jugar"
```

Qué muestra:

- `islands/phaser.island.js` re-exporta Phaser: Nexa lo empaqueta solo (esbuild) y lo declara en el import map, así
  `import Phaser from "phaser"` se resuelve en el navegador. Ver `docs/VENDORING.md`.
- El juego se monta con `ref` en un `<div>` y se destruye en `onCleanup` al salir de la página.
- **Juego -> UI**: `update()` escribe una señal de Nexa cada fotograma (`Frames: …`).
- **UI <- juego**: un clic sobre el sprite (`pointerdown`) suma a otra señal (`Clics: …`).

Phaser solo se descarga en la ruta del juego: la portada no lo carga.
