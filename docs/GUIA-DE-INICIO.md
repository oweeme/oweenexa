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
dinámicas (`[slug]`), que `nexa preview`/`nexa dev` renderizan al vuelo.

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
nexa add tauri      # genera src-tauri/ (empaqueta dist/ como app de escritorio)
nexa add capacitor  # genera capacitor.config.json (empaqueta dist/ como app móvil)
```

Ninguno de estos requiere `package.json` ni npm — se registran en
`nexa.toml`/`nexa.lock`. Usarlos sin declararlos con `nexa add` sigue
funcionando (cero configuración); `nexa build` solo avisa
(`NEXA-PKG-*`), nunca bloquea.

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
- Sin `getStaticPaths`: una ruta dinámica no se pre-renderiza con `nexa
  build` — se sirve al vuelo con `nexa preview`/`nexa dev`.
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
