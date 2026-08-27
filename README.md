# Nexa

Framework frontend HTML-first, SEO-first y backend-agnóstico. Compilador en
Rust, aplicación en TypeScript/JSX.

**📖 [Guía de instalación y uso](docs/GUIA-DE-INICIO.md)** — empezá acá si
es la primera vez que ves Nexa: instalación, tu primer proyecto,
sintaxis de una página, módulos oficiales, islas interactivas,
empaquetado para escritorio/móvil. Ver también `docs/Arquitectura SEO
Completo Framework.md` para la especificación completa y
`docs/FASES-DE-CONSTRUCCION.md` para el roadmap de construcción por fases.

**Estado actual: las 16 fases originales del roadmap (Fase 0 a Fase 15)
están completas**, más una Fase 16 añadida después, a partir de uso real
del framework en un proyecto propio. El resto de esta sección es un
resumen de lo que ya existe, fase por fase — para aprender a usarlo, la
guía de arriba es el punto de partida.

**Lo más reciente (Fase 16 — Islas interactivas):** el hueco real que
dejaba la Fase 15 — un proyecto con partes públicas/SEO y partes
genuinamente interactivas (un dashboard, un tablero tipo Trello) tenía
que repartirse entre Nexa y otro framework, sin ningún mecanismo para
mezclarlos en un mismo proyecto. Ahora `data-nexa-island="<specifier>"` +
`data-nexa-props={{...}}` marcan un subárbol como punto de montaje de una
isla: el servidor solo renderiza su fallback (contenido real, sin JS,
como siempre en Nexa — Rust nunca parsea ni interpreta lo que hay del
otro lado del specifier, así que esto no reabre la composición de
componentes rechazada desde la Fase 2), y lo interactivo se monta
enteramente en el cliente, con un contrato mínimo
(`export default function mount(el, props)`) que cualquier framework
puede implementar. `packages/islands` es el runtime; `packages/vue-island`
es la prueba de que un componente **Vue 3 real** puede montarse dentro de
un proyecto Nexa sin que el compilador sepa nada de Vue — mismo patrón de
"paquete de comunidad" que `packages/stripe` en la Fase 15. Verificado
con Chromium real: el fallback SSR es contenido real indexable, la
estrategia `visible` no carga nada hasta que el elemento entra en
pantalla, y ambas islas del proyecto de referencia (una escrita a mano
con `@nexa/reactivity`, otra con el componente Vue real) son interactivas
de verdad — ver `examples/oweeme-shop`. Esa misma verificación encontró
un bug real preexistente (un literal numérico entero se serializaba como
`1.0`, no `1`, en JSON-LD/props), corregido con test de regresión.

**Antes (Fase 15):** un mecanismo real de imports de terceros —
`nexa.toml` declara `[imports] stripe = "..."` (una URL de CDN o un
archivo en `public/`), y un handler que use `stripe.algo(...)` hace que
su chunk importe ese nombre "pelado", resuelto por un
`<script type="importmap">` real que `nexa-cli` genera en `<head>`. Es
la generalización de lo que hasta la Fase 13 era un caso especial
cableado a mano solo para `@nexa/platform`. Verificado con un navegador
real (Chromium headless) cargando el SDK real de Stripe.js desde su CDN
real — ver `examples/oweeme-shop`, un proyecto de referencia completo
(i18n + `load()` contra un backend real en **PHP** + SEO/schema + forms +
ui + platform + el import de Stripe, todo junto). Esa misma verificación
con navegador real encontró y corrigió cuatro bugs reales preexistentes:
atributos JSX dinámicos con plantillas de varias partes
(`href={\`/${a}/${b}\`}`) se perdían en silencio; archivos `.js` servidos
desde `public/` llegaban con el `Content-Type` equivocado; y
`@nexa/forms` no marcaba un campo inválido cuando el navegador cancela
`submit` de verdad (con una segunda recursión infinita al arreglarlo,
detectada por el propio test de regresión). `docs/POLITICA-LTS.md` fija
el versionado del framework completo y el compromiso de codemods para
la primera versión mayor que rompa algo (ninguna todavía, sería
prematuro).

**Todo lo anterior, resumido:** `nexa build`/`nexa dev`/`nexa preview`
compilan páginas reales (HTML + SEO + JSON-LD + i18n) contra `load()`
real (cualquier backend HTTP), con activación de JS por nodo
(`interaction`/`visible`/`idle`/`load`/`manual`) y cero JS si no hace
falta. `@nexa/ui` (design tokens + componentes en CSS, tree-shaking real
por sitio), `@nexa/forms` (validación nativa progresiva) y
`@nexa/platform` (`isTauri`/`isCapacitor`/`isWeb` + notify/storage/
share/capturePhoto) son opcionales y de costo cero si no se usan.
`nexa.toml`/`nexa.lock` (`nexa add <módulo>`) reemplazan
`package.json`/npm para todo esto, con una política de estabilidad
Stable/Experimental/Internal por módulo (`docs/POLITICA-DE-VERSIONES.md`).
`nexa dev` recarga sin recargar la página (auto-reload + panel de
diagnóstico `@nexa/devtools`, con errores de compilación legibles en el
propio navegador). `nexa.toml` `[performance]` hace fallar el build de
verdad si una página excede su presupuesto de JS/CSS/imagen.
`@nexa/telemetry` (Web Vitals + errores) y `@nexa/test`
(`mountChunk()`) son opt-in explícito. `nexa add tauri`/`nexa add
capacitor` empaquetan `dist/` como app de escritorio (binario real
compilado y ejecutado) o móvil (APK real compilado con Gradle, con el
SDK de Android instalado).

## Estructura

```
crates/                (Rust — el compilador/toolchain)
├── nexa-ast/         AST interno de Nexa (independiente de oxc)
├── nexa-parser/      TSX -> AST de Nexa (usa oxc_parser internamente)
├── nexa-ir/          Representación intermedia: Classification, DependencyGraph
├── nexa-analyzer/    AST -> IR clasificado (Static/Dynamic/Interactive)
├── nexa-renderer/    IR (+ datos/params reales) -> HTML (atributos dinámicos incluidos)
├── nexa-activation/  IR -> manifiesto de activación + chunks JS (con el
│                      handler real, extraído del código fuente)
├── nexa-router/      src/pages/**/*.tsx -> tabla de rutas + matching
├── nexa-loader/      ejecuta el `load` de una página (sustituye params + GET real)
├── nexa-seo/         resuelve `seo`/`schema` -> <head>, JSON-LD, sitemap.xml,
│                      robots.txt, warnings del SEO Analyzer
├── nexa-ui/          tree-shaking del CSS de @nexa/ui: qué clases `nx-*`
│                      usa de verdad el sitio -> qué CSS se envía
├── nexa-i18n/        carga src/locales/<locale>.json, descubre locales
│                      disponibles, calcula los `hreflang` alternate links
└── nexa-cli/         binario `nexa` — embebe @nexa/runtime, @nexa/router,
                       @nexa/forms y el CSS de @nexa/ui ya compilados (assets/)

packages/               (TypeScript — lo que corre en el navegador; workspace de npm)
├── reactivity/     state(), effect(), scheduler, bindText() — sin Virtual DOM
├── runtime/        initActivation() — lee el manifiesto y activa cada nodo
│                    según su estrategia (interaction/visible/idle/load/manual)
├── router/         initRouter()/initPrefetch() — navegación SPA + prefetch
├── http/           createApi(), query()/mutation() (sobre @nexa/reactivity)
├── ui/              CSS fuente (tokens + Button/Input/Card/Dialog) +
│                    comportamiento accesible del Dialog
├── forms/          initForms() — validación nativa (Constraint Validation
│                    API), errores, clases touched/dirty/invalid
├── dev-client/     initDevClient() — sondea /__nexa_dev__/version y
│                    reemplaza el <body> cuando algo bajo src/ cambió
├── platform/       platform.isTauri/isCapacitor/isWeb + notify/storage/
│                    share/capturePhoto — un solo código para los tres
├── devtools/       initDevtools() — panel de diagnóstico de `nexa dev`
│                    (clasificación, activación, avisos), nunca en build
├── test/           mountChunk() — probar un handler interactivo aislado
├── telemetry/      initTelemetry() — Core Web Vitals + errores, solo si
│                    nexa.toml declara [telemetry] endpoint
├── stripe/         paquete de comunidad de referencia (Fase 15): un
│                    envoltorio real sobre @stripe/stripe-js — NO es
│                    oficial de Nexa, prueba que `[imports]` funciona
│                    con cualquier paquete de un tercero
├── islands/        initIslands() (Fase 16) — lee [data-nexa-island] y
│                    monta lo que su specifier resuelva, con las mismas
│                    estrategias de @nexa/runtime
└── vue-island/     paquete de comunidad de referencia (Fase 16): un
                     adaptador delgado sobre Vue 3 real — la prueba de
                     que una isla puede montar cualquier framework, no
                     solo código escrito con @nexa/reactivity

examples/
└── oweeme-shop/    proyecto de referencia: i18n + load() (backend PHP
                     real) + seo/schema + forms + ui + platform + stripe +
                     dos islas interactivas (una a mano, otra con Vue)
```

Cada crate/paquete está partido en módulos de una sola responsabilidad
(parser, error, jsx, expr, text, loader, handlers, seo, schema, template,
object_literal, declaration / classification, node, dependency, component /
manifest, strategy, chunk, build / route, segment, scan, matcher / url,
error / navigate, prefetch, fetcher / client, query, mutation / resolve,
head, schema, sitemap, robots, analyzer / registry, scan / assets,
bootstrap, document, pipeline...) — nada vive en un único archivo gigante.
La única excepción deliberada es `reactive.ts` en `packages/reactivity`:
signal + effect + scheduler son un solo algoritmo mutuamente recursivo, y
partirlo ahí solo generaría imports circulares sin ganar claridad.

## Uso

Instala el binario en tu PATH (una vez, y de nuevo cada vez que cambie el
código):

```bash
cargo install --path crates/nexa-cli --root ~/.local --debug
```

(`~/.local/bin` debe estar en tu `PATH` — en este sistema ya lo está).
Alternativa sin instalar nada: usa la ruta completa al binario de
`cargo build`, `./target/debug/nexa`.

```bash
nexa create hello
cd hello
nexa build

cat dist/index.html
```

### `@nexa/ui` con tree-shaking real

```tsx
function contact() {
    console.log("contacto");
}

export default function About() {
    return (
        <main>
            <div class="nx-card">
                <h2 class="nx-card-title">Sobre nosotros</h2>
                <button class="nx-btn nx-btn-primary" onClick={contact}>Contactar</button>
            </div>
        </main>
    );
}
```

`nexa build` genera `dist/assets/nexa-ui.css` con los tokens + **solo**
`.nx-card`/`.nx-btn` (nada de `.nx-input`/`.nx-dialog`, que no se usaron),
y enlaza ese CSS únicamente en las páginas que de verdad usan algo — una
página 100% estática del mismo proyecto no lo enlaza en absoluto.

Nada de esto exige declarar nada — pero si quieres que `nexa.toml`
refleje qué usa el proyecto (y que `nexa build` deje de avisar):

```bash
nexa add ui
# Instalado @nexa/ui v0.1.0 (stable).
#   Design tokens + Button/Input/Card/Dialog en CSS, con tree-shaking real.
# Registrado en nexa.toml y nexa.lock — sin package.json, sin npm.
```

### Página de producto completa: datos, SEO y schema.org reales

```tsx
// src/pages/products/[slug].tsx
export const load = { url: "/products/:slug" };

export const seo = {
    title: `${data.name} | Oweeme`,
    description: data.description,
    canonical: `/products/${params.slug}`,
    openGraph: { title: data.name, image: data.image }
};

export const schema = {
    type: "Product",
    name: data.name,
    description: data.description,
    image: data.image,
    offers: {
        type: "Offer",
        price: data.price,
        priceCurrency: "USD",
        availability: "https://schema.org/InStock"
    }
};

function buy() {
    cart.add(product.id);
}

export default function ProductPage() {
    return (
        <article>
            <h1>{data.name}</h1>
            <img src={data.image} alt={data.name} />
            <button class="nx-btn nx-btn-primary" onClick={buy}>Comprar</button>
        </article>
    );
}
```

```bash
NEXA_API_URL="http://localhost:8080" NEXA_SITE_URL="https://tu-sitio.com" nexa preview

curl http://127.0.0.1:4321/products/iphone-17   # <title>, meta, canonical, OG, JSON-LD, datos reales
curl http://127.0.0.1:4321/products/no-existe   # HTTP 404 real si tu backend devuelve 404
curl http://127.0.0.1:4321/sitemap.xml
curl http://127.0.0.1:4321/robots.txt
```

### Servidor de desarrollo con auto-reload

```bash
nexa dev --port 4321
```

Edita cualquier `.tsx` bajo `src/pages/` y guarda: la próxima petición ya
sirve el cambio (nunca hay un `dist/` viejo de por medio), y si tienes la
página abierta en el navegador se refresca sola en menos de medio
segundo, sin recargar toda la página ni perder el scroll. Si el archivo
queda con un error de sintaxis, el navegador muestra el mensaje real del
compilador (no un 500 genérico) — y en cuanto lo corriges, la página se
reemplaza sola por la versión que sí compila. En la esquina de la
página verás un panel flotante (`@nexa/devtools` — solo en `nexa dev`,
nunca en `build`/`preview`) con la clasificación de nodos de la página
actual, su manifiesto de activación completo, y sus avisos SEO/paquetes,
todo en vivo.

### Presupuestos de rendimiento reales

```toml
# nexa.toml
[performance]
maxInitialJS = 15000   # bytes
maxCSS = 20000
maxImage = 200000      # por imagen, no la suma
```

```bash
nexa build
# Presupuesto de rendimiento excedido (1 página(s)):
#   /productos: maxInitialJS usa 18204 bytes, supera el presupuesto maxInitialJS de 15000 bytes
# Error: el build no cumple el presupuesto de rendimiento declarado en nexa.toml
```

Ausente por defecto (cero fricción); declarado, **bloquea el build de
verdad** si una página se pasa — no es un aviso más.

### Telemetría opt-in de verdad

```toml
# nexa.toml — sin esto, ni un byte de @nexa/telemetry llega al navegador
[telemetry]
endpoint = "/api/telemetry"
```

Con eso declarado, cada página envía Core Web Vitals (LCP, CLS, INP
aproximado) y errores sin capturar a ese endpoint en cuanto se miden —
nunca la URL completa, cookies, ni identificadores de usuario.

### Un mismo handler, tres plataformas: `@nexa/platform`

```tsx
function shareThis() {
    platform.share({ title: "Nexa", url: "https://example.com" });
}

export default function Home() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <button onClick={shareThis}>Compartir</button>
        </main>
    );
}
```

`nexa build` detecta `platform.` en el handler y antepone
`import { platform } from "platform";` al chunk generado — un specifier
"pelado" que resuelve un import map real que `nexa-cli` declara en
`<head>` (`{"imports": {"platform": "/assets/nexa-platform.js"}}`),
automático, sin tocar nada más. En Capacitor nativo llama al plugin real
(`@capacitor/share`); en web y en una app Tauri usa la Web Share API
estándar.

```bash
nexa add tauri
cd src-tauri && cargo build     # binario de escritorio real

nexa add capacitor
npm init -y && npm install @capacitor/core @capacitor/android @capacitor/cli
npx @capacitor/cli add android   # requiere el SDK de Android (ANDROID_HOME)
cd android && ./gradlew assembleDebug   # APK real en app/build/outputs/apk/debug/
```

### Un paquete de un tercero, con el mismo mecanismo (`[imports]`)

```toml
# nexa.toml
[imports]
stripe = "/vendor/nexa-stripe.js"
```

```tsx
async function checkout() {
    const client = await stripe.load("pk_test_...");
    // ...
}
```

`stripe` no es un módulo oficial de Nexa — es cualquier archivo que tú
pongas en `public/vendor/` (o una URL de CDN). El mismo mecanismo que usa
`platform` internamente (Fase 13) se generalizó en la Fase 15 para
funcionar con cualquier nombre que declares: el chunk antepone
`import { stripe } from "stripe";`, y el import map de `<head>` lo
resuelve. Ver `examples/oweeme-shop` (con `packages/stripe`, un
envoltorio real sobre `@stripe/stripe-js`) para la integración completa,
verificada con un navegador real cargando el SDK real de Stripe desde su
CDN.

### Islas interactivas (Fase 16): un dashboard/tablero en el mismo proyecto

```toml
# nexa.toml
[imports]
dashboardIsland = "/vendor/dashboard-island.js"
```

```tsx
<div data-nexa-island="dashboardIsland" data-nexa-strategy="load" data-nexa-props={{ tasks: data.tasks }}>
    <p>Cargando panel…</p>
</div>
```

```ts
// src/islands/dashboard.island.ts — cualquier framework, con este contrato:
export default function mount(el: Element, props: Record<string, unknown>): void | (() => void) {
    // montar lo que sea aquí — @nexa/reactivity a mano, o un adaptador
    // como @nexa/vue-island (createApp(Component, props).mount(el))
}
```

Rust solo ve `data-nexa-island="dashboardIsland"` como un `Element` más
— nunca abre ni interpreta el módulo al que apunta el specifier, así que
esto no reabre la composición de componentes rechazada desde la Fase 2.
El servidor solo renderiza el fallback (los hijos del `<div>`, contenido
real como cualquier otro nodo de Nexa); el montaje real ocurre 100% en
el cliente, con la misma estrategia de activación que ya usan los
eventos (`interaction`/`visible`/`idle`/`load`/`manual` — por defecto
`visible`, no `interaction`: una isla no tiene "el" evento obvio de un
`onClick`). Ver `examples/oweeme-shop` para dos islas reales: una
escrita a mano con `@nexa/reactivity`, y un dashboard con un componente
**Vue 3 real** montado vía `@nexa/vue-island` — la prueba de que un
proyecto Vue existente puede convivir con las páginas públicas de Nexa
sin reescribirse.

### Formulario con validación nativa + i18n con `hreflang`

```tsx
// src/pages/contact.tsx
export default function Contact() {
    return (
        <form data-nexa-form>
            <label for="email">Email</label>
            <input id="email" name="email" type="email" required />
            <p data-nexa-error-for="email"></p>
            <button class="nx-btn nx-btn-primary" type="submit">Enviar</button>
        </form>
    );
}
```

```tsx
// src/pages/[locale]/index.tsx  ->  /es, /en, ...
export const seo = {
    title: t("home.title"),
    canonical: `/${params.locale}`
};

export default function Home() {
    return (
        <main>
            <h1>{t("home.title")}</h1>
            <p>{t("home.subtitle")}</p>
        </main>
    );
}
```

```json
// src/locales/es.json
{ "home": { "title": "Bienvenido a Oweeme", "subtitle": "Tu tienda de confianza" } }
```

`initForms()` solo se inyecta si la página tiene un `<form
data-nexa-form>` de verdad (mismo costo cero que `@nexa/ui`).
`t("clave")` se reconoce tanto en el cuerpo JSX como dentro de
`seo`/`schema`, y `curl http://127.0.0.1:4321/es` frente a `/en` devuelve
`<title>`, `<h1>` y `<link rel="alternate" hreflang="...">` todos
traducidos.

`nexa build` imprime warnings del SEO Analyzer cuando falta algo
(`[NEXA-SEO-001] falta \`title\` en \`seo\``, `[NEXA-A11Y-001] <img> sin
\`alt\``) — nunca hacen fallar el build, solo avisan. También genera, por
sitio, `dist/sitemap.xml`, `dist/robots.txt` y `dist/assets/nexa-ui.css`
(URLs absolutas en el sitemap si defines `NEXA_SITE_URL`), y por página,
`dist/<ruta>/index.html` + `dist/<ruta>/nexa-manifest.json`, más en
`dist/assets/`: un `<Componente>-<id>.js` por cada nodo interactivo,
`nexa-runtime.js` y `nexa-router.js`.

Para los paquetes de TypeScript (workspace de npm — una sola instalación
para los doce):

```bash
npm install                              # desde la raíz del repo
cd packages/reactivity && npm test && npm run typecheck
cd packages/runtime     && npm test && npm run typecheck
cd packages/router      && npm test && npm run typecheck
cd packages/http        && npm test && npm run typecheck
cd packages/ui          && npm test && npm run typecheck
cd packages/forms       && npm test && npm run typecheck
cd packages/dev-client  && npm test && npm run typecheck
cd packages/platform    && npm test && npm run typecheck
cd packages/devtools    && npm test && npm run typecheck
cd packages/test        && npm test && npm run typecheck
cd packages/telemetry   && npm test && npm run typecheck
cd packages/stripe      && npm test && npm run typecheck
cd packages/islands     && npm test && npm run typecheck
cd packages/vue-island  && npm test && npm run typecheck
```

Si tocas `packages/runtime`, `packages/router`, `packages/forms`,
`packages/dev-client`, `packages/platform`, `packages/devtools`,
`packages/telemetry` o el CSS de `packages/ui`, regenera lo que
`nexa-cli` embebe:

```bash
npm run build:cli-assets   # esbuild -> crates/nexa-cli/assets/*.js
npm run build:ui-assets    # copia el CSS -> crates/nexa-ui/assets/*.css
```

### Probar un handler interactivo en aislamiento con `@nexa/test`

```ts
import { mountChunk } from "@nexa/test";
import activate from "../dist/assets/ProductPage-3.js";

test("el botón agrega el producto al carrito", () => {
    const { el, destroy } = mountChunk(activate, `<button>Comprar</button>`);
    el.click();
    // ...aserciones sobre el efecto del handler...
    destroy();
});
```

## Limitaciones conocidas (deliberadas)

- Un solo componente por página, sin composición (`<Otro/>` no soportado),
  sin props, sin condicionales — seguiría requiriendo rediseñar el modelo
  de componentes, evitado a propósito desde la Fase 2. Es también la razón
  de que `@nexa/ui` se use con `<button class="nx-btn">` y no `<Button>`.
- `load`/`seo`/`schema` son objetos literales estáticos, no funciones — sin
  headers, sin autenticación, sin mutaciones en el servidor. Es deliberado:
  Nexa Core no ejecuta JavaScript del desarrollador, solo lo analiza.
- No hay forma de enumerar qué `slug` existen (`getStaticPaths`): las
  rutas dinámicas con `load()` solo se sirven al vuelo vía `nexa preview`,
  nunca como HTML estático pre-generado por `nexa build`.
- Un chunk de evento solo tiene código real si el handler es `function
  nombre() {}` o `const nombre = () => {}` en el mismo archivo; si viene de
  un import o de un patrón más complejo, cae a un placeholder explícito.
- El tree-shaking de `@nexa/ui` solo ve clases `class="..."` estáticas
  (no `class={expr}`), y es a nivel de sitio completo (`nexa build` junta
  todas las páginas), no por página individual.
- `initRouter` reemplaza `<body>` completo, no un fragmento más fino: no
  existe todavía un contenedor de página estable.
- `t(...)` reconoce exactamente un patrón sintáctico: un identificador
  llamado `t` con un único argumento string literal (`t("clave")`). Ni
  interpolación (`t("hola {name}")`), ni pluralización, ni una clave
  dinámica (`t(variable)`) están soportadas todavía.
- Las traducciones son un JSON plano por locale, resuelto de la misma
  forma que `data.*` — no hay fallback automático a otro locale si falta
  una clave: se omite el campo (`seo`) o queda el placeholder
  `<!--nexa:t(clave)-->` (cuerpo JSX), nunca se inventa un valor.
- Una isla (Fase 16) nunca se renderiza en el servidor: solo su fallback
  (los hijos que escribas a mano) es HTML real desde el primer byte — el
  contenido interno de la isla no existe hasta que el módulo del cliente
  la monta. Como Nexa no tiene bucles/composición (ver el primer punto de
  esta lista), tampoco hay forma de generar ese fallback iterando un
  array (`data.products.map(...)`) — hay que escribirlo a mano, igual que
  cualquier otro contenido de una página Nexa.
- Las props de una isla (`data-nexa-props={{...}}`) se resuelven una sola
  vez, en el momento del render — no hay canal para que el servidor
  empuje props actualizadas a una isla ya montada, ni para que la isla
  escriba de vuelta a `data.*`.
- `@nexa/forms` valida con la Constraint Validation API nativa del
  navegador: sin reglas de validación compuestas/asíncronas (ej. "el email
  no está ya registrado") — eso requeriría ejecutar código del
  desarrollador, fuera del alcance de Nexa Core.
- `nexa dev` no es HMR en sentido estricto: reemplaza `<body>` completo
  (igual que la navegación SPA desde la Fase 6), no sustituye un solo
  componente conservando su estado en memoria — Nexa no tiene instancias
  de componente en el cliente todavía. Lo que sí se preserva es el
  scroll; el valor de un `<input>` que el usuario esté escribiendo no
  sobrevive a un auto-reload.
- La detección de cambios de `nexa dev` es por sondeo (compara el `mtime`
  más reciente bajo `src/` cada 400ms), no un watcher real basado en
  eventos del sistema de archivos — deliberado, para no añadir esa
  dependencia en esta fase.
- `nexa add` no descarga nada de una red: solo declara, en `nexa.toml`/
  `nexa.lock`, un módulo que ya vive embebido en el binario de
  `nexa-cli` (`ui`, `forms`). Un registro real de paquetes de terceros
  (`nexa add @alguien/stripe`, con descarga e instalación de verdad) es
  explícitamente la Fase 15, no esta.
- El `content_hash` de `nexa.lock` es FNV-1a sobre los bytes embebidos,
  pensado solo para notar cambios de contenido entre versiones del
  binario — no es una firma criptográfica ni protege contra
  manipulación deliberada.
- `@nexa/platform` no incluye un plugin nativo dedicado para
  notificaciones/share/cámara en Tauri (solo en Capacitor, vía sus
  plugins reales) — usa las Web APIs estándar, que el webview de Tauri
  soporta directamente. Añadir un plugin propio de Tauri queda para
  cuando se pueda verificar contra una app real.
- `nexa add capacitor` solo genera `capacitor.config.json` — el proyecto
  nativo `android/`/`ios/` lo genera `npx @capacitor/cli add android/ios`
  (requiere Node + el SDK de Android o Xcode, ninguno de los dos lo
  instala Nexa). El build de Android sí se verificó de punta a punta
  (APK real compilado con Gradle); iOS no se puede compilar ni verificar
  en Linux bajo ninguna circunstancia (Xcode no corre ahí).
- El binario de escritorio que genera `nexa add tauri` se verificó
  compilando y ejecutándolo de verdad (el proceso corre sin caerse), pero
  no con una captura de pantalla de la ventana — por respeto a no
  capturar el escritorio real de quien lo construya.
- `@nexa/test` no incluye un "renderer JSON→HTML" en TypeScript — ese
  renderizado es un paso de compilación en Rust (`nexa-renderer`), ya
  cubierto ahí por sus propios tests. `mountChunk()` prueba lo único que
  de verdad corre como JS de un desarrollador: los handlers interactivos.
- El INP de `@nexa/telemetry` es una aproximación (usa la señal de FID,
  ya retirada del estándar), no el algoritmo real de INP (percentil 98
  de todas las interacciones) — ese algoritmo es sustancialmente más
  complejo que lo que cabe en un bundle de un solo archivo sin la
  librería oficial `web-vitals`.
- `maxCSS` compara contra el `nexa-ui.css` compartido de *todo el sitio*
  (Fase 9), no un cálculo por página — una página que usa poco de
  `@nexa/ui` en un sitio que usa mucho verá el mismo número que las
  demás. Consecuencia directa de que el CSS se ensambla a nivel de sitio,
  no una limitación nueva de esta fase.
- `[imports]` (Fase 15) no descarga ni instala nada — solo declara, en
  `nexa.toml`, a qué URL/archivo resuelve un nombre. Conseguir que ese
  archivo exista (bundlear tu propio paquete de npm, como hace
  `packages/stripe` con `@stripe/stripe-js`) es responsabilidad del
  proyecto, no de `nexa-cli`.
- `examples/oweeme-shop` no es un negocio real en producción — es el
  proyecto de referencia de integración que pide la Fase 15. Un
  deployment real, con marca y datos propios, queda fuera de lo que este
  repositorio (o un agente) puede hacer por ti.

Todo esto es intencional: cada fase es un *vertical slice* mínimo que deja
algo ejecutable, sin reabrir las decisiones de las fases anteriores — ver
`docs/FASES-DE-CONSTRUCCION.md` (incluye un addendum con el detalle de qué
pendientes se cerraron y por qué el resto se dejó para más adelante),
`docs/GUIA-DE-INICIO.md` (para alguien sin este contexto) y
`docs/POLITICA-LTS.md` (versionado del framework completo).
