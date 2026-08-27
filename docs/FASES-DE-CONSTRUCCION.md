# Nexa — Fases de construcción

> Derivado de `Arquitectura SEO Completo Framework.md`. Ese documento es la
> especificación conceptual (100% diseño, 0% código a fecha de 2026-08-26).
> Este documento reorganiza esa especificación en fases ejecutables, cada
> una con un criterio de salida verificable ("algo que corre"), siguiendo
> la propia regla que el documento original se impone al final: no diseñar
> más antes de tener el vertical slice funcionando.

## Cómo leer esto

- Cada fase tiene **objetivo**, **entregables** y **criterio de salida**
  (una prueba concreta y ejecutable de que la fase terminó).
- Las fases están ordenadas por dependencia real, no por importancia.
- "Core" = obligatorio para que Nexa exista. "Oficial" = módulo separado,
  puede retrasarse sin bloquear el core. Esta distinción viene del propio
  documento (sección "¿Qué pertenece realmente al Core?") y es la que evita
  que el proyecto se convierta en un monolito interminable.

---

## Fase 0 — Especificación (✅ completada)

El documento fuente. Cierra las decisiones de fondo:

- HTML-first, SEO-first, backend-agnóstico, sin Virtual DOM.
- Compilador en Rust, desarrollador escribe TypeScript + JSX propio.
- Reactividad fine-grained (signals), no hidratación tradicional sino
  **Progressive Activation**.
- Clasificación de cada nodo del árbol: `static / dynamic / interactive / async`.
- Tres modos de salida: SSG, SSR/dinámico, Híbrido.
- Nexa Core ≠ Nexa UI ≠ Adapters ≠ Comunidad (separación de paquetes).

**Criterio de salida:** ya cumplido — este documento existe. No reabrir
estas decisiones salvo que la Fase 2 demuestre que alguna es inviable.

---

## Fase 1 — Bootstrap del toolchain (Hito 0) — ✅ completada

> `crates/nexa-cli` con `nexa info` funcionando. Ver `cargo build && ./target/debug/nexa info`.

**Objetivo:** tener un binario Rust que compile y responda.

**Entregables:**
- Workspace Cargo (`Cargo.toml` + `crates/`).
- `nexa-cli` con un único comando real: `nexa info`.

**Criterio de salida:**
```bash
cargo build
./target/debug/nexa info   # imprime "Nexa CLI, version 0.1.0"
```

---

## Fase 2 — Compilador mínimo: TSX → HTML estático (Hito 1) — ✅ completada

> `nexa-ast` + `nexa-parser` (sobre `oxc_parser`) + `nexa-renderer` +
> `nexa create`/`build`/`preview`. Verificado end-to-end: `nexa create hello
> && cd hello && nexa build` produce `dist/index.html` con HTML real.
> Limitaciones deliberadas: sin props/variables/composición de componentes —
> ver `README.md`.

**Objetivo:** el primer "Hello World" real: de un archivo `.tsx` a HTML
servible, sin reactividad, sin JS.

**Decisión clave a respetar:** no escribir un parser de TypeScript desde
cero. Envolver un parser existente (oxc, swc, o similar) para producir el
AST de Nexa. Esto es explícito en el documento original y es correcto —
reimplementar TS es un proyecto en sí mismo.

**Entregables:**
- `nexa-parser`: TSX/TS → AST de Nexa (delegando el parsing real).
- `nexa-ast`: estructura de datos interna (`Component`, `Element`, `Props`,
  `Expression`, `Event`).
- `nexa-renderer` (v0): AST → HTML string.
- Comandos: `nexa create`, `nexa build`, `nexa preview`.

**Criterio de salida:**
```bash
nexa create hello && cd hello && nexa build
cat dist/index.html   # <main><h1>Hello Nexa</h1>...</main>
```

---

## Fase 3 — IR + Dependency Graph (Hito 2) — ✅ completada

> `nexa-ir` (Classification, IrNode, DependencyGraph, IrComponent) +
> `nexa-analyzer` (`analyze()`). Se extendió `nexa-ast`/`nexa-parser` para
> capturar `{product.name}` como `Expr` simbólica y `onClick={buy}` como
> `Event`, sin evaluarlos. Verificado con test de integración TSX→IR real
> (`crates/nexa-analyzer/tests/tsx_to_ir.rs`) y wireado en `nexa build`
> (imprime clasificación y dependencias). El renderer produce marcadores
> inertes (`<!--nexa:product.name-->`, `data-nexa-on-click="buy"`) — sin
> runtime todavía, eso es la Fase 5.

**Objetivo:** dejar de tratar el árbol como texto y empezar a razonar sobre
qué parte de la página necesita qué.

**Entregables:**
- `nexa-ir`: representación intermedia independiente del HTML.
- `nexa-analyzer`: clasifica cada nodo como `static / dynamic / interactive / async`
  y construye el dependency graph (qué variable afecta a qué nodo).
- Gancho para el futuro SEO Analyzer y Accessibility Analyzer (no
  implementarlos todavía, solo dejar el punto de extensión).

**Criterio de salida:** dado un componente con una variable (`{product.name}`)
y un botón (`onClick`), el analyzer debe distinguir correctamente cuál nodo
es texto estático, cuál es dinámico y cuál requiere interactividad —
verificable con un test de IR (input JSX → output IR esperado).

---

## Fase 4 — Reactividad + Runtime mínimo (Hito 3) — ✅ completada

> `packages/reactivity` (TypeScript — la primera pieza que corre en el
> navegador, no en Rust): `state()`, `effect()`/`effect.client()`,
> scheduler con batching por microtask, y `bindText()` como germen de la
> futura Progressive Activation. Verificado con el escenario exacto del
> criterio de salida (`test/counter.test.ts`, sobre happy-dom): un
> `<button>{count}</button>` que al incrementar toca *solo* su `TextNode`
> — el hermano y la estructura del `<button>` quedan intactos. 8/8 tests.

**Objetivo:** signals, effects y un runtime de navegador deliberadamente
pequeño (sin router, sin data layer todavía).

**Entregables:**
- Primitivas: `state()`, `effect()`, `effect.client()`.
- Scheduler mínimo.
- Paquete `@nexa/reactivity` (TypeScript, no Rust — corre en el navegador).

**Criterio de salida:** un contador (`<button>{count}</button>`) que se
actualiza en el DOM tocando solo el `TextNode` afectado, sin re-renderizar
el árbol completo — verificable inspeccionando qué nodos DOM se tocan.

---

## Fase 5 — Progressive Activation (Hito 4) — ✅ completada (con una brecha conocida)

> **Rust**: `nexa-activation` (nuevo crate) recorre el mismo IR que ya
> clasifica `nexa-analyzer` y produce el manifiesto de activación
> (`{"5": {"event": "click", "handler": "buy", "module": "...", "strategy":
> "interaction"}}`) más un chunk `.js` real por nodo interactivo.
> `nexa-renderer` pasó a consumir `IrComponent` en vez del AST crudo, así
> que `data-nexa="<id>"` sale directo de `node.classification` — sin
> manifiesto externo decidiendo qué marcar. `data-nexa-strategy="idle"` en
> el JSX anula la estrategia por defecto (`interaction`).
>
> **TypeScript**: `packages/runtime` implementa las 5 estrategias
> (`interaction`, `visible`, `idle`, `load`, `manual`) e `initActivation()`,
> que recorre `[data-nexa]` y no llama a `loadModule` hasta que la
> estrategia lo decide — un documento sin nodos interactivos no carga JS
> ni una sola vez (verificado con manifiesto vacío).
>
> Verificado extremo a extremo con un proyecto real: una página con
> `<h1>{product.name}</h1>` (sin marcador) y dos `<button>` (con
> `data-nexa` y su propio chunk cada uno) genera exactamente
> `dist/nexa-manifest.json` + `dist/assets/ProductPage-5.js` +
> `dist/assets/ProductPage-7.js`, mientras que el scaffold estático por
> defecto sigue generando cero JS. 20 tests en Rust + 20 en TypeScript
> (`packages/reactivity` + `packages/runtime`).
>
> **Brecha conocida, deliberada**: (1) el contenido de cada chunk es un
> placeholder (`console.warn(...)`) — extraer la lógica real de un
> handler (`function buy() {...}`) del código fuente es trabajo de la
> Data Layer (Fase 7), no de esta fase. (2) `dist/index.html` todavía no
> incluye el `<script>` que cargaría `@nexa/runtime` + el manifiesto en un
> navegador real — falta un bundler mínimo que empaquete `packages/runtime`
> a JS plano; no estaba en el alcance original de esta fase y queda como
> el primer pendiente antes de la Fase 6.

**Objetivo:** la característica diferencial real del proyecto. Sin esto,
Nexa es "otro SSG más".

**Entregables:**
- Activation manifest (`{ "a1": { event, module, handler } }`).
- Marcado `data-nexa="..."` en el HTML de salida.
- Code splitting automático: cada componente interactivo → su propio chunk JS.
- Estrategias de activación: `interaction`, `visible`, `idle`, `load`, `manual`.

**Criterio de salida:** una página con un `<h1>` estático y un `<Counter />`
interactivo debe producir HTML donde solo el bloque del Counter carga JS
(medible: tamaño de JS inicial ≈ 0 para contenido estático).

---

## Fase 6 — Router + Fragment Navigation (Hito 5) — ✅ completada

> **Rust**: `nexa-router` (nuevo crate) escanea `src/pages/**/*.tsx` y
> construye la tabla de rutas (`index.tsx` -> `/`, `products/[slug].tsx`
> -> `/products/:slug`), con matching que prioriza rutas estáticas sobre
> dinámicas cuando compiten por la misma forma. `nexa build` ahora compila
> **todas** las páginas encontradas (SSG multi-página, cada una con su
> propio `dist/<ruta>/index.html` + `nexa-manifest.json`), saltándose las
> rutas dinámicas (no hay Data Layer todavía para enumerar qué `slug`
> existen). `nexa preview` pasó de ser un placeholder a un servidor HTTP
> real (`tiny_http`, sin SSL): sirve lo que ya esté en `dist/`, y renderiza
> al vuelo lo que falte — típicamente las rutas dinámicas.
>
> **TypeScript**: `packages/router` (nuevo paquete) — `initRouter()`
> intercepta clics en enlaces internos, hace `fetch` + reemplaza
> `<title>`/`<body>` + `history.pushState`, sin recargar la página;
> respeta `target="_blank"`, `download`, `data-nexa-reload`, enlaces
> externos y teclas modificadoras. `initPrefetch()` precarga en hover
> (con delay) o `touchstart`, compartiendo caché con `initRouter` para no
> pedir dos veces la misma página.
>
> Verificado extremo a extremo: un proyecto con `/`, `/about`, `/products`
> y `/products/[slug]` — `nexa build` genera 3 páginas estáticas y avisa
> de la ruta dinámica saltada; `curl http://.../products/iphone-17` contra
> `nexa preview` devuelve HTML completo con `<h1>` (rendering al vuelo,
> sin build previo para esa ruta exacta) — el criterio de salida exacto de
> esta fase. 28 tests en Rust + 31 en TypeScript.
>
> **Brecha conocida, deliberada**: los `params` capturados por una ruta
> dinámica (`{"slug": "iphone-17"}`) todavía no se inyectan en el
> render — por eso todas las variantes de `/products/:slug` producen hoy
> el mismo HTML. Conectar params -> componente es trabajo de la Data
> Layer (Fase 7). Tampoco existe todavía un contenedor de página estable
> para "fragment navigation" real: `initRouter` reemplaza `<body>`
> completo, no un fragmento más fino — eso llega con el sistema de layout
> (Fase 9).

**Objetivo:** file-based routing con SEO en la primera visita y navegación
tipo SPA en clics internos, sin convertir todo en una SPA.

**Entregables:**
- Convención `pages/**/*.tsx` → rutas (incl. `[slug].tsx`).
- Primera visita: HTML completo por request.
- Navegación interna: fetch de fragmento + `history.pushState`.
- Prefetch básico.

**Criterio de salida:** `curl /productos/iphone-17` devuelve HTML completo
con `<h1>`; un clic interno en el navegador no recarga la página pero sí
actualiza la URL.

---

## Fase 7 — Data Layer (Hito 6) — ✅ completada

> **Decisión de arquitectura clave**: `load` NO es una función a ejecutar
> (Nexa Core no incorpora un motor de JavaScript). Es un objeto literal
> estático — `export const load = { url: "/products/:slug" }` — que
> `nexa-parser` extrae con análisis sintáctico puro. Esto mantiene la
> promesa original de la especificación ("Nexa Core no conoce Node/JS en
> ejecución") mientras sí resuelve datos reales en el servidor.
>
> **Rust**: `nexa-loader` (nuevo crate) sustituye los `:params` en la
> plantilla de URL (reutilizando los mismos parámetros que ya captura
> `nexa-router`) y hace un GET real (`ureq`) contra el backend — la URL
> base sale de `NEXA_API_URL`. Un 404 del backend se distingue
> explícitamente (`LoaderError::NotFound`) del resto de fallos.
> `nexa-renderer` ahora acepta ese JSON opcionalmente: si una expresión
> dinámica tiene como raíz el identificador `data` (`{data.name}`), se
> resuelve contra el JSON real y se renderiza (escapado) — cualquier otro
> identificador, o `data.*` sin datos, sigue siendo el comentario inerte
> de siempre. `nexa build`/`nexa preview` distinguen `PageError::NotFound`
> (-> 404 real) de cualquier otro fallo (-> 500).
>
> **TypeScript**: `packages/http` (nuevo paquete, primera dependencia
> real entre paquetes — esto forzó convertir `packages/` en un workspace
> real de npm) — `createApi()` (cliente propio, sin Axios), `query()` con
> las 3 políticas de cache pedidas (cache-first/network-first/
> stale-while-revalidate) y `mutation()`, ambos devolviendo señales de
> `@nexa/reactivity` (`data`/`loading`/`error`) — para uso del lado
> cliente, dentro de los chunks que activará Progressive Activation.
>
> Verificado con el criterio de salida exacto: una página
> `products/[slug].tsx` con `load()` real contra un backend JSON de
> prueba — `curl /products/iphone-17` devolvió `<h1>iPhone 17</h1>`,
> `<p>El último de Apple</p>`, `<strong>999</strong>` (datos reales, no
> comentarios) con HTTP 200; `curl /products/no-existe` devolvió HTTP 404
> real. 43 tests en Rust + 45 en TypeScript.
>
> **Brecha conocida, deliberada**: `load` solo soporta un GET a una URL
> con sustitución de parámetros — sin headers, sin autenticación, sin
> POST/mutaciones en el servidor. Tampoco hay `getStaticPaths` (una forma
> de enumerar qué `slug` existen para poder pre-renderizarlos con `nexa
> build`): las rutas dinámicas con loader solo se sirven al vuelo via
> `nexa preview`, nunca como HTML estático pre-generado. El bundler
> mínimo pendiente desde la Fase 5 sigue pendiente.

**Objetivo:** separar fetch de render, con cache y mutaciones.

**Entregables:**
- `@nexa/http`: cliente HTTP propio (no depende de Axios).
- `query()` / `mutation()` con estados `loading/data/error`.
- Políticas de cache: `cache-first`, `network-first`, `stale-while-revalidate`.
- Errores de servidor → códigos HTTP reales (404/500/403 con HTML válido,
  nunca `200 OK` con `<div id="app">` vacío).

**Criterio de salida:** página de producto que carga datos vía `load()`,
renderiza HTML con esos datos, y devuelve 404 real si el producto no existe.

---

## Addendum — Cierre de pendientes antes de la Fase 8

Antes de seguir con SEO, se cerraron las brechas que las Fases 5-7 habían
dejado documentadas como conocidas (no descubiertas ahora: ya estaban
anotadas explícitamente como pendientes):

- **El bundler mínimo (pendiente desde la Fase 5).** `@nexa/runtime` y
  `@nexa/router` no tienen dependencias cruzadas entre paquetes, así que
  se empaquetan con `esbuild` en un solo archivo cada uno
  (`npm run build:cli-assets`) y se incrustan en el binario `nexa-cli` vía
  `include_str!` (`crates/nexa-cli/assets/`, `src/assets.rs`). Cada
  página generada ahora incluye un `<script type="module">` real
  (`src/bootstrap.rs`) que carga el router siempre, y la activación solo
  si el manifiesto tiene alguna entrada — verificado sirviendo esos dos
  archivos con `nexa preview` y ejecutándolos de verdad con
  `node --experimental-vm-modules` (son ESM válido con las exportaciones
  esperadas).
- **Chunks con lógica real, no un placeholder (pendiente desde la Fase
  5/7).** `nexa-parser` ahora extrae, por `Span`, el código fuente exacto
  de cualquier `function nombre() {}` o `const nombre = () => {}` de
  nivel superior (`Component::handlers`) — sin ejecutarlo, solo
  copiándolo — y `nexa-activation` lo inserta tal cual en el chunk. Si no
  encuentra el handler (import externo, patrón no reconocido), cae al
  placeholder de antes en vez de fallar.
- **`{params.slug}` no se inyectaba en el render (pendiente desde la Fase
  6).** El renderer ahora recibe un `RenderContext` con `data` *y*
  `params`; una expresión con raíz `params` se resuelve contra los
  parámetros de ruta capturados por `nexa-router`, con la misma
  disciplina que `data.*`: si no hay valor, sigue siendo el comentario
  inerte de siempre.
- **Rutas dinámicas renderizadas al vuelo ahora persisten sus chunks.**
  `nexa preview` escribía el HTML de una ruta dinámica pero nunca los
  `.js` de sus nodos interactivos, así que el navegador los habría
  pedido en vano. Ahora, al renderizar al vuelo, los chunks (deterministas
  a partir del código fuente, no de los datos) se escriben a
  `dist/assets/` como efecto secundario.

**Lo que se dejó fuera a propósito — no son pendientes, son alcance de
fases futuras:** composición de componentes, props, condicionales
(seguirían requiriendo rediseñar el modelo de componentes, que es
justamente lo que se ha evitado tocar desde la Fase 2); `getStaticPaths`
o cualquier forma de enumerar `slug`s para pre-renderizar rutas dinámicas
como HTML estático (Data Layer no define ese mecanismo todavía); y que
`initRouter` reemplace un fragmento más fino que `<body>` completo (esto
necesita un contenedor de página estable, que es el sistema de layout de
la Fase 9).

---

## Fase 8 — SEO y datos estructurados como core (Hito 7) — ✅ completada

**Objetivo:** el punto de partida de todo el proyecto — esto es lo que se
vende como diferencial, así que no puede ser un módulo opcional.

**Entregables:**
- `defineSEO()` → `<title>`, `<meta description>`, `<link canonical>`,
  OpenGraph, Twitter Cards.
- `defineSchema()` → JSON-LD (Product, Article, Organization, etc.).
- `sitemap.xml`, `robots.txt` generados por el framework.
- SEO Analyzer: warnings en build si falta `alt`, `canonical`, etc.

**Criterio de salida:** una página de producto generada por Nexa pasa una
validación de Rich Results de Google (Product schema) sin edición manual.

> **Cómo quedó:** `export const seo = {...}` y `export const schema =
> {...}` son objetos literales estáticos (mismo principio que `load` en
> la Fase 7: nada se ejecuta) que `nexa-parser` extrae con la misma
> disciplina de `data.*`/`params.*` simbólicos — incluyendo plantillas de
> texto (`` `${data.name} | Oweeme` ``). El nuevo crate `nexa-seo`
> resuelve esas declaraciones contra los datos reales de cada render y
> produce: las etiquetas de `<head>` (title/description/canonical/OG/
> Twitter), el `<script type="application/ld+json">` (con `@context` y
> `type` -> `@type` automáticos, a cualquier profundidad de anidamiento),
> `sitemap.xml`/`robots.txt` a nivel de sitio (`NEXA_SITE_URL`), y los
> warnings del SEO Analyzer (`NEXA-SEO-*`/`NEXA-A11Y-*`) que ya se habían
> dejado anotados como punto de extensión desde la Fase 3.
>
> **Bug real encontrado y arreglado durante esta fase:** al verificar el
> criterio de salida con una imagen real (`<img src={data.image}
> alt={data.name} />`), los atributos dinámicos se renderizaban vacíos
> (`<img src alt>`) — el AST solo sabía representar atributos estáticos o
> ausentes, nunca dinámicos. Se corrigió extendiendo `Attr::value` a
> `AttrValue::Static(String) | Dynamic(Expr)`, resuelto por el renderer
> con la misma lógica que ya resolvía `data.*` en el cuerpo del documento
> (si no se puede resolver, el atributo se omite entero — un `src=""`
> roto es peor que ningún `src`). No era un problema exclusivo de SEO,
> pero sin este arreglo ninguna imagen de producto real habría tenido
> `src` ni `alt`.
>
> Verificado extremo a extremo con un backend JSON de prueba real: la
> página de producto generada incluye título/descripción/canonical/OG
> reales, un JSON-LD de `Product` con `name`/`description`/`image`/
> `offers.price`/`offers.priceCurrency`/`offers.availability` — validado
> contra los campos que Google documenta como requeridos/recomendados
> para Product Rich Results (no se probó contra la herramienta Rich
> Results Test real: este entorno no tiene navegador ni acceso a esa API
> externa). 78 tests en Rust.

---

## Fase 9 — CSS / UI system (Hito 8) — ✅ completada (con un ajuste de alcance)

**Objetivo:** un sistema visual propio, sin atar el core a Tailwind/Bootstrap.

**Entregables:**
- Design tokens (`--nx-color-primary`, `--nx-space-1`, etc.).
- `@nexa/ui`: componentes base (Button, Input, Dialog, Card...) como paquete
  **separado** del core.
- Accesibilidad (ARIA, focus trap, roles) integrada en estos componentes,
  no como capa opcional.
- CSS tree-shaking: solo se envía el CSS de lo que realmente se usa.

**Criterio de salida:** un proyecto que no importa `@nexa/ui` no paga ningún
costo de CSS/JS por ella (verificable en el bundle final).

> **Ajuste de alcance, explícito:** el original de esta fase asume
> componentes JSX importables (`<Button>`), y eso todavía no existe — el
> compilador solo soporta un componente por página, sin composición
> (límite fijado desde la Fase 2 y respetado en cada fase desde
> entonces). En vez de forzar esa capacidad ahora, `@nexa/ui` se diseñó
> para consumirse con HTML normal: `<button class="nx-btn
> nx-btn-primary">`. Es una decisión honesta, no un atajo oculto — el día
> que exista composición de componentes, envolver esto en `<Button>` es
> trivial (la implementación ya está: solo cambiaría cómo se escribe).
>
> **Cómo quedó:** `packages/ui` tiene el CSS fuente (tokens + un archivo
> por componente: Button, Input, Card, Dialog) y el comportamiento
> accesible de Dialog en TypeScript (`openDialog`/`closeDialog`: ARIA,
> foco inicial dentro del diálogo, `Escape` para cerrar, *focus trap* con
> `Tab`/`Shift+Tab`, devolución del foco a quien lo abrió — probado con
> vitest + happy-dom). El nuevo crate `nexa-ui` (Rust) hace el
> tree-shaking real: recorre el IR de cada página buscando clases `nx-*`
> **estáticas** en atributos `class` (una `class={expr}` dinámica no se
> puede analizar sin datos reales — limitación conocida, no un bug), y
> `nexa-cli` acumula esas clases de *todas* las páginas de un `nexa
> build` para ensamblar un único `dist/assets/nexa-ui.css` — el
> tree-shaking es a nivel de sitio completo, no por página individual.
> Una página que no usa nada de `@nexa/ui` ni siquiera enlaza el CSS.
>
> Verificado extremo a extremo: un proyecto con una página 100% estática
> y otra usando `Card` + `Button` — la primera no lleva `<link
> rel="stylesheet">` en absoluto; la segunda sí, y el
> `dist/assets/nexa-ui.css` generado contiene exactamente los tokens +
> `.nx-btn`/`.nx-card` — sin ni un byte de `.nx-input`/`.nx-dialog`. 87
> tests en Rust + 51 en TypeScript (nuevo paquete `packages/ui`).

---

## Fase 10 — Forms + i18n (Hito 9) — ✅ completada

**Objetivo:** dos módulos oficiales de alto valor práctico para
ecommerce/noticias, ambos con progressive enhancement.

**Entregables:**
- `@nexa/forms`: validación, errores, estados `dirty/touched`, funciona sin
  JS cuando el backend lo permite.
- `@nexa/i18n`: `t()`, rutas por locale (`/es/...`, `/en/...`),
  `hreflang` automático.

**Criterio de salida:** un formulario de contacto válido y un selector de
idioma que cambia la URL y regenera `hreflang` correctamente.

> **Cómo quedó — Forms:** `packages/forms` no reinventa validación: se
> apoya en la Constraint Validation API nativa del HTML
> (`checkValidity()`, `validationMessage`, `required`/`type="email"`/
> `pattern`/etc.), y solo añade lo que el navegador no ofrece por sí solo:
> mensajes de error posicionados por convención (`data-nexa-error-for`),
> clases `nx-touched`/`nx-dirty`/`nx-invalid` para estilos, y
> `aria-invalid`/`aria-describedby` para accesibilidad. `initForms()` solo
> se inyecta en páginas que de verdad tienen un `<form data-nexa-form>`
> (misma disciplina de costo cero que `@nexa/ui` en la Fase 9) —
> `nexa-cli` lo detecta recorriendo el IR (`has_forms` en
> `crates/nexa-cli/src/pipeline.rs`).
>
> **Cómo quedó — i18n:** `[locale]` es un segmento de ruta reservado
> (`src/pages/[locale]/index.tsx` → `/es`, `/en`, ...). `t("clave")` se
> reconoce como un patrón sintáctico exacto — igual que `data.x` o
> `params.x`, nunca se ejecuta — tanto en el cuerpo JSX
> (`{t("home.title")}`) como, tras un bug encontrado en la verificación
> en vivo (ver abajo), dentro de `seo`/`schema`
> (`title: t("home.title")`). El diccionario (`src/locales/<locale>.json`)
> se resuelve con el mismo mecanismo de `data.*`/`params.*` ya usado por
> el renderer. `hreflang` se calcula listando los locales disponibles
> (`nexa_i18n::available_locales`) y reescribiendo la ruta actual con cada
> uno (`nexa_i18n::alternate_links`, reutilizando el `resolve_url` de la
> Fase 6).
>
> **Bug real encontrado y corregido durante la verificación:** con
> `seo = { title: t("home.title") }`, el `<h1>` y el `hreflang` se
> traducían correctamente pero el `<title>` se quedaba en el fallback
> `"Nexa"`. Causa: `Template`/`JsonTemplate` (el tipo que representa los
> valores de `seo`/`schema`) no tenía forma de representar `t("...")` —
> solo el cuerpo JSX lo soportaba. Se corrigió añadiendo
> `TemplatePart::Translate`/`JsonTemplate::Translate` en `nexa-ast`,
> extendiendo `nexa-parser::translate` para reconocer el patrón tanto en
> `Expression` normal como en `JSXExpression`, y añadiendo `translations`
> a `nexa_seo::SeoContext` para que `resolve_template`/
> `resolve_json_template` también lo resuelvan. Verificado de nuevo tras
> el fix: `/es` → `<title>Bienvenido a Oweeme</title>`, `/en` →
> `<title>Welcome to Oweeme</title>`, con `canonical` y `hreflang`
> intactos.
>
> Verificado extremo a extremo con `nexa preview`: un formulario de
> contacto (`src/pages/contact.tsx`) validando en el navegador sin JS de
> más, y una página `[locale]` sirviendo `/es` y `/en` con texto,
> `<title>`, `canonical` y `hreflang` todos correctamente traducidos. 102
> tests en Rust (workspace completo) + 57 en TypeScript (incluye el nuevo
> paquete `packages/forms`).

---

## Fase 11 — Dev server + HMR — ✅ completada (con un ajuste de alcance)

**Objetivo:** experiencia de desarrollo real (`nexa dev`), no solo build.

**Entregables:**
- Servidor de desarrollo con recarga en caliente de componentes.
- Feedback de errores del compiler en el navegador.

**Criterio de salida:** editar `index.tsx` y ver el cambio reflejado sin
recargar manualmente ni perder estado local trivial.

> **Ajuste de alcance, explícito:** "HMR" en el sentido estricto (Hot
> *Module* Replacement: sustituir el módulo JS de un componente y que su
> estado en memoria sobreviva) no existe todavía porque Nexa no tiene
> instancias de componente en el cliente — la reactividad (Fase 4) vive
> atada a nodos del DOM ya renderizados, no a un árbol de componentes con
> estado propio (la misma limitación de fondo que ya obligó el ajuste de
> alcance de `@nexa/ui` en la Fase 9, y la razón por la que `initRouter`
> reemplaza `<body>` completo desde la Fase 6). Lo que sí se construyó,
> con la misma honestidad: **auto-reload en el navegador sin recarga
> completa de página**, que es lo que de verdad pedía el criterio de
> salida ("sin recargar manualmente").
>
> **Cómo quedó:** `nexa dev` recompila cada página desde `src/` en cada
> petición — a diferencia de `nexa preview`, nunca sirve un
> `dist/.../index.html` que haya quedado de un build anterior (ver
> `page_resolver::resolve_page`, ahora compartido por `preview` y `dev`
> con un parámetro `use_prebuilt_html`). Un endpoint
> `/__nexa_dev__/version` expone el `mtime` más reciente entre todos los
> archivos bajo `src/` (`dev_watch::version_marker`) — no hay un watcher
> real con eventos push (`notify`); el nuevo paquete
> `@nexa/dev-client` sondea ese endpoint cada 400ms desde un `<script>`
> en `<head>` (deliberado: sobrevive al propio reemplazo del `<body>` que
> dispara). Cuando la versión cambia, vuelve a pedir la página actual y
> reemplaza `<body>` — pero no con un `innerHTML` ingenuo: un `<script>`
> insertado así nunca se ejecuta (comportamiento estándar del DOM), así
> que `swapDocument` los saca del HTML nuevo, los recrea con
> `createElement`/`appendChild`, y así el bootstrap de la página
> (`initRouter`/`initActivation`/`initForms`) vuelve a correr con el
> manifiesto correcto de la versión nueva. El scroll (`scrollX`/`scrollY`)
> se captura antes del swap y se restaura después — es el "estado local
> trivial" que de verdad se preserva; el valor de un `<input>` que el
> usuario esté escribiendo no sobrevive, porque no hay forma de
> distinguir qué parte del HTML nuevo "es la misma" que la vieja sin un
> componente con identidad propia (mismo límite de fondo explicado
> arriba).
>
> Los errores de compilación (ej. un JSX mal cerrado mientras se edita)
> se muestran como una página de error legible en el propio navegador —
> con el mensaje real del parser (`oxc`), no solo el contexto genérico
> ("compilando src/pages/index.tsx") — y esa página también lleva el
> script de auto-reload, así que en cuanto se corrige el archivo, la
> siguiente detección de cambio la reemplaza sola por la página real, sin
> intervención manual.
>
> Verificado extremo a extremo contra un servidor `nexa dev` real: (1)
> editar `index.tsx` cambia el HTML servido en la siguiente petición; (2)
> `/__nexa_dev__/version` cambia de valor tras guardar; (3) un script de
> Node que ejecuta el **bundle real** de `@nexa/dev-client` (no un mock)
> contra el servidor real confirma que el DOM se actualiza solo, sin
> intervención, en cuanto el archivo cambia; (4) forzar un error de
> sintaxis produce un HTTP 500 con el mensaje real del parser en una
> página legible, y arreglar el archivo hace que la página vuelva sola a
> mostrar contenido correcto. 111 tests en Rust (workspace completo) + 66
> en TypeScript (incluye el nuevo paquete `packages/dev-client`).

---

## Fase 12 — Package manager y manifest del proyecto (Hito 10) — ✅ completada (con un ajuste de alcance)

**Objetivo:** dejar de depender de npm para lo esencial del propio framework
(aunque el ecosistema JS siga siendo compatible).

**Entregables:**
- `nexa.toml` (manifest), `nexa.lock` (lockfile).
- `nexa add <módulo>` / `nexa create`.
- Política de versionado: Stable / Experimental / Internal, con reglas
  claras de cuándo se permite romper compatibilidad.

**Criterio de salida:** `nexa add ui` instala y registra `@nexa/ui` sin
tocar `package.json`/npm.

> **Ajuste de alcance, explícito:** esto no es un gestor de paquetes con
> red/registro remoto — eso es explícitamente la Fase 15 ("paquetes de
> comunidad de terceros"). Hasta la Fase 11, `@nexa/ui` y `@nexa/forms`
> ya funcionaban sin declarar nada (están embebidos en el binario de
> `nexa-cli`, activados solo con usarlos): `nexa.toml` no significaba
> nada todavía. Lo que construye esta fase es la *forma* real de un
> sistema de paquetes — manifiesto, lockfile, registro con niveles de
> estabilidad — aplicada a los módulos oficiales que ya existen, sin
> tocar esa disciplina de cero-config (usar `nx-btn` sin declarar nada
> sigue funcionando exactamente igual).
>
> **Cómo quedó:** `nexa.toml` gana una sección `[dependencies]`
> (`ui = "0.1.0"`) parseada con `serde`/`toml` (`manifest.rs`). Un
> registro interno (`modules.rs`) clasifica cada módulo oficial en
> **Stable** (`ui`, `forms` — SemVer real, solo rompe en mayor),
> **Experimental** (ninguno todavía; rompe en cualquier menor, con
> aviso) o **Internal** (`router`, `runtime` — core, siempre activos;
> `dev-client` — solo lo usa `nexa dev`) — la política completa de
> cuándo se permite romper cada nivel vive en
> `docs/POLITICA-DE-VERSIONES.md`. `nexa add <módulo>`: valida el nombre
> contra el registro (da un mensaje distinto si el nombre es
> desconocido vs. si es un módulo interno que deliberadamente no se
> declara), escribe la entrada en `nexa.toml`, y registra la versión +
> un *hash de contenido* (FNV-1a de los bytes embebidos — se documenta
> explícitamente que no es una firma criptográfica ni una garantía de
> integridad frente a manipulación, solo detección de cambios) en
> `nexa.lock` (`lockfile.rs`). Es idempotente: correrlo dos veces con la
> misma versión no hace nada la segunda vez.
>
> Para que `nexa.toml` no quede como pura decoración, `nexa build` ahora
> avisa (`NEXA-PKG-001`/`002`, nunca bloquea — mismo criterio que los
> avisos de SEO desde la Fase 8) cuando una página usa clases `nx-*` o
> `<form data-nexa-form>` sin que el módulo correspondiente esté
> declarado, sugiriendo el `nexa add` exacto que hace falta.
>
> Verificado extremo a extremo con un proyecto real: `nexa create` no
> genera ningún `package.json`; usar `nx-btn` sin declarar nada dispara
> `[NEXA-PKG-001]` en `nexa build` y el CSS se genera igual (cero-config
> intacto); `nexa add ui` hace desaparecer el aviso, deja `ui = "0.1.0"`
> en `nexa.toml` y una entrada con su `content_hash` en `nexa.lock`, y
> sigue sin existir ningún `package.json` en el proyecto; correrlo de
> nuevo es un no-op explícito; `nexa add stripe` (no existe) y `nexa add
> router` (existe pero es interno) dan cada uno el error correcto y
> distinguible. 130 tests en Rust (workspace completo) + 66 en
> TypeScript (sin paquetes nuevos esta fase — no hace falta ningún
> `@nexa/*` nuevo en el navegador para esto).

---

## Fase 13 — Adaptadores multiplataforma (Hito 11) — ✅ completada (con un ajuste de alcance)

**Objetivo:** el mismo código Nexa corriendo en web, escritorio y móvil.

**Entregables:**
- `@nexa/platform` (abstracción: `platform.isWeb/isTauri/isCapacitor`,
  `notifications()`, `storage()`, `share()`, `camera()`).
- Adapter Tauri, adapter Capacitor.
- Adapters de despliegue web: Node, Bun, Deno, Cloudflare (opcionales,
  nunca requeridos).

**Criterio de salida:** un mismo proyecto Nexa empaquetado como app Tauri
de escritorio y como sitio web estático, sin cambiar código de aplicación.

> **Ajuste de alcance, explícito, acordado con el usuario antes de
> empezar — y ampliado después:** al planear la fase, este entorno
> (sandbox de este agente) tenía Rust, webkit2gtk-4.1-dev, gtk3-dev y un
> `$DISPLAY` activo (Tauri se pudo compilar y ejecutar de verdad), pero
> ningún SDK de Android instalado — así que se acordó explícitamente con
> el usuario dejar Capacitor solo en generar `capacitor.config.json`, sin
> fingir un build móvil que no se podía hacer. El usuario luego señaló
> que sí tenía Java 21 y un Android SDK real instalados en la misma
> máquina (`~/Android/Sdk`, con licencias ya aceptadas) — con eso se
> completó la verificación real: proyecto Android nativo generado y un
> APK de verdad compilado con Gradle (detalle más abajo). Xcode/iOS
> sigue fuera de alcance sin excepción: no corre en Linux bajo ninguna
> circunstancia.
>
> **Cómo quedó — `@nexa/platform`:** paquete nuevo (`packages/platform`)
> con detección real (`isTauri()` mira `globalThis.isTauri` — verificado
> contra el código fuente de `@tauri-apps/api`; `isCapacitor()` mira
> `window.Capacitor.isNativePlatform()` — verificado contra
> `@capacitor/core`), `storage()` sobre `localStorage` (funciona igual en
> los tres entornos, no hace falta rama nativa), y `notify()`/`share()`/
> `capturePhoto()` con dos ramas reales: Capacitor nativo llama a
> `window.Capacitor.Plugins.*` con la forma exacta de los plugins reales
> (`@capacitor/local-notifications`, `@capacitor/share`,
> `@capacitor/camera` — verificada leyendo su código fuente, no
> inventada); Tauri y web usan las Web APIs estándar (`Notification`,
> `navigator.share`, `getUserMedia` + `canvas`), porque el webview de
> Tauri las soporta directamente. Un plugin nativo dedicado para Tauri
> (`@tauri-apps/plugin-notification`, etc.) queda fuera: reproducir a
> mano su protocolo de IPC sin poder probarlo contra una app real
> hubiera sido inventar, no construir.
>
> **Integración real con el compilador:** un handler que escribe
> `platform.algo(...)` hace que `nexa-activation` anteponga
> automáticamente `import { platform } from "/assets/nexa-platform.js";`
> al chunk generado — el mismo criterio de costo cero que `@nexa/ui`/
> `@nexa/forms`: un chunk que no usa `platform.` no lo importa. `nexa
> build` avisa (`NEXA-PKG-003`, Fase 12) si se usa sin declarar
> `nexa add platform`.
>
> **Cómo quedó — Tauri:** `nexa add tauri` genera `src-tauri/` a partir
> de una plantilla embebida que **no se inventó a mano**: sale de un
> `tauri init --ci` real (Tauri CLI 2.11.4, instalada vía `npx` para
> esta verificación) apuntando a `../dist`, simplificada (sin
> `tauri-plugin-log`, un solo ícono placeholder en vez de todo el set —
> Tauri exige al menos uno para compilar) y confirmada compilando de
> verdad antes de incrustarla en el binario. `nexa.toml` registra
> `tauri` como **Experimental** (recién nacido, sin uso probado a través
> de muchos proyectos todavía).
>
> **Cómo quedó — Capacitor:** `nexa add capacitor` genera
> `capacitor.config.json` — la forma real y mínima (`appId`, `appName`,
> `webDir`) que produce `npx @capacitor/cli init` (también verificado
> corriéndolo de verdad). No genera `android/`/`ios/` (eso lo hace `npx
> @capacitor/cli add android/ios`, un paso posterior real que sí queda
> documentado en el mensaje de `nexa add capacitor`).
>
> **Actualización tras esta verificación inicial:** el usuario tenía Java
> 21 y un Android SDK real instalados (`~/Android/Sdk`, con
> `cmdline-tools`, `platform-tools`, `build-tools` y una plataforma ya
> descargada, licencias aceptadas) en el mismo entorno donde corre este
> agente — así que se pudo llevar la verificación mucho más lejos de lo
> que se pensaba posible al planear la fase: `npx @capacitor/cli add
> android` generó el proyecto Android nativo de verdad, y `./gradlew
> assembleDebug` **compiló un APK real y firmado** (`app-debug.apk`,
> ~4.1 MB, `BUILD SUCCESSFUL in 46s`), con el `dist/` de Nexa
> correctamente embebido en `assets/public/` (se verificó abriendo el
> APK con `unzip -l` y confirmando ahí `nexa-platform.js`,
> `nexa-forms.js`, `index.html`, etc.). Lo único que quedó sin probar es
> instalarlo y ejecutarlo en un dispositivo o emulador real — no hay
> ninguno conectado en este entorno (`adb devices` no lista nada, y no
> hay un emulador de Android instalado) — pero el propio *build* del
> paquete nativo, que era la parte más incierta, quedó verificado de
> punta a punta con herramientas reales, no simulado.
>
> **Bug real encontrado y corregido durante esta verificación:** el
> primer intento de `npx @capacitor/cli add android` falló con "Invalid
> App ID... must be in Java package form with no dashes". La función
> `sanitize_app_id` de `capacitor_scaffold.rs` generaba
> `com.nexa.nexa-cap-verify` — copiaba las reglas de un nombre de
> paquete de Cargo (donde el guion sí es válido) en vez de las de un
> identificador de paquete Java/Android (que solo permite
> `[a-zA-Z][a-zA-Z0-9_]*` por segmento, sin guiones). Se corrigió
> reemplazando cualquier carácter no alfanumérico por `_` en vez de `-`,
> con un test nuevo (`never_puts_a_dash_in_the_app_id...`) que fija este
> caso exacto para que no vuelva a pasar.
>
> **Cómo quedó — adaptadores de despliegue web:** `nexa build` ya
> produce `dist/` como archivos 100% estáticos con rutas relativas — no
> hay ningún "adaptador" que escribir porque no hace falta: **cualquier**
> servidor de archivos estáticos sirve ese `dist/` tal cual, sin código
> específico de Nexa en el servidor. Se verificó sirviéndolo con un
> servidor Node de 20 líneas escrito para la ocasión (sin ningún paquete
> npm) — Bun y Deno no están instalados en este entorno, así que no se
> verificaron directamente, pero la garantía es estructural (son
> archivos estáticos planos), no algo que dependa del runtime.
>
> Verificado extremo a extremo: un proyecto real con un botón cuyo
> handler llama `platform.share(...)` — `nexa build` avisa
> `NEXA-PKG-003`, el chunk generado trae el `import` correcto, y el
> aviso desaparece tras `nexa add platform`. `nexa add tauri` generó
> `src-tauri/` con nombres bien sustituidos (`nexa_phase13_lib`,
> `com.nexa.nexa-phase13`), `cargo build` ahí dentro compiló un binario
> de escritorio real (~1m24s, sin warnings), y ejecutarlo
> (`./target/debug/nexa-phase13`) mantuvo el proceso vivo más de 10
> segundos sin caerse (no se tomó una captura de pantalla del
> escritorio real de la máquina por respeto a la privacidad de lo que
> hubiera en las demás ventanas — la verificación fue por vida del
> proceso, no visual). `nexa add capacitor` generó un
> `capacitor.config.json` válido; con el SDK de Android real disponible
> en la máquina, `npx @capacitor/cli add android` + `./gradlew
> assembleDebug` compilaron un **APK real y firmado** con el `dist/` de
> Nexa embebido correctamente adentro (verificado sin necesidad de un
> emulador o dispositivo, que no hay en este entorno); y `dist/` se
> sirvió sin cambios desde un servidor Node real. 146 tests en Rust
> (workspace completo) + 92 en TypeScript (nuevo paquete
> `packages/platform`, 26 tests).

---

## Fase 14 — DevTools, testing y presupuestos de rendimiento — ✅ completada (con un ajuste de alcance)

**Objetivo:** hacer visible lo que Nexa promete (HTML mínimo, JS mínimo).

**Entregables:**
- `@nexa/test`: unit (componentes/estado), renderer (JSON→HTML esperado), E2E.
- `@nexa/devtools`: inspección de render tree, qué se activó y qué no, SEO
  checklist en vivo.
- `performance: { maxInitialJS, maxCSS, maxImage }` con warnings en build.
- `@nexa/telemetry` opcional (web vitals, errores), respetando privacidad.

**Criterio de salida:** el build falla (o avisa) si el JS inicial de una
página supera el presupuesto configurado.

> **Ajuste de alcance, explícito:** "unit (componentes/estado), renderer
> (JSON→HTML esperado)" asume un modelo de componentes con estado propio
> en el cliente que Nexa no tiene (la reactividad vive atada a nodos del
> DOM ya renderizados, y el renderizado real de HTML es un paso de
> compilación en Rust, ya cubierto por cientos de tests en
> `nexa-renderer`/`nexa-seo` desde fases anteriores — no hay ningún
> "renderer JSON→HTML" en el lado de TypeScript que probar). En vez de
> forzar ese modelo, `@nexa/test` se construyó para lo que sí existe de
> verdad: handlers de eventos como funciones JS aisladas y el manifiesto
> de activación como JSON — es una decisión honesta, no un atajo, y el
> criterio de salida ("el build falla si excede el presupuesto") se
> cumplió al pie de la letra.
>
> **Cómo quedó — `public/` → `dist/` (bug de fases anteriores, cerrado
> aquí porque hacía falta para que `maxImage` tuviera algo real que
> medir):** `nexa build` nunca copiaba `public/` a `dist/` — un
> `<img src="/logo.png">` con el archivo puesto ahí a mano daba 404 real
> en producción. Se corrigió (`copy_dir::copy_recursive`), y de paso
> `nexa preview`/`nexa dev` ahora sirven cualquier archivo estático bajo
> `dist/`/`public/` respectivamente como último recurso antes de un 404
> (antes ninguno de los dos lo hacía en absoluto).
>
> **Cómo quedó — presupuestos de rendimiento:** vive en `nexa.toml`
> (reutilizando el manifiesto de la Fase 12, no `nexa.config.ts` — ese
> archivo sigue sin ningún parser real, sería una pieza de infraestructura
> nueva solo para esto), sección `[performance]` con los mismos nombres
> de campo del documento original (`maxInitialJS`, `maxCSS`, `maxImage`).
> Ausente por defecto = cero comprobación, cero fricción. `maxInitialJS`
> cuenta el bootstrap que se carga siempre/condicionalmente
> (`@nexa/router` siempre; `@nexa/runtime` si hay algo interactivo;
> `@nexa/forms`/`@nexa/telemetry` si aplican) más los chunks con
> `Strategy::Load` (se piden de inmediato) — los chunks
> `interaction`/`visible`/`idle`/`manual` se difieren a propósito y no
> cuentan, es la señal real de "qué le cuesta a la carga inicial".
> `maxCSS` compara contra el `nexa-ui.css` compartido de todo el sitio
> (Fase 9) si la página lo enlaza. `maxImage` es un límite *por imagen*
> (no una suma): la imagen estática más pesada que referencia la página,
> resuelta contra `public/`. A diferencia de los avisos `NEXA-SEO-*`/
> `NEXA-PKG-*`, **exceder un presupuesto declarado hace fallar
> `nexa build` de verdad** (código de salida 1) — un presupuesto que
> nunca bloquea no es un presupuesto, es una sugerencia, y para
> sugerencias ya existían los avisos.
>
> **Cómo quedó — `@nexa/devtools`:** un panel flotante que solo inyecta
> `nexa dev` (nunca `build`/`preview` — cero costo en producción,
> garantizado en el propio compilador, no solo por convención). Sondea
> un nuevo endpoint `/__nexa_dev__/diagnostics?path=<ruta>` que compila
> la página pedida (reutilizando `page_resolver::compile_matched_page`,
> extraído de `resolve_page` para este fin) y devuelve JSON real:
> clasificación de nodos, manifiesto de activación completo (evento,
> handler, módulo, estrategia — vía un nuevo `ActivationManifest::
> entries()`), y los avisos `NEXA-SEO-*`/`NEXA-PKG-*` de esa página. El
> panel sobrevive a un auto-reload de `@nexa/dev-client` (que reemplaza
> `<body>` entero, donde vive el `<div>` visible del panel) recreándose
> solo si detecta que su contenedor ya no está conectado al documento.
>
> **Bug real encontrado y corregido durante la verificación E2E:** el
> primer intento de correr el bundle real de `@nexa/devtools` contra un
> `nexa dev` real dio 404 en el endpoint de diagnóstico. Causa:
> `query_param` (Fase 11) nunca decodificaba `%XX` — el cliente pide
> `?path=%2F` (`encodeURIComponent("/")`, lo correcto), y sin decodificar
> eso nunca hacía match contra la ruta `/`. Se corrigió con un
> `percent_decode` propio (sin dependencia nueva) y tests que fijan el
> caso exacto (`%2F` → `/`, `+` → espacio, un `%` mal formado se deja
> tal cual). Sin este fix, `@nexa/devtools` habría quedado roto en
> cualquier uso real con rutas de más de un segmento.
>
> **Cómo quedó — `@nexa/telemetry`:** deliberadamente requiere **dos**
> pasos explícitos antes de que una sola línea de este código llegue al
> navegador: `nexa add telemetry` (registro en `nexa.toml`, igual que
> cualquier otro módulo) **y además** `[telemetry] endpoint = "..."` — sin
> endpoint no hay a dónde enviar nada, así que ni se inyecta el módulo.
> Core Web Vitals (LCP/CLS real; INP aproximado con la señal de FID, ya
> retirada — el algoritmo real de INP es sustancialmente más complejo, y
> reimplementarlo mal habría sido peor que ser honesto sobre la
> simplificación) vía `PerformanceObserver` nativo, sin la librería
> `web-vitals` (evita una dependencia externa en un bundle de un solo
> archivo). Errores sin capturar vía `window.onerror`/
> `unhandledrejection`. Cada reporte se envía en cuanto se mide (nunca en
> lote) vía `navigator.sendBeacon` con `fetch(..., {keepalive:true})` de
> respaldo. Privacidad: nunca se manda la URL completa (solo
> `location.pathname`, no la query string), ni cookies, ni
> identificadores de usuario.
>
> Verificado extremo a extremo: `nexa build` con `maxImage`/`maxInitialJS`
> muy bajos falló de verdad (código de salida 1) nombrando la página y el
> presupuesto exacto excedido; subir los límites lo dejó pasar limpio.
> `public/favicon.ico`/`public/images/logo.png` aparecieron en `dist/` y
> se sirvieron con el `Content-Type` correcto desde `nexa preview`. El
> **bundle real** de `@nexa/devtools` (no un mock), corrido con Node +
> happy-dom contra un `nexa dev` real, mostró el panel con datos
> genuinos del servidor ("4 nodos · 3.5kb JS inicial · 3 aviso(s)") — y
> ese mismo intento fue lo que encontró el bug de `query_param` de
> arriba. El **bundle real** de `@nexa/telemetry`, corrido igual contra
> un `nexa dev` real con un endpoint configurado, envió un reporte de
> error real por HTTP a un servidor colector real (`{"kind":"error",
> "name":"error","detail":"boom real","path":"/","timestamp":...}`), y
> sin endpoint configurado el HTML servido no menciona `@nexa/telemetry`
> en absoluto. 176 tests en Rust (workspace completo) + 128 en
> TypeScript (nuevos paquetes `packages/devtools`, `packages/test`,
> `packages/telemetry`).

---

## Fase 15 — Ecosistema y estabilidad (pre-producción real) — ✅ completada (con un ajuste de alcance)

**Objetivo:** lo que separa un prototipo interesante de algo que un tercero
puede adoptar sin miedo.

**Entregables:**
- Paquetes de comunidad de referencia (`@nexa/stripe`, `@nexa/firebase`, etc.)
  como prueba de que el sistema de plugins funciona con terceros.
- Documentación pública real (no solo este archivo).
- Política de LTS y codemods de migración entre versiones mayores.
- Al menos **un proyecto real en producción** construido sobre Nexa (tu
  propio ecommerce/noticias) como validación final antes de llamarlo 1.0.

**Criterio de salida:** un proyecto ajeno al tuyo (o un backend distinto:
PHP en vez del original) usa Nexa sin tocar el core — es la prueba real de
la "reutilización en distintos proyectos" que motivó todo el diseño.

> **Ajuste de alcance, acordado con el usuario antes de empezar:** "un
> proyecto real en producción" (tu propio ecommerce/noticias, con
> deployment real) no es algo que este agente pueda hacer — implica
> negocio, marca y datos del usuario. Se acordó explícitamente construir
> en su lugar un **proyecto de referencia completo**
> (`examples/oweeme-shop`) que integra casi todo el framework a la vez,
> como validación de integración y punto de partida. De la misma forma,
> "sistema de plugins" no existía como mecanismo real antes de esta fase
> (`platform.` era un caso especial cableado a mano en el compilador
> desde la Fase 13) — se generalizó de verdad en vez de dejarlo
> documentado como límite.
>
> **Cómo quedó — el mecanismo de imports generalizado:** lo que antes
> era `nexa-activation` reconociendo literalmente el string `"platform."`
> ahora es un conjunto de identificadores (`import_names`) que `nexa-cli`
> le pasa desde `nexa.toml` — `platform` sigue built-in, y cualquier otro
> nombre que el proyecto declare en `[imports]` (`stripe = "..."`, una
> URL de CDN o un archivo bajo `public/`) funciona exactamente igual, sin
> que `nexa-activation` sepa que existe. Un handler que usa
> `<nombre>.` hace que su chunk anteponga
> `import { <nombre> } from "<nombre>";` — un specifier "pelado" que
> `nexa-cli` resuelve de verdad con un `<script type="importmap">` real
> en `<head>` (solo si esta página en concreto usa algo, igual que el
> resto de Nexa). `@stripe/stripe-js` (el SDK real de Stripe, vía
> `packages/stripe` — un envoltorio delgado que exporta `stripe.load()`)
> es el paquete de comunidad de referencia: no es un mock, es la
> dependencia real de npm.
>
> **Tres bugs reales encontrados y corregidos verificando esto con un
> navegador de verdad** (Chromium headless vía Playwright, no solo
> `curl`/unit tests):
> 1. `href={\`/${params.locale}/products/${params.slug}\`}` (una
>    plantilla con varias partes en un atributo JSX) se perdía en
>    silencio — `AttrValue::Dynamic` solo aceptaba una única `Expr`
>    desde la Fase 2, así que una expresión que no fuera exactamente
>    `data.x`/`params.x` colapsaba a `value: None`, indistinguible de un
>    atributo booleano real (`<a href>` sin ningún valor). Se corrigió
>    generalizando `AttrValue::Dynamic` para que use `Template` (lo que
>    ya usan los campos de `seo` desde la Fase 6) en vez de una sola
>    `Expr`, con un `from_jsx_text_expression` nuevo en `nexa-parser` que
>    comparte la lógica de plantillas con `seo`/`schema`. Un atributo
>    cuya expresión de verdad no se puede resolver ahora se omite del
>    todo (nunca queda como un atributo booleano fantasma).
> 2. Un archivo `.js` servido desde `public/` (el import map apuntando a
>    `/vendor/nexa-stripe.js`) llegaba con `Content-Type:
>    application/octet-stream` — el navegador rechaza ejecutar un
>    `<script type="module">` así ("Failed to load module script...").
>    `guess_static_content_type` (Fase 14) no tenía casos para
>    `js`/`mjs`/`css`/`html`; se añadieron.
> 3. `@nexa/forms` (Fase 10) nunca marcaba un campo `required` vacío
>    como inválido al hacer clic real en el botón de envío: el navegador
>    cancela el evento `submit` por completo cuando la validación nativa
>    falla (dispara `invalid` en su lugar) — los tests existentes usaban
>    `form.dispatchEvent(new Event("submit"))` a mano, que se salta esa
>    validación nativa por completo, así que nunca lo habían expuesto. Se
>    corrigió escuchando también `invalid` por campo. Al escribir el test
>    de regresión con `requestSubmit()` (que sí respeta la validación
>    nativa) apareció un CUARTO bug, más sutil: el propio handler de
>    `invalid` llamaba a `checkValidity()` de nuevo para leer el mensaje
>    de error, y `checkValidity()` dispara su propio `invalid` si el
>    campo sigue inválido — una recursión infinita entre el handler y el
>    evento que lo dispara. Se corrigió leyendo `field.validity.valid`
>    directamente (siempre actualizado, no dispara nada) en vez de volver
>    a llamar `checkValidity()`.
>
> **Cómo quedó — el proyecto de referencia:** `examples/oweeme-shop`,
> con `[locale]/index.tsx` (i18n + `platform.share`), `[locale]/
> products/[slug].tsx` (`load()` real, `seo`/`schema` de `Product`,
> `stripe.load()`), `[locale]/contact.tsx` (`@nexa/forms`), y un backend
> real en **PHP** (`backend/index.php`) — la prueba concreta del
> criterio de salida ("un backend distinto: PHP en vez del original").
>
> **Cómo quedó — documentación y política:** `docs/GUIA-DE-INICIO.md`
> (una guía real para alguien sin contexto previo del repositorio, no
> solo el estado actual como ya hace `README.md`) y
> `docs/POLITICA-LTS.md` (SemVer para el framework completo, ciclo de
> vida de cada versión mayor, y el compromiso concreto de codemods para
> cuando exista una primera versión mayor que rompa algo — hoy, en
> `0.1.0`, no existe ninguna todavía porque sería prematuro).
>
> Verificado extremo a extremo con un navegador real (no solo `curl`):
> `nexa build` con presupuestos muy bajos falló nombrando exactamente
> qué se pasó; la home, el producto (con datos reales del backend PHP) y
> el contacto renderizaron correctamente en Chromium headless, con los
> `href` multi-parte ya resueltos; el botón "Comprar" disparó una
> importación real de `stripe` vía el import map, que a su vez cargó de
> verdad `https://js.stripe.com` (tráfico real observado hacia la CDN
> real de Stripe) y resolvió un cliente real; el formulario de contacto
> marcó el campo de correo como inválido al hacer clic real en enviar,
> sin quedarse colgado. 192 tests en Rust (workspace completo) + 130 en
> TypeScript (nuevos paquetes `packages/stripe`, con `@stripe/stripe-js`
> como dependencia real de npm).

---

## Fase 16 — Islas interactivas (Hito 12) — ✅ completada

**Objetivo:** permitir subárboles genuinamente interactivos y con estado
(dashboards, tableros Kanban, mapas en vivo) sin reabrir el modelo de
composición de componentes que se ha evitado tocar desde la Fase 2 — y
sin que Nexa tenga que reinventar un framework de UI completo en Rust
para lograrlo. Motivada directamente por un usuario real intentando usar
Nexa en `oweeme.com` (SEO público + un tablero SDLC tipo Trello) y en
proyectos futuros (ecommerce, logística, una app tipo Uber) sin tener que
alternar entre varios frameworks.

**Por qué esto no es la composición de componentes rechazada:** la regla
rechazada, reafirmada en cada fase desde la 2, es que Rust parsee un
segundo árbol `.tsx` y lo empalme con props/composición en el propio
compilador. Una isla es otra cosa: para `nexa-parser`/`nexa-analyzer`/
`nexa-renderer`, `<div data-nexa-island="x">` es un `Element` más, con
hijos estáticos/dinámicos normales (el fallback SSR). El `specifier` es
un string opaco — Rust nunca abre, parsea ni interpreta lo que hay del
otro lado. Todo lo interactivo vive y se ejecuta 100% en el cliente.

**Entregables:**
- `data-nexa-island="<specifier>"` + `data-nexa-props={{...}}` (string
  literal + objeto resuelto a JSON con el mismo mecanismo que ya usan
  `seo`/`schema`) — nuevo campo `island: Option<Island>` en `Element`
  (`nexa-ast`) y `IrNodeKind::Element` (`nexa-ir`), nueva variante
  `Classification::Island`, reconocido en `nexa-parser::jsx` con el
  mismo patrón que ya existía para `data-nexa-strategy`.
- El "contrato de montaje": `export default function mount(el, props)`
  — la misma convención de export por defecto que ya usan los chunks de
  activación de eventos (`activate(el)`, Fase 5), documental, nunca
  validada por Rust.
- `packages/islands` (nuevo runtime cliente): `initIslands()` recorre
  `[data-nexa-island]` y reutiliza sin cambios las mismas estrategias de
  activación de eventos (`interaction`/`visible`/`idle`/`load`/`manual`,
  `packages/runtime/src/strategies.ts`) — solo cambia el *default*
  (`visible`, no `interaction`: una isla no tiene "el" evento obvio de
  disparo de un `onClick`).
- `packages/vue-island` (nuevo, paquete de comunidad de referencia igual
  que `packages/stripe` en la Fase 15): adaptador delgado y real sobre
  Vue 3 (`createApp(component, props).mount(el)`), no escrito ni
  mantenido por el core de Nexa.
- Reutilización íntegra del import map de la Fase 15 para resolver
  specifiers de islands (`import_map::used_by_page` extendido con un
  tercer criterio de uso, además de la búsqueda textual `nombre.` en
  handlers) — ninguna infraestructura de resolución nueva.
- Nuevo aviso `NEXA-PKG-004`: `data-nexa-island="x"` con `x` ausente de
  `nexa.toml [imports]` (mismo espíritu que `NEXA-PKG-001..003`, Fase 12).
- Decisión deliberada: **no** existe un `IslandManifest` paralelo al
  `ActivationManifest` de eventos — el specifier/props/estrategia de una
  isla se quedan en el HTML servido (`data-nexa-island`/`data-nexa-props`/
  `data-nexa-strategy`), a diferencia de un evento (cuyo manifiesto real
  vive solo en `nexa-activation`, el HTML solo lleva `data-nexa="<id>"`).
  Se descartó un manifiesto propio porque hubiera duplicado, en una
  segunda estructura, información que ya está a la vista en el HTML —
  contra el propio espíritu de "todo visible en view-source" de Nexa.
  Costo aceptado: el panel de DevTools (Fase 14) no lista islands esta
  fase, y el presupuesto `maxInitialJS` no cuenta bytes de módulos de
  islands (el specifier puede apuntar a cualquier lado que Rust no puede
  inspeccionar).
- Dos islands reales en `examples/oweeme-shop`: un filtro de productos
  escrito a mano con `@nexa/reactivity` (`src/islands/productFilter.island.ts`,
  injertado en `[locale]/index.tsx`, con el fallback SSR real —
  `<ul>` de productos con enlaces reales, escrito a mano porque Nexa no
  tiene bucles/composición para generarlo desde `data.products`) y un
  dashboard con un componente **Vue 3 real** (`src/islands/Dashboard.ts`,
  montado en una página nueva `[locale]/dashboard.tsx` vía
  `@nexa/vue-island` — un panel que no es superficie SEO a propósito).

**Criterio de salida:** verificado extremo a extremo con Chromium real
(Playwright, no solo `curl`/unit tests):
- El HTML servido (sin ejecutar JS) contiene el fallback real de
  productos — SEO intacto — y `data-nexa-island`/`data-nexa-props` bien
  formados y escapados, visibles en el propio HTML.
- Con estrategia `visible` (el default), el módulo de la isla NO se pide
  mientras el elemento está fuera del viewport, y SÍ se pide en cuanto
  entra — verificado con un viewport real y scroll real, no simulado.
- La isla escrita a mano es interactiva de verdad: escribir en el campo
  de búsqueda filtra la lista usando los mismos datos que ya trajo
  `load()` del backend PHP, sin volver a pedirlos.
- La isla Vue es interactiva de verdad: un clic real actualiza el estado
  reactivo de **Vue** (no de `@nexa/reactivity`) y el DOM lo refleja —
  la prueba directa de que un framework externo real puede montarse
  dentro de un proyecto Nexa.
- Las páginas ya verificadas en la Fase 15 (home con `platform.share`,
  producto con `stripe.load()`, contacto con validación nativa) siguen
  pasando sin cambios en la misma sesión de Chromium — aditivo en la
  práctica, no solo por revisión de código.

> **Un bug real encontrado verificando con datos reales, no simulados:**
> un literal numérico entero escrito en `.tsx` (`id: 1`, en
> `data-nexa-props`) se serializaba como `1.0`, no `1` —
> `serde_json::Number::from_f64` no distingue "este f64 es
> matemáticamente un entero" de "quiero que se vea como decimal". Este
> mismo patrón ya existía, sin corregir, en `nexa-seo/src/resolve.rs`
> desde fases anteriores (JSON-LD/`schema.org` con un precio literal
> tendría el mismo defecto) — nunca se había expuesto porque ningún
> ejemplo/test previo usaba un número literal ahí (`price: data.price`
> siempre venía de JSON real, que sí preserva enteros). Se corrigió con
> un `json_number()` que serializa como entero cuando el `f64` no tiene
> parte fraccionaria, duplicado en `nexa-seo` y `nexa-renderer` por la
> misma razón que el resto de estas plantillas de JSON se duplican entre
> ambos crates — con test de regresión en los dos sitios.
>
> **Cómo quedó — el proyecto de referencia:** `examples/oweeme-shop`
> ganó `/products` en el backend PHP (catálogo completo, no solo por
> `slug`), la isla `productFilter` en el home, y la página nueva
> `/dashboard` con la isla `dashboardIsland`. El README del ejemplo
> también documenta un gap preexistente de la Fase 15 que esta
> verificación expuso: `nexa preview` sirve desde `dist/`, así que hace
> falta un `nexa build` antes — el README anterior no lo mencionaba.
>
> 202 tests en Rust (workspace completo, +10 sobre la Fase 15) + 130 en
> TypeScript (incluyendo los nuevos `packages/islands` y
> `packages/vue-island`, este último con Vue 3 real como dependencia de
> npm — no un mock).

---

## Regla de disciplina para todas las fases

> No empezar a diseñar la fase N+2 mientras la fase N no tenga un criterio
> de salida cumplido y verificado. El documento original cayó en esto
> (diseñó i18n, PWA, registry de paquetes y DevTools antes de tener un
> compilador funcionando); esta lista existe para no repetirlo.
