# Guía de inicio

Esta es la puerta de entrada para alguien que llega a Nexa por primera
vez — no ha visto el resto del repositorio, no sabe cómo se construyó.
`README.md` es el estado actual del proyecto (qué fase, qué se verificó);
`docs/FASES-DE-CONSTRUCCION.md` es la bitácora de cómo se construyó cada
pieza; esta guía es "cómo lo uso, sin ese contexto".

## Qué es Nexa

Un framework de frontend HTML-first, SEO-first, backend-agnóstico:

- **HTML-first**: cada página compila a HTML real en el servidor
  (`nexa build`/`nexa preview`/`nexa dev`, un binario de Rust) — el
  navegador nunca necesita ejecutar JavaScript para ver el contenido.
- **SEO-first**: título, meta, Open Graph, JSON-LD y `sitemap.xml` se
  declaran como datos (nunca código que Nexa ejecute) y se resuelven de
  verdad al compilar.
- **Backend-agnóstico**: `load()` es un GET HTTP real a la URL que
  declares — Node, Python, PHP, Go, lo que sea, da igual (ver
  `examples/oweeme-shop/backend/index.php` para una prueba concreta con
  PHP).
- **JS mínimo, a propósito**: una página sin nada interactivo no manda ni
  un byte de JavaScript. Lo interactivo se activa por nodo, con la
  estrategia que declares (`interaction` por defecto, o
  `visible`/`idle`/`load`/`manual`).

Lo que **no** es (deliberadamente — ver "Límites conocidos" más abajo):
un framework de componentes con props/composición/estado en el cliente.
Una página es un solo componente. Para partes genuinamente interactivas
y con estado (un chat, notificaciones en vivo, una tabla con orden/
filtro en el cliente, un dashboard completo) existen las **islas**
(ver más abajo) — no reabren la composición, viven aparte.

## Instalación

No hay (todavía) un binario publicado — se compila desde el código
fuente (necesitas [Rust](https://rustup.rs/) instalado, con `cargo` en
tu `PATH`).

```bash
git clone https://github.com/oweeme/oweenexa.git
cd oweenexa      # importante: los comandos de abajo asumen que estás DENTRO de esta carpeta
cargo install --path crates/nexa-cli --root ~/.local
```

Esto compila en modo release (tarda uno o dos minutos la primera vez) e
instala el binario en `~/.local/bin/nexa`. Agregá esa carpeta a tu
`PATH` si todavía no lo está:

```bash
export PATH="$HOME/.local/bin:$PATH"
# agregá la línea de arriba a tu ~/.bashrc o ~/.zshrc para que quede permanente
```

Verificá que funcionó:

```bash
nexa info
# Nexa CLI
# Version: 0.1.0
# Fase actual: 16 — Islas interactivas
```

Si en vez de eso ves algo como `error: ... is not a directory`, casi
siempre es que el comando `cargo install` se corrió desde afuera de la
carpeta `oweenexa/` (el `--path crates/nexa-cli` es relativo a donde
estés parado) — confirmá con `ls` que ves `crates/`, `packages/`,
`README.md` en el directorio actual antes de instalar.

> Si vas a modificar el propio framework (no solo usarlo), agregá
> `--debug` al final de `cargo install` para compilar más rápido — el
> binario resultante es más lento en tiempo de ejecución, así que no es
> lo recomendado para uso normal.

## Tu primer proyecto

```bash
nexa create mi-sitio
cd mi-sitio
nexa dev --port 4321
```

Abre `http://127.0.0.1:4321` — edita `src/pages/index.tsx` y guarda: el
navegador se actualiza solo, sin recargar a mano (`nexa dev`, con un
panel de diagnóstico flotante en la esquina). Cuando el sitio esté listo:

```bash
nexa build          # genera dist/ — HTML + JS/CSS estáticos
nexa preview         # sirve dist/ tal cual (para probar el build de producción)
```

`dist/` es 100% archivos estáticos: cualquier servidor (Node, Nginx,
Cloudflare Pages, lo que sea) lo sirve sin ningún adaptador especial —
las únicas rutas que necesitan algo más que archivos estáticos son las
dinámicas (`[slug]`) que **no** declaran `paths` (ver abajo); esas sí
necesitan `nexa preview`/`nexa dev` renderizándolas al vuelo.

### Pre-renderizar rutas dinámicas (`paths`)

Si conocés de antemano los valores posibles de un segmento dinámico
(qué `slug` existen), declará `paths` junto a `load` — mismo mecanismo
exacto (una URL, un GET real), pero el backend responde con la lista
completa en vez de un solo objeto:

```tsx
// src/pages/products/[slug].tsx
export const paths = { url: "/products" };          // GET real, responde:
// [{"slug": "iphone-17"}, {"slug": "pixel-10"}, ...]

export const load = { url: "/products/:slug" };      // igual que siempre
```

Con esto, `nexa build` genera un `index.html` real para cada entrada
(`dist/products/iphone-17/index.html`, `dist/products/pixel-10/index.html`,
...) — HTML puro, sin depender de que tu backend siga corriendo. Una
ruta con varios segmentos dinámicos (`[locale]/products/[slug].tsx`)
necesita que cada objeto de `paths` traiga **todas** las claves
(`{"locale":"es","slug":"iphone-17"}`). Sin `paths`, la ruta sigue
funcionando exactamente igual que antes — solo que `nexa build` la deja
para `nexa preview`/`nexa dev` en vez de pre-renderizarla.

## Convenciones de una página (`src/pages/**/*.tsx`)

Cada archivo exporta, como mucho, esto (todo opcional salvo el
`export default`):

```tsx
export const load = { url: "/products/:slug" };     // GET real contra NEXA_API_URL

export const seo = {
    title: `${data.name} | Mi Tienda`,               // plantillas con data./params./t() — Fase 6-10
    description: data.description,
    canonical: `/products/${params.slug}`,
};

export const schema = { type: "Product", name: data.name, /* ... */ };  // JSON-LD real

function buy() {                                      // se activa solo con la primera interacción
    cart.add(product.id);
}

export default function ProductPage() {
    return (
        <article>
            <h1>{data.name}</h1>
            <img src={data.image} alt={data.name} />   {/* atributos dinámicos, incluida una plantilla con varias partes */}
            <button onClick={buy}>Comprar</button>
        </article>
    );
}
```

- `data.*` — lo que devolvió `load()`. `params.*` — segmentos de ruta
  (`[slug]`). `t("clave")` — traducción, si la página vive bajo
  `[locale]` (`src/locales/<locale>.json`).
- Nada de esto se **ejecuta**: el compilador lo reconoce como patrón
  sintáctico y lo resuelve él mismo contra datos reales. Por eso
  `load`/`seo`/`schema` no pueden tener lógica arbitraria (sin
  condicionales, sin funciones propias) — son literales, no código.
- Un `onClick={handler}` cuyo `handler` esté declarado como función en el
  mismo archivo se activa de verdad en el navegador (el código se copia
  tal cual a un chunk propio); estrategia de activación configurable con
  `data-nexa-strategy="visible"` (o `idle`/`load`/`manual`).

## Módulos oficiales (`nexa add`)

```bash
nexa add ui        # design tokens + Button/Input/Card/Dialog (class="nx-btn", sin JSX de componentes)
nexa add forms     # validación nativa progresiva (data-nexa-form, data-nexa-error-for)
nexa add platform  # platform.isTauri/isCapacitor/isWeb + notify/storage/share/capturePhoto
nexa add telemetry # Web Vitals + errores — necesita además [telemetry] endpoint en nexa.toml
nexa add pwa       # manifest.webmanifest + service worker real, a partir de [pwa] en nexa.toml
nexa add tauri      # genera src-tauri/ (empaqueta dist/ como app de escritorio)
nexa add capacitor  # genera capacitor.config.json (empaqueta dist/ como app móvil)
```

Ninguno de estos requiere `package.json` ni npm — se registran en
`nexa.toml`/`nexa.lock`. Usarlos sin declararlos con `nexa add` sigue
funcionando (cero configuración); `nexa build` solo avisa
(`NEXA-PKG-*`), nunca bloquea.

### `@nexa/ui` en detalle

No hay componentes JSX (`<Button>`) — son clases CSS reales, con
tree-shaking: `nexa build` solo manda el CSS de lo que tu sitio usa de
verdad, junto (una sola vez) los tokens de diseño.

```tsx
<button class="nx-btn nx-btn-primary">Comprar</button>
<button class="nx-btn nx-btn-outline">Cancelar</button>
<button class="nx-btn" disabled>Deshabilitado</button>

<input class="nx-input" type="email" name="email" />

<div class="nx-card">
    <h2 class="nx-card-title">Producto</h2>
    <p>...</p>
</div>
```

**Dialog** es el único que trae comportamiento real en JS (foco atrapado
dentro del diálogo, `Escape` para cerrar, devuelve el foco al cerrar) —
se activa llamando a `ui.openDialog(el)`/`ui.closeDialog(el)` desde un
handler, igual que cualquier otro paquete con import (`platform.share`,
`stripe.load`):

```tsx
function open() {
    ui.openDialog(document.querySelector(".nx-dialog"));
}
function close() {
    ui.closeDialog(document.querySelector(".nx-dialog"));
}

export default function Home() {
    return (
        <main>
            <button class="nx-btn nx-btn-primary" onClick={open}>Abrir</button>

            <div class="nx-dialog" hidden>
                <p>Contenido del diálogo</p>
                <button class="nx-btn" onClick={close}>Cerrar</button>
            </div>
        </main>
    );
}
```

El `<div class="nx-dialog" hidden>` lo escribís vos (Nexa nunca genera
markup que no esté en tu `.tsx`) — `ui.openDialog` le quita el `hidden`
y le pone `role="dialog"`/`aria-modal="true"`, `ui.closeDialog` se lo
devuelve. `ui` no hace falta declararlo en `[imports]` ni en `nexa add`:
está disponible siempre, igual que `platform`.

Todas las variables de diseño (`--nx-color-primary`, `--nx-space-4`,
`--nx-radius-md`, etc., ver `packages/ui/src/tokens.css`) son custom
properties normales de CSS — sobreescribilas en tu propio `:root` si
querés otra paleta, sin tocar `@nexa/ui`.

### `@nexa/forms` en detalle

Progressive enhancement sobre la validación nativa del navegador — el
HTML solo, sin JS, ya valida (`required`, `type="email"`, `minlength`,
`pattern`...). `@nexa/forms` añade: mensajes de error visibles donde vos
digas, y clases CSS de estado.

```tsx
<form data-nexa-form action="/api/contact" method="post">
    <label for="email">Email</label>
    <input class="nx-input" id="email" name="email" type="email" required />
    <p data-nexa-error-for="email"></p>

    <label for="message">Mensaje</label>
    <input class="nx-input" id="message" name="message" type="text" required minlength="10"
           data-nexa-message="Contanos un poco más (mínimo 10 caracteres)." />
    <p data-nexa-error-for="message"></p>

    <button class="nx-btn nx-btn-primary" type="submit">Enviar</button>
</form>
```

- `data-nexa-form` en el `<form>` activa todo lo demás — sin esto, el
  formulario es HTML normal (sigue validando de forma nativa, pero sin
  los mensajes/clases de abajo).
- Cada campo necesita `name` — es la clave que conecta el campo con su
  `<p data-nexa-error-for="ese-name">` (puede estar en cualquier parte
  dentro del `<form>`, no tiene que ir justo al lado).
- `data-nexa-message="..."` (opcional, por campo) reemplaza el mensaje
  nativo del navegador ("Rellena este campo") por el tuyo. Sin esto, se
  usa el mensaje nativo tal cual.
- Se valida al perder el foco (`blur`) la primera vez, y en cada
  cambio (`input`) después de eso — no espera al `submit` para avisar,
  pero tampoco molesta antes de que el usuario haya tocado el campo.
- Al enviar: si algo es inválido, bloquea el envío, marca todos los
  campos como "tocados" y pone el foco en el primer campo inválido.
- Clases automáticas por campo: `nx-touched` (ya perdió el foco una
  vez), `nx-dirty` (el valor cambió), `nx-invalid` (inválido ahora
  mismo) — y el atributo `aria-invalid="true"/"false"`, que si además
  usás `class="nx-input"` te da el borde rojo automático de
  `.nx-input[aria-invalid="true"]` sin escribir CSS propio.
- Ningún backend está atado a esto: el `action`/`method` del `<form>`
  son tuyos — `@nexa/forms` solo mejora la experiencia antes de que el
  navegador haga su POST normal.

### Paquetes de un tercero (`[imports]`)

Cualquier paquete JS/npm real (no solo los oficiales) puede engancharse
igual, vía un import map real:

```toml
# nexa.toml
[imports]
stripe = "/vendor/mi-stripe.js"   # o una URL de CDN
```

```tsx
async function checkout() {
    const client = await stripe.load("pk_...");   // se activa solo si el handler usa "stripe."
}
```

Ver `examples/oweeme-shop` para una integración real y funcionando
contra el SDK real de Stripe.js (`packages/stripe`).

## Islas interactivas: chat, notificaciones, tablas, dashboards

Para lo que no es una página SEO — un chat en vivo, una tabla con
filtros en el cliente, un panel administrativo completo — una isla
monta lo que necesites, en el cliente, sin que el compilador de Nexa
sepa nada de lo que hay dentro:

```tsx
<div data-nexa-island="dashboard" data-nexa-strategy="load" data-nexa-props={{ tasks: data.tasks }}>
    <p>Cargando…</p>   {/* fallback: lo único que Rust renderiza aquí */}
</div>
```

```ts
// src/islands/dashboard.island.ts — el contrato es solo esta función:
export default function mount(el: Element, props: Record<string, unknown>): void | (() => void) {
    // escribe esto a mano con @nexa/reactivity, o monta un framework
    // real con un adaptador — @nexa/vue-island hace exactamente eso
    // sobre Vue 3: createApp(Componente, props).mount(el)
}
```

- El specifier (`"dashboard"`) se declara en `nexa.toml [imports]`, el
  mismo mecanismo que un paquete de un tercero.
- El fallback (los hijos del `<div>`) es lo único que existe en el HTML
  servido — real, pero no indexable como "la" isla en sí, así que una
  isla es para lo que **no** necesita ser superficie SEO (un dashboard,
  un chat) — el contenido que sí importa para SEO (artículos, perfiles,
  catálogo) sigue siendo una página Nexa normal.
- Estrategia de activación configurable (`data-nexa-strategy`), igual
  que un `onClick` — por defecto `visible` (no `interaction`: una isla
  no tiene "el" evento obvio de un botón).
- Ver `examples/oweeme-shop` para dos islas reales funcionando: un
  filtro de productos escrito a mano y un panel con un componente
  **Vue 3 real** montado vía `@nexa/vue-island` — la prueba de que un
  panel tipo Trello/SDLC que ya tengas en Vue puede vivir en el mismo
  proyecto Nexa, sin reescribirlo.

## PWA (`nexa add pwa`)

```bash
nexa add pwa
```

Esto agrega un `[pwa]` real a `nexa.toml`, con el nombre del proyecto ya
puesto. Completalo:

```toml
[pwa]
name = "Mi App"
themeColor = "#2563eb"
icon = "/icon-512.png"      # PNG cuadrado real en public/, idealmente 512x512

[pwa.cache]
"/assets" = "cache-first"              # JS/CSS con nombre de archivo estable
"/api" = "network-first"               # datos: intenta la red, cae al cache si no hay
"/" = "stale-while-revalidate"         # páginas: sirve cache al toque, actualiza atrás
```

`nexa build` genera `dist/manifest.webmanifest` y `dist/sw.js` de verdad
— no hay nada que escribir a mano. Cada página lleva automáticamente
`<link rel="manifest">` y el registro del service worker. Un prefijo sin
regla en `[pwa.cache]` (o sin `[pwa.cache]` en absoluto) cae a
`network-first`, el default más seguro. Verificado en Chromium real:
con la red completamente cortada después de una segunda visita, la
página sigue cargando con contenido real desde el cache.

## Imágenes optimizadas automáticamente

No hay que hacer nada — `nexa build` ya optimiza cualquier `<img
src="/algo.jpg">` (o `.jpeg`/`.png`) con un `src` estático de más de
480px de ancho:

```tsx
<img src="/img/hero.jpg" alt="Portada" />
```

se convierte, en `dist/`, en:

```html
<picture>
  <source type="image/avif" srcset="/img/hero-480.avif 480w, /img/hero-768.avif 768w, /img/hero-1280.avif 1280w, /img/hero-1920.avif 1920w" sizes="100vw">
  <img src="/img/hero.jpg" alt="Portada"
       srcset="/img/hero-480.jpg 480w, /img/hero-768.jpg 768w, /img/hero-1280.jpg 1280w, /img/hero.jpg 1920w"
       sizes="100vw" width="1920" height="1080" loading="lazy">
</picture>
```

- El navegador usa el AVIF si lo soporta (todos los navegadores
  modernos) — en una foto real de 1920x1080 probada de verdad, 51 KB en
  vez de 158 KB (68% más chico), y hasta 90% en los anchos menores.
  Verificado en Chromium real: con esto, el navegador descarga
  *únicamente* el AVIF del ancho que necesita — nunca el JPEG de
  respaldo, salvo en un navegador sin soporte AVIF.
- `width`/`height` se completan solos si no los pusiste — ayuda a
  Core Web Vitals (evita layout shift), y `loading="lazy"` también se
  agrega si no lo declaraste.
- Una imagen de 480px de ancho o menos no vale la pena tocarla — se deja
  tal cual. Un `src={expr}` dinámico tampoco se toca (no hay forma de
  saber en tiempo de build qué archivo es).
- Esto tiene un costo real de tiempo de build (codificar AVIF no es
  gratis) — una foto de 1920x1080 tarda unos segundos. Si tu proyecto
  tiene muchas fotos, esperá que `nexa build` tarde más que antes.

## Presupuestos de rendimiento

```toml
[performance]
maxInitialJS = 15000   # bytes
maxCSS = 20000
maxImage = 200000       # por imagen individual, no la suma
```

Con esto declarado, `nexa build` **falla** (no solo avisa) si una página
se pasa. Ausente por defecto.

## Empaquetado más allá de la web

- **Escritorio**: `nexa add tauri` + `cd src-tauri && cargo build` (o
  `cargo tauri dev`/`build` con `@tauri-apps/cli` instalado, para
  recarga en caliente e instaladores empaquetados).
- **Móvil**: `nexa add capacitor` + `npx @capacitor/cli add
  android`/`ios` + `./gradlew assembleDebug` (Android) — necesita el SDK
  correspondiente, que Nexa no instala.

## Límites conocidos (resumen — la lista completa, con el porqué de cada
uno, vive en `README.md`)

- Un componente por página: sin `<Otro/>`, sin props, sin composición.
- `load`/`seo`/`schema` son literales estáticos, nunca funciones.
- Una ruta dinámica se pre-renderiza con `nexa build` solo si declara
  `paths` (Fase 17) — sin eso, sigue sirviéndose al vuelo con `nexa
  preview`/`nexa dev`.
- `nexa dev` recarga reemplazando `<body>` completo — no hay estado de
  componente en memoria que preservar (Nexa no tiene ese modelo), así
  que un valor de `<input>` sin guardar no sobrevive a un auto-reload.

## Dónde seguir

- `docs/FASES-DE-CONSTRUCCION.md` — el roadmap completo, fase por fase,
  con lo que se verificó de cada una.
- `docs/POLITICA-DE-VERSIONES.md` — Stable/Experimental/Internal, por
  módulo.
- `docs/POLITICA-LTS.md` — la política de versionado del framework como
  un todo, y el compromiso sobre codemods futuros.
- `examples/oweeme-shop` — un proyecto de referencia usando la mayoría
  de estas piezas juntas.
