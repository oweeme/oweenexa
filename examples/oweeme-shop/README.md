# Oweeme Shop — proyecto de referencia (Fases 15-16)

Un sitio real construido con Nexa, usando prácticamente todo lo del
framework junto en un solo lugar coherente: `load()` contra un backend
real, `seo`/`schema` (JSON-LD), i18n (`[locale]`, `t()`, `hreflang`),
`@nexa/ui`, `@nexa/forms`, `@nexa/platform`, un paquete de comunidad de
terceros (`@nexa/stripe`, ver `packages/stripe` en la raíz del repo) vía
el mecanismo de imports de la Fase 15, y dos islas interactivas (Fase 16):
una escrita a mano con `@nexa/reactivity` y otra que monta un componente
Vue 3 real vía `@nexa/vue-island`.

No es un negocio real en producción — es la validación de integración
que pide la Fase 15 (`docs/FASES-DE-CONSTRUCCION.md`), y el punto de
partida si algún día quieres llevar algo parecido a producción de
verdad.

## Correrlo

```bash
# 1. El backend (PHP real — prueba concreta de "backend distinto al
#    original" que pide el criterio de salida de la Fase 15):
php -S 127.0.0.1:8090 backend/index.php
# o, si no tienes PHP instalado pero sí podman/docker:
podman run --rm -p 8090:8090 -v "$(pwd)/backend:/app:Z" -w /app \
  docker.io/library/php:8.3-cli php -S 0.0.0.0:8090 index.php

# 2. El sitio — `nexa preview` sirve desde dist/, así que primero hace
#    falta un build (copia public/vendor/*.js ahí):
NEXA_API_URL="http://127.0.0.1:8090" nexa build
NEXA_API_URL="http://127.0.0.1:8090" nexa preview --port 4950
```

Luego abre `http://127.0.0.1:4950/es` (o `/en`).

## Qué mirar

- `src/pages/[locale]/index.tsx` — home, i18n + `@nexa/platform`
  (compartir) + `@nexa/ui` (tarjeta/botones) + la isla `productFilter`
  (Fase 16): el `<ul>` de productos que ves sin JS es contenido real,
  generado con `<For each={data.products}>` (Fase 30) — un `<li>` real
  por cada producto que devolvió `load()`, sin código a mano ni límite
  de dos productos hardcodeados; las mismas props (`data.products`) se
  le pasan además a la isla para el filtro en el cliente, sin volver a
  pedirlas.
- `src/pages/[locale]/dashboard.tsx` — una página que NO es superficie
  SEO a propósito: monta la isla `dashboardIsland`, un componente **Vue
  3 real** (`src/islands/Dashboard.ts`) vía `@nexa/vue-island` — la
  prueba concreta de que un panel administrativo (tipo SDLC/Trello)
  puede vivir en el mismo proyecto Nexa sin reescribirlo.
- `src/islands/productFilter.island.ts` — isla escrita a mano con
  `@nexa/reactivity` (sin ningún framework externo).
- `src/pages/[locale]/products/[slug].tsx` — `load()` contra el backend
  PHP, `seo`/`schema` de `Product` reales, y el botón de "Comprar" que
  usa `stripe.load(...)` — el paquete de comunidad de referencia.
- `src/pages/[locale]/contact.tsx` — `@nexa/forms` con validación nativa.
- `nexa.toml` — `[dependencies]`, `[performance]` (presupuestos reales),
  y `[imports]` (`stripe`, `productFilter`, `dashboardIsland` — mismo
  mecanismo de las Fases 15 y 16).
- `public/vendor/nexa-stripe.js`, `product-filter-island.js`,
  `dashboard-island.js` — bundles reales (esbuild) de `packages/stripe`,
  `src/islands/productFilter.island.ts` y `src/islands/dashboard.island.ts`
  respectivamente. Regenéralos desde la raíz del repo si tocas esos
  archivos, por ejemplo:
  `npx esbuild examples/oweeme-shop/src/islands/productFilter.island.ts --bundle --format=esm --target=es2022 --outfile=examples/oweeme-shop/public/vendor/product-filter-island.js`
- `backend/index.php` — un backend real en un lenguaje distinto al que
  se usó en las fases anteriores (Node/Python), sin tocar el core de
  Nexa para nada. Incluye `/products` (catálogo completo, para la isla
  del home) además de `/products/:slug`.

## Limitaciones de este ejemplo (no de Nexa)

- Las claves de Stripe en `checkout()` son de ejemplo/no funcionales —
  no se completa un cobro real.
- `[locale]`/`[slug]` son rutas dinámicas: `nexa build` no las
  pre-renderiza (no hay forma de enumerar qué `slug` existen todavía,
  ver limitaciones conocidas del `README.md` raíz) — sírvelas con `nexa
  preview`/`nexa dev`.
