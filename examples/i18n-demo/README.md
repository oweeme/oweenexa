# i18n-demo — empieza en español, añade alemán e inglés cuando quieras

Todas las páginas viven bajo `src/pages/[locale]/` y los textos en `src/locales/<idioma>.json`.

```bash
nexa build           # /es, /es/blog, /de, /de/blog, /en, /en/blog  (+ la puerta de idioma en /)
nexa preview
```

**Para simular "solo español al principio"**: borra `src/locales/de.json` y `src/locales/en.json` y
vuelve a compilar: solo existen `/es/...`. Al volver a crearlos, `/de/...` y `/en/...` aparecen solos,
con `hreflang` en todas las páginas.

- Una clave que falta en un idioma **cae al español** (`de.json` y `en.json` de este ejemplo solo
  traducen algunas): puedes traducir poco a poco.
- Como no hay `src/pages/index.tsx`, `nexa build` genera una **puerta de idioma** en `/`: lleva al idioma
  del navegador si el sitio lo tiene, y si no al idioma por defecto (`[i18n] default`, `es`).
  `../../e2e/locale-gate.mjs` lo comprueba (7 comprobaciones).
