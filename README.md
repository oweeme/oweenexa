# Nexa

Framework frontend **HTML-first, SEO-first y backend-agnóstico**.
Compilador en Rust, aplicación en TypeScript/JSX. Compila páginas a HTML
real (con SEO, JSON-LD, i18n) contra cualquier backend HTTP, y solo envía
JavaScript al navegador para las partes de la página que de verdad son
interactivas — nunca para la página completa.

**📖 [Guía de instalación y uso](docs/GUIA-DE-INICIO.md)** — empezá acá si
es la primera vez que ves Nexa: instalación, tu primer proyecto, sintaxis
de una página, módulos oficiales, islas interactivas, empaquetado para
escritorio/móvil.

**📚 [Referencia completa](docs/REFERENCIA.md)** — cada función, cada
atributo, cada opción de `nexa.toml`, organizado por tema. Consultala
cuando ya conozcas lo básico y necesites la forma exacta de algo puntual.

## ¿Qué es Nexa?

La mayoría de los frameworks frontend parten de "toda la página es una
app de JavaScript" y después agregan SSR/hidratación para recuperar SEO
y rendimiento. Nexa parte al revés: **toda página es HTML real desde el
primer byte**, y el JavaScript es un añadido explícito, nodo por nodo,
solo donde el desarrollador lo pide.

Eso hace que Nexa sea especialmente bueno para sitios donde el SEO y el
tiempo de carga importan de verdad — catálogos, landings, blogs,
e-commerce — pero sin renunciar a tener partes genuinamente interactivas
(un dashboard, un carrito, un formulario con validación) cuando hacen
falta, en el mismo proyecto.

## Ventajas

- **Cero JavaScript por defecto.** Una página sin interacción se compila
  a HTML puro, sin runtime, sin hidratación, sin bundle vacío "por las
  dudas". El JS aparece únicamente cuando la página lo necesita, y solo
  el que necesita.
- **Activación progresiva por nodo.** Cada elemento interactivo declara
  su propia estrategia (`interaction`, `visible`, `idle`, `load`,
  `manual`): un botón bajo el pliegue no carga su handler hasta que
  entra en pantalla; uno crítico puede cargar de inmediato.
- **SEO real, no un plugin.** `seo`/`schema` por página generan
  `<title>`, meta tags, Open Graph y JSON-LD válidos; `nexa build`
  genera `sitemap.xml` y `robots.txt`, y un SEO Analyzer avisa en el
  build si falta algo (`title`, `alt`, etc.) sin bloquear el deploy.
- **Islas interactivas para lo que sí necesita un framework real.** Un
  dashboard o un carrito complejo puede montarse con `@nexa/reactivity`
  a mano, o con un componente **Vue real** (u otro framework) vía un
  adaptador delgado — sin que el resto del sitio deje de ser HTML-first,
  y sin reescribir un proyecto Vue existente para meterlo en Nexa.
- **Backend-agnóstico de verdad.** `load()` hace un `GET` HTTP a
  cualquier backend (PHP, Go, Node, lo que ya tengas) y le pasa el
  resultado a la página — Nexa no impone su propio backend ni su propio
  ORM.
- **Rutas dinámicas pre-renderizables.** Una ruta como `[slug].tsx`
  puede declarar `paths` y salir del build como HTML estático real, sin
  servidor Nexa corriendo en producción — o servirse al vuelo si el
  espacio de parámetros no se puede enumerar de antemano.
- **PWA declarativo.** `nexa add pwa` + un bloque `[pwa]` en `nexa.toml`
  generan un manifest y un service worker reales, con reglas de caché
  (`cache-first`/`network-first`/`stale-while-revalidate`) por prefijo
  de ruta — sin escribir el service worker a mano.
- **Cache-busting real.** Cada JS/CSS que genera `nexa build`
  (`nexa-runtime.js`, los chunks de cada nodo interactivo, `nexa-ui.css`)
  lleva un hash de su propio contenido en el nombre de archivo — un
  redeploy nunca sirve JS/CSS viejo desde el caché de un visitante, y
  `nexa add nginx` puede recomendar caché `immutable` de un año para esos
  archivos sin que sea una promesa vacía.
- **Optimización de imágenes automática.** Cualquier `<img>` estático se
  reescribe a `<picture>` con variantes AVIF reales en varios anchos, sin
  que el desarrollador toque nada.
- **Presupuestos de rendimiento que bloquean el build.** `nexa.toml`
  puede declarar límites de JS/CSS/imagen por página; si una página se
  pasa, `nexa build` falla — no es un aviso más.
- **i18n con `hreflang` real.** Traducciones por locale, resueltas tanto
  en el cuerpo de la página como en `seo`/`schema`, con los
  `<link rel="alternate" hreflang="...">` generados automáticamente.
- **Empaquetado nativo desde el mismo proyecto.** `nexa add tauri`
  genera un binario de escritorio real; `nexa add capacitor` + Gradle
  genera un APK real — mismo código fuente que la versión web.
- **Sin dependencias ocultas.** `nexa.toml`/`nexa.lock` reemplazan
  `package.json`/npm para los módulos oficiales de Nexa: no hay
  `node_modules` de por medio para usar `@nexa/ui`, `@nexa/forms` o
  `@nexa/platform`.

## Instalación rápida

Instalación completa (clonar, compilar, agregar al `PATH`) en
**[docs/GUIA-DE-INICIO.md](docs/GUIA-DE-INICIO.md)**. Con `nexa` ya
instalado:

```bash
nexa create hello
cd hello
nexa build

cat dist/index.html
```

## Uso

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
reemplaza sola por la versión que sí compila. En la esquina de la página
verás un panel flotante (`@nexa/devtools` — solo en `nexa dev`, nunca en
`build`/`preview`) con la clasificación de nodos de la página actual, su
manifiesto de activación completo, y sus avisos SEO/paquetes, todo en
vivo.

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
pongas en `public/vendor/` (o una URL de CDN). El chunk antepone
`import { stripe } from "stripe";`, y el import map de `<head>` lo
resuelve. Ver `examples/oweeme-shop` (con `packages/stripe`, un
envoltorio real sobre `@stripe/stripe-js`) para la integración completa,
verificada con un navegador real cargando el SDK real de Stripe desde su
CDN.

### Islas interactivas: un dashboard/tablero en el mismo proyecto

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
— nunca abre ni interpreta el módulo al que apunta el specifier. El
servidor solo renderiza el fallback (los hijos del `<div>`, contenido
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

### Desarrollo de los paquetes de TypeScript

Workspace de npm, una sola instalación para todos:

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
├── stripe/         paquete de comunidad de referencia: un envoltorio
│                    real sobre @stripe/stripe-js — NO es oficial de
│                    Nexa, prueba que `[imports]` funciona con cualquier
│                    paquete de un tercero
├── islands/        initIslands() — lee [data-nexa-island] y monta lo que
│                    su specifier resuelva, con las mismas estrategias
│                    de @nexa/runtime
└── vue-island/     paquete de comunidad de referencia: un adaptador
                     delgado sobre Vue 3 real — la prueba de que una
                     isla puede montar cualquier framework, no solo
                     código escrito con @nexa/reactivity

examples/
└── oweeme-shop/    proyecto de referencia: i18n + load() (backend PHP
                     real) + seo/schema + forms + ui + platform + stripe +
                     dos islas interactivas (una a mano, otra con Vue)
```

Cada crate/paquete está partido en módulos de una sola responsabilidad —
nada vive en un único archivo gigante. La única excepción deliberada es
`reactive.ts` en `packages/reactivity`: signal + effect + scheduler son
un solo algoritmo mutuamente recursivo, y partirlo ahí solo generaría
imports circulares sin ganar claridad.

## Limitaciones conocidas (deliberadas)

- Un solo componente por página, sin composición (`<Otro/>` no
  soportado), sin props, sin condicionales. Es también la razón de que
  `@nexa/ui` se use con `<button class="nx-btn">` y no `<Button>`.
- `load`/`seo`/`schema` son objetos literales estáticos, no funciones —
  sin headers, sin autenticación, sin mutaciones en el servidor.
  Deliberado: Nexa Core no ejecuta JavaScript del desarrollador, solo lo
  analiza.
- Una ruta dinámica (`[slug].tsx`) que declara `paths` sí se
  pre-renderiza a HTML estático real con `nexa build` — pero si NO
  declara `paths`, sigue sirviéndose solo al vuelo vía `nexa preview`,
  nunca como HTML estático. `paths` enumera valores conocidos de
  antemano (vía el backend); no hay forma de pre-renderizar rutas cuyos
  parámetros no se puedan listar así.
- Un chunk de evento solo tiene código real si el handler es `function
  nombre() {}` o `const nombre = () => {}` en el mismo archivo; si viene
  de un import o de un patrón más complejo, cae a un placeholder
  explícito.
- El tree-shaking de `@nexa/ui` solo ve clases `class="..."` estáticas
  (no `class={expr}`), y es a nivel de sitio completo (`nexa build` junta
  todas las páginas), no por página individual.
- `initRouter` reemplaza `<body>` completo, no un fragmento más fino: no
  existe todavía un contenedor de página estable. Sí reactiva
  correctamente eventos, formularios e islas de la página de destino, y
  limpia los de la página anterior antes de hacerlo.
- `t(...)` reconoce exactamente un patrón sintáctico: un identificador
  llamado `t` con un único argumento string literal (`t("clave")`). Ni
  interpolación (`t("hola {name}")`), ni pluralización, ni una clave
  dinámica (`t(variable)`) están soportadas todavía.
- Las traducciones son un JSON plano por locale, resuelto de la misma
  forma que `data.*` — no hay fallback automático a otro locale si falta
  una clave: se omite el campo (`seo`) o queda el placeholder
  `<!--nexa:t(clave)-->` (cuerpo JSX), nunca se inventa un valor.
- Una isla nunca se renderiza en el servidor: solo su fallback (los
  hijos que escribas a mano) es HTML real desde el primer byte — el
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
  navegador: sin reglas de validación compuestas/asíncronas (ej. "el
  email no está ya registrado") — eso requeriría ejecutar código del
  desarrollador, fuera del alcance de Nexa Core.
- `nexa dev` no es HMR en sentido estricto: reemplaza `<body>` completo,
  no sustituye un solo componente conservando su estado en memoria — Nexa
  no tiene instancias de componente en el cliente todavía. Lo que sí se
  preserva es el scroll; el valor de un `<input>` que el usuario esté
  escribiendo no sobrevive a un auto-reload.
- La detección de cambios de `nexa dev` es por sondeo (compara el `mtime`
  más reciente bajo `src/` cada 400ms), no un watcher real basado en
  eventos del sistema de archivos.
- `nexa add` no descarga nada de una red: solo declara, en `nexa.toml`/
  `nexa.lock`, un módulo que ya vive embebido en el binario de
  `nexa-cli` (`ui`, `forms`). Un registro real de paquetes de terceros
  con descarga e instalación de verdad está fuera de alcance por ahora.
- El `content_hash` de `nexa.lock` es FNV-1a sobre los bytes embebidos,
  pensado solo para notar cambios de contenido entre versiones del
  binario — no es una firma criptográfica ni protege contra manipulación
  deliberada.
- `@nexa/platform` no incluye un plugin nativo dedicado para
  notificaciones/share/cámara en Tauri (solo en Capacitor, vía sus
  plugins reales) — usa las Web APIs estándar, que el webview de Tauri
  soporta directamente.
- `nexa add capacitor` solo genera `capacitor.config.json` — el proyecto
  nativo `android/`/`ios/` lo genera `npx @capacitor/cli add android/ios`
  (requiere Node + el SDK de Android o Xcode, ninguno de los dos lo
  instala Nexa). El build de Android se verificó de punta a punta (APK
  real compilado con Gradle); iOS no se puede compilar ni verificar en
  Linux (Xcode no corre ahí).
- `@nexa/test` no incluye un "renderer JSON→HTML" en TypeScript — ese
  renderizado es un paso de compilación en Rust (`nexa-renderer`), ya
  cubierto ahí por sus propios tests. `mountChunk()` prueba lo único que
  de verdad corre como JS de un desarrollador: los handlers interactivos.
- El INP de `@nexa/telemetry` es una aproximación (usa la señal de FID,
  ya retirada del estándar), no el algoritmo real de INP (percentil 98
  de todas las interacciones) — ese algoritmo es sustancialmente más
  complejo que lo que cabe en un bundle de un solo archivo sin la
  librería oficial `web-vitals`.
- `maxCSS` compara contra el `nexa-ui.css` compartido de *todo el sitio*,
  no un cálculo por página — una página que usa poco de `@nexa/ui` en un
  sitio que usa mucho verá el mismo número que las demás. Consecuencia
  directa de que el CSS se ensambla a nivel de sitio.
- `[imports]` no descarga ni instala nada — solo declara, en
  `nexa.toml`, a qué URL/archivo resuelve un nombre. Conseguir que ese
  archivo exista (bundlear tu propio paquete de npm, como hace
  `packages/stripe` con `@stripe/stripe-js`) es responsabilidad del
  proyecto, no de `nexa-cli`.
- `examples/oweeme-shop` no es un negocio real en producción — es el
  proyecto de referencia que ejercita la integración completa de
  módulos. Un deployment real, con marca y datos propios, queda fuera de
  lo que este repositorio puede hacer por ti.

Todo esto es intencional: cada limitación es una decisión explícita para
mantener a Nexa Core simple y predecible, no un descuido — ver
**[docs/REFERENCIA.md](docs/REFERENCIA.md)** para el detalle completo de
cada mecanismo, **[docs/GUIA-DE-INICIO.md](docs/GUIA-DE-INICIO.md)** para
empezar desde cero, y `docs/POLITICA-LTS.md` para el versionado del
framework completo.
