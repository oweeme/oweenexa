# site — la documentación de Nexa (GitHub Pages)

El sitio **se genera** a partir de `../docs/*.md`: una sola fuente de verdad, que también se lee tal cual en
GitHub. Nada de esto hace falta para *usar* Nexa — es solo la web de documentación.

```bash
cd site
npm install
node build.mjs            # genera site/dist/   (falla si hay enlaces internos rotos)
node check-tutorial.mjs   # el tutorial tiene que coincidir con examples/tutorial-blog
node serve.mjs 4700 /oweenexa/    # verlo en http://127.0.0.1:4700/oweenexa/ (simula GitHub Pages)
```

## Qué hace `build.mjs`

- Convierte cada `docs/*.md` en una página con barra lateral, índice de la página, anterior/siguiente, tema
  claro/oscuro y buscador (`search-index.json`, en el cliente, sin servicios externos).
- **Parte `REFERENCIA.md` por secciones** (`##`) en páginas propias y reescribe los enlaces y anclas entre ellas.
- Resalta el código (`highlight.js`). Un bloque con ```` ```tsx title="src/pages/x.tsx" ```` muestra ese título en la barra.
- Enlaces: un `.md` de `docs/` se convierte en su página; una ruta del repositorio (`examples/…`, `crates/…`)
  en un enlace a GitHub. Todos los enlaces son **relativos**, así que el sitio funciona bajo cualquier subdirectorio.
- Comprueba que no haya enlaces ni anclas internas rotas, y que la estructura HTML de cada página sea válida
  (un `<main>` sin escapar dentro de un código en línea, por ejemplo, rompería la página).
- Genera `sitemap.xml`, `robots.txt`, `404.html` y `.nojekyll`.

La portada (`home.mjs`) es bilingüe ES/EN; la documentación está en español.

## Añadir o reordenar páginas

Edita la lista `GROUPS` de `build.mjs` (archivo de `docs/`, ruta y título). Un archivo nuevo en `docs/` que no esté
en la lista no se publica.

## Diseño

`assets/style.css`: paleta «papel» (claro) y «tinta» (oscuro), esquinas de 12 px y ventanas de doble línea con
botones de escama para tarjetas y bloques de código. Tipografía [Inter](https://rsms.me/inter/) (licencia OFL,
paquete `@fontsource/inter`, servida desde el propio sitio). Ninguna petición a servicios externos.

## Publicar

`.github/workflows/pages.yml` construye y despliega en cada cambio de `docs/` o `site/` en `main`.
Una sola vez: **Settings → Pages → Source: GitHub Actions**. La URL queda en
`https://<usuario>.github.io/<repositorio>/` (`SITE_URL` se calcula solo en el flujo).
