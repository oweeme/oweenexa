# Vendoring de paquetes npm para `[imports]`

`nexa.toml [imports]` (Fase 15) solo declara un nombre y una URL:

```toml
[imports]
stripe = "/vendor/nexa-stripe.js"
```

`nexa build` genera un `<script type="importmap">` con eso y confía en
que `/vendor/nexa-stripe.js` exista como un archivo ESM real, servible
tal cual — Nexa no tiene bundler ni resuelve `node_modules` en tiempo
de build. Conseguir que ese archivo exista ("vendoring") es
responsabilidad del proyecto. Esta guía documenta el proceso repetible,
para no reinventarlo cada vez.

## Por qué hace falta un paso de bundling

Un paquete de npm típico:

- puede estar en CommonJS, no ESM;
- puede tener sus propias dependencias (`node_modules` anidado);
- puede estar repartido en varios archivos.

Un import map de navegador solo entiende ESM real y specifiers
resueltos — no sabe nada de `node_modules`. El paso de bundling
(`esbuild --bundle`) convierte todo eso en **un único archivo ESM
autocontenido**, sin más dependencias que resolver.

Si el paquete *ya* publica su propio build ESM lo bastante autocontenido
(algunos SDKs lo hacen), este paso entero puede no hacer falta — un
CDN de ESM (`https://esm.sh/<paquete>`) o el propio archivo del paquete
alcanzan como valor de `[imports]` directamente. El bundling manual es
para el caso común: un paquete con dependencias propias o en CommonJS.

## El proceso, paso a paso

### 1. Instalar el paquete en algún lado con `npm`

No hace falta que sea dentro del proyecto Nexa — el paquete nunca
corre en Node, solo se usa para generar el bundle. Un directorio
temporal alcanza:

```bash
mkdir -p /tmp/vendor-build && cd /tmp/vendor-build
npm install marked
```

### 2. Un archivo de entrada que re-exporte lo que se necesita

```js
// entry.js
export { marked } from "marked";
```

Si el paquete necesita configuración inicial, o si se quiere exponer
una API más chica que la del paquete completo, este archivo es el
lugar — ver la sección "Con un envoltorio propio" más abajo para el
caso de Stripe, que sí necesita esto.

### 3. Bundlear a un único archivo ESM

```bash
npx esbuild entry.js --bundle --format=esm --target=es2022 --outfile=marked.js
```

Mismo comando (`esbuild --bundle --format=esm`) que ya usa
`build:cli-assets` en el `package.json` raíz del propio Nexa para sus
paquetes internos — no es nada especial de este proceso, es el comando
estándar de esbuild para "un árbol de módulos -> un archivo".

### 4. Copiar el resultado a `public/vendor/`

```bash
cp /tmp/vendor-build/marked.js mi-proyecto/public/vendor/marked.js
```

`public/` se copia tal cual a `dist/` en cada `nexa build` — cualquier
archivo ahí queda servible en esa misma ruta.

### 5. Declararlo en `[imports]`

```toml
[imports]
marked = "/vendor/marked.js"
```

### 6. Usarlo desde un handler

```tsx
function updatePreview() {
    const text = document.querySelector("[data-md-input]").value;
    document.querySelector("[data-md-preview]").innerHTML = marked.parse(text);
}
```

`nexa-activation` detecta el identificador `marked.` en el código
fuente del handler (igual que `platform.`/`ui.`) y antepone el
`import` real automáticamente — no hace falta escribir el `import`
a mano.

## Con un envoltorio propio (`packages/stripe`)

El caso de arriba alcanza cuando el paquete se usa tal cual lo publica
su autor. Cuando hace falta darle forma a su API, o combinarlo con
configuración propia del proyecto, un pequeño paquete wrapper (con su
propio `package.json`, dentro del workspace) es más prolijo que un
`entry.js` suelto — `packages/stripe/src/index.ts` es el ejemplo real:
un envoltorio delgado sobre `@stripe/stripe-js` que expone
`stripe.load(...)`/`stripe.checkout(...)`, bundleado exactamente con el
mismo comando del paso 3 (ver `build:cli-assets` en el `package.json`
raíz) y copiado a `examples/oweeme-shop/public/vendor/nexa-stripe.js`.

Ninguna de las dos formas es "la correcta" — un wrapper vale la pena
cuando hay API propia que agregar (autenticación, defaults del
proyecto); vendoring directo alcanza cuando el paquete ya expone lo
que se necesita.

## Ejemplo real de punta a punta: `marked`

`examples/oweeme-shop/src/pages/[locale]/articles.tsx` usa `marked`
(vendorizado sin wrapper, siguiendo el proceso de arriba) para una
vista previa en vivo de Markdown mientras se escribe un artículo — el
caso concreto que motivó este issue (#21): "`marked`, ya lo usás para
artículos". Corre 100% en el cliente (Nexa nunca ejecuta JavaScript en
build time, así que un paquete como este no puede generar HTML
servidor-renderizado — el cuerpo ya publicado de un artículo seguiría
siendo texto resuelto por `load()`, como cualquier otro campo de
`data.*`; esto es la herramienta de autoría, no el render final).

Para regenerar `public/vendor/marked.js` si se actualiza la versión de
`marked`:

```bash
mkdir -p /tmp/vendor-build && cd /tmp/vendor-build
npm install marked
echo 'export { marked } from "marked";' > entry.js
npx esbuild entry.js --bundle --format=esm --target=es2022 --outfile=marked.js
cp marked.js <ruta-del-proyecto>/public/vendor/marked.js
```

Verificado de punta a punta con un navegador real (Playwright): el
`<textarea>` trae su valor por defecto como HTML real (sin JS), y
escribir Markdown ahí actualiza la vista previa con el HTML que
produce `marked.parse(...)` de verdad (`<h1>`, `<em>`, `<strong>`
reales, no simulados).

## Fuera de alcance (explícito)

Esto es una guía y un proceso repetible, no un registro/paquete
automático de terceros (`nexa add <paquete-de-npm-cualquiera>`) — eso
es un problema más grande (resolución de versiones, actualizaciones,
un catálogo curado) y queda fuera de alcance por ahora.
