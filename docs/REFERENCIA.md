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

Además: [Recursos: islas y CSS](#recursos-islas-css-y-pasos-propios-fase-72) ·
[Contenido local](#contenido-local-content--un-sitio-sin-backend-fase-73) ·
[Redirecciones, cabeceras y CSP](#redirecciones-cabeceras-y-csp-fase-75) ·
[Docker](#despliegue-en-contenedor-docker) · [Componentes y `<Show>`](#componentes-y-show-fase-62) ·
[Modo App (SPA)](#modo-app-spa) · [Backend, proxy y sesión](#backend-proxy-y-sesión-proxy-createapi-persisted) ·
[Un solo ejecutable (`nexa bundle`)](#un-solo-ejecutable-nexa-bundle). Guías: [Elige tu camino](ELIGE-TU-CAMINO.md) ·
[Tutorial: blog y tienda](TUTORIAL-BLOG-TIENDA.md) · [Vendoring de terceros](VENDORING.md).

---

## El CLI

```
nexa info              Versión y fase actual del toolchain.
nexa create <nombre>   Crea un proyecto nuevo en ./<nombre> (--template app: landing + SPA).
nexa build             Compila src/pages/**/*.tsx (y src/app/) -> dist/. --target desktop: + binario Tauri.
nexa bundle            nexa build + UN ejecutable con servidor, caché y renderizado bajo demanda.
nexa preview [--port]  Sirve dist/; lo que falte se renderiza al vuelo. Puerto: 4321.
nexa dev [--port]      Recompila cada página en cada request + auto-reload. Puerto: 4321.
nexa add <módulo>      Declara un módulo oficial en nexa.toml/nexa.lock (`docker`, `nginx`, `tauri`, …).
nexa lint              Compila sin escribir dist/ (Core y Modo App); falla si hay avisos SEO/paquetes.
nexa test              Compila (dist/ fresco) y corre el script "test" de package.json.
```

Variables de entorno que el CLI lee:

- `NEXA_API_URL` — base para las URLs de `load`/`paths` (ej.
  `NEXA_API_URL=https://api.miapp.com nexa build`). Sin ella, las URLs
  se piden tal cual (útil si `load.url` ya es absoluta).
- `NEXA_SITE_URL` — si está definida, `sitemap.xml`/`robots.txt` usan
  URLs absolutas; si no, quedan relativas y `nexa build` avisa.

Las dos (y cualquier otra) se pueden poner en un archivo **`.env`** de la raíz del proyecto; `.env.local`
(que no se sube a git) manda sobre `.env`, y una variable ya definida en el entorno real manda sobre ambos.
El ejecutable de `nexa bundle` **no** lleva tu `.env`: en producción usa las variables reales del entorno.

`nexa dev` sirve además un panel de diagnóstico flotante (clasificación
de cada nodo, manifiesto de activación, avisos) y recarga el navegador
solo cuando algo bajo `src/` cambió.

## Estructura de un proyecto

```
mi-sitio/
├── nexa.toml           Manifiesto del proyecto — ver la sección dedicada.
├── nexa.lock           Generado por `nexa add` — no se edita a mano.
├── tsconfig.json       Solo para el editor (lo deja `nexa create`) — el compilador no lo lee.
├── nexa-env.d.ts       Tipos para el editor: `data`, `params`, `t`, `<For>`, `<Show>`, @nexa/app, platform.
├── src/
│   ├── layout.tsx      Opcional — envuelve toda página con header/footer (ver "Layouts").
│   ├── pages/          Cada .tsx es una ruta de Nexa Core (HTML con SEO) — ver "Rutas".
│   │   ├── 404.tsx     Opcional — tu página de error 404.
│   │   ├── docs/_layout.tsx   Opcional — layout SOLO de las páginas de esa carpeta (anidado).
│   │   └── [locale]/   Segmento dinámico especial para i18n (opcional).
│   ├── content/        Opcional — Markdown y JSON locales: `load = { url: "content:/posts" }`.
│   ├── styles/         Opcional — *.css: se empaquetan y se enlazan solos.
│   ├── app/pages/      Opcional — rutas de la SPA (Modo App); + app/components, app/config.ts.
│   ├── components/     Opcional — componentes de Nexa Core (se expanden al compilar).
│   ├── locales/        <locale>.json — diccionarios planos para t().
│   └── islands/        Opcional — *.island.ts (también vale `islands/` en la raíz): se empaquetan solas.
├── public/              Se copia tal cual a dist/ (imágenes, íconos, vendor/*.js de terceros).
├── .env                 Opcional — variables (NEXA_API_URL…); `.env.local` no va a git.
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
| `pages/docs/[...path].tsx` | `/docs/a`, `/docs/a/b/c` (uno o más segmentos; `path = "a/b/c"`) |
| `pages/404.tsx` | tu página de error 404 (ver abajo) |
| `pages/docs/_layout.tsx` | no es una ruta: un layout (ver "Layouts anidados") |

Una ruta puede tener varios segmentos dinámicos. `nexa build` compila
directo a HTML cualquier ruta **sin** segmentos dinámicos; una ruta
dinámica solo se pre-renderiza si declara `paths` (ver esa sección) —
si no, se sirve al vuelo con `nexa preview`/`nexa dev` — o, en producción,
**bajo demanda con `nexa bundle`** (ver "Un solo ejecutable"). Un ejemplo con
dos segmentos dinámicos, `src/pages/blog/[author]/[slug].tsx`, atiende
`/blog/hectorarchitect/desarrollando-oweelinux-conlinux`.

### Ruta comodín `[...nombre]`

`[...path]` captura **uno o más** segmentos finales: `src/pages/docs/[...path].tsx` atiende `/docs/guia/instalar`
con `params.path = "guia/instalar"`. Es el último recurso: una ruta exacta (`docs/intro.tsx`) o de un solo segmento
(`docs/[seccion]/index.tsx`) siempre gana. Con `paths`, cada entrada trae `{"path": "guia/instalar"}` y se genera
`dist/docs/guia/instalar/index.html` (los valores no pueden contener `..`, `\` ni segmentos vacíos). Sin `paths`, se
renderiza bajo demanda (`nexa bundle`, `nexa preview`).

Un `[...nombre]` dentro de `load` viaja codificado (`a%2Fb`) hacia tu API; con `content:` se decodifica a carpetas.

### Página 404 propia

`src/pages/404.tsx` es tu página «no encontrada». `nexa build` la publica como `dist/404.html` (no entra en el
sitemap), y `nexa dev`, `nexa preview` y `nexa bundle` la sirven **con estado 404**: para una ruta que no existe, o
cuando tu API respondió 404 para esos datos. Lleva el layout y el SEO como cualquier página. Sin ella, se usa una
página mínima.

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

### Layouts anidados (`_layout.tsx`)

Un `_layout.tsx` dentro de una carpeta de `src/pages/` envuelve **solo las páginas de esa carpeta** (y sus subcarpetas),
dentro del `src/layout.tsx` global. Se anidan de fuera hacia dentro:

```
src/layout.tsx                  cabecera y pie del sitio
src/pages/docs/_layout.tsx      barra lateral de la documentación
src/pages/docs/guia/x.tsx       → x dentro de la barra, dentro de la cabecera
```

Las mismas reglas que el layout global: un único `data-nexa-slot` vacío, sin eventos propios (sí islas). Los
archivos que empiezan por `_` no son páginas. La navegación sin recarga reemplaza el `data-nexa-slot` más externo,
así que cambiar de sección siempre muestra el layout correcto.

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
    stylesheets: ["/static/site.css"],    // un <link rel="stylesheet"> por entrada
    meta: [                               // Fase 37 — meta tags sueltos, ver más abajo
        { name: "google-site-verification", content: "abc123" }
    ]
};

export default function Layout() { /* ... */ }
```

Mismo patrón que `seo`/`schema`: un objeto literal, nunca código
ejecutado — `nexa-cli` lo resuelve y lo mezcla en el `<head>` real del
documento, antes de `</head>`. Todos los campos son opcionales.

**`meta` — escape hatch para lo que no tiene campo propio en `seo{}`
(Fase 37):** `seo{}` cubre title/description/canonical/OpenGraph/
Twitter card — pero verificación de Search Console, dominio de
Facebook, o un `theme-color` puntual no tenían ningún lugar declarativo
antes de esto:

```tsx
export const head = {
    meta: [
        { name: "google-site-verification", content: "abc123" },
        { property: "theme-color", content: "#C8102E" }  // property, no name — igual que Open Graph
    ]
};
```

Cada entrada necesita `content` y exactamente uno de `name`/`property`
— si no tiene `content`, se omite en silencio (nunca un `<meta
content="">` roto). **`export const head` no es exclusivo del
layout** — una página individual también puede declararlo (ej. un
`theme-color` distinto solo para esa ruta); el de la página se agrega
después del del layout en el HTML final.

### Header/nav con estado real: una isla dentro del layout (Fase 31)

Un evento interactivo directo (`onClick`, etc.) en `src/layout.tsx`
sigue sin soportarse — el build falla explícitamente. La razón es
técnica, no arbitraria: los ids de nodo de un `IrComponent` se numeran
desde 0, y el layout y cada página son `IrComponent`s compilados por
separado — mezclarlos en el mismo manifiesto de activación produciría
ids colisionados.

Una **isla** (`data-nexa-island`) sí está permitida — nunca tuvo ese
problema: el cliente lee specifier/props/estrategia directamente del
DOM, sin ninguna entrada en el manifiesto de activación. Es la forma
real de tener un header compartido por todas las páginas con estado de
verdad (toggle de tema, selector de idioma, menú hamburguesa móvil) sin
copiarlo a mano en cada página:

```tsx
// src/layout.tsx
export default function Layout() {
    return (
        <div class="site-shell">
            <div data-nexa-island="siteHeader" data-nexa-strategy="load">
                <header><nav><a href="/">Inicio</a></nav></header>
            </div>
            <div data-nexa-slot></div>
        </div>
    );
}
```

El specifier del layout se une al import map y al bootstrap de
`nexa-islands.js` exactamente igual que uno declarado en la propia
página — no hace falta nada especial en `nexa.toml [imports]` más allá
de declararlo como cualquier otro. Un `onClick` **dentro** del fallback
de esa isla (no en la propia isla montada del lado del cliente, sino en
el HTML que el servidor renderiza) sigue sin soportarse, por la misma
razón de ids colisionados.

**Esta isla sobrevive la navegación SPA, sin remount (Fase 32):**
`initRouter` (Fase 22) sabe distinguir el HTML que viene del layout del
que viene de `data-nexa-slot` — en cada navegación de cliente, solo se
reemplaza el contenido del slot. El header (con la isla ya montada, su
estado incluido) nunca se toca. Verificable con un `console.log` dentro
del `mount()` de la isla: se ejecuta una sola vez, sin importar cuántas
páginas se visiten después.

Tampoco hay layouts anidados por directorio (un solo `src/layout.tsx`
para todo el proyecto, no uno por carpeta de `src/pages/`).

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
  verdad en el navegador. Si `handler` usa además un `const NOMBRE =
  "algo compartido";` de nivel superior (una URL base, un límite), esa
  constante viaja junto con el handler a su chunk — no hace falta
  repetir el valor adentro de cada función.

## `load` — traer datos

```tsx
export const load = { url: "/products/:slug" };
```

Un GET HTTP real contra `NEXA_API_URL + url` (con `:params` sustituidos
por los valores reales de la ruta). El backend puede estar escrito en
cualquier lenguaje — Nexa solo espera JSON de vuelta. Los valores de los
`:params` se **codifican** (`%XX`) antes de ir a tu API, y un valor `.`/`..`
se rechaza: lo que escribe quien visita nunca puede inyectar `?query` ni
segmentos en tu endpoint. Cada llamada tiene un plazo de 10 s. Un 404 del
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

**Idiomas (Fase 66).** Si una ruta vive bajo `[locale]`:

- sin `paths`, y con `[locale]` como **único** segmento dinámico
  (`src/pages/[locale]/index.tsx`), `nexa build` genera una página por cada
  `src/locales/<idioma>.json` — no hace falta ningún endpoint;
- con `paths` cuyas entradas **no** traen `locale`
  (`[locale]/shop/[category]/[product].tsx` y `[{"category":…,"product":…}]`),
  se pre-renderiza **una vez por idioma**; si las entradas sí traen `locale`,
  se respeta tal cual.

## `seo` y `schema`

> Todo valor que viene de datos se escapa, y una URL con esquema ejecutable
> (`javascript:`, `data:`) se omite — ver [SEGURIDAD.md](SEGURIDAD.md).

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
  página vive bajo `[locale]`). **Fallback (Fase 66):** una clave que falta
  en un idioma cae al idioma por defecto, clave por clave (`[i18n] default`
  en `nexa.toml`, `es` si no se declara); si falta también ahí, se omite (en
  `seo`) o queda un placeholder inerte en el cuerpo — nunca se inventa un
  valor. Un idioma que no existe en `src/locales/` (`/xx/...`) es un 404
  real.
- `t(...)` reconoce dos formas: `t("clave")`, y desde la Fase 45,
  `t("clave", { variable: data.x })` — interpolación simple de
  `{variable}` dentro del texto del diccionario:

  ```json
  // src/locales/es.json
  { "profile": { "donateTo": "Apoyar a {name}" } }
  ```

  ```tsx
  <p>{t("profile.donateTo", { name: data.creatorName })}</p>
  ```

  Cada valor del segundo argumento es la misma referencia limitada que
  ya acepta cualquier otra posición dinámica de Nexa (`data.x`,
  `params.x`) — nunca una expresión arbitraria; un literal
  (`{ name: "Ada" }`) o una llamada no se reconocen, y el `t(...)`
  entero queda sin efecto. `t("clave")` sin segundo argumento sigue
  funcionando exactamente igual que antes. Sin pluralización, sin
  clave dinámica (`t(variable)`) — quedan fuera de alcance.

  `nexa build`/`nexa lint` fallan con un error claro si una clave tiene
  un `{placeholder}` sin su variable correspondiente, o si sobra una
  variable que la clave no usa:

  ```
  Error: src/pages/index.tsx: t("profile.donateTo", ...) usa "{name}"
  pero no se pasó esa variable — la clave dice: "Apoyar a {name}"
  ```

  Esta validación cubre `t(...)` en el cuerpo JSX; en `seo`/`schema`/
  `head` la interpolación se resuelve igual de bien en tiempo de render,
  pero todavía no se valida en build — ajuste de alcance explícito.
- `<html lang="...">` usa el locale real de la página (el idioma por
  defecto si no vive bajo `[locale]`).
- `<link rel="alternate" hreflang="...">` se genera solo si la ruta
  vive bajo `[locale]` y hay más de un `src/locales/*.json`; desde la Fase 67
  incluye `hreflang="x-default"` apuntando al idioma por defecto.
- **Puerta de idioma (Fase 67).** Con rutas bajo `[locale]` y **sin**
  `src/pages/index.tsx`, `nexa build` escribe `dist/index.html` (y `preview`/`dev`
  la sirven en `/`): un script lleva al visitante a su idioma (el del navegador
  si el sitio lo tiene, si no el idioma por defecto), conservando `?query` y
  `#hash`; sin JavaScript hay enlaces a cada idioma, `canonical` y `x-default`.
  Una portada propia (`src/pages/index.tsx`) siempre gana.
- El Modo App **no tiene** `t()`: sus textos los gestiona la propia app.
- Una página fuera de `[locale]` (Fase 29) — por ejemplo una gate de
  selección de idioma en la raíz — también resuelve `params.locale` y
  `t(...)` contra ese mismo `"es"` por defecto (antes, sin fase 29,
  quedaban `href`s sin resolver y placeholders `<!--nexa:t(...)-->`
  inertes solo por no tener segmento `:locale`). No configurable
  todavía — si tu proyecto necesita otro default, es un buen momento
  para pedirlo.

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

## Iteración de listas (`<For>`)

```tsx
export const load = { url: "/articles" };

export default function Articles() {
    return (
        <ul>
            <For each={data.items}>
                {(item) => (
                    <li>
                        <a href={`/articles/${item.slug}`}>{item.title}</a>
                    </li>
                )}
            </For>
        </ul>
    );
}
```

`<For>` no es un componente ni un elemento HTML — es un patrón
sintáctico fijo que el compilador reconoce por nombre exacto, igual que
reconoce `t("clave")`. Nunca se ejecuta un `.map()` de verdad: Rust
clasifica el cuerpo del callback **una sola vez** (es una plantilla) y
lo renderiza como HTML real una vez por cada elemento del array que
`each` resolvió, en `nexa build`/`nexa preview`.

- `each={data.items}` o `each={params.x}` — mismo `Expr` limitado que
  el resto de Nexa (`data.x`/`params.x`, sin expresiones arbitrarias).
  Es el único atributo que `<For>` admite.
- El callback debe ser un arrow function de **cuerpo conciso** —
  `(item) => (<jsx/>)`, nunca `(item) => { return <jsx/>; }` — con
  **un único parámetro sin destructuring** (`(item) =>`, nunca
  `({ title }) =>`) que devuelve **un único elemento JSX**.
- Dentro del cuerpo, `item.propiedad` (con el nombre que hayas elegido
  para el parámetro) se resuelve igual que `data.propiedad` — mismo
  mecanismo, alcance limitado a ese `<For>` en particular.
- Si `each` no resuelve a un array real (falta el dato, es `null`, es
  otro tipo) el `<For>` itera cero veces — nunca se inventa un
  elemento, mismo criterio que el resto del renderer.
- `nexa build` falla con un mensaje explícito ante cualquier forma no
  reconocida (`each` con otra raíz, más de un parámetro, destructuring,
  cuerpo con bloque, más de un hijo) — nunca compila "a medias".
- Un `onClick`/isla dentro del cuerpo del `<For>` recibe un único
  `NodeId`/entrada de manifiesto (el cuerpo se clasifica una sola vez),
  compartido por todas las copias que produce el renderer — el runtime
  de activación (`packages/runtime`) activa cada copia por separado con
  su propio elemento real, no solo la primera.
- `<For>` se puede anidar (Fase 62): el `each` interno puede ser un
  campo del item externo (`each={group.members}`). Un item interno
  tapa a uno externo del mismo nombre, igual que en JS.

## Componentes y `<Show>` (Fase 62)

Nexa Core compone UI **en tiempo de compilación**: `<Card title={...}>`
no es un componente en ejecución, el parser *expande* la llamada
sustituyendo los props en el JSX de `Card`. El resultado es exactamente
el árbol que habrías escrito a mano — el resto del compilador no se
entera, y no se envía ningún JavaScript por usar componentes.

```tsx
// src/components/ui.tsx
export function Tag({ text }) { return <span class="tag">{text}</span>; }
export function Button({ onPress, children }) {
    return <button class="nx-btn" onClick={onPress}>{children}</button>;
}
// src/components/Card.tsx
export default function Card(props) {
    return <section class="card"><h2>{props.title}</h2>{props.children}</section>;
}
```

```tsx
// src/pages/catalog.tsx
import Card from "../components/Card";
import { Tag, Button } from "../components/ui";

export const load = { url: "/api/catalog" };
function saludar() { console.log("hola"); }

export default function Catalog() {
    return (
        <main>
            <For each={data.groups}>
                {(group) => (
                    <Card title={group.name}>
                        <For each={group.members}>{(m) => <p><Tag text={m.role} /> {m.name}</p>}</For>
                        <Show when={group.open} fallback={<p>Cerrado</p>}>
                            <Button onPress={saludar}>Saludar</Button>
                        </Show>
                    </Card>
                )}
            </For>
        </main>
    );
}
```

| Se puede | Cómo |
|---|---|
| Definir un componente | `function Nombre(props)`, `const Nombre = ({ a, b }) => (...)` — en el mismo archivo, o importado con ruta relativa (`import X from "./X"`, `import { X } from "../ui"`) |
| Pasar un prop | Literal (`title="x"`, `n={3}`), `data.x`, `params.x`, `t("clave")`, una plantilla (`` `/p/${data.slug}` ``), o el item de un `<For>` (`person={p}`) |
| Usar un prop | `{props.title}`, `{title}` (destructurado), en atributos y plantillas (`href={`/x/${props.slug}`}`), en `each=` y en `when=` |
| Campos de un prop | `{props.user.name}` si `user` se pasó como `data.x` / el item de un `<For>` |
| Hijos | `{props.children}` / `{children}` |
| Eventos | La página pasa el nombre de su función (`onPress={saludar}`) y el componente la conecta (`onClick={props.onPress}`). Las funciones del propio archivo del componente también llegan al chunk de activación |
| Un prop no pasado | Se omite el atributo / no se renderiza el texto; en `when=` es falso |

`<Show when={data.x} fallback={<p/>}>...</Show>` se resuelve al compilar
contra los datos reales: `when` es la misma `Expr` limitada de siempre
(nada de comparaciones — calculá el valor en el backend). Es falso si el
dato falta, es `null`, `false`, `0`, `""` o un array vacío; verdadero en
cualquier otro caso. Solo la rama elegida llega al HTML.

No se puede (con mensaje de error explícito): recursión, `{...spread}`
de props, props con valores por defecto, un prop que sea una función o
una expresión calculada, `<Pkg.Componente>` con punto, un componente
que no devuelva un único elemento JSX.

## Contenido enriquecido: `<SafeHtml>` y `<Markdown>` (Fase 68)

`{data.body}` escapa todo (un `"<p>Hola</p>"` se ve como texto). Para el cuerpo de un post, una descripción
con formato, etc.:

```tsx
<SafeHtml html={data.bodyHtml} />       // HTML (de un editor WYSIWYG)
<Markdown text={data.bodyMarkdown} />   // Markdown (CommonMark + tablas + tachado)
```

Se sanean al renderizar: etiquetas de texto/listas/tablas/código/enlaces/imágenes sí; `<script>`, `<iframe>`,
`<svg>`, `<form>`, `on*`, `style`, `javascript:`… no. Funcionan dentro de `<For>` y de componentes (el prop
recibe `data.x`). Solo aceptan una referencia a datos, nunca un literal. Detalle y límites en
[SEGURIDAD.md](SEGURIDAD.md).

## Link activo automático (`aria-current`)

```tsx
<nav>
    <a href="/">Inicio</a>
    <a href="/dashboard" data-nexa-match="prefix">Panel</a>
</nav>
```

Nexa sabe qué ruta está renderizando en el momento de armar cada
página — lo resuelve él mismo, igual que `seo`/`schema`, sin que el
desarrollador escriba ningún condicional (una página/layout de Nexa no
admite lógica propia, así que ni siquiera sería posible escribirlo a
mano). Un `<a href="...">` cuyo `href` coincide con la ruta actual
recibe `aria-current="page"` automáticamente:

- Coincidencia **exacta** por defecto — `href="/"` nunca marca activo
  en `/dashboard`, ni en ninguna otra ruta que no sea `/` misma.
- `data-nexa-match="prefix"` la vuelve por prefijo, con límite de
  segmento: `/dashboard` matchea `/dashboard/settings` pero no
  `/dashboard-old`; `href="/"` en modo prefijo sigue matcheando *solo*
  `/` exacto (el límite de segmento lo evita, no es un caso especial
  aparte).
- El estilo es cosa del proyecto — Nexa nunca agrega una clase, solo el
  atributo semánticamente correcto: `a[aria-current="page"] { ... }`.
- Funciona igual dentro de `src/layout.tsx` que dentro de una página
  individual — mismo mecanismo, sin distinción.

**Con `src/layout.tsx` + navegación SPA:** un nav dentro del layout no
se vuelve a renderizar del lado del servidor en cada click (Fase 32,
el contenedor de página estable) — `packages/router` recalcula
`aria-current` del lado del cliente después de cada navegación, con el
mismo criterio de coincidencia (por eso `data-nexa-match`, a diferencia
de otros atributos de control de Nexa, sí queda en el HTML final: el
cliente lo necesita para releer el modo). Un nav dentro de
`data-nexa-slot` no necesita este recálculo — llega ya resuelto en el
HTML fresco de cada navegación, como cualquier otro contenido de
página.

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

El fallback (los hijos del `<div data-nexa-island>`) no está limitado a
HTML fijo: es un nodo más, resuelto con el mismo mecanismo que
cualquier otro contenido de la página — `data.*`, `params.*`, `t(...)`,
`<For>` incluido. Si ya tenés el dato al momento de renderizar la
página, mostralo de verdad en vez de un placeholder:

```tsx
<div data-nexa-island="commentCount" data-nexa-props={{ articleId: params.slug }}>
    <span>{data.commentCount} comentarios</span>   {/* HTML real, no un spinner */}
</div>
```

Un placeholder fijo (`<p>Cargando…</p>`) sigue siendo válido — tiene
sentido cuando el contenido de la isla no existe todavía en el
servidor (por ejemplo, un carrito que solo vive en `localStorage`) —
pero no es la única opción.

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

**Apilamiento** (Fase 42) — abrir un `Dialog` mientras otro ya está
abierto lo apila encima (z-index correcto, sin depender del orden en el
DOM) y oculta el de abajo de lectores de pantalla (`aria-hidden`)
mientras esté cubierto; cerrar el de arriba devuelve el foco al
elemento del diálogo de abajo que lo abrió, no a `document.body`. El
focus trap de cada diálogo ya estaba aislado por elemento desde la
Fase 9, así que dos diálogos abiertos a la vez nunca compiten por el
mismo `Tab`.

**`ui.confirm()` / `ui.alert()`** (Fase 42) — el caso común de
confirmar antes de una acción destructiva, o avisar que algo se guardó,
sin escribir el `<div class="nx-dialog">` a mano:

```ts
const ok = await ui.confirm("¿Seguro que querés borrar esto?");
if (ok) { /* borrar */ }

await ui.alert("Guardado con éxito");
```

```ts
ui.confirm(message: string, options?: { confirmLabel?: string; cancelLabel?: string }): Promise<boolean>
ui.alert(message: string, options?: { okLabel?: string }): Promise<void>
```

El HTML se crea en el momento y se descarta al resolver (igual que
`ui.notify()`) — `role="alertdialog"`. Escape en `confirm()` cuenta
como cancelar (resuelve `false`); en `alert()`, como aceptar. Llamado
desde dentro de otro `Dialog` ya abierto, se apila sobre él sin romper
ninguno de los dos.

**Utilidades de layout** (Fase 44) — un set chico y deliberado, no un
sistema de utilidades completo tipo Tailwind (`@nexa/ui` sigue siendo
`Button`/`Input`/`Card`/`Dialog`/`Drawer` + esto, no un kit completo):

```tsx
<nav class="nx-flex nx-gap-3">...</nav>
<ul class="nx-grid nx-grid-cols-3">...</ul>
<main class="nx-stack">...</main>
```

| Clase | Efecto |
| --- | --- |
| `.nx-flex` | `display: flex` |
| `.nx-flex-col` | `flex-direction: column` |
| `.nx-flex-wrap` | `flex-wrap: wrap` |
| `.nx-gap-1` .. `.nx-gap-4` | `gap` — mapea a `--nx-space-1`..`--nx-space-4`. Se combina con `.nx-flex` o `.nx-grid`, ambos aceptan `gap`. |
| `.nx-grid` | `display: grid` con `gap: var(--nx-space-4)` por defecto |
| `.nx-grid-cols-2` / `-3` / `-4` | columnas iguales; colapsan a 1 columna sola bajo 640px (responsivo, sin nada que declarar) |
| `.nx-stack` | espaciado vertical consistente entre hijos directos (`--nx-space-4`) |
| `.nx-stack-1` .. `.nx-stack-3` | mismo espaciado, con `--nx-space-1`..`--nx-space-3` |

Se tree-shakean igual que el resto de `@nexa/ui` (Fase 23): una página
que no usa ninguna clase de layout no paga nada de este CSS.

**Drawer** (Fase 41) — mismo contrato de accesibilidad que Dialog (foco
atrapado, Escape, devolución de foco), pensado para sidebars/menús
laterales en vez de una caja centrada:

```tsx
function openMenu() { ui.openDrawer(document.querySelector(".nx-drawer")); }
function closeMenu() { ui.closeDrawer(document.querySelector(".nx-drawer")); }
```

```tsx
<button onClick={openMenu}>Menú</button>
<div class="nx-drawer" hidden>
    <a href="/">Inicio</a>
    <a href="/productos">Productos</a>
    <button onClick={closeMenu}>Cerrar</button>
</div>
```

Un drawer dentro de `src/layout.tsx` (Fase 25) sobrevive abierto entre
navegaciones SPA (`initRouter` no toca el layout); uno dentro del
contenido de una página se descarta junto con el resto del slot al
navegar — en ambos casos `closeDrawer` es seguro de llamar dos veces o
sobre un nodo ya fuera del DOM.

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

**Tooltip / Progress / Skeleton** (Fase 56) — tres componentes más,
también puramente visuales:

```tsx
<span class="nx-tooltip">
    Pasá el mouse acá
    <span class="nx-tooltip-content">Info extra</span>
</span>

<div class="nx-progress">
    <div class="nx-progress-bar" style={`width: ${data.percent}%`}></div>
</div>

<div class="nx-skeleton" style="width: 200px; height: 1rem;"></div>
```

`.nx-tooltip-content` se muestra con `:hover`/`:focus-within` — sin
JavaScript. `.nx-progress-bar` necesita su `width` puesto a mano (fijo
o dinámico vía `data.*`) — Nexa no calcula ningún porcentaje.
`.nx-skeleton` es un bloque con una animación de "shimmer"; el
desarrollador pone el `width`/`height` que necesite.

**Table / Badge / Avatar / Breadcrumbs / Alert / Divider** (Fase 56) —
seis componentes puramente visuales, cero JavaScript, tree-shakeados
igual que el resto:

```tsx
<div class="nx-table-wrap">
    <table class="nx-table nx-table-zebra">
        <thead><tr><th>Cliente</th><th>Estado</th></tr></thead>
        <tbody>
            <tr><td>Hector M.</td><td><span class="nx-badge nx-badge-success">Pagado</span></td></tr>
        </tbody>
    </table>
</div>

<nav class="nx-breadcrumbs" aria-label="breadcrumb">
    <a class="nx-breadcrumbs-link" href="/">Inicio</a>
    <span class="nx-breadcrumbs-link" aria-current="page">Pedido #1024</span>
</nav>

<div class="nx-alert nx-alert-warning">
    <p class="nx-alert-title">Stock bajo</p>
    <p>Quedan 3 unidades.</p>
</div>

<span class="nx-avatar nx-avatar-sm">HM</span>
<hr class="nx-divider" />
```

| Componente | Clases |
| --- | --- |
| Table | `.nx-table`, `.nx-table-wrap` (scroll horizontal), `.nx-table-zebra` (filas alternadas) |
| Badge | `.nx-badge` + `.nx-badge-primary/success/warning/danger/neutral` |
| Avatar | `.nx-avatar` (círculo, 2.5rem por defecto) + `.nx-avatar-sm/-lg`; con `<img>` adentro recorta a `object-fit: cover` |
| Breadcrumbs | `.nx-breadcrumbs` (contenedor) + `.nx-breadcrumbs-link` (cada link/paso; `/` automático entre ellos vía `::after`); `aria-current="page"` marca el actual |
| Alert | `.nx-alert` + `.nx-alert-success/warning/danger` (borde izquierdo de color); `.nx-alert-title` opcional |
| Divider | `.nx-divider` (horizontal) / `.nx-divider-vertical` |

**Tabs, Accordion, Dropdown, y ordenar/filtrar una `Table`** (Fase 57)
— con estado real en cliente, pero **ninguno es una isla**: son
funciones normales de `ui`, llamadas desde un `onClick`/`onInput`
normal, exactamente igual que `Dialog`/`Drawer` desde la Fase 9. Ver
"Isla vs. función de `ui`" más abajo para el criterio general.

*Tabs* — `ui.selectTab(event)` lee `event.currentTarget.dataset.tab` y
el `.nx-tab-group` más cercano; no necesita el elemento explícito:

```tsx
function selectTab(event) { ui.selectTab(event); }

<div class="nx-tab-group">
    <div class="nx-tab-list" role="tablist">
        <button class="nx-tab" data-tab="uno" aria-selected="true" onClick={selectTab}>Uno</button>
        <button class="nx-tab" data-tab="dos" onClick={selectTab}>Dos</button>
    </div>
    <div class="nx-tab-panel" data-tab="uno">Contenido uno</div>
    <div class="nx-tab-panel" data-tab="dos" hidden>Contenido dos</div>
</div>
```

*Accordion* — `ui.toggleAccordionItem(event)`; con `data-exclusive="true"`
en el `.nx-accordion` contenedor, abrir un item cierra los demás (por
defecto, varios pueden estar abiertos a la vez):

```tsx
function toggle(event) { ui.toggleAccordionItem(event); }

<div class="nx-accordion" data-exclusive="true">
    <div class="nx-accordion-item">
        <button class="nx-accordion-trigger" aria-expanded="false" onClick={toggle}>Pregunta</button>
        <div class="nx-accordion-panel" hidden>Respuesta</div>
    </div>
</div>
```

*Dropdown* — `ui.toggleDropdown(event)` / `ui.selectDropdownOption(event)`;
se cierra solo con click afuera o `Escape` (igual que `Dialog`, pero sin
atrapar el foco — un dropdown no bloquea el resto de la página):

```tsx
function toggleMenu(event) { ui.toggleDropdown(event); }
function pick(event) { ui.selectDropdownOption(event); }

<div class="nx-dropdown">
    <button class="nx-dropdown-trigger" aria-haspopup="listbox" aria-expanded="false" onClick={toggleMenu}>
        Elegir talle…
    </button>
    <ul class="nx-dropdown-menu" role="listbox" hidden>
        <li class="nx-dropdown-option" role="option" data-value="m" onClick={pick}>M</li>
    </ul>
</div>
```

*Table: ordenar y filtrar* — `ui.sortTable(event)` sobre el `<th>`
clickeado (ordena como número si ambas celdas lo son, si no como
texto; alterna ascendente/descendente); `ui.filterTable(table, query)`
oculta filas de `<tbody>` cuyo texto no matchea — ambos client-side,
sobre datos que el servidor ya renderizó, sin re-pedir nada:

```tsx
function sortByColumn(event) { ui.sortTable(event); }
function filterOrders(event) { ui.filterTable(document.getElementById("orders"), event.target.value); }

<input type="search" onInput={filterOrders} placeholder="Buscar pedido…" />
<table id="orders" class="nx-table">
    <thead><tr><th data-sortable onClick={sortByColumn}>Cliente</th></tr></thead>
    <tbody>...</tbody>
</table>
```

**Buscador de página, reactivo — sin isla** (Fase 57) —
`ui.createPageSearch(container, itemSelector?)` filtra cualquier
conjunto de elementos marcados con `data-searchable` (selector
configurable) dentro de `container`, con un `Signal` real de
`@nexa/reactivity` por debajo. El chunk extraído de un handler es un
módulo aislado — no puede depender de una variable de módulo declarada
aparte en la página — por eso el controlador se cuelga del propio
elemento `<input>` (`event.currentTarget`), que sí persiste entre una
tecleada y la siguiente:

```tsx
function onSearchInput(event) {
    const input = event.currentTarget;
    input.pageSearch ??= ui.createPageSearch(document.getElementById("catalogo"));
    input.pageSearch.setQuery(event.target.value);
}

<input type="search" onInput={onSearchInput} placeholder="Buscar…" />
<div id="catalogo">
    <article data-searchable>Zapatillas Nezha</article>
    <article data-searchable>Camiseta Jade</article>
</div>
```

```ts
ui.createPageSearch(container: Element, itemSelector?: string): { setQuery(value: string): void }
```

**Isla vs. función de `ui`: cómo decidir** — regla general (Fase 57):
si el estado es local a un widget concreto y se puede modelar como
"manipular directamente el DOM que ya está en la página" (abrir/cerrar,
marcar una pestaña activa, ordenar/filtrar filas ya renderizadas), es
una función de `ui` + `onClick`/`onInput` — sin Virtual DOM, sin
re-render, activación progresiva normal (Fase 5). Reservá una isla
(Fase 16) para cuando de verdad hace falta un framework completo
(Vue/React con su propio ciclo de vida) o composición real con props/
children profundos (`@nexa/reactivity`, modelo de componentes, Fase 50,
dentro de la isla) — nunca como opción por defecto para "este botón
hace algo".

Design tokens (`--nx-color-primary`, `--nx-space-4`, `--nx-radius-md`,
`--nx-color-success`, `--nx-color-warning`, `--nx-color-zebra`, ...) son
custom properties de CSS normales — sobreescribilas en tu propio
`:root` para otra paleta.

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

const session = platform.sessionStorage()   // Fase 38 — mismo contrato, sobre sessionStorage
session.get(key) / session.set(key, value) / session.remove(key)
// Vive solo mientras dure la pestaña — a diferencia de storage()/localStorage, no sobrevive cerrar el navegador.

const cache = await platform.cache("nombre-de-cache")   // Fase 39 — Cache API directa, sin [pwa]
await cache.match(url) / await cache.put(url, response) / await cache.delete(url)
// Independiente de [pwa.cache] (que vive en el service worker) — para cachear algo puntual desde código normal.

const users = platform.db("users")   // Fase 40 — wrapper mínimo sobre IndexedDB, no un ORM
await users.get(id) / await users.set(id, valor) / await users.delete(id) / await users.list()
// Cada nombre de colección es su propia base IndexedDB — sin declarar nada de antemano.
// Web: IndexedDB real. Tauri/Capacitor: el IndexedDB del propio webview, sin rama nativa distinta.

platform.theme.get()          // Fase 43 — "dark" | "light" | "system" (default si nunca se guardó nada)
platform.theme.set("dark")    // persiste con platform.storage() y pone data-theme="dark" en <html>

const status = await platform.network.getStatus()   // Fase 48 — { online: boolean, type: "wifi"|"cellular"|"none"|"unknown" }
const unsubscribe = platform.network.onChange((status) => { ... })   // suscripción; llamar unsubscribe() para cancelarla

const state = await platform.lifecycle.getState()    // Fase 49 — { active: boolean }: false mientras la app está en segundo plano
const stop = platform.lifecycle.onChange((state) => { ... })   // suscripción a resume/pause; llamar stop() para cancelarla

const position = await platform.geolocation.getCurrentPosition()   // Fase 51 — { coords: { latitude, longitude, accuracy, ... }, timestamp }
const stopWatch = platform.geolocation.watchPosition((position) => { ... })   // suscripción a cambios de posición; llamar stopWatch() para cancelarla

const launch = await platform.deepLinks.getLaunchUrl()   // Fase 52 — { url: string }: "" en Web/Tauri (una página no se "lanza" con un deep link)
const stopOpen = platform.deepLinks.onOpen((event) => { ... })   // suscripción a que la app se reabra con una URL; no dispara nunca en Web/Tauri

const registration = await platform.push.register({ vapidPublicKey })   // Fase 53 — { platform: "capacitor", token } | { platform: "web", subscription }
platform.push.onReceived((n) => { ... })          // solo Capacitor — en Web lanza, hay que escuchar "push" en tu Service Worker
platform.push.onActionPerformed((a) => { ... })   // solo Capacitor — en Web lanza, hay que escuchar "notificationclick" en tu Service Worker

const status = await platform.biometrics.isAvailable()   // Fase 54 — { isAvailable, biometryType }
await platform.biometrics.authenticate({ reason: "Desbloquear" })   // dispara el prompt biométrico real; rechaza si falla/cancela

await platform.haptics.impact({ style: "MEDIUM" })       // Fase 55 — HEAVY (default) | MEDIUM | LIGHT
await platform.haptics.notification({ type: "ERROR" })  // SUCCESS (default) | WARNING | ERROR
await platform.haptics.vibrate({ duration: 250 })        // ms; default 300
await platform.haptics.selectionStart()   // hint de "empezar a arrastrar por una selección" (ej. un slider)
await platform.haptics.selectionChanged() // solo vibra si hubo un selectionStart() sin su selectionEnd() todavía
await platform.haptics.selectionEnd()
```

`platform.theme` funciona junto a los tokens de `@nexa/ui`
(`tokens.css`): sin llamar nunca `platform.theme.set()`, el sitio ya
respeta `prefers-color-scheme` del sistema operativo — el helper es
solo para dejar que el usuario elija explícitamente y esa elección le
gane al sistema. Para que la preferencia se vea también después de
recargar la página (no solo tras el primer click), el toggle de tema
necesita `data-nexa-strategy="load"` (Fase 5) — así el chunk de
`@nexa/platform` se importa en cada carga, y `theme.ts` reaplica lo
guardado apenas se evalúa:

```tsx
<button class="nx-btn" data-nexa-strategy="load" onClick={() => platform.theme.set("dark")}>
    Oscuro
</button>
```

Cada función lanza un `Error` con mensaje claro (`[nexa/platform] ...`)
si la capacidad no está disponible en el entorno actual — nunca falla
en silencio.

`platform.network` (Fase 48, primer plugin del issue #20, incremental):
Capacitor nativo usa el plugin real `@capacitor/network`; Web/Tauri usan
`navigator.onLine` + los eventos estándar `online`/`offline`. El
`type` (`"wifi"`/`"cellular"`) es exclusivo de Capacitor — en Web
siempre es `"unknown"`, porque no existe un equivalente estándar
confiable (`navigator.connection` no es parte de ningún spec estable).

`platform.lifecycle` (Fase 49, segundo plugin del issue #20): Capacitor
nativo usa `@capacitor/app` (`getState`/`appStateChange`); Web/Tauri
usan `document.hidden` + el evento estándar `visibilitychange` —
exactamente la misma señal que la propia documentación de
`@capacitor/app` dice que usa su rama web, no una aproximación propia.

`platform.geolocation` (Fase 51, tercer plugin del issue #20): Capacitor
nativo usa el plugin real `@capacitor/geolocation`; Web/Tauri usan la
Geolocation API estándar del navegador (`navigator.geolocation`) — la
misma API que usa la propia rama web de `@capacitor/geolocation` por
dentro. La forma de `Position` coincide en ambos casos, sin necesidad
de normalizar nada entre ramas.

`platform.deepLinks` (Fase 52, cuarto plugin del issue #20): Capacitor
nativo usa el plugin real `@capacitor/app` (`getLaunchUrl`/
`appUrlOpen`). Web/Tauri: `getLaunchUrl()` siempre devuelve `{ url: "" }`
y `onOpen()` nunca dispara — no es una limitación inventada, es
literalmente lo que hace la propia rama web de `@capacitor/app`
(`AppWeb`) para ambos casos, verificado leyendo su código fuente. Tiene
sentido: una página no se "lanza" con una URL de deep link distinta de
la que ya está cargada, ni se "reabre" con otra URL mientras sigue
corriendo — cada URL nueva en un navegador es una navegación distinta.

`platform.push` (Fase 53, quinto plugin del issue #20) — deliberadamente
más chico que el plugin completo de Capacitor: solo registro + recibir,
sin `checkPermissions`/canales/notificaciones ya entregadas. Distinto
de `platform.notify()` (notificación local, sin servidor). Capacitor y
Web no comparten forma de registro (un token de FCM/APNs no es una
`PushSubscription` del Web Push estándar), por eso `register()` devuelve
una unión discriminada (`{ platform: "capacitor", token }` |
`{ platform: "web", subscription }`) en vez de forzar una forma común
inventada. En Web hace falta un Service Worker ya registrado (`[pwa]`,
Fase 18) y una `vapidPublicKey` real de tu backend. `onReceived()`/
`onActionPerformed()` solo existen de verdad en Capacitor — en Web, un
push siempre llega al Service Worker (`self.addEventListener("push",
...)`), nunca a la página; llamar a estas funciones ahí lanza un error
explícito con la solución, en vez de no hacer nada en silencio.

`platform.biometrics` (Fase 54, sexto plugin del issue #20): Capacitor
nativo usa el plugin real `@aparajita/capacitor-biometric-auth`
(`checkBiometry`/`authenticate`). Web/Tauri usan WebAuthn real
(`PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable()`
para `isAvailable()`; un ciclo completo de `navigator.credentials.create`/
`.get()` contra un autenticador de plataforma para `authenticate()` —
sin backend, un gate local del dispositivo). **No** se replicó la
propia rama web de `@aparajita/capacitor-biometric-auth`: leyendo su
código fuente, esa rama es una simulación deliberada para tests (un
`confirm()` con estado falso puesto a mano) — usar WebAuthn real es más
fiel a la disciplina de "verificado contra algo real, nunca inventado"
que copiar esa simulación. El id de la credencial creada la primera
vez se guarda con `platform.storage()` (Fase 38, reutilizado tal cual).

`platform.haptics` (Fase 55, séptimo y último plugin del issue #20):
Capacitor nativo usa el plugin real `@capacitor/haptics`. Web/Tauri
usan la Vibration API estándar (`navigator.vibrate`) — la misma que
usa la propia rama web de `@capacitor/haptics` por dentro (a
diferencia de biometría, esta rama web sí es real). Los patrones de
vibración de cada `impact`/`notification`/`selectionChanged` son
exactamente los que usa `HapticsWeb`, verificados leyendo su código
fuente.

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

### Modelo de componentes liviano (Fase 50) — **solo dentro de islas nativas**

> **Esto no aplica a páginas Nexa Core.** Una página `.tsx` sigue
> siendo "un componente por página, sin `<Otro/>`, sin props" — esa
> regla no cambia. `nexa-parser` no entiende nada de este módulo; es
> exclusivamente una utilidad de runtime para organizar el código
> *dentro* del `mount(el, props)` de una isla escrita a mano (Fase 16).

```ts
import { mountComponent, type Component } from "@nexa/reactivity";

interface ItemProps { label: string }
const Item: Component<ItemProps> = (el, props) => {
    el.textContent = props.label;
};

interface ListProps { labels: string[] }
const List: Component<ListProps> = (el, props, ctx) => {
    for (const label of props.labels) {
        const li = document.createElement("li");
        el.appendChild(li);
        ctx.mount(Item, li, { label });   // un componente monta a otro, props tipadas
    }
};

export default function mount(el: Element, props: Record<string, unknown>) {
    return mountComponent(List, el, { labels: props.labels as string[] });
}
```

`mountComponent` devuelve una función de limpieza que desmonta, en
orden inverso, todo lo que se registró con `ctx.onCleanup`/`ctx.mount`
durante el montaje — incluidos los componentes hijos, transitivamente.
Sin Virtual DOM: cada componente manipula DOM real una sola vez al
montarse, nunca se "re-renderiza" solo. Una `Signal` pasada como prop
ya alcanza para que un hijo reaccione a cambios del padre (leerla con
`.value` dentro de su propio `effect()`) — no hace falta ningún
mecanismo de props reactivas aparte.

Ejemplo real, de punta a punta:
`examples/oweeme-shop/src/islands/taskList.island.ts` — el mismo panel
de tareas que `dashboardIsland` (Vue 3 real), reescrito con este
modelo. Comparación de peso de bundle real en
`docs/FASES-DE-CONSTRUCCION.md`, Fase 50.

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
chat.status  // Signal<"connecting" | "open" | "reconnecting" | "closed">
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

### Reconexión con backoff (Fase 35)

Sin declarar `reconnect`, el comportamiento es exactamente el de antes
— retrocompatible: un `WebSocket` que se corta queda `"closed"` para
siempre; un `EventSource` reintenta solo con el mecanismo nativo del
navegador (sin backoff configurable ni límite). Para reconexión
controlada:

```ts
const chat = connectSocket({
    url: "wss://miapp.com/chat",
    reconnect: true, // o un objeto para ajustar los valores por defecto
});

const notifications = connectSSE({
    url: "/events",
    reconnect: { maxAttempts: 10, baseDelayMs: 500, maxDelayMs: 15000 }, // estos son los defaults de `reconnect: true`
});
```

- `reconnect: true` usa los defaults (`maxAttempts: 10`, `baseDelayMs:
  500`, `maxDelayMs: 15000`) — un objeto parcial completa lo que falte
  con esos mismos defaults.
- `status` gana `"reconnecting"` (ambas conexiones, `connectSSE`
  también gana un `status` que antes no tenía) mientras se reintenta,
  con backoff exponencial + jitter entre intentos.
- Un reintento exitoso reinicia el contador — agotar `maxAttempts`
  requiere esa cantidad de fallos *consecutivos*, no acumulados a lo
  largo de toda la conexión.
- Agotados los intentos, `status` pasa a `"closed"` definitivo — mismo
  final que sin `reconnect`, solo que después de intentarlo.
- Llamar `close()` a mano nunca dispara un reintento, sin importar si
  hay `reconnect` configurado — es una desconexión deliberada, no un
  corte de red.
- `send()` (en `connectSocket`) siempre opera sobre la conexión
  vigente — si hubo una reconexión de por medio, es el `WebSocket`
  nuevo, nunca el que se cortó.

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

Conseguir que `/vendor/nexa-stripe.js` (o el archivo que sea) exista es
responsabilidad del proyecto — ver **`docs/VENDORING.md`** (Fase 46)
para el proceso paso a paso de tomar un paquete de npm cualquiera
(`marked`, Firebase Auth, el SDK de PayPal...) y bundlearlo a un único
archivo ESM servible, con dos ejemplos reales ya probados:
`packages/stripe` (con un envoltorio propio) y `marked` en
`examples/oweeme-shop/src/pages/[locale]/articles.tsx` (vendorizado
directo, sin wrapper).

## Recursos: islas, CSS y pasos propios (Fase 72)

Nexa trae esbuild integrado para lo que escribes tú. **Sin configuración**: las convenciones bastan.

| Archivo | Resultado |
|---|---|
| `islands/<nombre>.island.ts` (o `.tsx`, `.js`, `.jsx`; también en `src/islands/`) | `/assets/islands/<nombre>.<hash>.js`. `<nombre>` se puede usar ya en `data-nexa-island="<nombre>"` (entra solo al import map). |
| `src/styles/<nombre>.css` (los que empiezan por `_` son parciales) | `/assets/styles/<nombre>.<hash>.css`, enlazado **solo** en el `<head>` de todas las páginas y del shell del Modo App. |

Una isla puede importar otros archivos, **paquetes de npm** y CSS (`import "./counter.css"`: ese CSS se enlaza también).
Un CSS puede usar `@import`, y se minifica. Los archivos llevan hash en el nombre: caché inmutable.

Requisito único: `esbuild` en el proyecto (`npm install -D esbuild`; o en el `PATH`). Si hay islas o estilos y no está,
`nexa` lo dice con ese mismo comando. Sin islas ni estilos no se necesita nada.

- `nexa build` deja los archivos en `dist/assets/`; lo generado vive en `.nexa/` (ponlo en `.gitignore`, `nexa create` ya lo hace).
- `nexa dev` empaqueta al arrancar y **otra vez cuando cambias** una isla o un CSS.
- Para Tailwind, Sass o cualquier generador: `[build] before = ["comando", …]` en `nexa.toml` corre esos comandos antes
  de empaquetar. Si uno falla, el build se detiene. Ejemplo con Tailwind v4: `before = ["npx @tailwindcss/cli -i src/styles/tw.css -o src/styles/app.css"]`
  (y `src/styles/tw.css` empieza con `@import "tailwindcss";`; `app.css` es el que se empaqueta, así que ignora `tw.css` con `_tw.css`).
- Lo que ya escribías a mano (`public/vendor/x.js` + `[imports]`) **sigue funcionando**; un `[imports]` con el mismo nombre gana.

## Contenido local (`content:`) — un sitio sin backend (Fase 73)

`load` y `paths` aceptan, en lugar de una URL de API, un camino bajo `src/content/`:

```tsx
export const load = { url: "content:/posts" };          // lista de src/content/posts/*.md|json
export const load = { url: "content:/posts/:slug" };    // una entrada (404 si no existe)
export const paths = { url: "content:/posts" };         // pre-renderizar todas
```

Cada `.md` es **cabecera + cuerpo**:

```md
---
title: Hola desde un archivo
date: 2025-03-10
tags: [nexa, markdown]
draft: false
---
## Sin backend
Texto en **Markdown**.
```

La entrada que recibe la página tiene los campos de la cabecera, más `slug` (el nombre del archivo), `path` (la ruta
dentro de la colección, útil para `[...path]`) y `body` (el Markdown: pásalo a `<Markdown text={data.body} />`, que lo
convierte y **sanea**). Un `.json` con un objeto aporta sus campos; con cualquier otro valor se devuelve tal cual
(`content:/site` → `src/content/site.json`).

- **Cabecera**: un YAML sencillo — `clave: valor`, cadenas con o sin comillas, números, `true`/`false`/`null`, listas
  `[a, b]` o con `- elemento`. Nada de objetos anidados.
- **Orden**: por `date` (más reciente primero) si todas la tienen; si no, por nombre.
- **Borradores**: `draft: true` no existe (ni en la lista ni como página).
- **Anidado**: `content:/docs` lista también las subcarpetas; `docs/guia/index.md` es la página `guia`.
- **Paginación**: `content:/posts?perPage=5&page=:n` trae una página; `content:/posts?perPage=5&pages` devuelve
  `[{"n":"1"},{"n":"2"},…]`, que es justo lo que `paths` necesita para `src/pages/blog/page/[n].tsx`.
- **Seguridad**: los nombres solo admiten letras, números, `-`, `_` y `.`; `..`, `%2e%2e`, rutas absolutas y `\`
  son 404, nunca leen fuera de `src/content/`.
- `nexa bundle` embebe `src/content/` con las fuentes: cambiar un archivo = volver a empaquetar.

Ejemplo completo y probado en Chrome: `examples/content-demo`.

**Paginar contra una API** es igual de simple y no necesita nada especial: `paths` devuelve `[{"n":"1"},{"n":"2"}]` y
`load = { url: "/api/posts?page=:n" }`. (Un `?page=2` en la URL del visitante no cambia una página ya pre-renderizada:
la paginación de Nexa es por ruta, `/blog/page/2`, que además es lo que indexan los buscadores.)

## Redirecciones, cabeceras y CSP (Fase 75)

```toml
[redirects]
"/articulos" = "/blog"                                   # 301
"/tienda/:slug" = { to = "/shop/:slug", status = 302 }   # :nombre captura un segmento
"/docs/v1/*" = "/docs/*"                                 # * captura el resto

[headers."/*"]                                           # cabeceras por ruta (`*` final = el resto)
X-Robots-Tag = "noindex"
[headers."/assets/*"]
Cache-Control = "public, max-age=31536000, immutable"

[security]
csp = "default-src 'self'; img-src 'self' data:"
```

- **Redirecciones**: estados 301, 302, 303, 307 o 308. El destino debe empezar por `/`, `http://` o `https://`; lo
  capturado se codifica y nunca puede convertir `/x` en `//otro-sitio` (redirección abierta). Las aplican `nexa dev`,
  `nexa preview` y el ejecutable de `nexa bundle`. `nexa build` escribe además `dist/_redirects` (Netlify, Cloudflare
  Pages) y, para las exactas, una página con `<meta refresh>` que funciona hasta en GitHub Pages.
- **Cabeceras**: las del proyecto pisan las de serie. `nexa build` escribe `dist/_headers` (Netlify, Cloudflare Pages).
  Se rechazan nombres o valores con saltos de línea.
- **`[security] csp`**: una Content-Security-Policy **sin `'unsafe-inline'`**. Nexa pone scripts en línea (el import
  map, el arranque de la página), así que una CSP normal los bloquearía. Con `csp`, cada página lleva un
  `<meta http-equiv="Content-Security-Policy">` con tu política más `script-src 'self'` y el **hash SHA-256 de cada script
  en línea de esa página**: un script inyectado (XSS) no tiene hash y el navegador no lo ejecuta. Si tu política ya trae
  `script-src`, se amplía. Una `<meta>` no admite `frame-ancestors`, `report-uri` ni `sandbox`: esas van por `[headers]`.
  Comprobado en Chrome (`e2e/content-site.mjs`): la página funciona sin errores de consola. Ojo con **tus** recursos: si tu
  página usa `style="..."` o una fuente externa, tu política tiene que permitirlos (`style-src 'self' 'unsafe-inline'`…).

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
nexa build --target desktop            # dist/ + binario de escritorio real (Fase 63)
# -> src-tauri/target/release/<nombre>  (un solo archivo; ~11 MB en Linux)

nexa add capacitor
npx @capacitor/cli add android         # necesita el SDK de Android
cd android && ./gradlew assembleDebug  # APK real
```

Ambos son Experimental — generan archivos reales en el proyecto
(`src-tauri/`, `capacitor.config.json`), sin depender de que este
repositorio esté disponible en la máquina donde se usa el binario de
`nexa`.

### Escritorio (Tauri) con el Modo App

- **La ventana abre la SPA**: si el proyecto tiene `src/app/`, la ventana
  arranca en su `base` (`/app`); si no, en la portada.
- **API de la app**: dentro del webview no hay el proxy de `nexa dev` ni
  nginx (la página vive en `tauri://localhost`). `[desktop.proxy]` (o
  `[proxy]` si no se declara) se escribe en el shell como
  `window.__NEXA_PROXY__` y `createApi` reescribe las rutas con ese
  prefijo al backend real, con `credentials: "include"`:

  ```toml
  [proxy]            # web: nexa dev / preview
  "/api" = "http://127.0.0.1:9100"
  [desktop.proxy]    # escritorio: el backend real
  "/api" = "https://api.miapp.com"
  ```

  El mismo código (`createApi({ baseURL: "/api" })`) funciona en la web y
  en escritorio. El backend necesita **CORS** para `tauri://localhost`
  (`http://tauri.localhost` en Windows). Ojo: una cookie de sesión de otro
  *sitio* requiere `SameSite=None; Secure` (HTTPS); para escritorio suele
  ser más simple autenticar con una cabecera (`createApi({ headers })`).
- **Puente a Rust**: `platform.native.invoke("comando", { args })`
  (`@nexa/platform`) llama a una `#[tauri::command]` de
  `src-tauri/src/lib.rs`; la plantilla trae `app_info` (nombre, versión,
  SO, arquitectura) — `platform.native.appInfo()`. Fuera de Tauri lanza un
  error claro. Para agregar tu propia función: escríbela, regístrala en
  `generate_handler![...]` y llámala desde la página.
- **`nexa build --target desktop`** corre `nexa build` (shell de escritorio)
  y `cargo build --release --features custom-protocol` sobre `src-tauri/`.
  Con un `cargo build` a mano hay que pasar esa feature: sin ella Tauri
  carga `devUrl` en vez de los assets embebidos.
- Instaladores (`.deb`/`.msi`/`.dmg`), firma de código y actualizador
  automático: `cargo tauri build` (CLI de Tauri) sobre el mismo `src-tauri/`.
- `--target android|ios` todavía no existe: para móvil, `nexa add capacitor`.

## Un solo ejecutable (`nexa bundle`)

```bash
nexa bundle                       # nexa build + empaqueta -> ./<proyecto>-server
./mi-sitio-server --port 8080 --host 0.0.0.0 --proxy /api=https://api.miapp.com
```

Genera **un único archivo**: una copia del binario de `nexa` con `dist/`
(comprimido) y la configuración pegados al final. Al arrancar, `nexa`
detecta esa carga y actúa como el servidor de producción de ese sitio.
No hace falta Node, nginx, Rust ni ningún archivo suelto en la máquina
de destino: se copia el ejecutable y se corre.

| Qué hace el servidor | Detalle |
|---|---|
| Sirve desde memoria | Todo el sitio viaja dentro del ejecutable; varios hilos |
| gzip | Los archivos de texto se precomprimen al empaquetar (si ganan >5%); se sirve gzip si el cliente lo acepta (`Accept-Encoding`) |
| Caché | `immutable` de un año para `nombre.<hash>.js/css`; `no-cache` + `ETag` para HTML, `sw.js` y el manifest (responde `304` sin cuerpo); 1 h para el resto |
| SPA (Modo App) | Una ruta profunda bajo `/app/...` que no es un archivo devuelve el shell; un asset que falta sigue siendo `404` |
| `[proxy]` | Reenvía `/api/*` al backend (método, cuerpo, cabeceras, cookies); se reemplaza al arrancar con `--proxy PREFIJO=URL` |
| Seguridad | Las mismas 3 cabeceras que `nexa preview`/nginx; `405` salvo `GET`/`HEAD`; rechaza `..` y codificaciones inválidas |
| `404.html` | Si existe, se usa para los 404 reales |

Opciones del ejecutable: `--port` (por defecto `$PORT` o 8080), `--host`
(por defecto `127.0.0.1`: usa `0.0.0.0` para aceptar otras máquinas),
`--proxy`, `--quiet`, `--help`.

Opciones de `nexa bundle`: `--out <nombre>`, `--no-build` (usa el `dist/`
existente) y `--runtime <ruta-a-nexa>` (el binario base: para empaquetar
para otro sistema operativo, pasa el `nexa` compilado para ese SO).

**Tamaño:** el ejecutable pesa lo que el binario de `nexa` (~15 MB en
release en Linux) más el sitio ya comprimido: `examples/app-demo` = 15,3 MB;
`examples/phaser-demo` (Phaser 1,4 MB) = 16,9 MB, y Phaser viaja a 369 KB
por la red con gzip.

### Renderizado bajo demanda (Fase 66)

Si el proyecto tiene rutas dinámicas de Core (`blog/[author]/[slug]`, `[locale]`,
…), `nexa bundle` **embebe las fuentes** (`src/pages`, `src/components`,
`src/layout.tsx`, `src/locales`, `nexa.toml`; nunca se sirven por HTTP) y el
ejecutable renderiza al vuelo lo que no está pre-renderizado — un post
publicado después del build — con el mismo pipeline de `nexa preview`:

```bash
NEXA_API_URL=https://tu-api nexa build
nexa bundle --no-build --out site
./site --port 8080 --api-url https://tu-api --revalidate 60
```

| Comportamiento | Detalle |
|---|---|
| Caché (ISR) | Lo renderizado se reutiliza `--revalidate` segundos (60 por defecto; `0` = nunca; `[bundle] revalidate`; `$NEXA_REVALIDATE`). Cabecera `X-Nexa-Cache: miss/hit/stale`. |
| Una sola llamada | Con 24 visitas simultáneas a una página vencida, una renderiza y las demás esperan ese resultado. |
| API caída | Se sirve la última copia aunque esté vencida (`stale`); una ruta nunca vista da `502` con `Retry-After`, sin filtrar la URL interna. |
| Rutas inexistentes | Un 404 de tu API es un 404 real, recordado 10 s como máximo. Un idioma inexistente es 404. |
| Lo que no cuesta una llamada | Lo pre-renderizado, los estáticos y lo que no parece una página (`/wp-login.php`, `/.env`: cualquier último segmento con `.`). |
| Seguridad | Parámetros codificados hacia tu API; directorio de trabajo privado (modo `0700`, dueño verificado); el paquete no puede escribir fuera de él; plazo de 10 s por llamada. |

`nexa bundle --static` desactiva esto (solo lo pre-renderizado; las
rutas dinámicas no pre-renderizadas dan 404). Los chunks y el CSS que genera
un render se escriben en el directorio de trabajo y se sirven desde ahí.

**No incluye:** terminación TLS (HTTPS: ponlo detrás de un proxy inverso o
balanceador), HTTP/2, brotli, `Range`, ni optimización de imágenes (`<picture>`/AVIF)
en las páginas renderizadas bajo demanda (sí en las pre-renderizadas). `sitemap.xml`
lista solo lo pre-renderizado. Tampoco hay reinicio sin cortes: un redeploy es
reemplazar el ejecutable.

## Despliegue en contenedor (`docker`)

`nexa add docker` genera un `Dockerfile` y un `.dockerignore` para el ejecutable de `nexa bundle`:

```
nexa bundle --out mi-sitio-server        # en Linux
docker build -t mi-sitio .
docker run -p 8080:8080 -e NEXA_API_URL=https://tu-api mi-sitio
```

La imagen solo copia ese archivo (sin Node, sin nginx) y corre sin privilegios. Compílalo en una distribución con una
glibc igual o más antigua que la de la imagen base.

## Despliegue (`nginx`)

```bash
nexa add nginx
```

genera `deploy/nginx.conf` — gzip, las mismas tres cabeceras de
seguridad que `nexa preview`/`nexa dev` ya mandan
(`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`,
`Referrer-Policy: strict-origin-when-cross-origin`), y un `try_files`
que sirve cualquier ruta estática (incluidas las pre-renderizadas por
`paths`).

**Rutas dinámicas sin `paths` en producción (Fase 34):** `nexa add
nginx` escanea `src/pages` (mismo criterio que `nexa build`) y lista,
por nombre real, cuáles de tus rutas dinámicas no declaran `paths` —
esas son las que `try_files` nunca va a encontrar como archivo. El
archivo generado incluye un bloque `proxy_pass` concreto (con el
prefijo real de la primera ruta detectada, no un placeholder genérico)
apuntando a un `nexa preview` corriendo detrás — comentado a propósito,
porque un prefijo mal adivinado sería peor que ninguno (ej. un proyecto
con `[locale]` como primer segmento no tiene un prefijo fijo posible;
en ese caso el archivo lo marca explícitamente en vez de inventar uno).
Verificado de punta a punta con un nginx real: ruta estática servida
directo desde `dist/`, ruta dinámica sin `paths` reenviada de verdad a
un `nexa preview` corriendo detrás, con datos que nunca existieron en
ningún build anterior.

Si tu proyecto no tiene ninguna ruta dinámica sin `paths`, el archivo
lo dice explícitamente y no incluye ningún `proxy_pass` — no hay nada
que decidir a mano en ese caso.

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

[app]                    # Modo App (SPA)
base = "/app"
title = "Mi App"

[proxy]                  # nexa dev/preview/bundle y `nexa add nginx`: prefijo -> backend
"/api" = "https://api.miapp.com"

[desktop.proxy]          # nexa build --target desktop: el backend real de la app de escritorio
"/api" = "https://api.miapp.com"

[i18n]
default = "es"           # idioma por defecto y de respaldo

[bundle]
revalidate = 60          # nexa bundle: segundos de caché de lo renderizado bajo demanda

[build]
before = ["npx @tailwindcss/cli -i src/styles/_tw.css -o src/styles/app.css"]   # comandos previos (Fase 72)

[redirects]              # Fase 75
"/viejo" = "/nuevo"

[headers."/*"]           # Fase 75
X-Robots-Tag = "noindex"

[security]               # Fase 75
csp = "default-src 'self'"
```

Todas las secciones son opcionales salvo `[project]`. Ninguna requiere
`package.json` ni npm.

## Modo App (SPA)

Una carpeta `src/app/` activa el Modo App: TSX completo que se ejecuta
en el navegador sobre `@nexa/app` (DOM real con señales, sin Virtual
DOM). Convive con `src/pages/` (Nexa Core, HTML real con SEO).

```
src/app/pages/index.tsx        -> /app
src/app/pages/users/[id].tsx   -> /app/users/:id
src/app/components/*.tsx       -> imports relativos normales
src/app/config.ts              -> opcional: export guard / notFound / loading
```

```tsx
import { state, For, Show, navigate } from "@nexa/app";
import type { PageProps } from "@nexa/app";

export default function Users(props: PageProps) {
    const users = state<{ id: number; name: string }[]>([]);
    fetch("https://mi-backend/api/users").then((r) => r.json()).then((l) => (users.value = l));
    return (
        <ul>
            <For each={users} fallback={<li>Cargando…</li>}>
                {(u) => <li><a href={`/app/users/${u.id}`}>{u.name}</a></li>}
            </For>
        </ul>
    );
}
```

| API | Qué hace |
|---|---|
| `state(v)`, `computed(fn)`, `watch(src, cb)` | Señales (`@nexa/reactivity`) |
| `effect(fn)`, `onCleanup(fn)` | Ligados a la página: se limpian al navegar |
| `{() => expr}` / `{signal}` | Hijo o atributo reactivo — solo ese nodo se actualiza |
| `value={signal}` / `checked={signal}` | Enlace bidireccional en inputs |
| `<Show when fallback>` | Condicional (`when`: valor, señal o función) |
| `<For each fallback>{(item, i) => ...}</For>` | Lista; conserva los nodos de items que siguen |
| `<Switch>`/`<Match when>` | Primer caso verdadero |
| `navigate(path, {replace})` | Navegación programática |
| `route.path` / `route.params` / `route.query` | Señales de la ruta actual |
| `guard(to)` en `config.ts` | Devuelve `false` (cancelar) o una ruta (redirigir) |

### Backend, proxy y sesión (`[proxy]`, `createApi`, `persisted`)

```toml
[proxy]
"/api" = "http://127.0.0.1:9100"   # cualquier backend
```

`nexa dev`/`nexa preview` reenvían lo que empiece con `/api` al backend
(método, cuerpo, cabeceras, cookies; las `Set-Cookie` vuelven todas), así
la app y la API comparten origen: sin CORS y con cookies `HttpOnly`.
`nexa add nginx` genera el `location` + `proxy_pass` equivalente (y el
fallback `/app/200.html` de la SPA) para producción.

```ts
import { createApi, persisted, navigate } from "@nexa/app";

export const user = persisted<string | null>("user", null);   // sobrevive a recargar

export const api = createApi({
    baseURL: "/api",
    // headers: () => ({ "X-Tenant": tenant }),   // objeto o función
    // credentials: "include",                    // solo si el backend es de otro origen (+ CORS)
    onUnauthorized: () => { user.value = null; navigate("/login", { replace: true }); },  // 401
});

const users = await api.get<User[]>("/users");       // get/post/put/patch/delete
```

La credencial real debería ser una cookie `HttpOnly` que pone el
backend; `persisted` guarda solo el estado de la UI. `guard` protege la
interfaz — la seguridad real es que el backend responda 401.

`nexa.toml` (opcional):

```toml
[app]
base = "/app"      # prefijo de URL de la SPA (por defecto /app)
title = "Mi App"   # <title> del shell
```

`nexa build` deja `dist/app/index.html` (el shell, idéntico para toda
ruta) y `dist/app/200.html` (convención de hosts de SPA); un servidor
estático debe responder con él a cualquier `/app/**` que no sea un
archivo. Ver `examples/app-demo` y la Fase 60 en
`docs/FASES-DE-CONSTRUCCION.md` para los límites conocidos.

## Límites conocidos

- Componentes (Fase 62): se expanden en tiempo de compilación — sin
  recursión, sin spread de props, y un prop solo puede ser un valor
  simbólico (ver "Componentes y `<Show>`").
- `load`/`paths`/`seo`/`schema` son literales estáticos, nunca
  funciones — sin headers, sin autenticación, sin mutaciones en el
  servidor.
- Una ruta dinámica sin `paths` solo se sirve al vuelo (`nexa
  preview`/`nexa dev`), nunca como HTML estático de `nexa build`.
- Un chunk de evento solo tiene código real si el handler es una
  función de nivel superior en el mismo archivo; un patrón más complejo
  cae a un placeholder explícito. Si el handler usa además un `const
  NOMBRE = <valor>;` de nivel superior del mismo archivo (Fase 58, bug
  #27), esa constante se antepone al chunk automáticamente cuando
  `<valor>` es un literal simple (string/número/booleano/array u
  objeto compuesto solo de esos); si no es simple (una llamada, otra
  variable, algo calculado), el build falla explícito en vez de generar
  un chunk con un identificador sin definir.
- El tree-shaking de `@nexa/ui` es a nivel de sitio completo, no por
  página individual.
- `initRouter` reemplaza solo `data-nexa-slot` cuando el proyecto usa
  `src/layout.tsx` (Fase 32) — el header/footer del layout, y cualquier
  isla que tengan montada, sobreviven la navegación sin remount. Sin
  layout (o si la página de destino no trae el mismo slot), sigue
  reemplazando `<body>` completo, igual que antes de la Fase 32. Sí
  reactiva correctamente eventos, formularios e islas de la página de
  destino (Fase 22), y limpia los de la página anterior antes de
  hacerlo — todo esto acotado al contenedor real que cambió.
- `t(...)` no soporta interpolación, pluralización, ni clave dinámica
  (una clave sin traducir sí cae al idioma por defecto, ver "Internacionalización").
- `@nexa/forms` valida solo con la Constraint Validation API nativa —
  sin reglas async o entre varios campos a la vez.
- Un layout (global o `_layout.tsx`) no admite eventos interactivos propios (`onClick`,
  etc.) — sí admite islas desde la Fase 31 — ver "Layouts compartidos".
- Un `?parametro=` en la URL del visitante no llega a `load`: la página se renderiza (y se cachea) por ruta. Pagina por ruta (`/blog/page/2`).
- WebSocket/SSE: `connectSSE`/`connectSocket` (`@nexa/http`, con
  reconexión) se usan desde el Modo App o una isla — una página de Nexa
  Core es HTML estático y no puede suscribirse a nada por sí misma.
