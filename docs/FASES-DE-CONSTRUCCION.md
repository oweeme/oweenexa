# Nexa — Fases de construcción

> Derivado del intercambio de diseño original (una exploración conceptual,
> 100% diseño, 0% código a fecha de 2026-08-26 — no incluida en este
> repositorio por ser una transcripción informal, ya superada en la
> práctica por este mismo documento). Este documento reorganiza esa
> exploración en fases ejecutables, cada una con un criterio de salida
> verificable ("algo que corre"), siguiendo la propia regla que esa
> exploración original se impone al final: no diseñar más antes de tener
> el vertical slice funcionando.

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

## Fase 17 — Pre-render de rutas dinámicas (`paths`) (Hito 13) — ✅ completada

**Objetivo:** cerrar la brecha más señalada desde que Nexa empezó a
usarse en proyectos reales — una ruta dinámica (`[slug].tsx`) solo se
servía al vuelo con `nexa preview`/`nexa dev`, nunca como HTML estático
real generado por `nexa build`. Para un catálogo (ecommerce, blog,
directorio) cuyos slugs se conocen de antemano, eso significa que
`dist/` nunca es "de verdad" 100% estático, y hace falta un proceso Rust
corriendo en producción aunque el contenido no cambie a cada request.

**Entregables:**
- `export const paths = { url: "..." }`: misma forma sintáctica exacta
  que `load` (un objeto literal, nunca una función) — el backend
  responde con un array JSON de sets de parámetros
  (`[{"slug":"iphone-17"}, ...]`) en vez de un único objeto de datos.
  Reutiliza `nexa_loader::load` sin ningún cambio (la función ya
  devolvía JSON genérico; lo único nuevo es qué hace `nexa build` con
  el resultado).
- `nexa-ast::Component.paths: Option<Loader>` (mismo tipo `Loader` que
  `load`) y `nexa-parser::loader::find_paths` (mismo parseo que `load`,
  otro nombre de export) — cero AST/IR nuevo, cero riesgo de tocar la
  regla de "sin composición" (Fase 2): `paths` no cambia nada de cómo
  se interpreta el `.tsx`, solo qué hace `nexa build` antes de compilar.
- `nexa build`: para cada ruta dinámica que declara `paths`, pide la
  lista, valida que cada entrada tenga un campo string por cada
  segmento dinámico de la ruta (`[locale]/products/[slug].tsx` necesita
  `locale` y `slug` en cada entrada), y genera un `index.html` real por
  combinación — mismo pipeline de compilación que cualquier página
  estática, solo que llamado N veces con N sets de parámetros distintos.
  Una ruta dinámica sin `paths` sigue exactamente igual que antes
  (servida al vuelo).

**Criterio de salida:** un proyecto con una ruta `[slug].tsx` que
declara `paths` contra un backend real produce, con `nexa build`,
HTML real por cada slug — y ese HTML sigue sirviendo contenido correcto
con el backend completamente apagado.

> **Verificado end-to-end con un backend real (no mocks) apagado a
> propósito:** un proyecto con `src/pages/products/[slug].tsx`
> declarando `paths = { url: "/products" }` + `load = { url:
> "/products/:slug" }` contra un backend HTTP real (dos productos)
> generó `dist/products/iphone-17/index.html` y
> `dist/products/pixel-10/index.html` con datos reales, título/canonical
> resueltos, y ambas URLs en `dist/sitemap.xml`. Se sirvió ese `dist/`
> con un servidor estático puro (`python -m http.server`, sin ningún
> proceso de Nexa/Rust) y **con el backend ya apagado** — la página
> siguió respondiendo 200 con el contenido real. Eso es exactamente lo
> que el criterio de salida pedía: HTML verdaderamente estático, no una
> promesa.
>
> 206 tests en Rust (workspace completo, +2 sobre la Fase 16: parseo de
> `paths` presente/ausente en `nexa-parser`). El resto de la lógica
> (`nexa build`, orquestación de E/S) se verificó con el build real de
> arriba, no con tests unitarios — mismo criterio que el resto de
> `nexa-cli`, que se apoya en verificación end-to-end para sus comandos.

---

## Fase 18 — PWA declarativo (`nexa add pwa`) (Hito 14) — ✅ completada

**Objetivo:** que un sitio Nexa pueda instalarse y funcionar offline sin
que el desarrollador escriba un service worker a mano — `[pwa]` en
`nexa.toml` es la declaración, `nexa build` genera los dos artefactos
reales que un navegador necesita.

**Entregables:**
- `nexa add pwa`: registra la dependencia y agrega un `[pwa]` real (no
  texto suelto) a `nexa.toml`, pre-llenado con el nombre del proyecto —
  mismo patrón de "andamiaje" que `tauri`/`capacitor` (Fase 13), pero sin
  generar ningún directorio aparte.
- `nexa_cli::manifest::PwaSection` (`name`, `shortName`, `themeColor`,
  `backgroundColor`, `display`, `icon`, `[pwa.cache]`) — mismo patrón
  serde que `[performance]`/`[telemetry]`, ausente por defecto.
- `nexa build` genera `dist/manifest.webmanifest` (JSON real, con los
  íconos declarados dos veces — 192x192 y 512x512 — apuntando al mismo
  PNG) y `dist/sw.js` (un service worker real, no una plantilla vacía:
  instala, reclama las páginas de inmediato con `skipWaiting`/
  `clients.claim`, y por cada `fetch` GET elige entre tres estrategias
  reales — `cache-first`/`network-first`/`stale-while-revalidate` — según
  el prefijo de ruta más específico que matchea en `[pwa.cache]`, con
  `network-first` como default seguro si no hay ninguna regla).
- Cada página compilada lleva automáticamente `<link rel="manifest">` +
  `<meta name="theme-color">` (mismo mecanismo de fragmentos de `<head>`
  que ya usan SEO/hreflang) y el registro del service worker en el
  `<script>` de arranque (`bootstrap::inject`, mismo costo-cero que
  `has_forms`/`has_islands`: nada de esto aparece si el proyecto no
  declaró `[pwa]`).

**Criterio de salida:** un proyecto con `[pwa]` completo, compilado y
servido, se comporta como una PWA real — instala un service worker que
cachea de verdad, y sigue funcionando con la red completamente cortada.

> **Verificado en Chromium real (Playwright), no solo generando los
> archivos:** un proyecto con `[pwa]` (ícono real, `themeColor`, y
> `[pwa.cache]` con reglas `cache-first`/`network-first`) servido con
> `nexa preview` — el service worker llegó a estado `activated`, una
> segunda visita cacheó de verdad la página y `/assets/nexa-router.js`
> (confirmado leyendo `caches.open("nexa-pwa-v1")` directamente en el
> navegador), y con `context.setOffline(true)` (la red completamente
> cortada, no solo un mock) una nueva navegación a la misma URL siguió
> mostrando el `<h1>` real de la página. Esa es la prueba que de verdad
> importa para "esto es una PWA funcional" — no que los archivos
> `manifest.webmanifest`/`sw.js` existan con la forma correcta, sino que
> el navegador los usa de verdad para seguir funcionando sin red.
>
> 218 tests en Rust (workspace completo, +12 sobre la Fase 17): 8 en el
> nuevo `pwa.rs` (generación del manifest, validación de estrategias de
> cache, orden de prefijos más específico primero), 2 en `manifest.rs`
> (parseo de `[pwa]`/`[pwa.cache]`), 2 en `bootstrap.rs` (el script de
> registro solo aparece con `has_pwa`).

---

## Fase 19 — Pipeline de imágenes (AVIF + `<picture>` real) (Hito 15) — ✅ completada (con un ajuste de alcance)

**Objetivo:** que `<img src="/foto.jpg">` no viaje al navegador tal cual
si `nexa build` puede mandar bytes más chicos sin que el desarrollador
haga nada — mismo espíritu que el tree-shaking de `@nexa/ui` (Fase 9):
el compilador hace el trabajo para que el navegador descargue menos.

**Entregables:**
- `nexa build` decodifica cada `<img src="...">` estático que
  `image_scan` (Fase 14) ya sabía encontrar, genera variantes en varios
  anchos (480/768/1280 + el ancho original, los que sean menores que el
  original) y reescribe el HTML final a un `<picture>` real —
  post-procesando el HTML ya renderizado con `lol_html` (el mismo motor
  de reescritura de Cloudflare), sin tocar `nexa-renderer` para nada: el
  renderer sigue emitiendo el `<img>` tal cual el desarrollador lo
  escribió.
- `width`/`height`/`loading="lazy"` se completan automáticamente si no
  estaban (ayuda real a Core Web Vitals — evita layout shift).
- `VariantCache`: la misma imagen usada en varias páginas se decodifica
  y codifica una sola vez por `nexa build`.
- Un error generando una imagen en particular (archivo corrupto, etc.)
  se reporta como aviso y esa imagen queda sin optimizar — nunca rompe
  el build entero.

> **Ajuste de alcance real, encontrado construyendo esto, no antes:**
> el plan original incluía WebP además de AVIF. Verificando con una foto
> real de 1920x1080 se encontró que el encoder de WebP de la crate
> `image` **solo soporta el modo lossless** — para una foto, eso produjo
> un archivo de 764 KB, más grande que el JPEG original de 158 KB. Se
> sacó WebP del todo: solo queda AVIF (que sí soporta lossy de verdad) +
> el propio formato original en los anchos más chicos, para el `<img>`
> de respaldo.
>
> **Segundo hallazgo real, sobre la velocidad de AVIF:** a la
> configuración por defecto de `ravif` (velocidad 4, pensada para
> comprimir una sola foto), una sola imagen de 1920x1080 (cuatro anchos)
> tardó más de un minuto **en un binario release** — impracticable para
> un `nexa build` con varias fotos. Se subió a velocidad máxima (10):
> la misma foto tardó 3.8 segundos en total, y los AVIF resultantes
> siguieron siendo 68-90% más chicos que el JPEG original según el
> ancho. También se encontró que los tests de este módulo, corriendo en
> modo debug (`cargo test` normal), literalmente no terminaban con
> imágenes de prueba de tamaño realista — se redujeron a imágenes de 1
> a 4px de alto (el ancho, que es lo que la lógica de breakpoints
> necesita probar, se mantuvo realista) para que la suite completa
> siguiera corriendo en segundos.

**Criterio de salida:** una foto real, en una página real, sirve un
AVIF real y más chico en un navegador real — no que los archivos
`.avif` existan con el tamaño correcto, sino que el navegador de verdad
los prefiere y los usa.

> **Verificado en Chromium real (Playwright):** una página con
> `<img src="/img/hero.jpg">` (foto real de 1920x1080) compilada con el
> binario **release** de `nexa` — `nexa build` generó
> `dist/img/hero-{480,768,1280,1920}.avif` (51 KB el de ancho completo,
> contra 158 KB del JPEG original) y reescribió el HTML a `<picture>`.
> Sirviendo ese `dist/` y cargando la página en Chromium con un
> viewport de 1000px: `img.currentSrc` fue el AVIF de 1280px, y la
> *única* request de imagen que el navegador hizo en toda la carga fue
> ese AVIF — nunca tocó el JPEG de respaldo.
>
> 226 tests en Rust (workspace completo, +8 sobre la Fase 18) — toda la
> lógica pura (`is_supported`, filtrado de breakpoints, reescritura de
> HTML, cache de variantes, manejo de errores) vive en
> `image_pipeline.rs` con tests reales (deciden y verifican contra
> archivos reales en disco, no mocks); la integración con `nexa build`
> se verificó con el build real de arriba.

---

## Fase 20 — Adaptador nginx + cabeceras de seguridad (Hito 16) — ✅ completada

**Objetivo:** que salir de `nexa build` a producción no dependa de que
cada quien reinvente su propio `nginx.conf` a mano, y que las cabeceras
de seguridad más básicas estén puestas por defecto — en desarrollo
(`nexa preview`/`nexa dev`) y en el adaptador generado, de forma
consistente.

**Entregables:**
- `nexa add nginx`: genera `deploy/nginx.conf` real (mismo patrón que
  `tauri`/`capacitor`, Fase 13) — gzip (sin `text/html`, que nginx ya
  comprime siempre), cabeceras de seguridad, y un bloque comentado de
  `proxy_pass` para las rutas dinámicas que no declaran `paths` (Fase
  17) y por lo tanto siguen necesitando un `nexa preview` vivo detrás.
- `X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`,
  `Referrer-Policy: strict-origin-when-cross-origin` — las mismas tres
  cabeceras, aplicadas de verdad en cada respuesta de `nexa preview`/
  `nexa dev` (`serve::respond`/`respond_bytes`) y documentadas en el
  `nginx.conf` generado, para que ambos entornos se comporten igual.
- Deliberadamente **sin** `Content-Security-Policy` ni
  `Permissions-Policy`: una CSP por defecto rompería el import map de
  terceros (Fase 15 — Stripe, un CDN de Vue, etc., cuyos orígenes no se
  pueden adivinar de antemano), y `Permissions-Policy: camera=()`
  rompería `platform.capturePhoto()` en la web (Fase 13). Ninguna de
  las dos se puede poner "segura por defecto" sin saber qué necesita
  cada proyecto — se dejan fuera en vez de adivinar mal.

**Criterio de salida:** el `nginx.conf` generado es válido de verdad
para un nginx real (`nginx -t`), no solo "se ve razonable" — y sirve
contenido real con las cabeceras correctas.

> **Dos hallazgos reales, validando con un nginx de verdad (contenedor
> `docker.io/library/nginx:alpine`), no solo revisando el texto a
> mano:**
> 1. La primera versión listaba `text/html` en `gzip_types` — nginx ya
>    comprime `text/html` siempre que `gzip on` está activo, así que
>    `nginx -t` marcó "duplicate MIME type" como warning real de
>    sintaxis. Se sacó de la lista.
> 2. La receta típica de "assets con cache agresivo e `immutable`" no
>    aplica todavía a Nexa: los nombres de archivo (`nexa-runtime.js`,
>    `Home-15.js`) no llevan hash de contenido, así que un `max-age` de
>    un año serviría JS viejo para siempre después de un redeploy. El
>    `nginx.conf` generado usa un `max-age` corto a propósito, con un
>    comentario explicando por qué — no la plantilla genérica de
>    internet copiada sin pensar.
>
> Verificado de punta a punta: `nginx -t` contra el archivo generado
> por `nexa add nginx` (cero warnings tras el fix), y un contenedor real
> de nginx sirviendo `dist/` con ese `nginx.conf` — `curl -I` mostró las
> tres cabeceras de seguridad en la respuesta real, idénticas a las que
> ya manda `nexa preview`.
>
> 234 tests en Rust (workspace completo, +8 sobre la Fase 19): 7 en
> `nginx_scaffold.rs`, 2 en `serve.rs` (las cabeceras están en toda
> respuesta, sin excepción) — más 1 ajuste de test.

---

## Fase 21 — `computed()`/`watch()` en `@nexa/reactivity` (Hito 17) — ✅ completada

**Objetivo:** dos primitivas chicas que le faltaban a `@nexa/reactivity`
para escribir una isla a mano sin reinventar valores derivados o
reaccionar a un cambio — sin acercarse nunca a un Virtual DOM ni a un
modelo de componentes (`store`/`resource`/`context`, sugeridos junto
con estas dos, se dejaron fuera a propósito: para cualquier cosa que
los necesite de verdad, la respuesta ya construida es una isla con un
framework real adentro — Fase 16 —, no que Nexa reinvente Pinia/Redux).

**Entregables:**
- `computed(() => a.value + b.value)`: una `ReadonlySignal` derivada,
  implementada como una `Signal` interna actualizada dentro de un
  `effect()` — reutiliza el tracking y el batching que ya existían
  desde la Fase 4, cero mecanismo nuevo. Encadenar computeds funciona
  (`computed(() => otroComputed.value * 2)`), con un costo real
  documentado: cada salto de la cadena es una vuelta más de microtask
  para terminar de propagarse.
- `watch(() => fuente.value, (nuevo, viejo) => ...)`: igual que
  `effect`, pero **no** corre el callback al crearse — solo cuando la
  fuente cambia de verdad después. Reutiliza `effect()` por dentro.

**Criterio de salida:** ambas primitivas se comportan como se
documentan, incluida la propagación en cadena.

> Encontrado escribiendo los tests, no antes: dos de ellos fallaban al
> principio (`computed` encadenado, y `watch` sobre un `computed`)
> porque solo esperaban una vuelta de microtask — con dos saltos de
> efecto hacen falta dos. No es un bug de `computed`/`watch`: es el
> mismo comportamiento que ya tenía encadenar dos `effect()` a mano
> desde la Fase 4, solo que nunca antes había un test que encadenara
> dos niveles. Quedó documentado en el propio código, no solo en el test.
>
> 18 tests en TypeScript en `@nexa/reactivity` (+10 sobre lo que ya
> existía).

---

## Fase 22 — Reactivación real tras navegación SPA (Hito 18) — ✅ completada

**Objetivo:** un bug de correctitud real, encontrado con un navegador
real mientras se documentaba el framework para la Fase de referencia:
navegar por un link interno (`initRouter`, Fase 6) a otra página de
Nexa dejaba esa página de destino con el HTML correcto pero **cero JS
activado** — ni botones, ni formularios, ni islas. Es, con diferencia,
el bug más grave encontrado en toda la construcción de Nexa: afecta a
cualquier sitio con más de una página interactiva navegada por SPA.

**Por qué pasaba:** `initRouter` reemplaza `<body>` con
`element.innerHTML = ...` — un `<script>` insertado así **nunca se
ejecuta**, es comportamiento estándar de cualquier navegador, no algo
que `packages/router` pudiera evitar por su cuenta. El manifiesto de
activación de la página de destino, embebido como argumento de una
llamada dentro de ese `<script>`, quedaba inerte para siempre.

**Entregables:**
- `initRouter` acepta `onNavigate?: (root) => void`, llamado justo
  después de reemplazar `<body>`.
- El manifiesto de activación se embebe también como datos inertes
  (`<script type="application/json" data-nexa-manifest>`), que sí
  sobrevive el reemplazo — a diferencia de una llamada dentro de un
  `<script type="module">`. La activación inicial de la propia página
  ahora lee de ahí también (una sola fuente de verdad, en vez de
  embeber el JSON dos veces).
- **Toda página, tenga o no contenido interactivo propio**, lleva una
  función `reactivate(root)` — cualquier página puede navegarse *hacia*
  una que sí lo tenga. Usa `import()` dinámico a propósito: el costo de
  `@nexa/runtime`/`forms`/`islands` solo se paga si la página de
  destino de verdad los necesita, nunca antes — la disciplina de costo
  cero se mantiene, ahora evaluada por navegación en vez de fija en
  tiempo de compilación.
- Antes de reactivar, se limpian los disposers de la página anterior
  (`disposeActivation`/`disposeForms`/`disposeIslands`) — esto también
  resuelve, como efecto colateral correcto, la fuga que tenían las
  islas al navegar (documentada como límite conocido hasta esta fase).

**Criterio de salida:** un botón/formulario/isla en una página a la que
se llega navegando por un link interno funciona igual que si se hubiera
cargado con `nexa preview`/`nexa dev` directo — y no se activa dos
veces al volver a visitarla.

> **Encontrado y arreglado con un navegador real, no en el código:**
> un proyecto de dos páginas (`/` estática, `/other` con un botón) —
> `page.click("a[href='/other']")` seguido de un clic real en el
> botón — confirmó el bug (el texto nunca cambiaba) antes del fix, y
> lo confirmó arreglado después. En el camino, dos hallazgos más:
> 1. La primera versión del fix embebía el manifiesto una segunda vez,
>    sin escapar, directo en el `<script type="module">` — el propio
>    test de regresión (`</script><script>alert(1)` como nombre de
>    handler) lo encontró antes de llegar a ningún navegador.
> 2. Verificar el fix requirió recordar reconstruir
>    `crates/nexa-cli/assets/nexa-router.js` (`npm run
>    build:cli-assets`) — el binario de Rust embebe ese archivo ya
>    compilado; editar el TypeScript fuente no alcanza solo.
>
> Verificado además que un formulario en una página a la que se llega
> por navegación SPA valida de verdad (`@nexa/forms` reactivado), y que
> visitar la misma página varias veces no duplica el manejador de
> eventos (un solo clic sigue disparando el handler una sola vez).
>
> 240 tests en Rust (workspace completo, +3 sobre la Fase 21: la
> función `reactivate` presente en toda página, el manifiesto embebido
> como JSON inerte, y el escape de `</script>` dentro de un valor del
> manifiesto) + 2 nuevos en `@nexa/router` (`onNavigate` se llama tanto
> sirviendo por red como desde caché de prefetch).

---

## Fase 23 — Content hashing (cache-busting real) (Hito 19) — ✅ completada

**Objetivo:** cerrar la limitación documentada desde la Fase 20 — los
archivos que genera `nexa build` (`nexa-runtime.js`, chunks de
activación como `Home-15.js`, `nexa-ui.css`) no llevaban ningún hash de
su contenido en el nombre, así que un redeploy podía cambiar el
contenido de un archivo sin cambiar su nombre. Eso hacía insegura
cualquier recomendación de caché agresivo (`Cache-Control: immutable`,
`max-age` de un año) en `nexa add nginx`: un visitante que ya tenía el
archivo viejo en caché nunca vería el nuevo.

**Entregables:**
- Hash corto (FNV-1a, 8 hex) de contenido — mismo algoritmo que ya usa
  `nexa.lock` para su propio `content_hash`, duplicado como una función
  de una decena de líneas en cada crate que lo necesita (`nexa-cli`,
  `nexa-activation`) en vez de crear una dependencia cruzada nueva.
- Los siete assets de framework embebidos en el binario
  (`nexa-runtime`, `nexa-router`, `nexa-forms`, `nexa-platform`,
  `nexa-telemetry`, `nexa-islands`, `nexa-ui.js`) pasan de nombre fijo
  (`crate::assets::NEXA_RUNTIME_FILENAME` como constante) a nombre
  calculado (`crate::assets::nexa_runtime_filename() -> String`) a
  partir del hash de su propio contenido — determinista para una
  compilación dada de `nexa-cli`.
- Cada chunk de nodo interactivo (`nexa-activation::build::collect`) ya
  no nombra su archivo con solo `{componente}-{node_id}.js`: primero
  calcula el contenido real del chunk (que no depende del nombre —
  `chunk::content_for`, separado de `chunk::generate` para esto), lo
  hashea, y solo entonces arma `{componente}-{node_id}.{hash}.js`. El
  manifiesto de activación (`entry.module`) y el nombre en disco
  (`chunk.filename`) se construyen juntos a partir del mismo hash, así
  que nunca pueden desincronizarse.
- `nexa-ui.css` (la unión de clases `nx-*` de *todo el sitio*, que solo
  se conoce después de compilar todas las páginas) usa un hash
  "pendiente" (`pipeline::UI_CSS_HASH_PLACEHOLDER`) en el `<link>` de
  cada página mientras se compila; `nexa build` reemplaza ese
  placeholder por el hash real recién al final, antes de escribir cada
  `index.html` a disco. `nexa dev`/`nexa preview` (que compilan una
  sola página a la vez, y conocen su CSS de inmediato) nunca llegan a
  dejar el placeholder en el HTML que sirven.
- `nexa add nginx`: el `location /assets/` único se separa en dos —
  uno con una regex (`^/assets/.+\.[0-9a-f]{8}\.(js|css)$`) que
  matchea específicamente los archivos con hash y les da
  `Cache-Control: public, max-age=31536000, immutable`; el resto de
  `/assets/` (ej. algo copiado a mano desde `public/assets/`, sin
  hash) sigue con el `max-age` corto de antes.

**Criterio de salida:** dos handlers con código distinto nunca
comparten nombre de archivo; el HTML de cada página referencia
exactamente el nombre real (con hash) de cada asset que usa, sin
placeholders filtrados; `nginx.conf` generado recomienda caché
`immutable` solo para lo que de verdad lo soporta.

> **Verificado con un proyecto real, no solo con `cargo test`:** una
> página con un botón interactivo y clases `nx-card`/`nx-btn` de
> `@nexa/ui`, compilada con `nexa build` real. `dist/assets/` quedó con
> `nexa-ui.cfd9aa83.css`, `Home-6.80ad9e12.js`, y los siete assets de
> framework, cada uno con su propio hash de 8 hex — y el `<link>`/
> `<script>` de `dist/index.html` referencian exactamente esos mismos
> nombres, sin ningún placeholder sin resolver. `nexa preview` sirvió
> los cuatro (HTML, CSS, chunk, runtime) con 200 real. Un navegador real
> (Chromium vía Playwright) confirmó `border-radius` real aplicado desde
> el CSS con hash, el clic del botón disparando el handler real desde el
> chunk con hash, y cero respuestas con error. Por separado, se repitió
> el mismo build borrando `dist/` y sirviendo con `nexa dev` (que
> compila una sola página a la vez, sin la unión de todo el sitio): el
> `<link>` de `nexa-ui.css` ya traía el hash real, nunca el placeholder
> `pending`.
>
> 244 tests en Rust (workspace completo, +4 sobre la Fase 22: el hash
> determinista/distinto de `nexa-activation::content_hash`, y que dos
> handlers con contenido distinto producen chunks con nombre distinto).

---

## Fase 24 — `nexa lint` / `nexa test` (Hito 20) — ✅ completada

**Objetivo:** cerrar el otro pendiente documentado desde hacía varias
fases — `@nexa/test` (mountChunk, getEntry, expectEntry) existía como
librería para importar desde los tests de un proyecto, pero no había
ningún comando `nexa` que lo orquestara; y no existía ningún chequeo de
CI-friendly que hiciera fallar el proceso por los avisos que el SEO
Analyzer/`pkg_warnings` ya calculan (`nexa build` los imprime pero nunca
falla por ellos, a propósito).

**Entregables:**
- `nexa lint`: recorre las mismas rutas que `nexa build` (reutiliza
  `enumerate_paths`/`resolved_pattern` de `commands::build`, ahora
  `pub(crate)`), compila cada página con `pipeline::compile_page` sin
  escribir nada a `dist/`, imprime los avisos SEO/paquetes de cada una,
  y termina con código de salida distinto de cero si encontró alguno.
- `nexa test`: corre `nexa build` primero (asegura un `dist/` fresco,
  con los nombres de archivo con hash de la Fase 23), y después
  `npm test` (leyendo `package.json`) — con mensajes de error explícitos
  si no hay `package.json`, o si no declara un script `"test"`. No
  reemplaza a `vitest`/`node --test`/etc., los orquesta.
- El ejemplo de `@nexa/test` en el README/`docs/REFERENCIA.md` se
  corrigió: ya no importa un chunk por un nombre de archivo hardcodeado
  (`ProductPage-3.js`, que dejó de ser válido con la Fase 23) — ahora
  resuelve la ruta real leyendo `entry.module` con `getEntry(manifest,
  id)`, que es justo para lo que existía esa función.

**Criterio de salida:** `nexa lint` sobre un proyecto con `seo` completo
termina en 0 sin escribir `dist/`; sobre uno con `seo` incompleto
termina distinto de cero y lista los avisos reales. `nexa test` sin
`package.json` falla con un mensaje que explica qué falta; con un script
`"test"` real, compila y después corre ese script, propagando su código
de salida.

> **Verificado con un proyecto real, no solo con `cargo test`:** un
> proyecto scaffolded por `nexa create` con `seo` incompleto — `nexa
> lint` listó los tres avisos reales (`NEXA-SEO-001/002/003`) y terminó
> con código 1, sin crear `dist/`. Completando `seo`, `nexa lint` terminó
> en 0 con "sin avisos". Por separado, con un `package.json` real
> (`"test": "node --test test/smoke.test.mjs"`) y un test real que lee
> `dist/index.html`: `nexa test` compiló el proyecto, corrió `npm test`,
> y el resultado (pass/fail) se verificó en ambos sentidos — con el test
> fallando a propósito primero (propagó código de salida 1), y pasando
> después (código 0).
>
> 246 tests en Rust (workspace completo, +2 sobre la Fase 23: `report()`
> combina y cuenta avisos SEO + de paquetes correctamente, y no imprime
> nada cuando no hay ninguno).

---

## Fase 25 — Layouts compartidos (`src/layout.tsx`) (Hito 21) — ✅ completada

**Objetivo:** cerrar el último pendiente grande de la lista — un
proyecto real (header/footer/nav repetidos en cada página) no tenía
ninguna forma de compartir esa maqueta sin copiarla a mano en cada
`.tsx`. El riesgo explícito, señalado desde que se dejó pendiente, era
reabrir la composición de componentes rechazada desde la Fase 2 (props,
`<Otro/>`, anidar un componente dentro de otro).

**Entregables:**
- `src/layout.tsx`, opcional, único para todo el proyecto (sin anidar
  por directorio): se compila con el mismo parser/analyzer/renderer que
  cualquier página — Rust nunca aprende a pasar props ni a componer.
  Recibe los mismos `params`/traducciones que la página a la que
  envuelve, así que puede usar `{params.x}`/`t(...)`.
- El único mecanismo nuevo es un splice de texto sobre HTML ya
  renderizado: se exige exactamente un elemento `data-nexa-slot`, vacío,
  en todo el layout (si no, el build falla con un mensaje explícito); ahí
  se inserta, tal cual, el HTML que ya renderizó la página. Ni el layout
  ni la página se conocen entre sí antes de ese punto.
- Explícitamente rechazado, con un error claro en el build: un evento
  interactivo o una isla dentro del layout — sus ids de nodo viven en un
  `IrComponent` separado del de cada página (ambos numerados desde 0),
  así que compartir un manifiesto de activación colisionaría. Documentado
  como límite deliberado, no oculto.
- Las clases `nx-*` que el layout use se suman a las de cada página para
  decidir el contenido de `nexa-ui.css` — sin esto, un `nx-btn` en el
  header nunca habría enviado su CSS real.

**Criterio de salida:** dos páginas envueltas por el mismo layout
comparten header/footer/nav sin duplicar una línea de HTML, incluso a
través de una navegación SPA; un layout con un evento interactivo o sin
(o con más de un) `data-nexa-slot` falla el build con un mensaje que
explica por qué.

> **Bug real preexistente, encontrado construyendo esto — no en el
> código nuevo:** `nexa_ui::collect_used_classes` recogía *cualquier*
> `class="..."` estático del árbol, no solo clases `nx-*` reales. Una
> página (o, con esta fase, un layout) con `class="site-shell"` o
> cualquier clase propia del proyecto activaba en falso el aviso
> `NEXA-PKG-001` ("usa clases nx-* de @nexa/ui") y enlazaba un
> `<link rel="stylesheet" href="/assets/nexa-ui...css">` — que, como
> ninguna clase real de `@nexa/ui` estaba en juego,
> `build_stylesheet_from_classes` nunca llegaba a escribir. Con el
> hashing de contenido de la Fase 23, este bug se volvió mucho más
> visible: el HTML servido en producción habría llevado literalmente
> `<link href="/assets/nexa-ui.pending.css">` — el hash "pendiente"
> nunca resuelto, porque la resolución del placeholder solo ocurre
> cuando `write_ui_stylesheet` encuentra contenido real. Se corrigió en
> la raíz (`nexa-ui::scan::collect_used_classes` ahora filtra contra las
> familias reales de `registry::COMPONENTS`), no en cada consumidor —
> las dos decisiones (enlazar el `<link>`, generar el archivo) vuelven a
> ser consistentes por construcción.
>
> **Verificado con un proyecto real, no solo con `cargo test`:** dos
> páginas (`/`, `/about`) envueltas por un `src/layout.tsx` con
> header/nav/footer reales y una clase (`site-shell`) deliberadamente NO
> perteneciente a `@nexa/ui`. Antes del fix: `nexa build` emitía
> `NEXA-PKG-001` en falso y `dist/index.html` quedaba con
> `href="/assets/nexa-ui.pending.css"` literal. Después: cero avisos
> falsos, sin `<link>` de CSS en absoluto (correcto, no se usó
> `@nexa/ui` de verdad). Un navegador real (Chromium vía Playwright)
> confirmó el header/footer/nav presentes en ambas páginas, incluso
> navegando de una a otra por el router SPA (sin `page.goto` directo a
> la segunda), y cero peticiones con error. Por separado, tres builds
> reales de un layout inválido confirmaron los tres mensajes de error
> esperados: un `onClick` en el layout, cero `data-nexa-slot`, y (por
> revisión de código, ya cubierto por un test) más de uno.
>
> 250 tests en Rust (workspace completo, +4 sobre la Fase 24: el splice
> del slot inserta el body y falla con un mensaje claro sin slot
> (`layout.rs`), y `collect_used_classes` ignora clases ajenas a
> `@nexa/ui` pero conserva las reales mezcladas con otras (`scan.rs`)).

---

## Fase 26 — Tiempo real: `connectSSE`/`connectSocket` en `@nexa/http` (Hito 22) — ✅ completada

**Objetivo:** cerrar el último pendiente de la lista — sin ningún
mecanismo de primera clase para notificaciones/chat/dashboards en vivo,
un proyecto tenía que escribir su propio wrapper de `EventSource`/
`WebSocket` desde cero cada vez. Relevante en particular para los otros
proyectos reales del usuario (logística tipo Uber, notificaciones).

**Entregables:**
- `connectSSE<T>({ url })`: envuelve `EventSource`, expone
  `data: Signal<T | undefined>` (el último mensaje, parseado como JSON
  si se puede, texto crudo si no) y `error: Signal<unknown>`, más
  `close()`.
- `connectSocket<T>({ url })`: envuelve `WebSocket`, expone `data`,
  `status: Signal<"connecting"|"open"|"closed">`, `error`, `send(msg)`
  (serializa objetos como JSON, manda un string tal cual) y `close()`.
- Ambas son 100% cliente, cero cambios en Rust/Nexa Core — ni protocolo
  propio, ni un endpoint que Nexa genere: el backend real del proyecto
  implementa el SSE/WS, igual que ya hace con `load()`. `eventSourceImpl`/
  `webSocketImpl` inyectables (mismo patrón que `fetchImpl` en
  `createApi`) para poder testear sin abrir una conexión real.
- Fuera de alcance a propósito: sin reconexión automática ni backoff —
  `status` pasa a `"closed"` y se queda ahí; reconectar es
  responsabilidad del proyecto.

**Criterio de salida:** un mensaje real empujado por un servidor SSE
real llega a `data.value`; un mensaje enviado por `send()` a un
WebSocket real y devuelto (echo) llega igual a `data.value`; `status`
refleja el ciclo de vida real de la conexión.

> **Verificado con servidores reales, no solo con fakes en
> `vitest`:** además de los tests unitarios (con `EventSourceImpl`/
> `webSocketImpl` inyectados, fakes controlados a mano), se levantó un
> servidor Node real (`http` + `ws`) sirviendo un endpoint SSE real
> (tres eventos reales, con un `setInterval` real) y un echo de
> WebSocket real, y una página real cargada en Chromium (vía
> Playwright) que importa el bundle real de `@nexa/http` (compilado con
> `esbuild`, sin mocks). El navegador recibió los tres eventos SSE
> reales en orden, abrió el WebSocket real (`status` pasó de
> `"connecting"` a `"open"` de verdad), mandó un mensaje real y recibió
> su eco real de vuelta — sin ningún doble, sin usar la API nativa
> directamente en el test, solo a través de `connectSSE`/`connectSocket`.
>
> 250 tests en Rust (sin cambios — esta fase es 100% TypeScript) + 23
> tests en `@nexa/http` (+13 sobre lo que ya existía: parseo de JSON y
> fallback a texto crudo en SSE, error real expuesto, `close()` real;
> ciclo de vida completo de `connectSocket` incluyendo `send()` con
> objeto vs. string).

---

## Fase 27 — `export const head` en `src/layout.tsx` (Hito 23) — ✅ completada

**Objetivo:** cerrar un hueco real de la Fase 25 — un layout solo puede
aportar HTML de `<body>` (todo lo que devuelve su componente se
inserta, vía splice, dentro de `data-nexa-slot`). Un `<link rel="icon">`
o un `<link rel="stylesheet">` puesto ahí queda atrapado en `<body>`,
donde no todos los navegadores lo detectan — descubierto usando
`tutorial-nexa.pages.dev` real: el favicon nunca se aplicaba pese a
estar presente en el HTML servido.

**Entregables:**
- `nexa-ast::Component` gana un campo `head: Option<JsonTemplate>` —
  mismo tipo que ya usa `schema`, no una estructura nueva.
- `nexa-parser::head::find_head` extrae `export const head = { ... }`
  con el mismo patrón que `schema.rs`/`seo.rs`: un objeto literal, nunca
  código ejecutado.
- `nexa-seo::render_layout_head` resuelve `head` (reutilizando
  `resolve_json_template`, el mismo mecanismo que ya resuelve `schema`)
  a HTML real: `icon` → `<link rel="icon">` (con el `type` inferido de
  la extensión), `appleTouchIcon` → `<link rel="apple-touch-icon">`,
  `stylesheets` (array) → un `<link rel="stylesheet">` por entrada.
- `nexa-cli::layout::render_for_page` devuelve ahora también
  `head_html`; `pipeline::compile_page` lo mezcla en el `<head>` real
  del documento (junto a `seo`/`hreflang`/`pwa`) — a diferencia del
  resto de lo que devuelve un layout, esto nunca pasa por el splice del
  slot.

**Criterio de salida:** un layout que declara
`export const head = { icon: "/static/logo.svg", stylesheets: [...] }`
produce esos `<link>` dentro de `<head>` en el HTML final — no dentro
de `<body>`.

> **Verificado con un proyecto real, no solo con `cargo test`:**
> `tutorial-nexa` (el sitio de presentación de Nexa) tenía exactamente
> este problema en producción — confirmado con `view-source:` en un
> navegador real, el `<link rel="icon">` aparecía dentro de `<body>`.
> Se migró su `src/layout.tsx` a `export const head`, se reconstruyó
> con el binario de esta fase, y el mismo `view-source:` confirmó los
> tres `<link>` (icon, apple-touch-icon, stylesheet) ya dentro de
> `<head>`, antes de `</head>`.
>
> 253 tests en Rust (workspace completo, +3 sobre la Fase 26: los tres
> campos de `head` se resuelven a `<link>` reales, `head` ausente no
> produce nada, y un valor con comillas embebidas no rompe el HTML).

---

## Fase 28 — Toast/snackbar (`ui.notify`) en `@nexa/ui` (Hito 24) — ✅ completada

**Objetivo:** una notificación efímera *dentro* de la página (tipo
`$q.notify()` de Quasar) para dashboards y SPAs — distinta de
`platform.notify()` (Fase 13), que es una notificación real del sistema
operativo. `@nexa/ui` tenía 4 componentes (Button/Input/Card/Dialog);
este es el primero cuyo HTML no lo escribe el desarrollador, lo crea la
llamada misma.

**Entregables:**
- `ui.notify({ message, variant?, duration? })` en `packages/ui` —
  crea el toast, lo apila en un contenedor fijo (`bottom-right`), lo
  auto-cierra a los 4s por defecto (`duration: 0` lo desactiva), y
  devuelve `{ close() }` para cerrarlo a mano. Accesible: `role="status"`
  + `aria-live="polite"`, más un botón de cierre real.
- Mismo mecanismo de siempre para que `nexa-activation` lo detecte
  cero-config: `ui.notify(...)` en un handler dispara
  `import { ui } from "ui";` automático (`ui` ya es builtin desde antes
  de esto, junto con `openDialog`/`closeDialog`) — sin declarar nada en
  `nexa.toml`.
- CSS auto-inyectado en runtime (un único `<style data-nexa-ui-toast>`,
  chequeado por presencia real en el DOM, no por un booleano en
  memoria) — a propósito, porque el tree-shaking a nivel de sitio de
  `nexa-ui.css` solo detecta `class="..."` estático en el JSX de cada
  página, y el HTML de un toast no existe hasta que `notify()` corre.

**Criterio de salida:** un handler que llama `ui.notify(...)` no
necesita declarar nada; el toast aparece, es accesible, se apila con
otros, se autocierra o se cierra a mano, y no deja el `<style>`
duplicado sin importar cuántas veces se llame.

> **Bug real encontrado por el propio test, no por revisión de
> código:** la primera versión usaba un booleano en memoria
> (`stylesInjected`) para no reinyectar el `<style>` dos veces. El test
> "solo inyecta el `<style>` una vez" fallaba porque el `beforeEach` de
> otro test había *limpiado el DOM* sin que nadie avisara al booleano
> — quedaba `true` para siempre aunque el `<style>` real ya no
> estuviera. Corregido chequeando la presencia real en el DOM
> (`document.head.querySelector(...)`) en vez de un flag separado — más
> simple y se autorepara si algo externo llega a borrar el `<style>`.
>
> **Verificado con un proyecto real y un navegador real, no solo con
> `vitest`:** un dashboard de prueba con dos botones (`Guardar` /
> `Forzar error`), compilado con `nexa build` real — el chunk generado
> confirmó `import { ui } from "ui";` automático, sin declarar nada. En
> Chromium real (Playwright): los dos toasts se apilaron sin
> reemplazarse, el de `duration` default desapareció solo a los ~4.2s
> mientras el de `duration: 0` seguía ahí, y el botón de cerrar lo sacó
> del DOM al instante — cero errores de consola, cero requests
> fallidos.
>
> 14 tests en `@nexa/ui` (+8 sobre los 6 que ya existían de Dialog):
> creación real en el DOM, accesibilidad, apilado, auto-cierre por
> `duration`, `duration: 0` lo desactiva, cierre manual por botón y por
> `close()` del handle, y el `<style>` nunca se duplica.

---

## Fase 29 — Locale por defecto fuera de `[locale]` (Hito 12) — ✅ completada

**Objetivo:** una página que vive fuera de `[locale]` (por ejemplo
`src/pages/index.tsx`, una gate de selección de idioma) pero comparte
`src/layout.tsx` con el resto del sitio — y ese layout usa
`params.locale`/`t(...)` como cualquier otra página — no debe quedar con
`href`s sin resolver ni con placeholders `<!--nexa:t(clave)-->` visibles
en producción solo por no tener un segmento de ruta `:locale`.

**Cómo se encontró:** construyendo `tutorial-nexa` (proyecto real hecho
con Nexa), su `src/pages/index.tsx` (gate ES/EN) comparte el
`src/layout.tsx` del resto del sitio. El header de ese layout usa
`params.locale` y `t("nav.home")` etc. — funciona perfecto en las 40
páginas que viven bajo `[locale]`, pero en la página suelta de la raíz
el nav quedaba con enlaces sin `href` (atributo omitido entero, no
`"/undefined"`) y el texto reemplazado por comentarios HTML inertes
(`<!--nexa:t(nav.home)-->`) — visible en producción, real, no un caso de
laboratorio.

**Entregables:**
- `crates/nexa-cli/src/pipeline.rs`: `compile_page` ahora completa
  `params["locale"]` con `DEFAULT_LOCALE` (`"es"`) cuando la key no
  existe, antes de cargar traducciones y construir `RenderContext`/
  `SeoContext` — una sola vez, al principio de la función, para que
  todo lo que sigue (render, SEO, hreflang, `<html lang>`) vea un
  locale real sin tener que lidiar cada uno por separado con la
  ausencia.
- Mismo valor (`"es"`) que ya estaba hardcodeado para `<html lang>`
  (Fase 9) — se unifican dos lugares que antes coincidían por
  casualidad (uno resolvía a `"es"`, el otro simplemente no existía).

**Criterio de salida:** una página fuera de `[locale]` que reutiliza un
layout con `params.locale`/`t(...)` genera HTML completo — `href`s
reales, texto traducido real — usando el locale por defecto del
proyecto, en vez de placeholders inertes.

> **Ajuste de alcance, decidido antes de empezar:** no se agregó
> ningún `default_locale` configurable en `nexa.toml` — no existe hoy
> ninguna sección `[i18n]`, y agregar una para un solo valor hardcodeado
> sería más superficie de configuración que necesidad real. Se
> reutilizó el mismo literal `"es"` que Fase 9 ya hardcodeaba para
> `<html lang>`, así los dos caminos quedan consistentes entre sí. Si en
> el futuro un proyecto real necesita otro default (ej. `"en"`), es el
> momento de agregar el campo — no antes.
>
> **Verificado con un proyecto real, no solo `cargo test`:** en
> `tutorial-nexa`, antes del fix, `curl` sobre `/` mostraba
> `<nav class="nx-nav"><a><!--nexa:t(nav.home)--></a>...` — enlaces sin
> `href`, texto vacío. Con el binario reconstruido, la misma página
> compila a `<a href="/es">Inicio</a><a href="/es/tutorial">Tutorial</a>
> <a href="/es/compatibility">Compatibilidad</a>
> <a href="..." class="nx-nav-github">GitHub</a>`, `<html lang="es">`
> correcto, y sin ningún `hreflang` espurio (la ruta de esta página no
> tiene `:locale`, así que `hreflang_head` sigue devolviendo vacío,
> como corresponde). En Chromium real (Playwright): los cuatro enlaces
> del nav de la gate son clickeables y navegan de verdad (`Tutorial` →
> `/es/tutorial`); se re-verificaron las 40 páginas bajo `[locale]`
> (`/es`, `/en`, click-through completo hero → tutorial → step card →
> Siguiente/Anterior → cambio de idioma → Compatibility) sin ninguna
> regresión, cero errores de consola, cero requests fallidos.
>
> Los 253 tests del workspace de Rust siguen en verde (`cargo test`,
> todas las suites) — este fix no tiene un test unitario propio porque
> `compile_page` se verifica en este proyecto por convención vía E2E
> real (`nexa build`/`nexa preview` + Playwright sobre un proyecto de
> verdad) en vez de fixtures sintéticas, el mismo patrón que el resto
> del pipeline de `nexa-cli`.

---

## Fase 30 — Iteración de listas (`<For>`) (Hito 12) — ✅ completada

**Objetivo:** cerrar el gap más grave del framework, señalado en un
análisis de gaps hecho por el propio usuario intentando migrar su
proyecto real (Oweeme) a Nexa: *"Nexa hoy no tiene ninguna forma de
iterar una lista dentro de una página"* — confirmado literal en
"Límites conocidos" (`sin bucles (.map())`). Sin esto, un listado de
artículos, un catálogo de productos, un directorio — el caso de uso
insignia que el propio README promociona — no se podía escribir como
página real de Nexa (HTML estático, SEO real); la única salida
documentada era una isla, perdiendo exactamente el beneficio de
SEO/HTML-real que es la razón de ser del framework. Trackeado como
issue #1 en `oweenexa` (ver también #9-#21, el resto del backlog que
salió de ese mismo análisis).

**Diseño:** un primitivo de iteración literal y segura, resuelto por el
compilador igual que `t()` — nunca un `.map()` real ni código
arbitrario del desarrollador:

```tsx
<For each={data.items}>
    {(item) => (<li>{item.title}</li>)}
</For>
```

`<For>` no reabre la composición de componentes rechazada desde la
Fase 2: no es un componente, es un patrón sintáctico reconocido por
nombre exacto. El cuerpo del callback se clasifica **una sola vez**
(es una plantilla, un único `NodeId` por nodo interactivo adentro) — el
*renderer*, no el *analyzer*, es quien produce una copia de HTML por
cada elemento real del array, en tiempo de build/request. Esto evita
por completo la pregunta de "¿cómo le doy un `NodeId` distinto a cada
copia?": no hace falta, todas comparten el mismo, y es el runtime de
activación quien las activa una por una.

**Entregables:**
- `nexa-ast`: `Node::For(ForLoop { each, item_name, body })` —
  `each: Expr` (mismo tipo limitado de siempre), `body: Box<Node>` (el
  callback de cuerpo conciso solo puede devolver una expresión, nunca
  múltiples hermanos).
- `crates/nexa-parser/src/for_loop.rs` (nuevo): reconoce `<For>` en
  `jsx.rs` **antes** del match que rechaza "composición de
  componentes" — un nombre JSX capitalizado llega como
  `JSXElementName::IdentifierReference`, no `Identifier` (bug real
  encontrado por los tests: mi primer intento interceptaba el nombre
  equivocado y ningún `<For>`, ni siquiera el caso feliz, parseaba).
  Valida en tiempo de parseo: `each` debe tener `data`/`params` como
  raíz; el hijo debe ser exactamente un arrow function de cuerpo
  conciso con un único parámetro sin destructuring/default, cuyo
  cuerpo es un único elemento JSX — cualquier otra forma es un
  `ParseError::Unsupported` explícito, nunca un compilado a medias.
- `nexa-ir::IrNodeKind::For { each, item_name, body: Box<IrNode> }` +
  `IrNode::walk()` actualizado para bajar a `body` — sin esto, todo lo
  que recorre el IR manualmente (no vía `walk()`) se queda ciego a lo
  que hay dentro de un `<For>`.
- `nexa-analyzer::classify_node`: clasifica el `For` como `Dynamic`
  (depende de `data`/`params`, se resuelve sin JS) y agrega
  `for_loop.each.root_identifier()` al grafo de dependencias — el
  cuerpo se clasifica recursivamente una sola vez.
- `nexa-renderer`: `RenderContext` gana `loop_binding:
  Option<(&str, &Value)>` — un solo nivel a propósito, `<For>` anidado
  queda fuera de alcance. `resolve()`/`resolve_expr_json()` prueban
  `data`/`params` primero, y si la raíz de la expresión coincide con el
  `item_name` del `loop_binding` activo, resuelven contra ese valor —
  cualquier otro identificador (o el mismo nombre fuera de su propio
  `<For>`) sigue siendo un marcador inerte, sin cambios. `render_for`
  resuelve `each` a un array real (`Vec::new()` si no es un array —
  nunca se inventa un elemento) y concatena el HTML de renderizar
  `body` una vez por elemento, cada vez con un `RenderContext` nuevo
  apuntando a ese elemento.
- Cuatro sitios con recursión manual sobre el IR (no exhaustiva, el
  compilador no los detecta al agregar la variante) necesitaron un
  brazo `For` explícito para no quedarse ciegos a lo que hay dentro de
  un `<For>`: `nexa-activation::build::collect` (sin esto, un
  `onClick` dentro de un `<For>` quedaba completamente ausente del
  manifiesto — el bug más serio de los cinco, encontrado por un test
  real antes de llegar a producción), `nexa-seo::analyzer::check_tree`
  (un `<img>` sin `alt` dentro de un `<For>` no generaba aviso),
  `nexa-cli::pipeline::has_forms`, `nexa-cli::image_scan::visit`,
  `nexa-ui::scan::walk`.
- `packages/runtime/src/activate.ts`: `initActivation` cambia
  `querySelector` (un solo elemento) por `querySelectorAll` + activar
  cada uno por separado — un mismo `data-nexa="<id>"` ahora puede
  aparecer en más de un elemento (todas las copias que produce un
  `<For>` comparten id). Cada copia recibe su propio `el` real vía su
  propio closure `trigger()`, así que un handler que lee
  `event.currentTarget` actúa sobre el elemento correcto — sin
  necesitar ningún mecanismo nuevo de "closures por índice": el modelo
  de activación de Nexa nunca pasó contexto además del elemento, así
  que ya alcanzaba con activar cada copia de verdad.

**Criterio de salida:** una página con `<For each={data.items}>`
compila a HTML real en `nexa build`, un nodo por cada elemento real del
array; un `onClick` dentro del callback se activa de forma
independiente por copia; `nexa build` falla con un mensaje explícito
ante cualquier forma no reconocida.

> **Ajuste de alcance, decidido antes de empezar:** sin soporte para
> `<For>` anidado (un solo `loop_binding` en `RenderContext`, sin
> pila) — no había ningún caso de uso real pidiéndolo todavía. Sin
> `index`/segunda variable del callback (`(item, i) => ...`) — el
> parser rechaza explícitamente más de un parámetro; se puede agregar
> después si hace falta, sin romper nada de esto. `each={params.x}`
> se acepta sintácticamente (mismo `Expr` que `data.x`) pero
> `params` nunca es un array real (viene de segmentos de URL) — itera
> cero veces siempre hoy, documentado como comportamiento explícito en
> el test correspondiente, no como caso soportado de verdad.
>
> **Dos bugs reales encontrados por los tests, no por revisión de
> código:** (1) mi primera versión de `for_loop.rs`/`jsx.rs`
> interceptaba `<For>` con el patrón de nombre equivocado
> (`JSXElementName::Identifier`, reservado a tags en minúscula como
> `<div>`) — cualquier nombre JSX capitalizado llega como
> `IdentifierReference`, así que ni el caso feliz parseaba hasta que
> los dos primeros tests de `nexa-parser` fallaron y señalaron
> exactamente por qué. (2) `nexa-activation::build::collect` hacía
> early-return en cualquier `IrNodeKind` que no fuera `Element` — sin
> el brazo `For` explícito, un `onClick` dentro de un `<For>` nunca
> llegaba al manifiesto de activación en absoluto (no un bug de
> "solo se activa la primera copia": ni siquiera existía la entrada).
> Atrapado por un test de `nexa-activation` escrito a propósito para
> este caso, antes de llegar a verificación E2E.
>
> **Verificado con un proyecto real y un navegador real, no solo
> `cargo test`:** un proyecto de prueba con `load()` real (HTTP a un
> `.json` servido por `python3 -m http.server`) y tres artículos reales
> — `nexa build` generó las tres `<li>` reales con sus `href`/título
> resueltos, `nexa lint` sin avisos. En Chromium real (Playwright): un
> botón "Quitar" dentro de cada `<li>` del `<For>`, clickeado en el
> del medio primero — **solo ese** `<li>` desapareció (no el primero,
> no los tres), confirmando que las N copias se activan de forma
> verdaderamente independiente, no que "la primera acapara todos los
> clics". El ejemplo de referencia `examples/oweeme-shop` migró su
> catálogo de dos productos hardcodeados a `<For each={data.products}>`
> sobre el backend PHP real — verificado con el mismo backend
> corriendo de verdad (`podman run ... php:8.3-cli`) + Playwright,
> confirmando además que la isla `productFilter` (Fase 16) sigue
> recibiendo el array completo sin romperse.
>
> Los 270 tests del workspace de Rust (+17 sobre los 253 de la Fase 29:
> 9 en `nexa-parser`, 5 en `nexa-renderer`, 1 en `nexa-analyzer`, 1 en
> `nexa-seo`, 1 en `nexa-activation`, más los que ya existían
> actualizados) siguen en verde, más 13 tests en `packages/runtime`
> (+1 sobre los 12 de antes, cubriendo el caso de varios elementos con
> el mismo id activándose por separado).

---

## Fase 31 — Islas dentro de `src/layout.tsx` (Hito 12) — ✅ completada

**Objetivo:** issue #6 del backlog de gaps (mismo origen que la Fase
30) — un header/nav compartido con estado real (toggle de tema,
selector de idioma, menú hamburguesa móvil) hoy había que copiarlo a
mano en cada página, porque `src/layout.tsx` rechazaba cualquier
`onClick` **o isla** con el mismo error. El propio comentario de la
Fase 25 ya señalaba el escape valve: una isla no usa ids de manifiesto
de activación, así que el problema de colisión de ids (la razón real
del rechazo) nunca le aplicó — solo no se había habilitado todavía.

**Entregables:**
- `crates/nexa-cli/src/layout.rs::validate`: separa el chequeo — sigue
  rechazando `Classification::Interactive` (un `onClick` en cualquier
  parte del layout, incluido dentro del fallback de una isla, donde el
  problema de ids colisionados sigue aplicando igual) pero ya no
  rechaza `Classification::Island`.
- `RenderedLayout` gana `island_specifiers: BTreeSet<String>` — los
  specifiers que el layout mismo declara, recogidos con la misma
  función que ya usaba `pipeline.rs` para la página
  (`island_specifiers`, subida a `pub(crate)` para reutilizarla tal
  cual en vez de duplicar la lógica).
- `pipeline::compile_page` une esos specifiers a los de la página antes
  de calcular `used_imports`/`has_islands`/`pkg_warnings` — sin esto,
  un `data-nexa-island` en el layout habría compilado sin error pero
  jamás se habría activado en ninguna página (el import map no lo
  tendría, y el bootstrap de `nexa-islands.js` ni se cargaría si la
  página en sí no tenía islas propias).

**Criterio de salida:** un `data-nexa-island` dentro de
`src/layout.tsx` compila sin error y se activa correctamente en cada
página del sitio; un `onClick` directo (dentro o fuera de una isla)
sigue fallando con un mensaje explícito.

> **Verificado con un proyecto real y un navegador real, no solo
> `cargo test`:** un layout con una isla `siteHeader` (`data-nexa-island`
> + `data-nexa-strategy="load"`) montando un toggle de tema real, dos
> páginas distintas compartiendo el mismo layout, cero islas propias en
> ninguna de las dos páginas. `nexa build` generó el import map
> (`{"siteHeader":"/vendor/site-header.js"}`) y el bootstrap con
> `initIslands` en **ambas** páginas — la prueba de que el mecanismo de
> unión de specifiers layout+página funciona, no solo que compila. En
> Chromium real (Playwright): la isla montó (`data-mounted="true"`),
> clickear el header cambió el texto y `<html data-theme="dark">` de
> verdad, sin errores de consola. Un segundo proyecto con un `onClick`
> directo en el layout siguió fallando con el mensaje esperado,
> confirmando que esto es aditivo, no un aflojamiento general.
>
> Los 273 tests del workspace de Rust (+3 sobre los 270 de la Fase 30,
> los tres en `nexa-cli::layout::tests` — isla aceptada, `onClick`
> directo rechazado, `onClick` dentro del fallback de una isla también
> rechazado) siguen en verde.

---

## Fase 32 — Contenedor de página estable en `initRouter` (Hito 12) — ✅ completada

**Objetivo:** issue #7 del backlog — complemento directo de la Fase 31.
`initRouter` (Fase 22) ya daba navegación SPA real, pero reemplazaba
`<body>` completo en cada click interno — una isla montada en
`src/layout.tsx` (Fase 31) se remontaba en cada navegación, perdiendo
cualquier estado que hubiera acumulado (un tema elegido, un dropdown
abierto). Confirmado con mi propia verificación de la Fase 31: el tema
volvía a "claro" al navegar a otra página.

**Diseño:** `initRouter` distingue el HTML que viene de `layout.tsx`
del que viene de `data-nexa-slot` — en una navegación de cliente, si
tanto el documento actual como el de destino tienen el mismo
`data-nexa-slot`, solo se reemplaza *su* contenido (`slot.innerHTML =
...`), nunca `<body>` entero. Sin layout (o si la página de destino no
trae el mismo slot, caso borde), cae al comportamiento de siempre.

**Entregables:**
- `packages/router/src/navigate.ts::applyPage` devuelve el contenedor
  real que cambió (el slot, o `document` si no hay slot) — antes no
  devolvía nada, `onNavigate` siempre recibía `document`.
- `InitRouterOptions.onNavigate` cambia de `(root: Document) => void` a
  `(root: ParentNode) => void` — `Document` y `Element` ya comparten
  esa interfaz, así que `initActivation`/`initForms`/`initIslands`
  (todos tipados `root?: ParentNode` desde que existen) no necesitaron
  ningún cambio: ya aceptaban un contenedor más chico que el documento
  completo, solo que nadie se los pasaba todavía.
- `syncManifestScript`: el `<script data-nexa-manifest>` (Fase 5) vive
  siempre al final de `<body>`, fuera de cualquier slot — un swap
  acotado al slot no lo toca, así que hay que actualizarlo a mano al de
  la página de destino (o insertarlo/quitarlo si una de las dos páginas
  no tiene contenido interactivo propio).
- `crates/nexa-cli/src/bootstrap.rs::reactivate_fn`: `reactivate(root)`
  ahora busca el manifiesto siempre en `document` (nunca en `root` —
  si `root` es el slot, el manifiesto no vive ahí adentro) pero escanea
  activación/formularios/islas dentro de `root`. Los eventos y
  formularios no tienen este problema (estructuralmente no pueden vivir
  en el layout, Fase 31), pero las islas sí: `reactivate` gestiona su
  propia `disposeSlotIslands`, **separada** de la `disposeIslands` de la
  carga inicial — así la primera navegación nunca desmonta lo que la
  carga inicial montó fuera del slot.

**Criterio de salida:** navegar entre dos páginas no desmonta ni
remonta una isla del layout; el contenido del slot se actualiza
correctamente con sus propios eventos/formularios/islas reactivados; el
scroll de lo que vive fuera del slot no se ve afectado.

> **Bug real encontrado por un navegador real, no por `cargo test`:**
> al renombrar la variable de disposición de islas dentro de
> `reactivate()` a `disposeSlotIslands`, la línea de la carga inicial
> (`disposeIslands = initIslands();`, fuera de `reactivate`) quedó
> asignando a una variable que ya no se declaraba en ningún lado —
> `ReferenceError: disposeIslands is not defined` en un `<script
> type="module">` real (los módulos ES son siempre modo estricto,
> donde asignar a una variable no declarada lanza en vez de crear una
> global silenciosa). Invisible en `cargo test`/`vitest` porque ninguno
> ejecuta el script generado completo en un navegador real — solo
> apareció al abrir la página en Chromium. Corregido declarando
> `disposeIslands` como una variable separada, nunca reasignada desde
> `reactivate`.
>
> **Verificado con un proyecto real y Chromium real:** una isla de
> header con un contador de montajes (`window.__mountCount`) y un
> toggle de tema real, dos páginas navegables entre sí por un `<a>`
> real interceptado por `initRouter`. Después de dos navegaciones
> (ida y vuelta): `__mountCount` siguió en `1`, el nodo DOM del header
> siguió siendo el mismo (marcado con un atributo propio antes de
> navegar, verificado que sobrevivió), el tema elegido antes de navegar
> siguió activo después, y el header siguió respondiendo a clicks
> (togglear el tema de nuevo funcionó) — no solo "no se rompió", sino
> que el estado real de la isla persistió de punta a punta. El scroll
> de un contenedor fuera del slot, con contenido interno, se preservó
> exacto (1500px antes y después) cuando el link de navegación vivía
> fuera de ese contenedor — un primer intento con el link *adentro* del
> área con scroll dio un falso negativo (Playwright hace
> scroll-into-view del elemento antes de clickearlo, un artefacto del
> test, no un bug real; confirmado moviendo el link fuera).
>
> Los 274 tests del workspace de Rust (+1 sobre los 273 de la Fase 31,
> en `nexa-cli::bootstrap::tests`) siguen en verde, más 17 tests en
> `packages/router` (+4 sobre los 13 de antes: swap acotado al slot con
> el resto de `<body>` intacto, `onNavigate` recibe el slot no el
> documento, el manifiesto se sincroniza al de la página de destino, y
> el caso sin layout sigue cayendo al reemplazo de `<body>` completo de
> siempre).

---

## Fase 33 — Link activo automático (`aria-current`) (Hito 12) — ✅ completada

**Objetivo:** issue #8 del backlog, el último del trío de nav/layout
(#6 islas en layout, #7 contenedor estable, #8 este). Sin composición
ni condicionales en el cuerpo de una página/layout, ni siquiera era
posible escribir a mano `class={path === href ? 'activo' : ''}` — el
mismo bug que hay que arreglar a mano en cualquier router de verdad
(un link "Inicio" marcado activo en cualquier página) no tenía forma de
corregirse en Nexa.

**Diseño:** resolución determinística en el renderer, mismo espíritu
que `seo`/`schema` — Rust ya sabe qué ruta está renderizando
(`route_pattern` + `params`, reconstruidos a la ruta real con
`nexa_loader::resolve_url`, la misma función que ya usa `nexa-i18n`
para `hreflang`). Un `<a href="...">` cuyo `href` resuelto coincide con
esa ruta recibe `aria-current="page"`; `data-nexa-match="prefix"` lo
vuelve por prefijo con límite de segmento.

**Entregables:**
- `RenderContext` gana `current_path: Option<&str>`, calculado una vez
  en `pipeline::compile_page` y pasado tanto al render de la página
  como al de `src/layout.tsx` (mismo mecanismo en ambos, sin
  distinción).
- `nexa-renderer::html::active_link_attr` — compara el `href` ya
  resuelto (reutilizando la resolución de `AttrValue::Static`/`Dynamic`
  que `render_attr` ya hacía, ahora extraída a `resolve_attr_string`
  para no duplicarla) contra `current_path`.
- `is_prefix_match` con límite de segmento: `current_path == href` o
  (`current_path` empieza con `href` **y** el siguiente carácter es
  `/`) — esto resuelve gratis el caso que preocupaba en el issue
  (`href="/"` en modo prefijo no matchea "toda ruta empieza con /",
  porque ninguna ruta real empieza con `//`).

**Criterio de salida:** un `<a href="/x">` recibe `aria-current="page"`
solo al renderizar `/x`; `data-nexa-match="prefix"` marca también las
subrutas; funciona igual en `layout.tsx` que en una página.

> **Ajuste de alcance, encontrado a mitad de la fase, no al planearla:**
> el diseño original asumía que resolver esto en el servidor alcanzaba
> — pero la Fase 32 (contenedor de página estable) significa que un nav
> dentro de `src/layout.tsx` **nunca se vuelve a renderizar del lado
> del servidor** en una navegación SPA. Verificado con Chromium real
> *antes* de dar la fase por terminada: después de un click interno, el
> nav seguía marcando la página con la que había cargado el sitio la
> primera vez — exactamente el mismo bug de Vue Router que esta fase
> pretendía resolver, reaparecido por la interacción con la Fase 32.
> Arreglado con `packages/router/src/active-links.ts::updateActiveLinks`
> — recalcula `aria-current` en todo el documento después de cada
> navegación, con el mismo criterio de coincidencia que el servidor. Por
> esto `data-nexa-match` **sí queda** en el HTML final (a diferencia de
> `data-nexa-strategy` para eventos, que se descarta): el cliente
> necesita releerlo para saber si le toca coincidencia exacta o por
> prefijo. Un nav dentro de `data-nexa-slot` no necesita este recálculo
> — llega resuelto en el HTML fresco de cada navegación, como cualquier
> otro contenido; el recálculo del cliente es redundante ahí pero
> inofensivo (misma respuesta, dos veces).
>
> **Verificado con un proyecto real y Chromium real:** un layout con un
> nav de dos links, dos páginas navegables entre sí por un `<a>` real.
> Antes del fix de arriba: navegar a "Otra" dejaba "Inicio" marcado
> activo. Después: el link activo cambia correctamente en cada
> navegación, ida y vuelta, en ambas direcciones. Coincidencia por
> prefijo (`data-nexa-match="prefix"`) verificada con una ruta de
> verdad tres niveles más profunda que el `href` declarado
> (`/other/deep` activando `<a href="/other" data-nexa-match="prefix">`).
> `nexa lint` sin avisos sobre el proyecto completo.
>
> Los 283 tests del workspace de Rust (+9 sobre los 274 de la Fase 32,
> todos en `nexa-renderer`) siguen en verde, más 23 tests en
> `packages/router` (+6 sobre los 17 de antes: 5 en un archivo nuevo,
> `active-links.test.ts`, más 1 de integración confirmando el
> recálculo tras una navegación SPA real).

---

## Fase 34 — Rutas dinámicas sin `paths` en producción, de verdad (Hito 12) — ✅ completada

**Objetivo:** issue #3 del backlog — un sitio con contenido creado en
vivo (un perfil nuevo, un artículo recién publicado) no puede enumerar
todos sus valores posibles en `paths` en tiempo de build. La única
salida documentada era `nexa preview` corriendo detrás de nginx, pero
el `proxy_pass` del `nginx.conf` generado estaba comentado, con un
ejemplo genérico (`/products/`) nunca verificado contra un nginx real.

**Lo que se encontró antes de llegar al objetivo real:** validando el
`deploy/nginx.conf` que ya generaba `nexa add nginx` (Fase 20/23) con
un nginx de verdad (no solo leyendo el texto), `nginx -t` falló:
`unknown directive "8}\.(js|css)$"`. La regex del `location` para
assets con hash de contenido (`^/assets/.+\.[0-9a-f]{8}\.(js|css)$`)
estaba sin comillas — nginx tokeniza `{`/`}` como delimitadores de
bloque de su propio archivo de configuración **siempre**, sin importar
que estén dentro de una regex, así que el `{8}` del cuantificador
rompía el parseo. Bug real, pre-existente desde la Fase 23, nunca
atrapado porque las verificaciones anteriores de ese archivo revisaban
contenido de texto (`cargo test`) pero no habían corrido `nginx -t` con
esta regla específica contra un nginx real. Arreglado envolviendo la
regex completa entre comillas dobles — eso alcanza para esconderle las
llaves al tokenizer de nginx.

**Entregables:**
- El bug de arriba, corregido en `nginx_scaffold.rs::TEMPLATE`.
- `nexa add nginx` ahora escanea `src/pages` (misma función que usa
  `nexa build`, `find_paths_declaration`) y lista, por nombre real,
  qué rutas dinámicas de *este proyecto* no declaran `paths` — en vez
  de un ejemplo genérico sin relación con el proyecto real. El bloque
  `proxy_pass` sigue comentado (sigue siendo una decisión del
  desarrollador ajustarlo a su estructura real), pero ahora trae el
  prefijo correcto pre-completado para la primera ruta detectada.
- Si el patrón detectado no tiene ningún segmento fijo antes del primer
  parámetro (ej. `/:locale/articles/:slug`, un proyecto con `[locale]`
  como primer segmento), el archivo generado lo marca explícitamente
  (`CAMBIAR-ESTE-PREFIJO`) en vez de inventar un prefijo que sería
  incorrecto con certeza.
- Si el proyecto no tiene ninguna ruta dinámica sin `paths`, el archivo
  lo dice explícitamente y no incluye ningún `proxy_pass` en absoluto.

**Criterio de salida:** un flujo real, probado con curl contra un
nginx real, sirviendo una ruta dinámica sin `paths` reenviada a un
`nexa preview` corriendo detrás — verificado, no solo documentado.

> **Ajuste de alcance, decidido explícitamente:** de las dos
> propuestas del issue, solo se construyó la primera (documentar y
> verificar el flujo con `nexa preview` detrás de nginx). La segunda
> (`nexa build --path <ruta>`, rebuild incremental de una sola ruta) se
> deja fuera a propósito — tiene una tensión real con el diseño de
> `nexa-ui.css` de la Fase 23: ese archivo es el hash del CSS de *todo
> el sitio junto*, conocido recién después de compilar todas las
> páginas (`commands::build::write_ui_stylesheet`). Un rebuild
> incremental de una sola ruta que introduce una clase `nx-*` nueva no
> podría recalcular ese hash sin tocar (o al menos releer) el resto del
> sitio ya compilado — resolverlo bien es una fase en sí misma, no un
> agregado chico a esta. El issue original ya marcaba esta segunda
> parte como condicional ("si se implementa"), así que no bloquea el
> criterio de salida real.
>
> **Verificado con un proyecto real, un nginx real (no solo `nginx
> -t`) y un `nexa preview` real corriendo detrás, no solo `cargo
> test`:** un proyecto con una página estática y una ruta
> `/articles/[slug]` con `load()` real contra un backend HTTP real (sin
> `paths`). `nexa add nginx` detectó y listó `/articles/:slug` por su
> nombre real. Con el bloque `proxy_pass` descomentado y nginx corriendo
> de verdad (`podman run ... nginx:alpine`, `--network=host`): la ruta
> estática (`/`) la sirvió nginx directo desde `dist/`, sin tocar el
> proceso de Nexa; `/articles/recien-publicado` (un slug que nunca
> existió en ningún build anterior) la reenvió de verdad a `nexa
> preview`, devolviendo el título real de un archivo JSON creado
> *después* del build; el `location` con hash de contenido devolvió
> `Cache-Control: immutable` de verdad sobre un asset real generado por
> el mismo build — la prueba de que el fix de la regex no solo pasa
> `nginx -t`, sirve tráfico real correctamente.
>
> Los 289 tests del workspace de Rust (+6 sobre los 283 de la Fase 33,
> todos en `nexa-cli::nginx_scaffold`) siguen en verde.

---

## Fase 35 — Reconexión con backoff en `connectSSE`/`connectSocket` (Hito 12) — ✅ completada

**Objetivo:** issue #5 del backlog — `@nexa/http` documentaba
explícitamente que un `WebSocket` cortado queda `"closed"` para
siempre, sin reconectar. Para cualquier feature de tiempo real real
(chat en vivo, notificaciones, un dashboard con datos live) una red
inestable mata la conexión hasta que el usuario refresca a mano.

**Entregables (`packages/http/src/realtime.ts`):**
- `reconnect?: boolean | ReconnectOptions` en ambos, `connectSSE` y
  `connectSocket` — `true` usa defaults (`maxAttempts: 10, baseDelayMs:
  500, maxDelayMs: 15000`), un objeto parcial completa lo que falte con
  esos mismos defaults. Sin `reconnect`, comportamiento idéntico a
  antes (retrocompatible).
- `status` gana `"reconnecting"` en ambas conexiones — `connectSSE` no
  tenía ningún `status` antes de esta fase, se agregó como parte de
  "unificar el comportamiento" que pedía el issue.
- Backoff exponencial con jitter (`backoffDelay`): mitad fija + mitad
  aleatoria del delay calculado, para que muchos clientes reconectando
  a la vez no lo hagan todos en el mismo instante.
- Para `connectSocket`: cada reconexión crea un `WebSocket` nuevo (uno
  cerrado no se puede reabrir) — `send()` sigue operando sobre la
  conexión vigente porque `socket` es una variable mutable capturada
  por referencia en el closure, no copiada al momento de crear `send`.
- Para `connectSSE`: el `EventSource` nativo ya reconecta solo sin
  `reconnect` configurado (comportamiento de siempre) — con
  `reconnect`, se cierra el nativo en `onerror` (para que no reintente
  por su cuenta sin backoff ni límite) y se maneja la reconexión a
  mano, mismo mecanismo que `connectSocket`.
- Un `closedByUser` interno distingue una desconexión deliberada
  (`close()`) de un corte real — solo el segundo dispara reconexión.

**Criterio de salida:** con `reconnect` declarado, cortar la conexión
dispara reintentos con backoff; `status` refleja `"reconnecting"` y
vuelve a `"open"` al reconectar; agotado `maxAttempts`, `status` pasa a
`"closed"` definitivo.

> **Verificado con `vitest` + fake timers (deterministico) Y con un
> servidor WebSocket real, matado y reiniciado de verdad, y un
> navegador real (no solo mocks) — los dos niveles, no uno solo:**
> con fake timers: 9 tests nuevos cubriendo backoff creciente, éxito
> reinicia el contador, agotar intentos cierra definitivo, `close()` a
> mano nunca reconecta, y `reconnect: true` usa los defaults. Con un
> servidor `ws` real (Node) + `nexa preview` real + Chromium real: una
> isla real con `connectSocket({ reconnect: {...} })`, conectada de
> verdad — matar el proceso del servidor (`kill -9`, no un mock) hizo
> que `status` pasara a `"reconnecting"` en el navegador real; con el
> servidor todavía caído por más tiempo del presupuesto de reintentos,
> `status` llegó a `"closed"` definitivo; en una corrida separada,
> reiniciando el servidor real *dentro* de la ventana de reintentos,
> `status` volvió a `"open"` un segundo después de que el servidor
> volviera a estar arriba — reconexión real de punta a punta, no
> simulada.
>
> Los 32 tests de `packages/http` (+9 sobre los 23 de antes) siguen en
> verde.

---

## Fase 36 — Texto JSX pegado a una expresión conserva su espacio (Hito 12) — ✅ completada

**Objetivo:** issue #22 — `normalize_jsx_text` le hacía `.trim()` a las
dos puntas de *cualquier* línea de texto JSX, incluida la única línea
de un bloque de una sola línea — así que `{a} — {b}` compilaba a
`"valorA— valorB"` (el espacio pegado a cada expresión, perdido).
Encontrado dos veces escribiendo ejemplos legítimos en fases anteriores
(Fase 30, Fase 33), trabajado alrededor con el idiom `{" "}` de React
en su momento — este issue lo arregla de raíz.

**Diagnóstico real:** JSX de verdad (Babel, `cleanJSXElementLiteralChild`)
no recorta las dos puntas de toda línea por igual — recorta el espacio
**inicial** solo si la línea *no* es la primera del bloque, y el
espacio **final** solo si *no* es la última. Un bloque de una sola
línea es simultáneamente su primera y su última línea, así que ningún
borde se toca. `normalize_jsx_text` no distinguía la posición de la
línea — trataba a todas por igual.

**Entregables:**
- `crates/nexa-parser/src/text.rs::normalize_jsx_text` reescrita con el
  mismo algoritmo posicional (recorte por posición de línea, no
  incondicional). El caso "indentación pura entre hermanos en líneas
  separadas colapsa a nada" sigue funcionando igual que antes — el fix
  es aditivo en el sentido de que solo cambia el caso de una sola línea
  con contenido real.
- Limpiado el único uso real del workaround `{" "}` que quedaba en el
  repo (`examples/oweeme-shop`) y el ejemplo correspondiente en
  `docs/REFERENCIA.md` — ya no hace falta.

**Criterio de salida:** `<p>{a} — {b}</p>` compila con el espacio a
ambos lados del guion preservado, sin `{" "}`; la indentación real
entre elementos en líneas separadas se sigue colapsando igual que
siempre.

> **Verificado con un proyecto real, no solo `cargo test`:** `nexa
> build` sobre una página con `<p>{"iPhone 17"} — ${"999"}</p>`
> compiló a `<p>iPhone 17 — $999</p>` — ambos espacios preservados.
> `examples/oweeme-shop` (con su backend PHP real corriendo) generó
> `"iPhone 17 — $999"` / `"Pixel 10 — $799"` sin el workaround que
> tenía antes.
>
> Los 295 tests del workspace de Rust (+6 sobre los 289 de la Fase 34:
> 5 en `nexa-parser::text::tests` cubriendo cada posición de línea por
> separado, más 1 de integración vía `parse_component`) siguen en
> verde.

---

## Fase 37 — Meta tags arbitrarios en `export const head` (Hito 12) — ✅ completada

**Objetivo:** issue #13 — `seo{}` cubre title/description/canonical/
OpenGraph/Twitter card, pero no tiene escape hatch para un meta tag
suelto que un servicio de terceros pida (verificación de Search
Console, dominio de Facebook, `theme-color` puntual).

**Entregables:**
- `nexa-seo::render_layout_head` (Fase 27) gana un campo `meta: [{name
  | property, content}]` — reutiliza la infraestructura de resolución
  ya existente (`JsonTemplate` → `resolve_json_template`), no un
  mecanismo nuevo. Acepta `name` (la forma común) o `property` (la que
  usan Open Graph/`theme-color`); una entrada sin `content` se omite en
  silencio.
- **De paso, un hueco real encontrado revisando el código, no en el
  issue original:** `component.head` (el campo del propio `nexa-ast`)
  se parseaba para *cualquier* componente desde la Fase 27, pero
  `pipeline::compile_page` solo lo usaba para el layout — una página
  individual podía escribir `export const head = {...}` y quedaba
  parseado y **completamente descartado**, sin error ni aviso. Arreglado:
  ahora `compile_page` también resuelve el `head` de la propia página,
  agregado al `<head>` final después del del layout.

**Criterio de salida:** `export const head = { meta: [...] }` en
`src/layout.tsx` o en una página produce `<meta>` reales dentro de
`<head>` — verificado con `curl`/lectura directa del HTML servido, no
solo del cuerpo.

> **Verificado con un proyecto real, no solo `cargo test`:** `nexa
> build` sobre una página con `head.meta` de dos entradas (`name` y
> `property`) generó ambos `<meta>` reales, confirmados dentro de
> `<head>...</head>` (no en `<body>`) por posición literal en el HTML
> servido. `nexa lint` sin avisos. Reverificado que un proyecto con
> layout (Fase 31) sigue compilando sin error tras agregar la segunda
> llamada a `render_layout_head` — sin regresión.
>
> Los 300 tests del workspace de Rust (+5 sobre los 295 de la Fase 36,
> todos en `nexa-seo::layout_head::tests`) siguen en verde.

---

## Fase 38 — `platform.sessionStorage()` (Hito 12) — ✅ completada

**Objetivo:** issue #15 — `platform.storage()` solo envuelve
`localStorage`, sin variante de sesión para datos que no deberían
sobrevivir cerrar la pestaña (un paso de wizard, un filtro temporal).

**Entregables:** `packages/platform/src/storage.ts` extrae un
`wrapStorage()` compartido (ambas funciones envuelven un objeto
`Storage`-shaped, la única diferencia es cuál usan por defecto) y
agrega `createSessionStorage()`, expuesta como `platform.sessionStorage()`
— mismo contrato exacto (`get`/`set`/`remove`) que `platform.storage()`.

**Criterio de salida:** `platform.sessionStorage()` funciona igual que
`platform.storage()` pero sobre `sessionStorage` real.

> **Verificado con un navegador real, no solo `vitest`:** una página
> con un handler que escribe en ambos (`platform.sessionStorage()` y
> `platform.storage()`) — en Chromium real, confirmado con
> `sessionStorage.getItem()`/`localStorage.getItem()` directos (no solo
> leyendo de vuelta con el propio wrapper) que cada uno escribió en el
> almacenamiento real correspondiente, sin cruzarse.
>
> 5 tests nuevos en `packages/platform` (31 total, todos en verde).

## Fase 39 — Acceso directo a Cache API (Hito 12) — ✅ completada

**Objetivo:** issue #17 — el único acceso a la Cache API era indirecto,
vía `[pwa.cache]` dentro del service worker que genera `nexa add pwa`.
Un proyecto sin PWA que igual quisiera cachear puntualmente una
respuesta pesada no tenía ninguna vía declarativa.

**Entregables:** `packages/platform/src/cache.ts::openCache(name,
backend?)` — envoltorio delgado sobre la Cache API nativa
(`caches.open`/`match`/`put`/`delete`), expuesto como
`await platform.cache("nombre")`. Independiente por completo de
`[pwa]`/`[pwa.cache]` — ambos mecanismos pueden convivir sin pisarse
porque operan sobre cachés con nombre propio.

**Criterio de salida:** se puede cachear y leer una respuesta puntual
sin depender de `[pwa]`/service worker.

> **Verificado con un navegador real:** un handler que hace
> `platform.cache("test-cache")`, guarda una `Response` real con
> `put()` y la relee con `match()` — en Chromium real, confirmado
> además con `caches.open()` directo del navegador (no solo el propio
> wrapper) que la entrada quedó en la Cache API real, con el nombre de
> caché correcto.
>
> 5 tests nuevos en `packages/platform` (36 total, todos en verde).

---

## Fase 40 — Wrapper mínimo sobre IndexedDB (Hito 12) — ✅ completada

**Objetivo:** issue #16 — sin ningún wrapper, cachear datos
estructurados grandes offline era 100% código del proyecto contra la
API nativa de IndexedDB (verbosa, basada en callbacks/eventos).

**Entregables:** `packages/platform/src/db.ts::openCollection(name,
backend?)`, expuesto como `platform.db("nombre")` — contrato
`get`/`set`/`delete`/`list` basado en promesas, sin ningún concepto de
ORM/schema/índices. Cada nombre de colección abre su **propia** base
IndexedDB (`nexa-db-<nombre>`) con un único object store fijo adentro
— evita el problema real de que IndexedDB exige declarar de antemano
todos los object stores de una base al crearla/subirle de versión, algo
que no tiene sentido pedirle a una función que se llama dinámicamente
con cualquier nombre. Tauri/Capacitor: sin rama nativa distinta, mismo
criterio que `platform.storage()` — el IndexedDB del propio webview
alcanza.

**Criterio de salida:** `platform.db("x").set/get/delete/list()`
funcionan contra IndexedDB real en un navegador real.

> **Decisión de testing, explícita:** en vez de una fake hecha a mano
> (arriesgado — la semántica async/transaccional de IndexedDB es
> notoriamente fácil de simular mal), se sumó `fake-indexeddb` como
> dependencia de desarrollo — implementa el mismo motor que un
> navegador real, no una aproximación. A propósito sin
> `fake-indexeddb/auto` (que inyectaría un `indexedDB` global): cada
> test pasa su propia instancia explícita, así el test de "IndexedDB no
> disponible" sigue siendo real (el entorno de test genuinamente no
> tiene ningún `indexedDB` global — mismo hallazgo que ya sirvió para
> verificar el caso "no disponible" de `platform.cache()`, Fase 39).
>
> **Verificado además con un navegador real (Playwright), no solo
> `fake-indexeddb`:** un handler que hace `platform.db("users").set()`
> y relee con `.get()`/`.list()` — en Chromium real, confirmado con
> `indexedDB.open()` directo del navegador (no el propio wrapper) que
> el dato quedó en la base IndexedDB real, con el nombre y el object
> store correctos.
>
> 7 tests nuevos en `packages/platform` (43 total, todos en verde).

## Fase 41 — `ui.openDrawer`/`ui.closeDrawer` (Hito 12) — ✅ completada

**Objetivo:** issue #9 — `@nexa/ui` ya tenía `Dialog` con comportamiento
real (Fase 9: foco atrapado, Escape, devolución de foco), pero cualquier
sidebar/drawer de un proyecto se escribía a mano contra `@nexa/reactivity`
sin nada de eso — inconsistente entre proyectos y sin accesibilidad real.

**Entregables:**

- `packages/ui/src/focus-trap.ts` (nuevo): la lógica de focus trap de
  `Dialog` (selector de elementos enfocables, atrapar `Tab`) extraída a
  un módulo compartido — un drawer es, en accesibilidad, el mismo
  contrato que un diálogo modal (`role="dialog"`, `aria-modal="true"`,
  foco atrapado, Escape, devolver el foco), solo cambia el CSS. `dialog.ts`
  se refactorizó para usarlo, sin cambiar su comportamiento observable
  (sus 6 tests existentes siguen pasando sin tocarlos).
- `packages/ui/src/drawer.ts`: `openDrawer(el)`/`closeDrawer(el)`, mismo
  contrato que `openDialog`/`closeDialog`.
- `packages/ui/src/drawer.css` (familia `nx-drawer`, registrada en
  `crates/nexa-ui/src/registry.rs` junto a `nx-btn`/`nx-input`/`nx-card`/
  `nx-dialog` — solo se envía si el proyecto usa la clase de verdad,
  Fase 25).
- `ui.openDrawer`/`ui.closeDrawer` expuestos en el objeto agrupado
  `packages/ui/src/index.ts` (mismo mecanismo de detección textual que ya
  usan `platform`/`stripe`/`ui.openDialog`).
- 8 tests nuevos en `packages/ui/test/drawer.test.ts` (22 tests totales
  en el paquete, todos en verde) + 2 tests nuevos en
  `crates/nexa-ui/src/tests.rs` (familia `nx-drawer` aislada del resto).
- Documentado en `docs/REFERENCIA.md`, sección `@nexa/ui`.

**Criterio de salida:** `ui.openDrawer(el)` atrapa el foco de verdad
(Tab real repetido no escapa del drawer), Escape cierra y devuelve el
foco, `role`/`aria-*` correctos — verificado con Playwright/Chromium
real, no solo unitario.

> **Navegación SPA (el punto explícito del issue):** el estado de un
> drawer vive en dos `WeakMap` clavadas por su propio nodo DOM, nunca en
> una variable global — el mismo diseño que ya tenía `Dialog` desde la
> Fase 9. Si el drawer está dentro del `[data-nexa-slot]` que
> `initRouter` reemplaza (Fase 32), el nodo viejo y sus entradas en los
> `WeakMap` se descartan juntos con la navegación, sin fuga ni listener
> huérfano. Si vive en `src/layout.tsx` (fuera del slot, el caso típico
> de un menú lateral persistente, Fase 31), sobrevive intacto entre
> páginas — comportamiento correcto para un drawer de navegación. Se
> sumó un test explícito (`closeDrawer` sobre un nodo ya desconectado
> del DOM) para dejar esto verificado, no solo argumentado.
>
> **Verificado con Chromium real (Playwright), no solo `vitest`:**
> `ui.openDrawer`/`ui.closeDrawer` invocados desde un handler real
> activado por Fase 5 (import dinámico del chunk al hacer click) contra
> un `<div class="nx-drawer">` real — ARIA correcto, foco inicial en el
> primer elemento enfocable, `Tab` repetido reconfirmado sin escapar del
> drawer, `Escape` cierra y devuelve el foco al botón que lo abrió.

## Fase 42 — Apilamiento de Dialog + `ui.confirm()`/`ui.alert()` (Hito 12) — ✅ completada

**Objetivo:** issue #10 — `Dialog` (Fase 9) no contemplaba abrir un
segundo diálogo sobre uno ya abierto (z-index/foco indefinidos), y
confirmar una acción destructiva requería cablear `openDialog`/
`closeDialog` a mano con callbacks en vez de `await ui.confirm(...)`.

**Entregables:**

- `packages/ui/src/dialog.ts`: `dialogStack` (array a nivel de módulo).
  `openDialog()` fija `dialog.style.zIndex` según la profundidad de la
  pila (independiente del orden de inserción en el DOM) y pone
  `aria-hidden="true"` en el diálogo justo debajo, mientras esté
  cubierto; `closeDialog()` deshace ambas cosas al desapilar. El focus
  trap en sí **ya estaba aislado por diálogo desde la Fase 9** (cada
  `trapFocus(dialog, ...)` solo mira dentro de su propio elemento, y un
  `keydown` dentro de B nunca burbujea a través de A porque no son
  ancestro/descendiente) — no hizo falta tocar esa parte.
  También se agregó `onDialogClose(dialog, cb)`, un punto de extensión
  interno (no exportado desde `index.ts`) para que `confirm()`/`alert()`
  se enteren de que un diálogo se cerró sin importar la vía (botón o
  Escape).
- `packages/ui/src/confirm-dialog.ts` (nuevo): `ui.confirm(mensaje,
  opciones?)` y `ui.alert(mensaje, opciones?)`, devuelven una `Promise`
  real. El HTML del diálogo se crea en el momento y se descarta al
  resolver — mismo principio que `ui.notify()` (Fase 28): el
  desarrollador no lo escribe a mano. `role="alertdialog"` (no
  `"dialog"`) para distinguirlos de un `Dialog` de contenido normal.
  Escape en `confirm()` cuenta como cancelar (resuelve `false`), igual
  que clickear "Cancelar".
- `ui.confirm`/`ui.alert` sumados al objeto agrupado `ui` en
  `packages/ui/src/index.ts`.
- 4 tests nuevos de apilamiento en `packages/ui/test/dialog.test.ts` +
  9 tests nuevos en `packages/ui/test/confirm-dialog.test.ts` (33 tests
  totales en el paquete, todos en verde).
- Documentado en `docs/REFERENCIA.md`.

**Criterio de salida:** abrir un segundo diálogo no rompe el focus trap
de ninguno de los dos; cerrar el de arriba devuelve el foco al de
abajo (no a `document.body`); `ui.confirm()`/`ui.alert()` resuelven de
verdad ante Aceptar/Cancelar/Escape — verificado con Playwright/
Chromium real.

> **Bug real encontrado en el propio test E2E, no en la implementación:**
> el primer intento de verificación asumía que cerrar cualquier diálogo
> lo saca del DOM, y esperaba `document.querySelectorAll(".nx-dialog").length
> === 0` tras cerrar A con Escape. Eso nunca se cumple para un diálogo
> que escribió el desarrollador (`closeDialog()` solo le pone `hidden`,
> nunca lo remueve) — a diferencia de `confirm()`/`alert()`, que sí se
> autoeliminan del DOM al resolver por diseño (no tiene sentido dejar
> húerfano un `<div>` que nadie más va a reabrir). El test se corrigió
> para comprobar `hidden`/`:not([hidden])` según corresponda; quedó
> como recordatorio de no asumir el mismo ciclo de vida de DOM para un
> elemento estático del desarrollador y uno efímero creado en runtime.
>
> **Verificado con Chromium real (Playwright):** abrir A, disparar
> `ui.confirm()` desde un botón dentro de A (queda apilado con
> `z-index` mayor y A con `aria-hidden`), `Tab` real repetido
> confirmado sin escapar del diálogo de confirm hacia A, click en
> "Aceptar" resuelve `true` y devuelve el foco al botón de A (con su
> `aria-hidden` restaurado), un segundo `confirm()` cancelado con
> `Escape` resuelve `false`, Escape sobre A lo oculta (sigue en el DOM),
> y `ui.alert()` independiente se abre, resuelve y se remueve del DOM
> al aceptar.

## Fase 43 — Tema light/dark de primera clase (Hito 12) — ✅ completada

**Objetivo:** issue #11 — sin andamiaje de tema, cualquier proyecto que
quería un switch light/dark tenía que duplicar a mano cada color de
`@nexa/ui` en su variante oscura, y armar su propio `data-theme` +
persistencia.

**Entregables:**

- `packages/ui/src/tokens.css`: variante oscura de todos los tokens de
  color, en dos capas — `@media (prefers-color-scheme: dark)` (sin JS,
  respeta el sistema operativo, salvo override manual a `light`) y
  `:root[data-theme="dark"]` (con JS, gana siempre). Se comprobó
  leyendo cada `.css` de componente que `Button`/`Input`/`Card`/
  `Dialog`/`Drawer` ya usan 100% tokens para sus colores — cero cambios
  en esos archivos, la variante oscura de los tokens les alcanza sola.
- `packages/platform/src/theme.ts` (nuevo): `platform.theme.get()`/
  `platform.theme.set("dark"|"light"|"system")`, persistido con
  `platform.storage()` (Fase 38, reutilizado tal cual, sin cambios) y
  aplicado como `document.documentElement.dataset.theme`.
- Expuesto en `packages/platform/src/index.ts` (`platform.theme`).
- 8 tests nuevos en `packages/platform/test/theme.test.ts` (51 totales
  en el paquete) + 2 tests nuevos en `crates/nexa-ui/src/tests.rs`
  verificando que `full_source()`/`build_stylesheet()` incluyen la
  variante oscura.
- Documentado en `docs/REFERENCIA.md`.

**Criterio de salida:** con `[data-theme="dark"]` en `<html>`, los 4
componentes existentes se ven correctos en oscuro sin CSS propio del
proyecto; `platform.theme.set("dark")` persiste entre recargas reales;
sin JS, el tema por defecto respeta `prefers-color-scheme` — todo
verificado con Playwright/Chromium real (contextos con
`colorScheme`/`javaScriptEnabled` emulados, no solo aserciones sobre el
CSS fuente).

> **Cómo persiste sin tocar el compilador (ajuste de diseño explícito):**
> `theme.ts` reaplica la preferencia guardada como efecto de nivel de
> módulo, apenas el chunk de `@nexa/platform` se evalúa — no hace falta
> que el desarrollador llame nada al cargar la página. Para que ese
> chunk se importe en cada carga (no solo al hacer click, que es el
> default de activación por interacción, Fase 5) el toggle de tema debe
> usar `data-nexa-strategy="load"`, mecanismo que ya existe desde la
> Fase 5 — se documentó el patrón en `docs/REFERENCIA.md` en vez de
> tocar `bootstrap.rs`/`pipeline.rs` para inyectar un script bloqueante
> de pre-paint (la solución "clásica" contra el flash de tema, pero
> fuera del tamaño S–M que pedía el propio issue). Costo aceptado:
> puede haber un breve flash del tema por defecto antes de que el chunk
> cargue en la recarga — no cero-flash, pero sí "el tema elegido se ve
> sin que el proyecto escriba nada".
>
> **Verificado con Chromium real (Playwright), variando el tema real
> del sistema operativo emulado (`colorScheme` del contexto), no solo
> forzando el atributo a mano:** sin JS (`javaScriptEnabled: false`)
> con el SO en dark, `--nx-color-primary` y el `background-color`
> computado del botón primario son los tonos oscuros; con el SO en
> light, los claros. Con JS: `platform.theme.set("dark")` aplica el
> token oscuro de inmediato, sobrevive a un `page.reload()` real,
> cambiar a `"light"` le gana al `prefers-color-scheme` del sistema, y
> `"system"` quita el atributo y vuelve a ceder el control al sistema.

## Fase 44 — Utilidades de layout flex/grid/stack (Hito 12) — ✅ completada

**Objetivo:** issue #12 — `@nexa/ui` no tenía ninguna utilidad de
layout; todo proyecto real (incluido `tutorial-nexa`, ver
`public/static/site.css`) terminaba escribiendo su propio CSS de flex/
grid/spacing desde cero, duplicando el mismo trabajo en cada proyecto.

**Entregables:**

- 4 archivos CSS nuevos en `packages/ui/src/` (`flex.css`, `gap.css`,
  `grid.css`, `stack.css`), cada uno su propia familia en
  `crates/nexa-ui/src/registry.rs` — mismo tree-shaking granular que
  ya tienen `Button`/`Card`/`Dialog`/`Drawer`: `.nx-gap-*` es una
  familia separada de `.nx-flex`/`.nx-grid` a propósito (`gap` es
  válido en ambos, así no se duplica).
  - `.nx-flex` / `.nx-flex-col` / `.nx-flex-wrap`.
  - `.nx-gap-1`..`.nx-gap-4` (mapeados a los tokens `--nx-space-*`
    existentes).
  - `.nx-grid` + `.nx-grid-cols-2/3/4`, con colapso responsivo a 1
    columna bajo 640px.
  - `.nx-stack` (+ `.nx-stack-1/2/3`): espaciado vertical consistente
    entre hijos directos, vía `> * + *` (sin margin duplicado contra
    el borde del contenedor).
- 4 tests nuevos en `crates/nexa-ui/src/tests.rs` (18 totales en el
  crate) verificando que cada familia nueva se incluye/excluye de forma
  independiente, igual que el resto.
- Uso real en `examples/oweeme-shop/src/pages/[locale]/index.tsx`:
  `nx-stack` en el contenedor de la página, `nx-flex nx-gap-3`/`nx-gap-2`
  en la navegación y la fila de botones, `nx-grid nx-grid-cols-2` en la
  grilla de productos del catálogo.
- Documentado en `docs/REFERENCIA.md`, con la tabla completa de clases.

**Criterio de salida:** las nuevas clases se tree-shakean igual que
`nx-btn`/`nx-card` — verificado tanto en Rust (`build_stylesheet`) como
con un `nexa build` real; documentadas con tabla completa; uso real en
`examples/`.

> **Bug real encontrado durante la propia verificación E2E — binario
> desactualizado, no la implementación:** el primer `nexa build` de
> prueba no incluyó ninguna clase de layout en el CSS generado (solo
> `.nx-card`), a pesar de que `cargo test` ya pasaba en verde. Causa:
> se había corrido `cargo build --release` pero no se había reinstalado
> el binario en `~/.local/bin/nexa` con la nueva versión — el CLI
> instalado todavía era el de la Fase 43, sin las 4 familias nuevas en
> el registro. Reinstalar el binario lo arregló al instante. Queda
> como recordatorio explícito de por qué la disciplina de este proyecto
> exige reinstalar el CLI antes de cada verificación E2E, no solo
> compilar y confiar en los tests unitarios.
>
> **Verificado con Chromium real (Playwright), no solo el CSS fuente:**
> con viewport ancho, `nav` es `display: flex` de verdad con
> `column-gap: 12px` (`.nx-gap-3` = `--nx-space-3`), el grid tiene 3
> columnas computadas, y el segundo hijo de `.nx-stack` tiene
> `margin-top: 16px` computado (`--nx-space-4`). Con viewport angosto
> (375px), el mismo grid de 3 columnas colapsa a 1 sola — el
> `@media (max-width: 640px)` funciona de verdad, no solo está escrito.

## Fase 45 — Interpolación en `t("clave", { name })` (Hito 12) — ✅ completada

**Objetivo:** issue #14 — `t(...)` solo reconocía un identificador con
un único argumento string literal; cualquier texto con una variable
(`"Apoyar a {name}"`, `"{count} artículos"`) había que partirlo a mano
en piezas separadas en cada idioma.

**Entregables:**

- `nexa-ast`: nuevo `struct Translate { key: String, args: Vec<(String, Expr)> }`
  (`crates/nexa-ast/src/translate.rs`), reemplaza el `String` suelto que
  usaban `Node::Translate`, `TemplatePart::Translate` y
  `JsonTemplate::Translate` — aditivo en la práctica (`args` vacío es
  exactamente `t("clave")` de siempre).
- `nexa-parser/src/translate.rs`: reconoce un segundo argumento objeto
  (`{ name: data.x }`) cuyos valores son la misma `Expr` limitada que ya
  acepta cualquier otra posición dinámica de Nexa — un literal, una
  llamada, o cualquier otra cosa en esa posición hace que el `t(...)`
  entero no se reconozca (mismo criterio "todo o nada" que el resto del
  parser). 6 tests nuevos, incluyendo que `seo.title = t("...", {...})`
  también funciona (comparten `from_call`, ver el comentario del
  archivo).
- `nexa-renderer`/`nexa-seo`: `resolve_translation` ahora sustituye cada
  `{nombre}` del texto resuelto por el valor de su `Expr` — si un
  argumento no se puede resolver, el `{placeholder}` literal queda tal
  cual (nunca inventa un valor, mismo criterio que el resto del
  renderer). 5 tests nuevos en `nexa-renderer` (42 totales).
- `nexa-cli::pipeline::validate_translate_calls` (nuevo): antes de
  renderizar, camina el IR (`IrNode::walk`, ya existente) buscando
  nodos `Translate`, compara los `{placeholder}` del diccionario contra
  los argumentos dados, y si no coinciden hace fallar `compile_page`
  con `PageError::Other` — el mismo canal de error que ya usa esa
  función para `LoaderError`/errores de layout, así que tanto `nexa
  build` como `nexa lint`/`nexa preview` lo heredan gratis. 9 tests
  nuevos.
- Documentado en `docs/REFERENCIA.md`, sección de i18n.

**Criterio de salida:** `t("clave", { name: data.x })` compila a HTML
real con el valor sustituido; `nexa build` falla con un error claro
ante un `{placeholder}` sin variable correspondiente (o al revés);
`t("clave")` sin segundo argumento sigue funcionando igual que antes —
verificado con `nexa build`/`nexa lint` reales, no solo `cargo test`.

> **Ajuste de alcance, explícito:** la validación de build solo cubre
> `t(...)` en el cuerpo JSX (`IrNodeKind::Translate`, alcanzable con
> `IrNode::walk`) — un `t(...)` con interpolación dentro de `seo`/
> `schema`/`head` **sí** interpola correctamente en tiempo de render
> (comparte el mismo `resolve_translation`/`resolve_translation_json`),
> pero su validación de build queda pendiente. Se documentó así en vez
> de ampliar el walk a `SeoConfig`/`JsonTemplate` (que hubiera hecho
> falta para cubrir esos tres casos), para mantener el tamaño M que
> pedía el propio issue.
>
> **Por qué un placeholder sin argumento correspondiente rompe el build
> en vez de degradar con gracia** (a diferencia del resto del
> renderer, que siempre produce algo): un dato faltante (`data.name`
> ausente) es normal — el JSON del `load()` puede legítimamente no
> traer todo. Un `{placeholder}` sin argumento es distinto: es una
> combinación código+diccionario que nunca puede tener sentido en
> ningún request — o el desarrollador se olvidó de pasar la variable, o
> el texto cambió y el código no. Dejarlo pasar solo pospone el error
> hasta que alguien lo vea en producción como un `{name}` literal.
>
> **Verificado con `nexa build`/`nexa lint` reales** (proyecto scratch,
> `src/locales/es.json` con `"greeting": "Hola desde {locale}"` y
> `"broken": "Hola {name}"`): `t("greeting", { locale: params.locale })`
> compiló a `<p>Hola desde es</p>` real en `dist/index.html`;
> `t("broken")` (sin el argumento `name`) hizo fallar tanto `nexa lint`
> como `nexa build` con exit code 1 y el mensaje `t("broken", ...) usa
> "{name}" pero no se pasó esa variable — la clave dice: "Hola
> {name}"`; un argumento de sobra (`t("home.title", { unused: ... })`
> contra una clave sin ningún `{unused}`) también falló con un mensaje
> igual de claro.

## Fase 46 — Guía de vendoring de paquetes npm (Hito 12) — ✅ completada

**Objetivo:** issue #21 — `[imports]` solo declara un nombre y una URL;
conseguir que el archivo exista era "responsabilidad del proyecto" sin
ninguna guía, y el único ejemplo real era Stripe (que además necesita
un envoltorio propio, no el caso más simple).

**Entregables:**

- `docs/VENDORING.md` (nuevo): el proceso paso a paso (instalar el
  paquete en cualquier lado con npm, un `entry.js` que re-exporte lo
  necesario, `esbuild --bundle --format=esm`, copiar a `public/vendor/`,
  declarar en `[imports]`, usar desde un handler) — con dos casos
  documentados: vendoring directo (sin wrapper) y con un envoltorio
  propio (`packages/stripe`, ya existente, citado como ejemplo real).
- Enlazado desde `docs/REFERENCIA.md`, sección `[imports]`.
- Segundo ejemplo real, `marked` (npm real, sin wrapper), vendorizado
  siguiendo la guía: `examples/oweeme-shop/public/vendor/marked.js`
  (bundle real de esbuild, 55kb), declarado en `nexa.toml [imports]`, y
  usado en una página nueva,
  `src/pages/[locale]/articles.tsx` — una vista previa en vivo de
  Markdown mientras se escribe un artículo (el caso concreto que cita
  el propio issue: "`marked`, ya lo usás para artículos").

**Criterio de salida:** guía documentada paso a paso; segundo ejemplo
real (además de Stripe) vendorizado y probado de punta a punta.

> **Por qué la vista previa de Markdown corre 100% en el cliente:**
> Nexa nunca ejecuta JavaScript en build time (es un compilador/
> analizador estático en Rust) — `marked.parse(...)` no puede formar
> parte del HTML servidor-renderizado de un artículo ya publicado (ese
> seguiría siendo texto resuelto por `load()`, un campo más de
> `data.*`). Lo que sí es un caso de uso real y honesto es la
> herramienta de *autoría*: alguien escribiendo Markdown y viendo el
> resultado antes de guardarlo — que es justo lo que demuestra el
> ejemplo.
>
> **Verificado de punta a punta con un navegador real (Playwright):**
> el `<textarea>` trae su valor por defecto como HTML real generado por
> el servidor (sin JS); escribir Markdown ahí (`# Hola`, `*artículo*`,
> `**marked**`) actualiza la vista previa con el HTML real que produce
> `marked.parse(...)` (`<h1>Hola</h1>`, `<em>artículo</em>`,
> `<strong>marked</strong>` — no simulado ni mockeado).
>
> **Bug real encontrado en el propio proceso de verificación:** el
> primer intento contra `nexa preview` dio 404 en `/vendor/marked.js`
> — `public/vendor/marked.js` existía, pero `nexa preview` sirve desde
> `dist/`, y `dist/vendor/` recién se genera con un `nexa build` previo
> (documentado ya en el README del ejemplo para los otros vendors, pero
> lo pasé por alto al agregar uno nuevo). Un `nexa build` antes de
> `nexa preview` lo resolvió — no es un bug de Nexa, es exactamente el
> comportamiento que el propio README de `examples/oweeme-shop` ya
> documentaba para `nexa-stripe.js`.

## Fase 47 — [Diseño] Cookies en `load()` (Hito 12) — ✅ completada (solo diseño, sin código)

**Objetivo:** issue #18 — decidir, antes de escribir una sola línea, si
`load()` puede leer cookies de sesión server-side, y cómo eso convive
con la premisa central de Nexa ("todo es un patrón sintáctico literal,
nunca código arbitrario"). El propio issue pide explícitamente un
documento de diseño primero, no una implementación directa.

**Entregable:** `docs/DISENO-COOKIES-EN-LOAD.md` — decisión completa:
`cookies.*` con allowlist explícita en `nexa.toml [cookies]`, resoluble
solo dentro de `load.headers` (nunca en JSX/`seo`/`schema`, para no
poder filtrar un token de sesión al HTML), sin redirects ni control de
flujo condicional dentro de `load`, y un bosquejo concreto de
implementación para cuando/si se decida seguir adelante.

> **El hallazgo que no estaba en el issue original:** una cookie es
> información de una petición concreta, y `nexa build` no tiene ninguna
> petición — un `load()` que dependa de `cookies.*` no puede
> pre-renderizarse como HTML estático nunca. Es, por diseño, una
> capacidad exclusiva de rutas dinámicas servidas en el momento (mismo
> bucket que las rutas sin `paths` de la Fase 34), nunca del build
> estático. Esta restricción condiciona toda la decisión y no era obvia
> antes de escribir el documento.
>
> Sin código de esta fase — el issue de implementación queda como
> siguiente paso explícito, a abrir cuando el usuario confirme la
> dirección.

## Fase 48 — `platform.network` (Hito 12) — ✅ completada

**Objetivo:** issue #20 — primer plugin de la lista incremental
sugerida ("estado de red → ciclo de vida → geolocalización → deep
links → push real → biometría → haptics"). `@nexa/platform` no tenía
ninguna forma de saber si el dispositivo está online, ni de
reaccionar a un cambio de conectividad.

**Entregables:**

- `packages/platform/src/network.ts` (nuevo): `getNetworkStatus()`
  (`{ online, type }`) y `onNetworkChange(callback)` (devuelve una
  función para cancelar la suscripción — mismo contrato que
  `connectSSE`/`connectSocket` de `@nexa/http`, Fase 26). Capacitor
  nativo: el plugin real `@capacitor/network` (`getStatus`/
  `addListener("networkStatusChange", ...)`/`PluginListenerHandle.remove()`
  — forma verificada instalando el paquete real y leyendo sus
  `.d.ts`, no inventada). Web/Tauri: `navigator.onLine` + eventos
  `online`/`offline` estándar; `type` siempre `"unknown"` en esa rama
  (`navigator.connection` no es un estándar estable, a diferencia de
  `onLine`).
- Expuesto como `platform.network.getStatus`/`platform.network.onChange`
  en `packages/platform/src/index.ts`.
- 11 tests nuevos en `packages/platform/test/network.test.ts` (61
  totales en el paquete).
- Documentado en `docs/REFERENCIA.md`, junto al resto de
  `@nexa/platform`.

**Criterio de salida:** las dos ramas (Web + Capacitor) sin inventar la
forma del plugin nativo; tests; documentado — mismos tres criterios
que pide el issue para cada plugin agregado.

> **Verificado con conectividad real en Chromium (Playwright), no
> simulada:** `context.setOffline(true)`/`setOffline(false)` de
> Playwright dispara los eventos `offline`/`online` reales del
> navegador — `platform.network.getStatus()` y una suscripción activa
> de `platform.network.onChange()` reflejaron el cambio real en ambos
> sentidos, y cancelar la suscripción de verdad dejó de recibir
> actualizaciones ante un cambio de conectividad posterior.
>
> **Bug real encontrado en el propio test, no en la implementación:**
> el primer intento de E2E guardaba la función de cancelación
> (`unsubscribe`) en una variable `let` de nivel de módulo compartida
> entre los tres handlers de la página (`checkStatus`/`watchStatus`/
> `stopWatching`) — pero Nexa empaqueta cada handler interactivo en su
> **propio chunk** (Fase 5), así que esa variable nunca era
> compartida de verdad entre ellos (cada chunk tiene su propia copia).
> Guardar la referencia en `window` (el único ámbito realmente
> compartido entre chunks separados) lo arregló. Queda como
> recordatorio para cualquier ejemplo futuro que necesite estado
> compartido entre más de un handler de la misma página.
>
> **Quedan pendientes, mismo issue, a resolver de a uno:** ciclo de
> vida (resume/pause), geolocalización, deep links, push real,
> biometría, haptics — orden sugerido por el propio issue, ajustable
> según necesidad real.

---

## Regla de disciplina para todas las fases

> No empezar a diseñar la fase N+2 mientras la fase N no tenga un criterio
> de salida cumplido y verificado. El documento original cayó en esto
> (diseñó i18n, PWA, registry de paquetes y DevTools antes de tener un
> compilador funcionando); esta lista existe para no repetirlo.
