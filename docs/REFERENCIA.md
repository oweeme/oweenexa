# Referencia de Nexa

Esto es lo que Nexa hace y cómo funciona — cada función real, cada
opción de `nexa.toml`, cada atributo, con su forma exacta. No es una
guía paso a paso (esa es [`GUIA-DE-INICIO.md`](GUIA-DE-INICIO.md)) ni la
bitácora de cómo se construyó (esa es
[`FASES-DE-CONSTRUCCION.md`](FASES-DE-CONSTRUCCION.md)).

## Índice

1. [El CLI](#el-cli)
2. [Estructura de un proyecto](#estructura-de-un-proyecto)
3. [Rutas](#rutas)
4. [Layouts compartidos](#layouts-compartidos)
5. [Una página, de arriba a abajo](#una-página-de-arriba-a-abajo)
6. [`load` — traer datos](#load--traer-datos)
7. [`paths` — pre-renderizar rutas dinámicas](#paths--pre-renderizar-rutas-dinámicas)
8. [`seo` y `schema`](#seo-y-schema)
9. [Internacionalización (`t()`, `[locale]`)](#internacionalización-t-locale)
10. [El cuerpo de la página: estático, dinámico, interactivo](#el-cuerpo-de-la-página-estático-dinámico-interactivo)
11. [Islas interactivas](#islas-interactivas)
12. [`@nexa/ui`](#nexaui)
13. [`@nexa/forms`](#nexaforms)
14. [`@nexa/platform`](#nexaplatform)
15. [`@nexa/reactivity`](#nexareactivity)
16. [`@nexa/http`](#nexahttp)
17. [`@nexa/test`](#nexatest)
18. [Paquetes de terceros (`[imports]`)](#paquetes-de-terceros-imports)
19. [Imágenes optimizadas automáticamente](#imágenes-optimizadas-automáticamente)
20. [PWA](#pwa)
21. [Telemetría](#telemetría)
22. [Presupuestos de rendimiento](#presupuestos-de-rendimiento)
23. [Empaquetado nativo (escritorio y móvil)](#empaquetado-nativo-escritorio-y-móvil)
24. [Despliegue (`nginx`)](#despliegue-nginx)
25. [`nexa.toml` — referencia completa](#nexatoml--referencia-completa)
26. [Límites conocidos](#límites-conocidos)

---

## El CLI

```
nexa info              Versión y fase actual del toolchain.
nexa create <nombre>   Crea un proyecto nuevo en ./<nombre>.
nexa build             Compila src/pages/**/*.tsx -> dist/.
nexa preview [--port]  Sirve dist/; lo que falte se renderiza al vuelo. Puerto: 4321.
nexa dev [--port]      Recompila cada página en cada request + auto-reload. Puerto: 4321.
nexa add <módulo>      Declara un módulo oficial en nexa.toml/nexa.lock.
nexa lint              Compila sin escribir dist/; falla si hay avisos SEO/paquetes.
nexa test              Compila (dist/ fresco) y corre el script "test" de package.json.
```

Variables de entorno que el CLI lee:

- `NEXA_API_URL` — base para las URLs de `load`/`paths` (ej.
  `NEXA_API_URL=https://api.miapp.com nexa build`). Sin ella, las URLs
  se piden tal cual (útil si `load.url` ya es absoluta).
- `NEXA_SITE_URL` — si está definida, `sitemap.xml`/`robots.txt` usan
  URLs absolutas; si no, quedan relativas y `nexa build` avisa.

`nexa dev` sirve además un panel de diagnóstico flotante (clasificación
de cada nodo, manifiesto de activación, avisos) y recarga el navegador
solo cuando algo bajo `src/` cambió.

## Estructura de un proyecto

```
mi-sitio/
├── nexa.toml           Manifiesto del proyecto — ver la sección dedicada.
├── nexa.lock           Generado por `nexa add` — no se edita a mano.
├── nexa.config.ts      Config mínima de la app (nombre) — no crece más que esto hoy.
├── src/
│   ├── layout.tsx      Opcional — envuelve toda página con header/footer (ver "Layouts").
│   ├── pages/          Cada .tsx es una ruta — ver "Rutas".
│   │   └── [locale]/   Segmento dinámico especial para i18n (opcional).
│   ├── locales/        <locale>.json — diccionarios planos para t().
│   └── islands/        Convención libre — donde pongas tus *.island.ts.
├── public/              Se copia tal cual a dist/ (imágenes, íconos, vendor/*.js de terceros).
└── dist/                Salida de `nexa build` — 100% archivos estáticos servibles por cualquier servidor.
```

## Rutas

File-based routing sobre `src/pages/`:

| Archivo | Ruta |
|---|---|
| `pages/index.tsx` | `/` |
| `pages/about.tsx` | `/about` |
| `pages/products/index.tsx` | `/products` |
| `pages/products/[slug].tsx` | `/products/:slug` |
| `pages/[locale]/index.tsx` | `/:locale` |
| `pages/[locale]/products/[slug].tsx` | `/:locale/products/:slug` |

Una ruta puede tener varios segmentos dinámicos. `nexa build` compila
directo a HTML cualquier ruta **sin** segmentos dinámicos; una ruta
dinámica solo se pre-renderiza si declara `paths` (ver esa sección) —
si no, se sirve al vuelo con `nexa preview`/`nexa dev`.

## Layouts compartidos

`src/layout.tsx`, opcional — si existe, envuelve el HTML de *toda*
página del proyecto:

```tsx
export default function Layout() {
    return (
        <div class="site-shell">
            <header><nav><a href="/">Inicio</a></nav></header>
            <div data-nexa-slot></div>
            <footer><p>© 2026</p></footer>
        </div>
    );
}
```

Se compila con el mismo parser/analyzer/renderer que cualquier página —
no es un mecanismo nuevo de composición, es un `.tsx` más. Lo único
especial es el splice final: `nexa-cli` busca el único elemento marcado
`data-nexa-slot` (debe estar **vacío** — si tiene hijos, o si no hay
exactamente uno en todo el archivo, el build falla con un mensaje
explícito) y ahí inserta, tal cual, el HTML que ya renderizó la página.
Ni el layout ni la página se conocen entre sí en ningún punto anterior a
ese splice.

Puede usar `{params.x}`/`t(...)` igual que cualquier página (recibe los
mismos `params`/traducciones que la página a la que envuelve). Sus
clases `nx-*` de `@nexa/ui`, si las usa, se suman a las de cada página
para decidir qué entra en `nexa-ui.css`.

### `export const head` — lo único que SÍ llega a `<head>` de verdad

Todo lo demás que devuelve el componente del layout termina dentro de
`<body>` (vía el splice del slot) — un `<link rel="icon">` puesto ahí
queda atrapado en `<body>`, donde no todos los navegadores lo detectan
de forma confiable. Para favicon y hojas de estilo compartidas, declará
`head` en el layout:

```tsx
export const head = {
    icon: "/static/logo.svg",            // <link rel="icon"> — el type se infiere de la extensión
    appleTouchIcon: "/static/icon-180.png", // <link rel="apple-touch-icon">
    stylesheets: ["/static/site.css"]     // un <link rel="stylesheet"> por entrada
};

export default function Layout() { /* ... */ }
```

Mismo patrón que `seo`/`schema`: un objeto literal, nunca código
ejecutado — `nexa-cli` lo resuelve y lo mezcla en el `<head>` real del
documento, antes de `</head>`. Los tres campos son opcionales.

**No soportado esta fase:** un evento interactivo (`onClick`, etc.) o
una isla dentro de `src/layout.tsx` — el build falla explícitamente. La
razón es técnica, no arbitraria: los ids de nodo de un `IrComponent` se
numeran desde 0, y el layout y cada página son `IrComponent`s
compilados por separado — mezclarlos en el mismo manifiesto de
activación produciría ids colisionados. El escape valve es el de
siempre: un nav con estado (ej. un menú hamburguesa) se maqueta como
contenido normal de cada página, o queda para una fase futura con
islas (las islas no usan ids de manifiesto, así que no tienen este
problema de raíz — solo no se habilitó todavía para no mezclar dos
cambios en la misma fase). Tampoco hay layouts anidados por directorio
(un solo `src/layout.tsx` para todo el proyecto, no uno por carpeta de
`src/pages/`).

## Una página, de arriba a abajo

Un archivo de página exporta, todo opcional salvo `export default`:

```tsx
export const load = { url: "/products/:slug" };
export const paths = { url: "/products" };

export const seo = {
    title: `${data.name} | Mi Tienda`,
    description: data.description,
    canonical: `/products/${params.slug}`,
    openGraph: { title: data.name, description: data.description, image: data.image },
    twitter: { card: "summary_large_image" },
};

export const schema = {
    type: "Product",
    name: data.name,
    offers: { type: "Offer", price: data.price, priceCurrency: "USD" },
};

function buy() {
    cart.add(data.id);
}

export default function ProductPage() {
    return (
        <article>
            <h1>{data.name}</h1>
            <img src={data.image} alt={data.name} />
            <button onClick={buy}>Comprar</button>
        </article>
    );
}
```

Nada de esto se **ejecuta** — `nexa-parser` lo reconoce como patrón
sintáctico puro (objetos/plantillas literales, nunca funciones
arbitrarias) y lo resuelve él mismo contra datos reales en tiempo de
build/request. Por eso `load`/`paths`/`seo`/`schema` no pueden tener
lógica: sin condicionales, sin funciones propias, sin `.map()`.

Dentro del JSX tenés disponibles, sin importar nada:

- `data.*` — lo que devolvió `load()`.
- `params.*` — los segmentos capturados de la ruta (`params.slug`,
  `params.locale`).
- `t("clave")` — traducción (ver la sección de i18n).
- Un `onClick={handler}` donde `handler` es una `function` o `const...
  = () => {}` de nivel superior en el mismo archivo — se activa de
  verdad en el navegador.

## `load` — traer datos

```tsx
export const load = { url: "/products/:slug" };
```

Un GET HTTP real contra `NEXA_API_URL + url` (con `:params` sustituidos
por los valores reales de la ruta). El backend puede estar escrito en
cualquier lenguaje — Nexa solo espera JSON de vuelta. Un 404 del
backend se convierte en un 404 real de la página (o, en `nexa build`,
en un error claro: no se puede generar HTML estático para un dato que
no existe).

## `paths` — pre-renderizar rutas dinámicas

```tsx
export const paths = { url: "/products" };
```

Misma forma exacta que `load` — un GET real, pero el backend responde
con un **array** de sets de parámetros:

```json
[{ "slug": "iphone-17" }, { "slug": "pixel-10" }]
```

Con esto, `nexa build` genera un `index.html` real por cada entrada
(`dist/products/iphone-17/index.html`, ...) — HTML puro, sin depender
de que el backend siga corriendo. Una ruta con varios segmentos
dinámicos (`[locale]/products/[slug].tsx`) necesita que cada objeto
traiga **todas** las claves (`{"locale":"es","slug":"iphone-17"}`). Sin
`paths`, la ruta se sigue sirviendo al vuelo, exactamente igual que
antes.

## `seo` y `schema`

```tsx
export const seo = {
    title: "...",            // <title>
    description: "...",      // <meta name="description">
    canonical: "/ruta",      // <link rel="canonical">
    openGraph: {
        title: "...", description: "...", image: "https://...",
    },
    twitter: { card: "summary_large_image" },
};
```

Cada campo es un string, una plantilla (`` `${data.x}` ``), o `t("...")`
— nunca una función. Un campo que no se puede resolver se **omite**
(nunca se rellena con un valor a medias). `nexa-seo` avisa
(`NEXA-SEO-001..`) si falta `title`/`description`/`canonical`, o si una
imagen no tiene `alt` (`NEXA-A11Y-*`) — nunca bloquea el build.

```tsx
export const schema = {
    type: "Product",          // -> "@type": "Product"
    name: data.name,
    offers: { type: "Offer", price: data.price, priceCurrency: "USD" },
};
```

Se resuelve a un `<script type="application/ld+json">` real
(JSON-LD/schema.org). `@type`/`type` es intercambiable; un `@type`
explícito nunca se sobreescribe.

`nexa build` también genera `dist/sitemap.xml` (con URLs absolutas si
`NEXA_SITE_URL` está definida) y `dist/robots.txt` para todas las rutas
estáticas compiladas.

## Internacionalización (`t()`, `[locale]`)

- `src/locales/<locale>.json` — un diccionario plano por idioma
  (`{"home": {"title": "Bienvenido"}}`).
- `t("home.title")` en el JSX, en `seo`, o en `schema` — se resuelve
  contra el diccionario del locale actual (`params.locale`, si la
  página vive bajo `[locale]`). Sin fallback a otro idioma: una clave
  ausente se omite (en `seo`) o queda un placeholder inerte en el
  cuerpo — nunca se inventa un valor.
- `t(...)` reconoce exactamente un patrón: un identificador llamado
  `t` con un único argumento string literal. Sin interpolación
  (`t("hola {name}")`), sin pluralización, sin clave dinámica
  (`t(variable)`).
- `<html lang="...">` usa el locale real de la página (`"es"` si no
  vive bajo `[locale]`).
- `<link rel="alternate" hreflang="...">` se genera solo si la ruta
  vive bajo `[locale]` y hay más de un `src/locales/*.json`.

## El cuerpo de la página: estático, dinámico, interactivo

Cada nodo del JSX se clasifica en tiempo de compilación:

| Clasificación | Cuándo | JS que manda |
|---|---|---|
| `Static` | Texto/HTML fijo | Ninguno |
| `Dynamic` | Depende de `data`/`params`/`t()` | Ninguno — se resuelve en el servidor |
| `Interactive` | Tiene un `onClick`/evento | Solo el handler, activado según su estrategia |
| `Island` | `data-nexa-island` | Todo el módulo de la isla, según su estrategia |

Un `onClick={handler}` se activa con una estrategia — por defecto
`interaction` (espera al primer evento real; ese mismo evento se
reproduce sobre el handler ya activado), configurable con:

```tsx
<button data-nexa-strategy="visible" onClick={track}>...</button>
```

Valores válidos: `interaction` (default), `visible` (entra al
viewport), `idle` (`requestIdleCallback`), `load` (de inmediato),
`manual` (solo si algo más llama a `activateManually(el)`
explícitamente).

## Islas interactivas

```tsx
<div data-nexa-island="dashboard" data-nexa-strategy="load" data-nexa-props={{ tasks: data.tasks }}>
    <p>Cargando…</p>   {/* fallback: lo único que el servidor renderiza acá */}
</div>
```

- `data-nexa-island="<specifier>"` — un string literal, resuelto contra
  `nexa.toml [imports]` (ver esa sección) — Nexa nunca abre ni
  interpreta el módulo al que apunta.
- `data-nexa-props={{...}}` — un objeto literal (mismo mecanismo que
  `schema`), se resuelve a JSON real en el servidor.
- `data-nexa-strategy` — mismos valores que un evento, pero el default
  es `visible` (no `interaction`: una isla no tiene "el" evento obvio
  de un botón).

El módulo al que apunta el specifier debe exportar por defecto:

```ts
export default function mount(
    el: Element,
    props: Record<string, unknown>,
): void | (() => void);
```

El valor de retorno, si es una función, es la limpieza (se llama si la
isla se desmonta explícitamente — no hay limpieza automática al navegar
a otra página de Nexa, ver "Límites conocidos"). Podés escribir el
`mount` a mano con `@nexa/reactivity`, o montar un framework real:

```ts
// packages/vue-island — mismo patrón para cualquier framework
import { createApp } from "vue";
export function defineVueIsland(component: Component) {
    return function mount(el: Element, props: Record<string, unknown>) {
        const app = createApp(component, props);
        app.mount(el);
        return () => app.unmount();
    };
}
```

Un Vue Router (o cualquier router del lado del cliente) montado dentro
de una isla convive sin conflicto con la navegación SPA de Nexa: los
clics internos de tu router quedan `event.defaultPrevented`, y Nexa
respeta eso.

## `@nexa/ui`

`nexa add ui` — clases CSS reales con tree-shaking (`nexa build` solo
manda el CSS de lo que tu sitio usa de verdad):

```tsx
<button class="nx-btn nx-btn-primary">Comprar</button>
<button class="nx-btn nx-btn-outline">Cancelar</button>
<input class="nx-input" type="email" />
<div class="nx-card">
    <h2 class="nx-card-title">Título</h2>
</div>
```

`.nx-input[aria-invalid="true"]` ya tiene borde rojo — se activa solo si
usás `@nexa/forms` en el mismo campo.

**Dialog**, el único componente con JS real:

```tsx
function open() { ui.openDialog(document.querySelector(".nx-dialog")); }
function close() { ui.closeDialog(document.querySelector(".nx-dialog")); }
```

```tsx
<button onClick={open}>Abrir</button>
<div class="nx-dialog" hidden>
    <p>Contenido</p>
    <button onClick={close}>Cerrar</button>
</div>
```

`ui.openDialog(el)` le quita `hidden`, pone `role="dialog"`/
`aria-modal="true"`, atrapa el foco (`Tab`) dentro del diálogo, cierra
con `Escape`, y devuelve el foco al elemento anterior al cerrar.
`ui` está disponible siempre (no hace falta declararlo en
`[imports]`), igual que `platform`.

**Toast / snackbar** (Fase 28) — no confundir con `platform.notify()`
(notificación real del sistema operativo): `ui.notify()` es una
notificación efímera *dentro* de la página, para dashboards y SPAs.

```tsx
function saveOk() {
    ui.notify({ message: "Guardado con éxito", variant: "success" });
}
```

```ts
ui.notify(options: {
    message: string;
    variant?: "info" | "success" | "warning" | "danger"; // default: "info"
    duration?: number; // ms antes de auto-cerrarse; 0 lo desactiva. Default: 4000
}): { close(): void }
```

Varios `ui.notify(...)` se apilan (no se reemplazan entre sí), cada uno
con `role="status"`/`aria-live="polite"`, y un botón para cerrarlo antes
de tiempo. A diferencia de Button/Input/Card/Dialog, el HTML de un
toast no lo escribe el desarrollador — lo crea `notify()` en el
momento, así que su CSS no pasa por el tree-shaking a nivel de sitio:
se inyecta una sola vez, en runtime, la primera vez que se llama
`notify()` — cero costo si nunca se usa.

Design tokens (`--nx-color-primary`, `--nx-space-4`, `--nx-radius-md`,
...) son custom properties de CSS normales — sobreescribilas en tu
propio `:root` para otra paleta.

## `@nexa/forms`

`nexa add forms` — progressive enhancement sobre la Constraint
Validation API nativa del navegador (el HTML solo, sin JS, ya valida):

```tsx
<form data-nexa-form action="/api/contact" method="post">
    <input class="nx-input" name="email" type="email" required />
    <p data-nexa-error-for="email"></p>

    <input class="nx-input" name="message" required minlength="10"
           data-nexa-message="Contanos un poco más." />
    <p data-nexa-error-for="message"></p>

    <button type="submit">Enviar</button>
</form>
```

- `data-nexa-form` en el `<form>` activa todo lo demás.
- Cada campo necesita `name` — conecta el campo con
  `<p data-nexa-error-for="ese-name">` (puede estar en cualquier parte
  dentro del form).
- `data-nexa-message` (opcional) reemplaza el mensaje nativo del
  navegador por uno propio.
- Se valida al perder el foco la primera vez, y en cada cambio después
  de eso.
- Al enviar: si algo es inválido, bloquea el envío, marca todo como
  "tocado" y enfoca el primer campo inválido.
- Clases automáticas por campo: `nx-touched`, `nx-dirty`, `nx-invalid`
  — más `aria-invalid="true"/"false"`.
- El `action`/`method` son tuyos — esto solo mejora la experiencia antes
  del POST normal del navegador.

## `@nexa/platform`

`nexa add platform` — un objeto `platform` disponible siempre (builtin,
como `ui`), con un solo código para Web/Tauri/Capacitor:

```ts
platform.isTauri: boolean
platform.isCapacitor: boolean
platform.isWeb: boolean

await platform.share({ title, url, text? })
// Web: Web Share API. Capacitor: plugin @capacitor/share. Tauri: no soportado (lanza).

await platform.notify({ title, body? })
// Web: Notification API (pide permiso). Capacitor: @capacitor/local-notifications.

await platform.capturePhoto()
// -> { dataUrl }. Web: getUserMedia + canvas. Capacitor: @capacitor/camera.

const storage = platform.storage()
storage.get(key) / storage.set(key, value) / storage.remove(key)
// Web: localStorage. Mismo contrato en cualquier entorno.
```

Cada función lanza un `Error` con mensaje claro (`[nexa/platform] ...`)
si la capacidad no está disponible en el entorno actual — nunca falla
en silencio.

## `@nexa/reactivity`

Sin Virtual DOM — señales que notifican directo a quien las lea. Para
escribir una isla a mano.

```ts
import { state, effect, computed, watch, bindText, type Signal, type ReadonlySignal } from "@nexa/reactivity";

const count = state(0);              // Signal<number> — .value (lee y suscribe), .peek() (lee sin suscribir)
count.value = 1;                     // dispara los efectos que lo leyeron (agrupado por microtask)

const doubled = computed(() => count.value * 2);   // ReadonlySignal<number> — se recalcula solo
doubled.value;                                      // no tiene setter

const stop = effect(() => {
    console.log(count.value);        // corre ya, y de nuevo cada vez que count cambie
});
stop();                              // desactiva el efecto

watch(
    () => count.value,
    (nuevo, viejo) => console.log(nuevo, viejo),
);                                    // como effect, pero NO corre al crearse
```

Encadenar `computed`s funciona (`computed(() => otroComputed.value * 2)`),
con un costo real: cada salto de la cadena es una vuelta más de
microtask para propagarse del todo.

`bindText(node, () => expr)` es lo que el compilador genera
automáticamente para `{data.x}` en el cuerpo de una página — no es algo
que normalmente escribas a mano.

No hay `store`/`resource`/`context` — para eso, la respuesta es montar
un framework real dentro de una isla.

## `@nexa/http`

Sin dependencias — cliente HTTP + capa de datos reactiva, para usar
dentro de una isla o un handler:

```ts
import { createApi, query, mutation, HttpError } from "@nexa/http";

const api = createApi({ baseURL: "https://api.miapp.com" });
await api.get<Producto[]>("/products");
await api.post<Producto>("/products", { name: "..." });
// también: api.put(...), api.delete(...)
// Un status no-2xx lanza HttpError { status, message }.

const users = query({
    key: "users",
    fetch: () => api.get("/users"),
    cachePolicy: "cache-first",   // | "network-first" | "stale-while-revalidate" (default: cache-first)
});
users.data      // Signal<T | undefined>
users.loading   // Signal<boolean>
users.error     // Signal<unknown>
users.refetch() // vuelve a pedir

const createUser = mutation({
    execute: (nombre: string) => api.post("/users", { nombre }),
});
await createUser.execute("Ana");
createUser.data / .loading / .error   // mismos Signal que query()
```

`query`/`mutation` devuelven `Signal`s de `@nexa/reactivity` — se leen
igual (`.value`) dentro de un `effect`/`computed`/una isla.

### Tiempo real: SSE y WebSocket

Mismo espíritu que `query`/`mutation` — un envoltorio delgado sobre las
APIs nativas del navegador (`EventSource`/`WebSocket`), sin protocolo
propio, que expone `Signal`s en vez de que cada proyecto reinvente su
propio wrapper:

```ts
import { connectSSE, connectSocket } from "@nexa/http";

const notifications = connectSSE<{ mensaje: string }>({ url: "/events" });
notifications.data   // Signal<T | undefined> — el último mensaje, parseado como JSON si se puede
notifications.error  // Signal<unknown>
notifications.close();

const chat = connectSocket<{ from: string; text: string }>({ url: "wss://miapp.com/chat" });
chat.status  // Signal<"connecting" | "open" | "closed">
chat.data    // Signal<T | undefined> — el último mensaje recibido
chat.send({ text: "hola" });   // objetos se serializan como JSON; un string se manda tal cual
chat.close();
```

Como cualquier código de `@nexa/http`, esto corre 100% en el cliente —
dentro de una isla, o de un handler de evento normal (via `[imports]`,
igual que `platform`/`stripe`). Nexa Core (Rust) no sabe nada de
WebSocket/SSE ni de ningún protocolo — no hay nada que compilar ni
analizar del lado del servidor para esto, es una librería de cliente
como cualquier otra.

**Fuera de alcance de esta fase:** sin reconexión automática si la
conexión se cae (`status` pasa a `"closed"` y se queda ahí — reconectar
es responsabilidad del proyecto, llamando `connectSSE`/`connectSocket`
de nuevo), sin backoff, sin un canal del lado del servidor en Rust (el
backend real del proyecto es quien implementa el endpoint SSE/WS —
Nexa no trae uno).

## `@nexa/test`

Para probar un chunk de activación aislado, fuera del navegador real:

```ts
import { mountChunk, getEntry, expectEntry } from "@nexa/test";

const manifest = JSON.parse(fs.readFileSync("dist/products/x/nexa-manifest.json"));
expectEntry(manifest, 3, { event: "click", strategy: "interaction" });

// El nombre de archivo del chunk lleva un hash de su contenido (Fase 23,
// cache-busting) — nunca lo hardcodees en el import: resuélvelo leyendo
// `module` de la entrada real del manifiesto.
const entry = getEntry(manifest, 3);
const activate = (await import(`../dist${entry.module}`)).default;

const { el, destroy } = mountChunk(activate, `<button>Comprar</button>`);
el.click();
destroy();
```

`nexa test` (Fase 23) orquesta esto: compila el proyecto (`dist/`
fresco, con los nombres de archivo reales) y después corre el script
`"test"` de `package.json` (`npm test`) — no reemplaza a
`vitest`/`node --test`/etc., solo se asegura de que corran contra un
build consistente. `nexa lint` (Fase 23) compila cada página sin
escribir `dist/` y falla (código de salida distinto de cero) si el SEO
Analyzer o los avisos de paquetes encontraron algo — a diferencia de
`nexa build`, que nunca falla por esto.

## Paquetes de terceros (`[imports]`)

Cualquier paquete JS real (no solo los oficiales) se engancha con el
mismo mecanismo que usan `ui`/`platform` por dentro:

```toml
# nexa.toml
[imports]
stripe = "/vendor/nexa-stripe.js"   # un archivo bajo public/, o una URL de CDN
```

```tsx
async function checkout() {
    const client = await stripe.load("pk_...");   // el chunk detecta el uso de "stripe." y antepone el import
}
```

Solo entra al `<script type="importmap">` del `<head>` lo que la página
de verdad usa — una página sin `stripe.` no lo declara ni lo carga.

## Imágenes optimizadas automáticamente

Sin configuración: `nexa build` optimiza todo `<img src="/foto.jpg">`
(o `.jpeg`/`.png`) estático de más de 480px de ancho.

```tsx
<img src="/img/hero.jpg" alt="Portada" />
```

se convierte en `dist/` en:

```html
<picture>
  <source type="image/avif" srcset="/img/hero-480.avif 480w, .../hero-768.avif 768w, .../hero-1280.avif 1280w, .../hero-1920.avif 1920w" sizes="100vw">
  <img src="/img/hero.jpg" alt="Portada"
       srcset="/img/hero-480.jpg 480w, .../hero-768.jpg 768w, .../hero-1280.jpg 1280w, /img/hero.jpg 1920w"
       sizes="100vw" width="1920" height="1080" loading="lazy">
</picture>
```

- Solo AVIF (no WebP — el encoder disponible en Rust es lossless-only,
  inútil para fotos).
- `width`/`height`/`loading="lazy"` se completan solos si faltaban.
- Un `src={expr}` dinámico no se toca (no hay forma de saber en build
  qué archivo es). Una imagen de 480px o menos tampoco (no vale la
  pena).
- Tiene costo real de tiempo de build — varios segundos por foto
  grande.

## PWA

```bash
nexa add pwa
```

agrega `[pwa]` a `nexa.toml`, ya pre-llenado con el nombre del proyecto:

```toml
[pwa]
name = "Mi App"
shortName = "MiApp"          # opcional — si falta, se recorta `name`
themeColor = "#2563eb"        # opcional
backgroundColor = "#ffffff"   # opcional
display = "standalone"        # opcional, default "standalone"
icon = "/icon-512.png"        # opcional — un PNG cuadrado real en public/

[pwa.cache]
"/assets" = "cache-first"
"/api" = "network-first"
"/" = "stale-while-revalidate"
```

`nexa build` genera `dist/manifest.webmanifest` y `dist/sw.js` reales.
Cada página lleva `<link rel="manifest">` y el registro del service
worker automáticamente. Estrategias válidas en `[pwa.cache]`:
`cache-first`, `network-first`, `stale-while-revalidate` — el prefijo
más específico que matchea gana; sin ninguna regla, cae a
`network-first`.

## Telemetría

```toml
[telemetry]
endpoint = "/api/telemetry"
```

Sin esto, `@nexa/telemetry` nunca se carga — hace falta el `endpoint`
explícito para que tenga sentido. Con él, cada página manda Core Web
Vitals (LCP/CLS/INP) y errores sin capturar a esa URL.

## Presupuestos de rendimiento

```toml
[performance]
maxInitialJS = 15000    # bytes
maxCSS = 20000
maxImage = 200000         # por imagen individual, no la suma
```

Con esto declarado, `nexa build` **falla** (no solo avisa) si una
página se pasa — nombrando exactamente qué se excedió. Ausente por
defecto (no hay presupuesto si no lo pedís).

## Empaquetado nativo (escritorio y móvil)

```bash
nexa add tauri
cd src-tauri && cargo build            # binario de escritorio real

nexa add capacitor
npx @capacitor/cli add android         # necesita el SDK de Android
cd android && ./gradlew assembleDebug  # APK real
```

Ambos son Experimental — generan archivos reales en el proyecto
(`src-tauri/`, `capacitor.config.json`), sin depender de que este
repositorio esté disponible en la máquina donde se usa el binario de
`nexa`.

## Despliegue (`nginx`)

```bash
nexa add nginx
```

genera `deploy/nginx.conf` — gzip, las mismas tres cabeceras de
seguridad que `nexa preview`/`nexa dev` ya mandan
(`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`,
`Referrer-Policy: strict-origin-when-cross-origin`), y un `try_files`
que sirve cualquier ruta estática (incluidas las pre-renderizadas por
`paths`). Trae un bloque comentado de `proxy_pass` para rutas dinámicas
sin `paths`, que necesitan un `nexa preview` corriendo detrás.

El cacheo de `/assets/` tiene dos reglas: los archivos que `nexa build`
genera con hash de contenido en el nombre (`nexa-runtime.<hash>.js`,
chunks de activación, `nexa-ui.<hash>.css`) usan `Cache-Control:
immutable` con `max-age` de un año — seguro de verdad, porque el nombre
cambia si el contenido cambia; cualquier otro archivo bajo `/assets/`
(ej. algo copiado a mano desde `public/assets/`, sin hash) cae a un
`max-age` corto. Sin CSP ni `Permissions-Policy` por defecto — romperían
`[imports]` de terceros y `platform.capturePhoto()` respectivamente si se
adivinaran mal.

## `nexa.toml` — referencia completa

```toml
[project]
name = "mi-sitio"
version = "0.1.0"

[dependencies]
# lo que registra `nexa add <módulo>` — nombre = versión.
ui = "0.1.0"

[imports]
# specifier -> URL real (CDN, o archivo bajo public/). Ver "Paquetes de terceros".
stripe = "/vendor/nexa-stripe.js"

[performance]
maxInitialJS = 15000
maxCSS = 20000
maxImage = 200000

[telemetry]
endpoint = "/api/telemetry"

[pwa]
name = "Mi App"
shortName = "MiApp"
themeColor = "#2563eb"
backgroundColor = "#ffffff"
display = "standalone"
icon = "/icon-512.png"

[pwa.cache]
"/assets" = "cache-first"
```

Todas las secciones son opcionales salvo `[project]`. Ninguna requiere
`package.json` ni npm.

## Límites conocidos

- Un componente por página: sin `<Otro/>`, sin props, sin composición,
  sin bucles (`.map()`) — es deliberado, no un pendiente. Una lista
  dinámica hay que escribirla a mano, o resolverla dentro de una isla.
- `load`/`paths`/`seo`/`schema` son literales estáticos, nunca
  funciones — sin headers, sin autenticación, sin mutaciones en el
  servidor.
- Una ruta dinámica sin `paths` solo se sirve al vuelo (`nexa
  preview`/`nexa dev`), nunca como HTML estático de `nexa build`.
- Un chunk de evento solo tiene código real si el handler es una
  función de nivel superior en el mismo archivo; un patrón más complejo
  cae a un placeholder explícito.
- El tree-shaking de `@nexa/ui` es a nivel de sitio completo, no por
  página individual.
- `initRouter` reemplaza `<body>` completo en cada navegación — no hay
  contenedor de página más fino. Sí reactiva correctamente eventos,
  formularios e islas de la página de destino (Fase 22), y limpia los
  de la página anterior antes de hacerlo.
- `t(...)` no soporta interpolación, pluralización, ni clave dinámica —
  y no hay fallback automático a otro locale si falta una clave.
- `@nexa/forms` valida solo con la Constraint Validation API nativa —
  sin reglas async o entre varios campos a la vez.
- `src/layout.tsx` es único para todo el proyecto (sin anidar por
  directorio) y no admite eventos ni islas propias — ver "Layouts
  compartidos".
- Sin WebSocket/SSE de primera clase — la isla es el mecanismo hoy
  (montar tu propio código o un framework real adentro).
