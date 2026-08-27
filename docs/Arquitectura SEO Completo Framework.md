Sí. **Se puede crear**, y la idea que estás planteando es técnicamente viable. Pero yo no lo llamaría simplemente "otro framework SSR". Lo que estás describiendo es una arquitectura distinta: **un frontend que genera HTML SEO completo y que después se vuelve interactivo, pero cuyo backend puede ser PHP, Go, Python, Rust, Java, etc.**

La clave es separar **renderizado**, **backend** e **hidratación**.

### La arquitectura que propondría

```text
                    BACKEND
          PHP / Go / Python / Rust
                    │
                    │ JSON
                    ▼
        ┌──────────────────────────┐
        │      FRONTEND ENGINE     │
        │                          │
        │  TypeScript              │
        │  Templates / Components  │
        │  Router                  │
        │  State                   │
        │  SEO                     │
        └────────────┬─────────────┘
                     │
                     │ render()
                     ▼
              HTML COMPLETO
                     │
                     ▼
                NAVEGADOR
                     │
                     ▼
             "hidratar" solamente
             lo que necesita JS
```

Y aquí está la diferencia fundamental:

**El backend nunca tiene que conocer Vue, React, Astro, etc.**

Puede devolver:

```json
{
  "product": {
    "id": 123,
    "slug": "iphone-17",
    "name": "iPhone 17",
    "description": "....",
    "price": 599
  }
}
```

El frontend transforma eso en:

```html
<html>
<head>
  <title>iPhone 17 | Mi tienda</title>
  <meta name="description" content="...">
  <link rel="canonical" href="...">
</head>

<body>
  <main>
    <article>
      <h1>iPhone 17</h1>

      <p>...</p>

      <span>$599</span>

      <button>Comprar</button>
    </article>
  </main>
</body>
</html>
```

Google recibe **ese HTML**, no un `<div id="app"></div>` esperando JavaScript.

------

# Pero hay una cuestión importante

Dices:

> "que funcione en el lado del frontend"

Aquí hay que hacer una pequeña corrección conceptual.

Si quieres que el usuario solicite:

```text
/producto/iphone-17
```

y que **el primer HTML que recibe ya contenga `<h1>`, `<p>`, imágenes, precios, etc.**, entonces ese HTML tiene que generarse **antes de llegar al navegador**.

Eso significa que necesitas algún proceso de renderizado.

Pero **no necesariamente un servidor Node**.

Podrías tener:

```text
PHP backend
      │
      │ JSON
      ▼
Frontend Renderer
      │
      ▼
HTML
```

El renderer podría ejecutarse:

- durante build
- en un edge runtime
- en un worker
- en Node
- en Bun
- en Deno
- en Rust
- en WASM
- incluso dentro de PHP mediante un motor adecuado

La cuestión importante es:

> **el backend de negocio no tiene por qué ser el renderer.**

------

# Y aquí es donde tu framework podría ser interesante

Yo lo diseñaría con **dos motores completamente separados**.

## 1. Compiler

Escrito en Rust, por ejemplo:

```text
myframework/
│
├── compiler/
│   └── Rust
│
├── runtime/
│   └── TypeScript
│
├── router/
│   └── TypeScript
│
├── renderer/
│   └── Rust/WASM
│
└── cli/
    └── Rust
```

El usuario escribe:

```vue
<script setup lang="ts">
const props = defineProps<{
  product: Product
}>()

async function buy() {
  await api.post('/cart', {
    product_id: props.product.id
  })
}
</script>

<template>
  <article>
    <h1>{{ product.name }}</h1>

    <p>{{ product.description }}</p>

    <strong>{{ product.price }}</strong>

    <button @click="buy">
      Comprar
    </button>
  </article>
</template>
```

Y el compilador produce algo parecido a:

```text
Component
     │
     ├── static HTML
     ├── dynamic HTML
     ├── events
     ├── state
     └── dependencies
```

------

# 2. Renderer

El renderer genera HTML.

Por ejemplo:

```ts
const html = render(ProductPage, {
  product
})
```

Resultado:

```html
<article>
  <h1>iPhone 17</h1>
  <p>...</p>
  <strong>$599</strong>
  <button>Comprar</button>
</article>
```

Pero hay algo mucho más interesante que podrías agregar:

## El renderer genera un "hydration manifest"

Por ejemplo:

```json
{
  "components": {
    "buy-button": {
      "id": "c1",
      "events": ["click"],
      "module": "/assets/product.js"
    }
  }
}
```

Entonces el navegador recibe:

```html
<h1>iPhone 17</h1>

<p>Descripción...</p>

<button data-hydrate="c1">
  Comprar
</button>
```

Y **no necesitas hidratar todo el árbol DOM**.

------

# Esto es fundamental

Tu framework podría diferenciar:

### Static

```html
<h1>Producto</h1>
<p>Descripción</p>
<img>
```

No necesita JavaScript.

### Interactive

```html
<button>
```

Necesita JS.

### Reactive

```html
<CartCounter>
```

Necesita runtime.

### Async

```html
<ProductReviews>
```

Puede cargar después.

Entonces tendrías:

```text
                    PAGE
                     │
        ┌────────────┼─────────────┐
        │            │             │
        ▼            ▼             ▼
     STATIC      INTERACTIVE     ASYNC
        │            │             │
       HTML          JS          lazy JS
```

Esto es mucho más fino que:

```text
hydrate entire application
```

------

# Y sí puedes tener Axios

Incluso puedes crear tu propio cliente:

```ts
import { api } from '@framework/http'

const product = await api.get('/products/123')
```

Internamente podría ser:

```text
api.get()
    │
    ├── fetch()
    ├── cache
    ├── interceptors
    ├── retry
    ├── serialization
    └── state integration
```

Y no necesitas Axios obligatoriamente.

Podrías hacer:

```ts
const api = createApi({
  baseURL: 'https://api.example.com'
})
```

y usar:

```ts
const product = await api.get<Product>(
  `/products/${id}`
)
```

------

# El router sería una pieza fundamental

Podrías definir:

```text
pages/
├── index.page.ts
├── products/
│   ├── index.page.ts
│   └── [slug].page.ts
├── category/
│   └── [slug].page.ts
└── blog/
    └── [slug].page.ts
```

El compilador genera:

```text
/
/products
/products/:slug
/category/:slug
/blog/:slug
```

Pero el router debería tener **dos comportamientos**.

### Primera visita

```text
GET /products/iphone-17
        ↓
HTML completo
        ↓
Browser
```

### Navegación interna

```text
/products
      ↓
click
      ↓
router
      ↓
fetch JSON
      ↓
actualiza DOM
```

Entonces tienes:

**SEO de HTML + navegación tipo SPA.**

Esto es exactamente lo que quieres.

------

# Y aquí aparece tu "hidratación"

Yo incluso cambiaría el nombre.

Porque la hidratación tradicional significa:

```text
HTML generado
     ↓
descargar aplicación
     ↓
recrear árbol virtual
     ↓
comparar
     ↓
conectar eventos
```

Tu framework podría hacer:

# Progressive Activation

Primero:

```html
<h1>Producto</h1>
<p>Descripción</p>
```

Luego:

```html
<button data-fw="buy">
```

El runtime encuentra:

```text
data-fw="buy"
```

y solamente carga:

```text
buy-button.js
```

Entonces:

```text
HTML
 │
 ├── static → nada
 │
 ├── interactive → activar
 │
 └── async → cargar cuando corresponda
```

Esto puede reducir muchísimo el JavaScript inicial.

------

# El SEO podría ser una característica de primera clase

Y aquí creo que tienes una oportunidad importante.

No limitarte a:

```ts
<title>
```

sino generar automáticamente:

```html
<title>
<meta name="description">
<meta name="robots">
<link rel="canonical">

<meta property="og:title">
<meta property="og:description">
<meta property="og:image">

<meta name="twitter:card">

<script type="application/ld+json">
```

Y además:

```text
<h1>
<h2>
<p>
<article>
<section>
<nav>
<main>
<footer>
<img alt="">
<a href="">
```

El framework podría tener:

```ts
defineSEO({
  title: product.name,
  description: product.description,
  canonical: `/producto/${product.slug}`,
  image: product.image
})
```

y producir todo automáticamente.

------

# Incluso Schema.org

Para ecommerce:

```ts
defineSchema({
  type: 'Product',

  name: product.name,

  offers: {
    price: product.price,
    currency: 'BOB'
  }
})
```

Renderer:

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "Product",
  ...
}
</script>
```

Para noticias:

```ts
defineSchema({
  type: 'NewsArticle'
})
```

Para empresas:

```ts
defineSchema({
  type: 'Organization'
})
```

Para hoteles:

```ts
defineSchema({
  type: 'Hotel'
})
```

Eso convierte SEO en parte de la arquitectura y no en un parche.

------

# Ahora viene la parte más interesante: Rust

Sí, **Rust sería una excelente opción para el compilador**.

No necesariamente haría todo en Rust.

Haría:

```text
                 Rust
                  │
        ┌─────────┼──────────┐
        │         │          │
     compiler   parser    bundler
        │         │          │
        └─────────┼──────────┘
                  │
                 WASM
                  │
                  ▼
              Browser
                  │
                  ▼
             TypeScript
               runtime
```

Rust podría encargarse de:

- parser
- compiler
- AST
- template compiler
- dependency graph
- bundling
- minification
- tree shaking
- code splitting
- manifest
- generación de rutas
- generación de metadata
- generación de HTML estático

Mientras TypeScript sería el lenguaje del desarrollador:

```ts
<script setup lang="ts">
```

------

# Y podrías llegar incluso más lejos

El framework podría funcionar con:

```text
myframework build
```

y generar:

```text
dist/
├── index.html
├── products/
│   ├── iphone.html
│   └── samsung.html
│
├── assets/
│   ├── app.js
│   ├── product.js
│   └── cart.js
│
└── manifest.json
```

Eso sería SSG.

Pero también:

```text
myframework serve
```

para SSR.

Y aquí puedes definir un **adapter**.

```text
Framework
    │
    ├── adapter-node
    ├── adapter-deno
    ├── adapter-bun
    ├── adapter-cloudflare
    ├── adapter-rust
    ├── adapter-php
    └── adapter-custom
```

Pero el backend de datos sigue siendo independiente:

```text
PHP API
Go API
Python API
Rust API
Java API
```

------

# Tu arquitectura podría ser esta

```text
                         YOUR FRAMEWORK
                              │
              ┌───────────────┼────────────────┐
              │               │                │
           Compiler          Router           SEO
              │               │                │
              ├───────────────┼────────────────┤
              │                                │
           Renderer                    Progressive Runtime
              │                                │
              ▼                                ▼
             HTML                         JavaScript
              │                                │
              └───────────────┬────────────────┘
                              │
                           Browser
                              │
                  ┌───────────┴───────────┐
                  │                       │
              SEO crawler             Usuario
                  │                       │
             HTML completo          App interactiva
```

Y externamente:

```text
              YOUR FRAMEWORK
                    │
                    │ JSON / GraphQL / RPC
                    ▼
          ┌─────────────────────┐
          │      BACKEND        │
          ├─────────────────────┤
          │ PHP                 │
          │ Go                  │
          │ Python              │
          │ Rust                │
          │ Java                │
          │ etc.                │
          └─────────────────────┘
```

**Sin Node obligatorio en producción.**

------

# ¿Es posible realmente?

**Sí.**

Pero hay que establecer una distinción muy importante:

### Imposible

Que el navegador reciba:

```text
JSON
```

y que **antes de ejecutar JavaScript** ese mismo navegador mágicamente tenga:

```html
<h1>
<p>
<article>
```

Eso no puede ocurrir.

### Totalmente posible

Que tu framework tenga:

```text
API JSON
     ↓
Framework Renderer
     ↓
HTML completo
     ↓
Browser
     ↓
Progressive Activation
```

y que el renderer sea independiente de tu backend.

------

# Yo lo plantearía como un proyecto serio

Y además encaja bastante bien con lo que vienes buscando con Rust.

Un posible nombre conceptual:

**Hydra** — *HTML-first Universal Frontend Framework*

o algo más descriptivo:

**Nexa**

```text
Nexa
├── Nexa Compiler      Rust
├── Nexa CLI            Rust
├── Nexa Router         TS
├── Nexa Runtime        TS
├── Nexa Renderer       Rust/WASM
├── Nexa SEO            TS/Rust
├── Nexa HTTP           TS
├── Nexa State          TS
└── Nexa Adapters       *
```

La filosofía sería:

> **HTML first. JavaScript when needed. Backend agnostic.**

Y tendría como principios:

```text
1. HTML completo desde el primer response
2. SEO semántico por defecto
3. Backend completamente independiente
4. JSON como fuente de datos
5. SPA navigation opcional
6. Progressive activation
7. Islands opcionales
8. TypeScript para el desarrollador
9. Rust para compiler/toolchain
10. Zero Node.js requirement in production
11. SSG + SSR + SPA + hybrid
12. Code splitting automático
```

**Y esto sí me parece un proyecto con suficiente sustancia para construirlo**, no simplemente otro clon de Vue. La parte difícil —y la que habría que diseñar muy bien— es el **modelo de componentes + compiler + renderer + mecanismo de activación**, porque ahí es donde realmente se decidiría si consigue una hidratación mucho más eficiente que Vue/React tradicional.

Si lo desarrollamos, yo empezaría **antes del CLI y antes del router**, definiendo primero el **modelo de renderizado y el protocolo HTML ↔ runtime ↔ JSON**. Esa decisión va a determinar prácticamente todo el framework.

----------------

Sí. En ese caso conviene **olvidarnos inicialmente de Vue, React, Angular, Astro, etc.** y diseñar el framework desde sus propios principios.

Yo empezaría por definir el **modelo de ejecución**, porque de ahí salen el compilador, los componentes, el router, el SEO, el runtime y finalmente la API de TypeScript.

# 1. Concepto fundamental del framework

La idea central sería:

> **El HTML es el producto principal. JavaScript es una capa de comportamiento que se activa solamente cuando es necesaria.**

No queremos:

```text
JavaScript
   ↓
crear aplicación
   ↓
crear DOM
   ↓
hidratar
   ↓
mostrar contenido
```

Queremos:

```text
Datos
  ↓
Renderer
  ↓
HTML completo
  ↓
Browser
  ↓
activar comportamiento
```

Por tanto, nuestro framework sería **HTML-first**.

------

# 2. Las cuatro capas

Propongo dividirlo inicialmente en cuatro sistemas:

```text
┌──────────────────────────────────────────────┐
│                 APPLICATION                  │
│                                              │
│  TypeScript                                  │
│  Components                                  │
│  Pages                                       │
│  Routes                                      │
│  State                                       │
│  API                                         │
└──────────────────────┬───────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────┐
│                   COMPILER                   │
│                                              │
│ TypeScript + Templates → AST → IR            │
└──────────────────────┬───────────────────────┘
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
┌──────────────────────┐ ┌─────────────────────┐
│       RENDERER       │ │       RUNTIME       │
│                      │ │                     │
│ JSON → HTML          │ │ Events              │
│ SEO                  │ │ State               │
│ Metadata             │ │ Navigation          │
│ Structured Data      │ │ Effects              │
└──────────┬───────────┘ └──────────┬──────────┘
           │                        │
           ▼                        ▼
        HTML                  Browser JS
```

Y el **Compiler** probablemente lo escribiría en Rust.

El desarrollador, sin embargo, trabaja exclusivamente con TypeScript.

------

# 3. No usaría `.vue`

No queremos heredar el modelo mental de Vue.

Podríamos tener:

```text
pages/
components/
layouts/
stores/
services/
```

y componentes `.tsx` o un formato propio.

Pero personalmente empezaría con **TypeScript + JSX propio** porque nos permite avanzar mucho más rápido con el compilador.

Por ejemplo:

```tsx
import { component } from "nexa";

export const Product = component((props) => {
    return (
        <article>
            <h1>{props.product.name}</h1>

            <p>{props.product.description}</p>

            <strong>
                {props.product.price}
            </strong>

            <button onClick={props.onBuy}>
                Comprar
            </button>
        </article>
    );
});
```

Pero hay una diferencia fundamental:

**No sería React.**

El JSX solamente sería la sintaxis.

El compilador produciría nuestro propio IR.

------

# 4. Nuestro concepto más importante: Render Tree

No quiero que nuestro framework dependa de Virtual DOM.

En vez de:

```text
Component
   ↓
Virtual DOM
   ↓
Diff
   ↓
DOM
```

tendríamos:

```text
Component
   ↓
Render Tree
   ↓
HTML
```

Y el compilador sabría qué partes son:

```text
STATIC
DYNAMIC
INTERACTIVE
ASYNC
```

Por ejemplo:

```tsx
<article>

    <h1>{product.name}</h1>

    <p>{product.description}</p>

    <img src={product.image} />

    <button onClick={buy}>
        Comprar
    </button>

</article>
```

El compilador podría producir:

```text
Article
│
├── H1
│   └── DYNAMIC: product.name
│
├── P
│   └── DYNAMIC: product.description
│
├── IMG
│   └── DYNAMIC: product.image
│
└── BUTTON
    ├── STATIC: "Comprar"
    └── INTERACTIVE: buy()
```

Esto es muchísimo más importante que la sintaxis.

------

# 5. Tres estados de un nodo

Cada elemento del árbol debería clasificarse.

## Static

```html
<h1>Mi tienda</h1>
```

No necesita runtime.

------

## Dynamic

```tsx
<h1>{product.name}</h1>
```

Se genera en el renderer.

Pero **no necesariamente necesita JavaScript en el navegador**.

Esto es importante.

Si:

```text
product.name = "iPhone 17"
```

el servidor/renderizador produce:

```html
<h1>iPhone 17</h1>
```

y termina ahí.

Google recibe:

```html
<h1>iPhone 17</h1>
```

No necesita conocer TypeScript.

------

## Interactive

```tsx
<button onClick={buy}>
```

Este sí necesita runtime.

El HTML podría contener:

```html
<button data-nexa-action="buy">
    Comprar
</button>
```

y nuestro runtime posteriormente conecta:

```text
data-nexa-action="buy"
             ↓
      buy() module
```

------

# 6. Esto elimina gran parte de la hidratación tradicional

No hacemos:

```text
hydrate(App)
```

sobre toda la aplicación.

En cambio:

```text
HTML
 │
 ├── static ──────────── nada
 │
 ├── dynamic ─────────── nada
 │
 ├── interactive ─────── activar
 │
 └── async ───────────── cargar
```

Podemos llamarlo:

# Selective Activation

o:

# Progressive Activation

Me gusta más **Progressive Activation**.

Porque no estamos intentando reconstruir una aplicación.

Estamos **activando comportamiento sobre HTML existente**.

------

# 7. El servidor/renderizador

Ahora llegamos al punto que diferencia el framework.

Nuestro backend podría responder:

```json
{
    "id": 123,
    "name": "iPhone 17",
    "description": "Smartphone...",
    "price": 599,
    "image": "/images/iphone.jpg"
}
```

Nuestro renderer recibe:

```text
ProductPage
+
JSON
```

y produce:

```html
<article>

    <h1>iPhone 17</h1>

    <p>Smartphone...</p>

    <img
        src="/images/iphone.jpg"
        alt="iPhone 17"
    >

    <strong>$599</strong>

    <button data-nexa-action="buy">
        Comprar
    </button>

</article>
```

Esto es **SEO real**.

------

# 8. El framework debe diferenciar Data y UI

Esto también es fundamental.

No queremos:

```tsx
fetch(...)
render(...)
SEO(...)
router(...)
```

mezclados.

Tendremos:

```text
Data Layer
      ↓
UI Layer
      ↓
SEO Layer
      ↓
Renderer
```

Por ejemplo:

```tsx
export async function loadProduct(
    id: string
): Promise<Product> {

    return api.get(`/products/${id}`);
}
```

La página:

```tsx
export default page(async ({ params }) => {

    const product = await loadProduct(params.id);

    return (
        <Product product={product} />
    );
});
```

Y SEO:

```tsx
export const seo = ({ product }) => ({
    title: product.name,
    description: product.description
});
```

------

# 9. SEO no será un plugin

Esto también lo definiría desde ahora.

SEO será parte del **core del framework**.

Una página puede declarar:

```tsx
export const seo = (data) => ({
    title: `${data.product.name} | Oweeme`,
    description: data.product.description,

    canonical: `/producto/${data.product.slug}`,

    openGraph: {
        title: data.product.name,
        image: data.product.image
    }
});
```

El renderer produce:

```html
<head>

<title>iPhone 17 | Oweeme</title>

<meta
    name="description"
    content="Smartphone..."
>

<link
    rel="canonical"
    href="https://example.com/producto/iphone-17"
>

<meta
    property="og:title"
    content="iPhone 17"
>

<meta
    property="og:image"
    content="/images/iphone.jpg"
>

</head>
```

------

# 10. Semantic HTML también será responsabilidad del framework

No quiero que el framework trate todo como:

```text
<div>
```

Nuestro objetivo debería ser que el desarrollador pueda escribir:

```tsx
<article>
    <header>
        <h1>{article.title}</h1>
    </header>

    <section>
        {article.content}
    </section>
</article>
```

y el renderer genere HTML normal.

Google ve:

```html
<article>
<h1>
<section>
<p>
```

No un árbol generado posteriormente por JavaScript.

------

# 11. Router

El router también debe ser HTML-first.

Estructura:

```text
src/
├── pages/
│   ├── index.tsx
│   ├── products/
│   │   ├── index.tsx
│   │   └── [slug].tsx
│   └── blog/
│       └── [slug].tsx
│
├── components/
├── layouts/
├── services/
└── stores/
```

Entonces:

```text
/products/iphone-17
```

resuelve:

```text
pages/products/[slug].tsx
```

Primera visita:

```text
HTTP
 ↓
route
 ↓
data
 ↓
renderer
 ↓
HTML
```

Navegación interna:

```text
click
 ↓
Nexa Router
 ↓
fetch data
 ↓
render fragment
 ↓
DOM update
```

Por tanto:

**SEO + navegación SPA.**

------

# 12. El concepto de Fragment Navigation

Esto es importante.

Supongamos:

```text
/products
```

HTML:

```html
<main id="page">
    ...
</main>
```

El usuario pulsa:

```text
Producto X
```

No queremos:

```text
reload completo
```

Tampoco:

```text
descargar toda la aplicación
```

Queremos:

```text
GET /products/x
        ↓
HTML/fragment
        ↓
#page
        ↓
replace
```

Y actualizar:

```js
history.pushState(...)
```

Así tenemos navegación SPA sin convertir toda la página en una SPA.

------

# 13. El runtime debería ser pequeño

Objetivo conceptual:

```text
HTML = 100%
JS = mínimo
```

No:

```text
Framework runtime = 150 KB
Application = 300 KB
Hydration = 100 KB
```

Nuestro runtime debería encargarse solamente de:

```text
Router
Events
State
Effects
Fetch
Activation
```

Y cada componente interactivo puede cargar su propio código.

------

# 14. Code splitting automático

Por ejemplo:

```tsx
<button onClick={buy}>
```

El compilador detecta:

```text
buy()
```

y genera:

```text
product-page.js
```

Pero:

```tsx
<Comments />
```

puede generar:

```text
comments.js
```

Y:

```tsx
<Chat />
```

genera:

```text
chat.js
```

Entonces:

```text
Initial JS
     │
     ├── Router
     └── Runtime

Lazy
     ├── Product
     ├── Comments
     └── Chat
```

------

# 15. Async Components

También necesitamos esto desde el diseño inicial.

```tsx
<Comments productId={product.id} />
```

Podría ser:

```tsx
async function Comments() {

    const comments =
        await api.get("/comments");

    return (
        <section>
            ...
        </section>
    );
}
```

El renderer puede generar:

```html
<section>
    ...
</section>
```

desde el principio si el dato está disponible.

O:

```text
HTML inicial
     ↓
<section>
Loading...
</section>
     ↓
runtime
     ↓
comments.js
```

según la estrategia.

------

# 16. Estado

No quiero un Redux/Vuex obligatorio.

Nuestro estado debería ser extremadamente simple:

```tsx
const count = state(0);

function increment() {
    count.value++;
}
```

Y:

```tsx
<button onClick={increment}>
    {count}
</button>
```

El compilador sabe que:

```text
count
```

afecta solamente a:

```text
TextNode
```

Entonces no necesita rerenderizar:

```text
App
 ├── Header
 ├── Product
 ├── Sidebar
 └── Footer
```

Solamente:

```text
TextNode(count)
```

Esto sería **fine-grained reactivity**.

------

# 17. Effects

También:

```tsx
effect(() => {

    document.title =
        `Cart (${cart.value})`;

});
```

Pero los efectos deben clasificarse:

```text
SERVER SAFE
CLIENT ONLY
```

Por ejemplo:

```tsx
effect.client(() => {
    localStorage.setItem(...)
});
```

El compilador sabe que esto jamás debe ejecutarse durante renderizado SEO.

------

# 18. La frontera Server / Client

Esta sería una de las piezas más importantes del framework.

```text
             COMPONENT
                 │
       ┌─────────┴─────────┐
       │                   │
    SERVER              CLIENT
       │                   │
    HTML/SEO             JS
       │                   │
       └─────────┬─────────┘
                 │
              Browser
```

Podríamos tener:

```tsx
export default component({
    server: true
});
```

o:

```tsx
export default component({
    client: true
});
```

Pero incluso mejor:

**el compilador debería inferirlo cuando sea posible.**

Si encuentra:

```tsx
onClick
```

sabe:

```text
CLIENT
```

Si encuentra:

```tsx
product.name
```

puede permanecer:

```text
SERVER
```

------

# 19. El compilador Rust

Aquí empezaría realmente el proyecto.

No empezaría haciendo:

```text
CLI
router
HTTP client
```

Primero:

```text
Parser
   ↓
AST
   ↓
IR
   ↓
Static/Dynamic/Interactive analysis
   ↓
HTML renderer
   ↓
Activation manifest
```

Por ejemplo:

```text
Input:

<button onClick={buy}>
    Comprar
</button>
```

IR:

```text
Element(button)
│
├── Attribute
│   └── activation = buy
│
└── Text
    └── "Comprar"
```

Output:

```html
<button data-nexa="a1">
    Comprar
</button>
```

Manifest:

```json
{
    "a1": {
        "event": "click",
        "module": "/assets/product-buy.js",
        "handler": "buy"
    }
}
```

El navegador solamente necesita cargar ese módulo cuando corresponde.

------

# 20. Primera especificación del framework

Yo establecería ahora estos principios como **Nexa Architecture v0.1**:

```text
NEXA v0.1

01. HTML-first
02. SEO-first
03. Backend-agnostic
04. API-first
05. TypeScript-first
06. Rust compiler
07. No Virtual DOM
08. No mandatory hydration
09. Progressive Activation
10. Fine-grained reactivity
11. File-based routing
12. SPA navigation optional
13. SSG supported
14. SSR supported
15. HTML-only supported
16. Streaming supported
17. Automatic code splitting
18. Server/client boundaries
19. SEO integrated into compiler
20. Structured data integrated into framework
```

Y una regla que considero **la más importante de todas**:

> **Si una página puede funcionar correctamente sin JavaScript, el framework debe producirla correctamente sin JavaScript.**

Es decir, si tenemos:

```text
GET /noticia/123
```

y JavaScript está desactivado, el usuario debería seguir viendo:

```html
<h1>La noticia...</h1>
<p>Contenido...</p>
<img>
```

La interactividad es una **mejora**, no un requisito para descubrir el contenido.

------

## La siguiente etapa

Ahora que ya tenemos el modelo conceptual, el siguiente paso no debería ser todavía escribir cientos de líneas de código.

Hay que definir la **especificación técnica del núcleo**, concretamente:

```text
Nexa Core
│
├── 1. Component Model
├── 2. Template/JSX Syntax
├── 3. Render Tree
├── 4. Intermediate Representation (IR)
├── 5. Server/Client Boundary
├── 6. Progressive Activation Protocol
├── 7. State Model
├── 8. Router Protocol
├── 9. Data Fetching Protocol
├── 10. SEO Model
├── 11. HTML Serialization
├── 12. Browser Runtime
└── 13. Rust Compiler
```

**Yo empezaría por los puntos 1–5**, porque ahí está el verdadero corazón de este framework. Si esos cinco quedan bien diseñados, después podemos construir el compilador Rust, el runtime TypeScript, el router y finalmente un primer proyecto real de e-commerce para comprobar que obtenemos **HTML SEO completo + navegación SPA + JavaScript mínimo**, sin necesitar que PHP/Go/Python se conviertan en un servidor SSR de Node.

-------------

Sí. Y con esos detalles ya podemos definir mejor qué sería **Nexa**. Hay una aclaración importante: **Nexa no debería depender de un servidor Node para hacer SSR**. Pero eso no significa que Nexa elimine por completo la necesidad de un proceso de renderizado cuando quieres SSR dinámico.

La arquitectura que propones puede evitar que **tu backend de negocio** sea un servidor SSR.

------

# 1. La arquitectura de Nexa

Yo la definiría así:

```text
                    ┌─────────────────────┐
                    │      MariaDB        │
                    └──────────┬──────────┘
                               │
                               ▼
                     ┌──────────────────┐
                     │ Backend          │
                     │ PHP / Go / Python │
                     │ Rust / Java ...  │
                     └────────┬─────────┘
                              │
                       JSON / API / RPC
                              │
                              ▼
                    ┌─────────────────────┐
                    │       NEXA          │
                    │                     │
                    │ Compiler            │
                    │ Renderer            │
                    │ Router              │
                    │ SEO                 │
                    │ Runtime             │
                    └──────────┬──────────┘
                               │
                               ▼
                         HTML + JS
                               │
                ┌──────────────┼──────────────┐
                ▼              ▼              ▼
              Web           Tauri         Capacitor
```

La idea fundamental sería:

> **Nexa no es tu backend. Nexa es el motor que convierte datos + componentes en una experiencia web HTML-first.**

------

# 2. ¿Entonces no necesitamos Node SSR?

### Para el backend: no.

Puedes tener:

```text
PHP → JSON
Go → JSON
Python → JSON
Rust → JSON
```

Nexa consume:

```text
GET /api/products/123
```

y recibe:

```json
{
  "id": 123,
  "name": "Producto",
  "description": "...",
  "price": 99
}
```

Eso es totalmente independiente del lenguaje del backend.

### Pero hay una distinción

Si quieres:

```text
Request
 ↓
API JSON
 ↓
Nexa Renderer
 ↓
HTML
 ↓
Browser
```

el renderer tiene que ejecutarse en algún lugar.

**Eso es inevitable.**

Lo que queremos evitar es que tu arquitectura sea:

```text
Browser
   ↓
Node SSR
   ↓
API PHP
   ↓
MariaDB
```

Queremos poder tener distintas estrategias.

------

# 3. Nexa podría tener tres modos

## Modo A — Static

```text
API
 ↓
Nexa build
 ↓
HTML
```

Se genera:

```text
dist/
├── index.html
├── products/
│   ├── iphone.html
│   └── samsung.html
└── blog/
    └── article.html
```

No necesitas Node en producción.

Puedes servirlo con:

- Apache
- Nginx
- Caddy
- CDN
- cualquier servidor HTTP.

Este modo sería excelente para:

- landing pages
- blogs
- documentación
- catálogos
- noticias
- páginas corporativas.

------

# 4. Modo B — Hybrid

Aquí es donde Nexa puede ponerse interesante.

Algunas páginas:

```text
/products
/blog
/about
```

son HTML estático.

Mientras:

```text
/dashboard
/admin
/chat
```

son aplicaciones interactivas.

Por tanto:

```text
Nexa
│
├── Static
├── HTML-only
├── Interactive
└── SPA
```

Una misma aplicación puede mezclar todo.

------

# 5. Modo C — Dynamic rendering

Si necesitas:

```text
GET /product/123
```

y el producto cambia constantemente, puedes utilizar un renderer.

Pero Nexa debería tener **adapters**:

```text
@nexa/adapter-node
@nexa/adapter-bun
@nexa/adapter-deno
@nexa/adapter-cloudflare
@nexa/adapter-rust
```

Y eventualmente:

```text
@nexa/adapter-php
```

Pero esto sería opcional.

El punto es que **Nexa Core no conoce Node**.

------

# 6. Y aquí aparece una idea todavía mejor

Podemos hacer que el backend entregue **datos y una representación de página**, no necesariamente HTML.

Por ejemplo:

```json
{
  "page": "product",
  "data": {
    "id": 123,
    "name": "iPhone",
    "price": 599
  }
}
```

Nexa sabe:

```text
page = product
```

y ejecuta:

```text
ProductPage
   +
data
   ↓
HTML
```

Así el backend solamente dice:

> "Estos son los datos."

Nexa decide:

> "Así se presenta."

Eso mantiene una separación arquitectónica muy limpia.

------

# 7. Tauri encaja MUY bien

Aquí sí veo una ventaja de haber escogido Rust para el compiler.

Nexa podría tener:

```text
                 NEXA
                   │
        ┌──────────┼───────────┐
        │          │           │
        ▼          ▼           ▼
       WEB       TAURI      CAPACITOR
```

### Web

```text
Nexa
 ↓
HTML
 ↓
Browser
```

### Tauri

```text
Nexa
 ↓
WebView
 ↓
Rust/Tauri
 ↓
Windows/Linux/macOS/Android/iOS
```

### Capacitor

```text
Nexa
 ↓
WebView
 ↓
Capacitor
 ↓
Android/iOS
```

Y el mismo código TypeScript:

```ts
import { api } from "@nexa/http";

const products =
    await api.get("/products");
```

podría funcionar en los tres.

------

# 8. Pero no mezclaría Tauri dentro del Core

Esto es importante.

No quiero que Nexa sea:

```text
Nexa = Web + Tauri + Android + iOS
```

Quiero:

```text
Nexa Core
   │
   ├── Web Adapter
   ├── Tauri Adapter
   └── Capacitor Adapter
```

Así Nexa sigue siendo un framework frontend.

------

# 9. UI: sí, puedes utilizar una biblioteca externa

Y esto también lo diseñaría desde el principio.

Nexa **no debería obligarte a usar un UI framework**.

Por ejemplo:

```tsx
import { Button } from "@some-ui/button";
```

o:

```tsx
import { QButton } from "...";
```

si alguien quisiera adaptar Quasar.

Pero además podríamos crear:

```text
@nexa/ui
```

con componentes básicos.

------

# 10. Nexa UI

Podríamos tener:

```text
@nexa/ui
│
├── Button
├── Input
├── Select
├── Dialog
├── Drawer
├── Card
├── Tabs
├── Table
├── Dropdown
├── Modal
├── Tooltip
├── Avatar
├── Badge
├── Alert
└── ...
```

Pero no quiero que el framework dependa de ellos.

Sería:

```text
Nexa Core
    │
    ├── optional → Nexa UI
    │
    ├── optional → Tailwind
    │
    ├── optional → Bootstrap
    │
    ├── optional → DaisyUI
    │
    └── optional → custom UI
```

------

# 11. Y sí: podemos tener algo parecido a Quasar

Aquí creo que tu observación es muy buena.

Quasar tiene conceptos que son realmente útiles:

```text
boot
css
icons
fonts
plugins
config
notifications
dialog
loading
```

Nexa debería tener algo equivalente, pero diseñado desde cero.

Por ejemplo:

```text
nexa.config.ts
export default defineConfig({

    app: {
        name: "My Store",
        version: "1.0.0"
    },

    css: [
        "./src/css/app.css"
    ],

    fonts: [
        "Inter"
    ],

    plugins: [
        notifications(),
        dialogs()
    ]

});
```

------

# 12. Boot system

Sí, tendría mucho sentido.

Por ejemplo:

```text
src/
├── boot/
│   ├── api.ts
│   ├── auth.ts
│   ├── analytics.ts
│   └── notifications.ts
```

Y:

```ts
export default boot(() => {

    api.configure({
        baseURL: "/api"
    });

});
```

Pero el compilador podría eliminar automáticamente boot modules que no se utilicen.

------

# 13. Notifications

También podemos tener:

```ts
import { notify } from "@nexa/notifications";

notify.success("Producto agregado");
```

Y:

```ts
notify.error("No se pudo guardar");
```

La implementación puede cambiar según plataforma:

```text
                 notify()
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
       Web        Tauri      Capacitor
        │           │           │
      Toast      Native       Native
```

Eso es muy potente.

El desarrollador no necesita saber cómo funciona cada plataforma.

------

# 14. Pero también deberíamos diferenciar UI Notification de Push Notification

No son lo mismo.

### Notification

```ts
notify.success(...)
```

Es una notificación dentro de la aplicación.

### Push

```ts
push.subscribe(...)
```

Puede utilizar:

- Web Push
- FCM
- APNs
- Tauri
- Capacitor.

Entonces:

```text
@nexa/notifications
@nexa/push
```

serían módulos separados.

------

# 15. Fonts

También podemos incluir un sistema:

```ts
fonts: {
    families: [
        "Inter",
        "Roboto"
    ]
}
```

Pero no quiero que Nexa descargue automáticamente fuentes desde Google.

Podría soportar:

```text
local
remote
self-hosted
```

y generar automáticamente:

```html
<link rel="preload">
```

cuando sea apropiado.

Esto incluso puede formar parte de la optimización SEO/performance.

------

# 16. Icons

Algo similar:

```ts
import { Icon } from "@nexa/ui";

<Icon name="search" />
```

y el compiler puede incluir únicamente:

```text
search.svg
```

en vez de cargar una biblioteca entera.

Eso es otra oportunidad para el compiler Rust.

------

# 17. Plugins

Sí, pero **no convertiría todo en plugin**.

Separaría:

### Core

```text
@nexa/compiler
@nexa/runtime
@nexa/router
@nexa/http
@nexa/seo
@nexa/state
```

### Official modules

```text
@nexa/ui
@nexa/notifications
@nexa/dialog
@nexa/icons
@nexa/forms
@nexa/i18n
@nexa/auth
@nexa/pwa
@nexa/push
```

### Community

```text
@nexa/firebase
@nexa/supabase
@nexa/stripe
@nexa/paypal
@nexa/maps
@nexa/analytics
```

Esto permitiría un ecosistema.

------

# 18. Y algo que quiero añadir: Forms

Para tus proyectos sería importante.

```tsx
<Form onSubmit={save}>

    <Input
        name="email"
        required
    />

    <Button type="submit">
        Guardar
    </Button>

</Form>
```

El framework podría encargarse de:

```text
validation
serialization
loading
errors
success
accessibility
```

Y podría funcionar sin JavaScript cuando sea posible.

Esto vuelve a nuestra filosofía:

> Progressive Enhancement.

------

# 19. Accessibility debería formar parte del compiler

Esto sería una característica diferencial interesante.

Por ejemplo, detectar:

```tsx
<img src={image} />
```

y advertir:

```text
NEXA-A11Y-001:
Image requires alt attribute.
```

O:

```tsx
<div onClick={...}>
```

y advertir:

```text
NEXA-A11Y-014:
Interactive element should use <button>.
```

El compiler Rust puede hacer este análisis.

Entonces Nexa no solamente sería:

**SEO-first**

sino:

**SEO + Accessibility + Performance first.**

------

# 20. Y ahora: ¿qué significa NEXA?

Aquí hay una cosa importante:

**Nexa no tiene que significar algo específico.**

Puede funcionar como nombre de marca.

Pero conceptualmente podemos darle una interpretación:

### NEXA

**Next Experience Architecture**

o:

**Next Experience Framework**

Me gusta especialmente:

> **NEXA — Next Experience Architecture**

Porque describe exactamente lo que queremos construir.

No es simplemente:

> "otro JavaScript framework".

Es una arquitectura:

```text
HTML
+
Data
+
Progressive JavaScript
+
Cross-platform
```

También "Nexa" transmite la idea de:

```text
connection
nexus
next
```

que encaja muy bien con un framework que conecta:

```text
Backend
   ↕
Nexa
   ↕
Web
   ↕
Desktop
   ↕
Mobile
```

------

# 21. ¿Y OweeNex?

También tiene sentido si quieres relacionarlo con tus proyectos.

Pero yo separaría **framework** y **empresa/ecosistema**.

Por ejemplo:

```text
Oweeme
│
├── Nexa
│   └── Frontend framework
│
├── OneOcean
│   └── Social platform
│
└── ...
```

Eso permite que Nexa sea eventualmente utilizado por terceros sin que parezca un framework exclusivo de Oweeme.

**OweeNex** podría ser perfectamente el nombre del proyecto interno o de una distribución, pero **Nexa** me parece mucho mejor como nombre técnico independiente.

------

# 22. Y hay una decisión que tomar ahora

Antes de continuar con el diseño del compiler, yo fijaría estas decisiones:

```text
NEXA
│
├── Language
│   └── TypeScript
│
├── Compiler
│   └── Rust
│
├── Template
│   └── JSX-like
│
├── Rendering
│   ├── HTML-only
│   ├── SSG
│   ├── Hybrid
│   └── optional SSR
│
├── Backend
│   └── agnostic
│
├── Data
│   ├── JSON
│   ├── REST
│   ├── GraphQL
│   └── custom
│
├── Runtime
│   └── TypeScript
│
├── Reactivity
│   └── fine-grained
│
├── Hydration
│   └── Progressive Activation
│
├── Router
│   └── HTML + SPA navigation
│
├── SEO
│   └── Core
│
├── UI
│   └── optional
│
├── Plugins
│   └── optional
│
├── Web
│   └── first-class
│
├── Tauri
│   └── adapter
│
└── Capacitor
    └── adapter
```

Y una última decisión que considero **muy importante**: **no diseñaría Nexa alrededor de "SSR"**. SSR sería solamente una de las estrategias de rendering.

El concepto central sería:

> **Nexa = un compilador + renderer + runtime para construir aplicaciones HTML-first, backend-agnostic y progresivamente interactivas.**

Eso nos permite conseguir exactamente lo que buscas: **un backend PHP/Go/Python/Rust que solamente entrega datos, mientras Nexa controla la representación, SEO, navegación e interactividad sin convertir todo en una SPA ni obligarte a tener Node como backend de producción.**

---------------

Sí. Antes de escribir el primer código de Nexa, **todavía hay varias decisiones de Core que conviene cerrar**. Y ahora que decidimos que Nexa tendrá su propia UI base estilo Quasar, yo ampliaría la especificación.

La regla sería:

> **Nexa Core debe ser pequeño y estable; todo lo que no sea imprescindible debe vivir como módulo oficial.**

Eso evita terminar creando un "monolito" como framework.

------

# 1. Arquitectura definitiva de Nexa

Yo propondría esta estructura:

```text
NEXA
│
├── Compiler
│   ├── Parser
│   ├── AST
│   ├── IR
│   ├── Analyzer
│   ├── Optimizer
│   ├── Code Generator
│   └── Bundler
│
├── Core Runtime
│   ├── Reactivity
│   ├── Events
│   ├── DOM
│   ├── Lifecycle
│   └── Activation
│
├── Renderer
│   ├── HTML
│   ├── SSG
│   ├── Streaming
│   └── Serialization
│
├── Router
│
├── Data
│   ├── Fetch
│   ├── Cache
│   ├── Loaders
│   └── Mutations
│
├── SEO
│
├── UI
│   ├── Components
│   ├── Layout
│   ├── Theme
│   ├── Icons
│   └── CSS
│
└── Platform
    ├── Web
    ├── Tauri
    └── Capacitor
```

Y encima:

```text
Nexa Application
```

------

# 2. UI propia: sí, pero no la metería toda en Core

Esto es importante.

No quiero:

```text
@nexa/core
    └── 200 componentes
```

Quiero:

```text
@nexa/core
@nexa/ui
@nexa/forms
@nexa/icons
@nexa/notifications
@nexa/dialog
```

El usuario instala:

```bash
nexa add ui
```

Y obtiene nuestra biblioteca.

------

# 3. Nexa UI debería ser realmente independiente de CSS frameworks

Esto me parece una excelente decisión.

No:

```text
Nexa UI
   ↓
Tailwind
   ↓
DaisyUI
```

Ni:

```text
Nexa UI
   ↓
Bootstrap
```

Sino:

```text
Nexa UI
   ↓
HTML
   +
CSS propio
   +
Nexa Runtime
```

Por ejemplo:

```tsx
<Button
    variant="primary"
    size="lg"
>
    Comprar
</Button>
```

produce:

```html
<button class="nx-btn nx-btn-primary nx-btn-lg">
    Comprar
</button>
```

con CSS propio.

------

# 4. Design System

Aquí yo tomaría bastante inspiración de Quasar, pero haciendo nuestro propio sistema.

Necesitamos definir:

```text
Colors
Typography
Spacing
Radius
Elevation
Breakpoints
Transitions
Icons
Dark mode
Density
```

Por ejemplo:

```ts
theme: {
    primary: "#1976D2",
    secondary: "#26A69A",
    accent: "#9C27B0",

    dark: true,

    radius: "medium",

    density: "comfortable"
}
```

Pero no debemos fijarnos todavía en colores concretos.

Primero hay que definir **tokens**.

------

# 5. CSS Tokens

Nexa podría generar:

```css
:root {
    --nx-color-primary: ...;
    --nx-color-secondary: ...;

    --nx-space-1: ...;
    --nx-space-2: ...;

    --nx-radius-sm: ...;
    --nx-radius-md: ...;

    --nx-font-body: ...;
    --nx-font-heading: ...;
}
```

Entonces cualquier UI externa puede coexistir con Nexa.

------

# 6. Layout system

Quasar tiene algo muy bueno: su sistema de layout.

Nexa debería tener algo equivalente:

```tsx
<App>
    <Header />
    <Drawer />
    
    <Main>
        ...
    </Main>

    <Footer />
</App>
```

Pero podríamos hacerlo mucho más semántico:

```tsx
<NexaLayout>
    <NexaHeader />
    <NexaSidebar />
    <NexaMain />
    <NexaFooter />
</NexaLayout>
```

Generando HTML:

```html
<header>
<aside>
<main>
<footer>
```

Esto beneficia también accesibilidad y SEO.

------

# 7. Responsive system

Necesitamos decidir si tendremos un sistema tipo:

```ts
breakpoints
```

y helpers:

```tsx
<Show when="mobile">
```

o:

```ts
useScreen()
```

Pero aquí hay que tener cuidado.

No quiero que Nexa termine enviando JavaScript solamente para saber si la pantalla es móvil.

Cuando sea posible:

**CSS debe resolver responsive.**

JavaScript solamente cuando realmente sea necesario.

------

# 8. Accessibility como Core

Esto sí lo pondría en Core.

No como módulo opcional.

Nexa UI debería producir:

```text
ARIA
keyboard navigation
focus management
labels
roles
semantic HTML
```

Por ejemplo:

```tsx
<Dialog
    title="Eliminar producto"
>
```

debe generar correctamente:

```html
role="dialog"
aria-modal="true"
aria-labelledby="..."
```

Y gestionar:

```text
Tab
Shift+Tab
Escape
Focus trap
Focus restore
```

------

# 9. Forms

Yo también lo consideraría parte del ecosistema oficial desde el principio.

```tsx
<Form onSubmit={save}>

    <Input
        name="email"
        type="email"
        required
    />

    <Button type="submit">
        Guardar
    </Button>

</Form>
```

Debe soportar:

```text
validation
errors
async validation
loading
dirty
touched
reset
serialization
```

Y, muy importante:

### Progressive enhancement

Un formulario básico debería poder funcionar incluso sin JavaScript cuando el backend lo permita.

------

# 10. Data Layer

Esto todavía necesita definición profunda.

No quiero que Nexa sea simplemente:

```ts
fetch()
```

Necesitamos algo como:

```ts
const product = await data<Product>(
    `/products/${id}`
);
```

Y posiblemente:

```text
loader
cache
mutation
revalidation
prefetch
abort
retry
optimistic update
```

Por ejemplo:

```ts
const products = query({
    key: ['products'],
    fetch: () => api.get('/products')
});
```

Y:

```ts
products.data
products.loading
products.error
```

Esto sería una parte importante del Core/Data.

------

# 11. Cache

Para ecommerce y noticias esto será muy importante.

Necesitamos distinguir:

```text
Browser Cache
Memory Cache
Persistent Cache
Server Cache
CDN Cache
```

Y definir políticas:

```text
cache-first
network-first
stale-while-revalidate
no-cache
```

Esto además afecta directamente al SEO/performance.

------

# 12. Mutations

Leer:

```ts
query()
```

no es igual que modificar:

```ts
mutation()
```

Por ejemplo:

```ts
const addProduct = mutation({
    execute: (product) =>
        api.post('/cart', product)
});
```

Y:

```tsx
<Button
    loading={addProduct.loading}
    onClick={() => addProduct.execute(product)}
>
    Comprar
</Button>
```

------

# 13. Error boundaries

También hay que definirlos.

```tsx
<ErrorBoundary>
    <Product />
</ErrorBoundary>
```

Pero nuevamente:

### Server error

Debe poder producir:

```text
HTTP 404
HTTP 500
HTTP 403
```

con HTML válido.

Por ejemplo:

```text
/producto/no-existe
        ↓
404
        ↓
<h1>Producto no encontrado</h1>
```

No:

```text
200 OK
<div id="app"></div>
```

Esto es importantísimo para SEO.

------

# 14. Loading states

Necesitamos una estrategia para:

```text
initial loading
navigation loading
component loading
mutation loading
background refresh
```

Y que cada uno sea independiente.

------

# 15. Internationalization

Yo lo definiría temprano aunque no lo implementemos inmediatamente.

```text
@nexa/i18n
```

Por ejemplo:

```ts
t('product.buy')
```

Pero también:

```text
URL:
 /es/producto/123
 /en/product/123
 /de/produkt/123
```

Y SEO:

```html
<link rel="alternate" hreflang="es">
<link rel="alternate" hreflang="en">
```

Esto es especialmente importante si queremos que Nexa sea útil para ecommerce y sitios internacionales.

------

# 16. SEO debe incluir Routing

No basta con `<title>`.

Necesitamos:

```text
canonical
alternate
hreflang
robots
sitemap
structured data
OpenGraph
Twitter cards
pagination
404
redirects
status codes
```

Y eventualmente:

```text
sitemap.xml
robots.txt
RSS
Atom
```

El framework debería poder generar estos recursos.

------

# 17. Security

También hay que definir esto antes del runtime.

Como mínimo:

```text
XSS
CSRF
CSP
Trusted Types
sanitization
URL validation
HTML escaping
cookie policies
```

Y una regla:

> Los valores dinámicos deben escaparse automáticamente.

Si tenemos:

```tsx
<h1>{product.name}</h1>
```

Nexa debe escapar correctamente.

Nunca:

```text
HTML injection by default
```

------

# 18. Environment system

Necesitaremos:

```text
.env
.env.development
.env.production
```

Pero con separación:

```text
PUBLIC_API_URL
SECRET_KEY
DATABASE...
```

Nexa jamás debe enviar accidentalmente:

```text
SECRET_KEY
```

al navegador.

El compiler podría detectar ciertos casos.

------

# 19. PWA

Como tú trabajas bastante con PWA, yo crearía:

```text
@nexa/pwa
```

que pueda generar:

```text
manifest.json
service worker
offline cache
install metadata
icons
```

Pero nuevamente:

**opcional.**

------

# 20. Tauri y Capacitor

Yo añadiría un concepto:

```text
@nexa/platform
```

con:

```ts
platform.isWeb
platform.isTauri
platform.isCapacitor
platform.isMobile
platform.isDesktop
```

Y APIs abstractas:

```ts
platform.notifications()
platform.storage()
platform.share()
platform.files()
platform.camera()
```

Así:

```text
Nexa Application
       │
       ▼
Platform API
       │
 ┌─────┼────────┐
 Web  Tauri  Capacitor
```

------

# 21. Plugin system

Sí necesitamos plugins, pero no desde el primer día como prioridad.

El plugin debería poder registrar:

```text
compiler transforms
routes
components
CLI commands
config
runtime modules
CSS
assets
SEO providers
platform adapters
```

Por ejemplo:

```ts
definePlugin({
    name: "maps",

    components: {
        Map
    },

    runtime: "./runtime.ts",

    css: "./styles.css"
});
```

------

# 22. DevTools

Esto es algo que **no debemos olvidar**.

Eventualmente:

```text
Nexa DevTools
```

debería permitir ver:

```text
Routes
Components
Render tree
State
Queries
Network
Activation
Loaded modules
SEO
Accessibility
Performance
```

Especialmente algo como:

```text
Page: /products/iphone

HTML:
  42 KB

JS:
  8 KB

Activated:
  ProductBuyButton
  Cart

Not activated:
  Reviews
  Recommendations

SEO:
  ✓ title
  ✓ description
  ✓ canonical
  ✓ schema
```

Eso sería brutalmente útil.

------

# 23. Testing

Necesitamos tres niveles:

```text
@nexa/test
```

### Unit

```text
components
state
utilities
```

### Renderer

```text
JSON → HTML
```

Este es particularmente importante.

Podríamos tener:

```text
Input
JSON
Expected HTML
```

y comprobar que el renderer genera exactamente lo esperado.

### E2E

```text
browser
navigation
forms
activation
```

------

# 24. Performance budget

Otra característica que me gustaría mucho.

Nexa podría permitir:

```ts
performance: {
    maxInitialJS: "50kb",
    maxCSS: "100kb",
    maxImage: "500kb"
}
```

Y el compiler avisa:

```text
NEXA-PERF-001

Initial JavaScript:
72 KB

Budget:
50 KB

Exceeded by:
22 KB
```

Esto encaja perfectamente con nuestra filosofía.

------

# 25. Observabilidad

Para aplicaciones reales:

```text
@nexa/telemetry
```

opcionalmente:

```text
errors
performance
navigation
web vitals
API latency
```

Pero siempre respetando privacidad.

------

# 26. CLI

Finalmente tendríamos:

```bash
nexa create my-app

cd my-app

nexa dev

nexa build

nexa preview

nexa add ui

nexa add pwa

nexa add i18n

nexa generate page products

nexa generate component ProductCard
```

Y:

```bash
nexa doctor
```

para diagnosticar problemas.

------

# 27. Entonces, ¿qué pertenece realmente al Core?

Yo haría esta separación:

### 🔴 Core — obligatorio

```text
Compiler
Runtime
Renderer
Component Model
Reactivity
Events
Router
Data primitives
HTML serialization
Server/Client boundary
Progressive Activation
SEO primitives
Accessibility analysis
Error handling
```

### 🟡 Official — mantenido por Nexa

```text
@nexa/ui
@nexa/forms
@nexa/icons
@nexa/notifications
@nexa/dialog
@nexa/i18n
@nexa/pwa
@nexa/auth
@nexa/push
@nexa/devtools
```

### 🟢 Adapters

```text
@nexa/tauri
@nexa/capacitor
@nexa/node
@nexa/bun
@nexa/deno
@nexa/cloudflare
```

### 🔵 Comunidad

```text
@nexa/firebase
@nexa/maps
@nexa/supabase
@nexa/...
```

------

# 28. Y el principio más importante de todos

Yo agregaría formalmente al diseño de Nexa:

> **Nexa no obliga a una aplicación a convertirse en una SPA.**

Una aplicación Nexa puede ser:

```text
HTML
```

o:

```text
HTML + pequeñas interacciones
```

o:

```text
HTML + islands
```

o:

```text
HTML + navegación SPA
```

o:

```text
SPA completa
```

según lo que necesite cada página.

Eso nos permite algo como:

```text
                  NEXA APP
                     │
       ┌─────────────┼──────────────┐
       │             │              │
       ▼             ▼              ▼
    Noticias      Ecommerce      Dashboard
       │             │              │
    HTML-first    Hybrid           SPA
       │             │              │
    0 JS/low JS   selective       full JS
```

Y **esa flexibilidad debería ser una propiedad del compilador**, no cuatro frameworks diferentes.

------

## Antes de empezar a programar, yo cerraría todavía 6 especificaciones

No necesitamos seguir agregando funcionalidades indefinidamente. El siguiente paso debería ser congelar la arquitectura base.

```text
NEXA CORE SPECIFICATION
│
├── 01. Component Model       ← siguiente
├── 02. Template / JSX
├── 03. Render Tree
├── 04. AST + IR
├── 05. Progressive Activation
├── 06. Reactivity
├── 07. Server/Client Boundary
├── 08. Data Model
├── 09. Router
├── 10. SEO Model
├── 11. UI Architecture
├── 12. CSS / Theme System
├── 13. Plugin Architecture
├── 14. Platform API
├── 15. Compiler Architecture
└── 16. Package / CLI Architecture
```

**No empezaría todavía con UI, Tauri, Capacitor ni plugins.** Primero debemos conseguir que un componente Nexa pueda pasar por esta cadena:

```text
TypeScript
    ↓
Parser
    ↓
AST
    ↓
Nexa IR
    ↓
Analyzer
    ↓
HTML Renderer
    ↓
HTML SEO completo
    ↓
Progressive Activation Manifest
    ↓
Browser Runtime
```

Cuando esa cadena funcione con un componente sencillo como:

```tsx
<Product
    product={product}
/>
```

y consigamos simultáneamente **HTML sin JavaScript + eventos selectivos + actualización reactiva + navegación**, tendremos realmente el corazón de Nexa. A partir de ahí, UI estilo Quasar, forms, notificaciones, PWA, Tauri y Capacitor pasan a ser capas construidas sobre una base sólida.

-----

Sí. Y aquí haría una última revisión **como si estuviéramos redactando la arquitectura 1.0 de un framework**, porque hay cosas que no son visibles al principio pero que, si no las definimos ahora, después obligan a romper el Core.

También cambiaría una cosa de lo que dijimos antes: **Forms, i18n, UI, PWA, notifications, etc. pueden ser paquetes oficiales de Nexa, instalables con `nexa add ...`, pero debemos diseñar desde ahora sus contratos con el Core.**

La pregunta ahora no es "¿qué funcionalidades tiene Nexa?", sino:

> **¿Qué decisiones arquitectónicas podrían limitarnos dentro de 3–5 años?**

Y ahí todavía hay varias.

------

# 1. El modelo de datos debe ser independiente del transporte

Hasta ahora hablamos de JSON, pero no deberíamos hacer que Nexa dependa de JSON.

El Core debería entender:

```text
Data
 │
 ├── JSON
 ├── FormData
 ├── HTML
 ├── Text
 ├── ArrayBuffer
 ├── Stream
 └── Custom serializer
```

Por ejemplo:

```ts
const product = await data<Product>('/api/product/123')
```

El desarrollador no debería preocuparse por si el backend utiliza:

```text
REST
GraphQL
RPC
JSON
XML
Protobuf
custom API
```

Nexa solamente necesita un **Data Adapter**.

Esto deja abierta la posibilidad de que posteriormente tengamos:

```text
@nexa/rest
@nexa/graphql
@nexa/grpc
@nexa/rpc
```

------

# 2. Streaming debería estar en el Core

Esto es importante para páginas grandes.

No queremos esperar:

```text
API
 ↓
todo el contenido
 ↓
renderizar todo
 ↓
enviar HTML
```

Queremos eventualmente:

```text
HTTP
 ↓
<html>
<head>
...
</head>

<body>

<header>...</header>

<main>
<h1>Producto</h1>

...
```

y posteriormente:

```text
Reviews
Recommendations
Related products
```

Esto permite **Streaming HTML**.

Por eso el Renderer debe diseñarse desde el principio para soportar:

```text
render()
renderStream()
```

aunque la primera versión solamente implemente `render()`.

------

# 3. Islands no debería ser una arquitectura obligatoria

Nexa puede soportar algo parecido a Islands, pero no quiero que el framework quede atrapado en ese modelo.

Debe poder hacer:

```text
Static
Interactive
Async
Island
SPA
```

individualmente.

Por ejemplo:

```tsx
<Product />

<CartButton />

<Comments />

<Chat />
```

podrían terminar como:

```text
Product
 └── HTML

CartButton
 └── Interactive

Comments
 └── Async

Chat
 └── Island
```

El compilador decide.

------

# 4. Partial rendering

Esto es diferente de hydration.

Supongamos:

```text
/products/iphone
```

Tenemos:

```html
<main id="product">
...
</main>
```

Una acción puede actualizar solamente:

```text
#product
```

sin recargar:

```text
<header>
<footer>
sidebar
```

Por eso necesitamos un protocolo interno de:

```text
HTML Fragment
```

Esto será importantísimo para la navegación.

------

# 5. Streaming + Partial Rendering + Progressive Activation

Estos tres deberían formar una especie de **Rendering Engine**:

```text
                 Rendering Engine
                       │
       ┌───────────────┼────────────────┐
       │               │                │
       ▼               ▼                ▼
   Full HTML       Fragments         Streaming
       │               │                │
       └───────────────┼────────────────┘
                       ▼
               Progressive Activation
```

Esto diferencia a Nexa de una SPA tradicional.

------

# 6. HTTP Core

Necesitamos definir nuestra abstracción HTTP.

No depender directamente de Axios.

Algo como:

```ts
const response = await http.get<Product>(
    '/products/123'
)
```

Y:

```ts
http.post()
http.put()
http.patch()
http.delete()
```

Pero también:

```ts
http.stream()
http.upload()
http.download()
```

Y middleware:

```ts
http.use(authMiddleware)
http.use(cacheMiddleware)
http.use(loggingMiddleware)
```

Entonces Axios podría incluso ser un adapter:

```text
@nexa/http
@nexa/http-axios
```

Pero Nexa no depende de Axios.

------

# 7. Abort / cancellation

Parece pequeño, pero es fundamental.

Ejemplo:

```text
Usuario busca:

iphone
   ↓
iphone 1
   ↓
iphone 17
```

No queremos que las respuestas viejas sobrescriban la última.

Nexa debería tener:

```ts
const request = http.get(...)

request.abort()
```

y esto debe integrarse con:

```text
Router
Queries
Components
Navigation
Forms
```

------

# 8. Prefetching

Nuestro router debería poder hacer:

```text
usuario mueve mouse sobre enlace
             ↓
        prefetch()
             ↓
      datos + código
             ↓
click
             ↓
navegación inmediata
```

Pero debe ser configurable.

```tsx
<Link
    href="/product/123"
    prefetch
>
```

Esto puede hacer que Nexa se sienta como una SPA aunque entregue HTML tradicional.

------

# 9. Cache inteligente del router

Si ya visitaste:

```text
/product/1
```

y vuelves:

```text
/product/1
```

Nexa podría reutilizar:

```text
HTML
data
component code
```

dependiendo de la política.

Necesitamos definir desde Core:

```text
navigation cache
data cache
component cache
```

------

# 10. SEO dinámico después de navegación

Esto es muy importante.

Si navegamos:

```text
/product/1
```

a:

```text
/product/2
```

sin recargar la página, debemos actualizar:

```text
<title>
meta description
canonical
OpenGraph
structured data
```

Por tanto, SEO no solamente es generación HTML.

También tiene un **runtime SEO manager**.

------

# 11. HTTP status codes

Esto debería ser Core.

Una página puede devolver:

```text
200
201
301
302
304
307
308
404
403
410
500
503
```

Por ejemplo:

```ts
return notFound();
```

debe producir:

```text
HTTP 404
```

y HTML:

```html
<h1>Producto no encontrado</h1>
```

No un falso:

```text
HTTP 200
```

Esto es crítico para SEO.

------

# 12. Redirect system

Igual:

```ts
return redirect('/login', 302);
```

y:

```ts
return permanentRedirect('/new-url');
```

Esto debe estar en Core.

------

# 13. Headers

También:

```ts
setHeader(...)
```

para:

```text
Cache-Control
ETag
Last-Modified
Content-Type
CSP
```

Esto será importante para performance y seguridad.

------

# 14. Assets

Aquí hay otra parte grande que debemos definir.

Nexa debe saber manejar:

```text
JS
CSS
Images
Fonts
SVG
JSON
Videos
WebAssembly
```

Y el compiler debería producir:

```text
hash
manifest
dependency graph
```

Por ejemplo:

```text
assets/
├── app.a8f2.js
├── product.91ca.js
├── app.23fa.css
└── logo.8ad3.svg
```

------

# 15. Image system

Para ecommerce/noticias esto es importantísimo.

Yo crearía:

```text
@nexa/image
```

con:

```tsx
<Image
    src={product.image}
    alt={product.name}
    width={800}
    height={800}
    loading="lazy"
/>
```

Y podría generar:

```text
srcset
sizes
width
height
preload
lazy loading
formats
```

Esto ayuda a:

- SEO
- Core Web Vitals
- performance
- CLS.

------

# 16. Head management

No solamente SEO.

También:

```text
title
meta
link
style
script
preload
prefetch
modulepreload
```

Necesitamos un sistema:

```ts
head({
    title: 'Product'
})
```

y que funcione:

```text
SSR
SSG
SPA navigation
```

------

# 17. Web Components

Aquí hay una decisión interesante.

Yo permitiría que Nexa pueda **consumir Web Components externos**.

Por ejemplo:

```html
<some-external-widget />
```

Nexa no necesita saber cómo está construido.

Esto permitiría integrar:

```text
Web Components
Lit
custom elements
third-party widgets
```

sin convertirlos en parte del Core.

------

# 18. Microfrontends

No lo implementaría inicialmente, pero el Core debería no impedirlo.

Por ejemplo:

```text
Nexa application
│
├── ecommerce
├── analytics
└── external widget
```

Pero no metería esto en v1.

Simplemente no diseñaría APIs que hagan imposible integrarlo posteriormente.

------

# 19. Workers

Nexa debería considerar:

```text
Web Worker
Service Worker
Shared Worker
```

especialmente para:

```text
offline
background processing
large calculations
sync
```

Pero como módulos oficiales:

```text
@nexa/worker
@nexa/pwa
```

------

# 20. WebSocket / SSE

Esto es importante para:

- chat
- notificaciones
- dashboards
- tiempo real.

El Core debería tener una abstracción:

```ts
const stream = realtime('/notifications');

stream.on('message', ...)
```

Y adaptadores:

```text
WebSocket
SSE
custom
```

No necesariamente implementaríamos todo en la primera versión.

Pero el contrato debería existir.

------

# 21. Storage

Necesitamos una API común:

```ts
storage.get()
storage.set()
storage.remove()
```

que pueda mapear:

```text
Web:
localStorage
IndexedDB

Tauri:
filesystem / secure storage

Capacitor:
native storage
```

Pero aquí hay que separar:

```text
Storage
SecureStorage
Cache
```

No debemos guardar secretos en `localStorage`.

------

# 22. Platform capabilities

Esto puede convertirse en uno de los puntos fuertes de Nexa.

Por ejemplo:

```ts
import { platform } from '@nexa/platform';

if (platform.camera.available) {
    ...
}
```

Pero idealmente:

```ts
const image = await device.camera.capture();
```

El usuario no sabe si está en:

```text
Web
Android
iOS
Tauri
```

El adapter resuelve la implementación.

------

# 23. Internacionalización completa

Como dices que `i18n` será un módulo oficial, yo lo haría bastante completo:

```text
@nexa/i18n
```

Debe soportar:

```text
translations
pluralization
formatting
dates
numbers
currency
locale detection
URL localization
hreflang
RTL
lazy translations
```

Especialmente:

```text
/es/producto/123
/en/product/123
```

------

# 24. Authentication

También creo que debe existir como módulo oficial:

```text
@nexa/auth
```

Pero **no dentro del Core**.

Debe abstraer:

```text
session
cookies
tokens
refresh
login
logout
permissions
roles
guards
```

Y ser backend agnostic.

Por ejemplo:

```ts
auth.user
auth.isAuthenticated
auth.login()
auth.logout()
```

------

# 25. Authorization

Separado de Authentication.

```text
Authentication
    ↓
¿Quién eres?

Authorization
    ↓
¿Qué puedes hacer?
```

Por ejemplo:

```ts
can('product.edit')
```

Esto será importante para dashboards y SaaS.

------

# 26. Environment / configuration

Nexa necesita una configuración fuertemente tipada:

```ts
export default defineConfig({
    app: {},
    build: {},
    css: {},
    seo: {},
    i18n: {},
    ui: {},
    plugins: {}
});
```

Pero quiero que el compiler valide la configuración.

------

# 27. Package system

Esta es una decisión MUY importante.

No deberíamos depender de un package manager específico para la arquitectura.

Pero inicialmente podemos utilizar:

```text
npm / pnpm / bun
```

para paquetes TypeScript.

Mientras:

```text
nexa CLI
```

se encarga de la experiencia del framework.

Por ejemplo:

```bash
nexa add ui
```

podría internamente instalar:

```text
@nexa/ui
```

------

# 28. CLI debe ser extensible

Los plugins deberían poder agregar comandos:

```bash
nexa generate
nexa add
nexa doctor
nexa analyze
```

Y un plugin:

```text
@nexa/shop
```

podría agregar:

```bash
nexa shop generate-product
```

Esto será muy útil si algún día haces módulos específicos para ecommerce.

------

# 29. Migration system

Esto casi nadie piensa al principio.

Pero si Nexa llega a v2, v3:

```bash
nexa migrate
```

debería poder transformar:

```text
Nexa 1.x
    ↓
Nexa 2.x
```

Por eso el compiler debe conocer versiones del AST/IR.

------

# 30. Compatibility policy

Antes de programar debemos decidir:

```text
Nexa 0.x
Nexa 1.x
Nexa 2.x
```

y qué significa breaking change.

Yo usaría:

```text
Core API → muy estable
Compiler internals → pueden cambiar
Plugins → versionados
UI → versionada independientemente
Adapters → independientes
```

------

# 31. Tree shaking obligatorio

Nexa UI puede tener:

```text
100 componentes
```

pero si utilizas:

```tsx
<Button />
```

el build debe incluir solamente:

```text
Button
Button CSS
dependencies
```

No:

```text
todo Nexa UI
```

------

# 32. Zero-JS como objetivo medible

Esto lo convertiría en una característica oficial.

Una página:

```text
/home
```

podría producir:

```text
HTML: 35 KB
CSS: 8 KB
JS: 0 KB
```

Otra:

```text
/product/123
```

podría producir:

```text
HTML: 52 KB
CSS: 12 KB
JS inicial: 4 KB
lazy JS: 18 KB
```

Y DevTools debería mostrarlo.

------

# 33. SEO Analyzer

Incluso podemos tener:

```bash
nexa analyze seo
```

y recibir:

```text
SEO ANALYSIS

✓ <title>
✓ meta description
✓ canonical
✓ h1
✓ semantic HTML
✓ image alt
✓ structured data
✓ sitemap
✓ robots

⚠ Missing hreflang
⚠ Image without dimensions
```

Eso sería una herramienta realmente útil.

------

# 34. Accessibility Analyzer

Igual:

```bash
nexa analyze a11y
A11Y

✓ Buttons accessible
✓ Form labels
✓ Keyboard navigation

⚠ Image missing alt
⚠ Dialog missing accessible name
```

------

# 35. Performance Analyzer

Y:

```bash
nexa analyze performance
Initial JS       12 KB
Initial CSS      18 KB
HTML             42 KB
Images           240 KB

LCP               1.1s
CLS               0.02
INP               80ms
```

Esto puede integrarse con DevTools posteriormente.

------

# 36. Entonces, ¿qué queda realmente en Core?

Después de toda esta revisión, yo congelaría **Nexa Core** así:

```text
NEXA CORE
│
├── Component Model
├── TypeScript Compiler Integration
├── AST / IR
├── Render Tree
├── HTML Renderer
├── Streaming Renderer
├── Progressive Activation
├── Fine-Grained Reactivity
├── Lifecycle
├── Events
├── Router primitives
├── Navigation
├── Fragment rendering
├── Data primitives
├── HTTP abstraction
├── Cache primitives
├── Error handling
├── Status codes
├── Redirects
├── Headers
├── Head management
├── SEO primitives
├── Accessibility analysis
├── Asset graph
├── Code splitting
├── Environment system
└── Security primitives
```

Eso es suficiente.

**No metería UI, Forms, i18n, Auth, PWA, Tauri, Capacitor, etc. dentro del Core.**

------

# 37. Y el ecosistema oficial de Nexa

Aquí sí podemos tener algo mucho más grande:

```text
NEXA OFFICIAL
│
├── @nexa/ui
├── @nexa/forms
├── @nexa/icons
├── @nexa/notifications
├── @nexa/dialog
├── @nexa/loading
├── @nexa/i18n
├── @nexa/auth
├── @nexa/push
├── @nexa/pwa
├── @nexa/image
├── @nexa/realtime
├── @nexa/storage
├── @nexa/devtools
├── @nexa/testing
├── @nexa/eslint
├── @nexa/formatter
├── @nexa/tauri
├── @nexa/capacitor
├── @nexa/rest
├── @nexa/graphql
└── @nexa/...
```

Y los adapters:

```text
NEXA ADAPTERS
│
├── Node
├── Bun
├── Deno
├── Rust
├── Cloudflare
├── Tauri
├── Capacitor
└── Custom
```

La diferencia es que **Nexa Core no depende de ninguno de ellos**.

------

# 38. Y finalmente: la arquitectura completa

Llegados a este punto, la visión de Nexa ya queda bastante clara:

```text
                         NEXA
                          │
              ┌───────────┴───────────┐
              │                       │
           CORE                  OFFICIAL
              │                       │
       ┌──────┼──────┐         ┌──────┼───────┐
       │      │      │         │      │       │
    Compiler Router Renderer   UI    i18n   Forms
       │      │      │         │      │       │
       │      │      │         └──────┼───────┘
       │      │      │                │
       └──────┼──────┘                │
              │                       │
              ▼                       ▼
        HTML + Runtime          Application APIs
              │
       ┌──────┼──────────┐
       │      │          │
      Web    Tauri   Capacitor
```

Y por debajo:

```text
                 BACKEND
        ┌─────────┼─────────┐
        │         │         │
       PHP       Go       Python
        │         │         │
        └─────────┼─────────┘
                  │
              JSON / RPC
                  │
                  ▼
                NEXA
```

### Y aquí ya tenemos una frontera muy clara:

**Nexa Core**
→ cómo se construye y ejecuta una aplicación.

**Nexa Official**
→ herramientas y funcionalidades que casi todos necesitan.

**Nexa Adapters**
→ integración con plataformas/runtime externos.

**Nexa Community**
→ cualquier integración adicional.

------

## Lo único que todavía considero obligatorio definir antes de escribir el compiler

Hay **cinco decisiones de muy bajo nivel** que no conviene improvisar:

1. **Cómo se representa un componente en TypeScript.**
2. **Cómo se transforma ese componente en nuestro AST/IR.**
3. **Cómo se representa un nodo interactivo en el HTML.**
4. **Cómo el navegador descubre qué código debe activar.**
5. **Cómo funciona la reactividad sin Virtual DOM.**

Esas cinco decisiones son el verdadero **ADN de Nexa**.

Una vez definidas, podemos pasar a diseñar el **Nexa Core Specification v0.1**, incluyendo ejemplos concretos de código TypeScript → AST → IR → HTML → manifest → runtime. Ahí ya estaríamos dejando de hablar de una idea y empezando a diseñar formalmente el framework.

----------------

Exactamente. Si queremos que Nexa sea un framework serio para 10+ años, **la compatibilidad no puede ser una promesa de marketing; tiene que ser una propiedad de su arquitectura**.

Yo establecería ahora una regla fundamental:

> **Nexa prioriza estabilidad de la API del desarrollador sobre la evolución interna del framework.**

El compiler, runtime, optimizador e incluso partes del renderer podrán cambiar internamente, pero el código que escribe el desarrollador debe permanecer válido.

Definamos las cinco decisiones.

------

# 1. Modelo de componentes

Esta es probablemente la decisión más importante.

No copiaría Vue, React ni Angular. Nexa tendrá un modelo propio, pero debe sentirse natural para alguien que conoce TypeScript.

Propongo:

```tsx
export component ProductCard(props: ProductProps) {

    return (
        <article class="product-card">
            <img
                src={props.product.image}
                alt={props.product.name}
            />

            <h2>{props.product.name}</h2>

            <p>{props.product.description}</p>

            <Button onClick={() => addToCart()}>
                Comprar
            </Button>
        </article>
    );
}
```

La palabra:

```ts
component
```

sería parte del lenguaje/framework de Nexa.

No:

```ts
defineComponent()
```

ni:

```ts
React.FC
```

ni una clase.

------

## Props

Fuertemente tipadas:

```ts
interface ProductProps {
    product: Product;
    featured?: boolean;
}

export component ProductCard(props: ProductProps) {
    ...
}
```

TypeScript hace el trabajo.

------

## Estado

Aquí quiero evitar completamente el modelo de Virtual DOM.

Por ejemplo:

```ts
const count = signal(0);
```

y:

```tsx
<Button onClick={() => count.value++}>
    {count.value}
</Button>
```

El compilador sabe exactamente que:

```text
count.value
      ↓
Button text
```

depende de ese estado.

Por lo tanto no necesita:

```text
Component
   ↓
Virtual DOM
   ↓
diff
   ↓
patch
```

Puede hacer:

```text
signal
  ↓
dependency
  ↓
DOM node
```

Esto será **fine-grained reactivity**.

------

# 2. AST + IR

Aquí usaría una arquitectura de compiler real.

```text
TypeScript / Nexa
       ↓
     Parser
       ↓
     Nexa AST
       ↓
    Analyzer
       ↓
      IR
       ↓
   Optimizer
       ↓
 Code Generator
```

Y el compiler puede estar escrito en Rust.

Pero hay una regla:

> **El AST interno y el IR NO son APIs públicas.**

Esto es crucial para la estabilidad.

Podemos cambiar:

```text
IR v1
```

por:

```text
IR v2
```

en Nexa 5, 10 o 20 sin romper:

```ts
export component ProductCard(...)
```

------

## ¿Por qué Rust?

Porque el compiler puede encargarse de:

```text
parsing
type analysis
dependency analysis
tree shaking
code splitting
SEO analysis
a11y analysis
optimization
asset graph
```

y hacerlo muy rápido.

Pero el desarrollador sigue escribiendo:

```text
TypeScript
```

No tiene que aprender Rust.

------

# 3. ¿Cómo representamos interactividad en HTML?

Aquí está una de las decisiones que realmente diferenciarán a Nexa.

No quiero:

```html
<div id="app"></div>
```

y luego:

```text
JavaScript → construir toda la aplicación
```

Nexa debe generar HTML real:

```html
<article class="product-card">
    <h1>Producto</h1>

    <button
        data-nx-action="cart.add"
        data-nx-id="product-123"
    >
        Comprar
    </button>
</article>
```

Pero no quiero llenar el HTML de información innecesaria.

El compiler genera un **Activation Manifest**.

Por ejemplo:

```json
{
  "components": {
    "CartButton": {
      "module": "/assets/cart-button.js"
    }
  }
}
```

Y el HTML tiene solamente las referencias necesarias.

------

# 4. Progressive Activation

Aquí está la diferencia fundamental con hydration tradicional.

No hacemos:

```text
HTML
 ↓
descargar TODO JS
 ↓
recrear TODO
 ↓
comparar TODO
 ↓
activar TODO
```

Nexa hace:

```text
HTML
 ↓
Browser muestra inmediatamente
 ↓
Nexa Runtime analiza activation map
 ↓
solo descarga lo necesario
 ↓
solo activa lo necesario
```

Por ejemplo:

```text
Página ecommerce
│
├── Header
│   └── HTML solamente
│
├── Product
│   └── HTML solamente
│
├── Gallery
│   └── Interactive
│
├── AddToCart
│   └── Interactive
│
├── Reviews
│   └── Lazy
│
└── Chat
    └── Lazy
```

Resultado:

```text
HTML → inmediatamente visible
JS → únicamente donde hace falta
```

------

# 5. El navegador debe saber cuándo activar

Definiría cuatro estrategias oficiales:

### `static`

Nunca necesita JavaScript.

```tsx
<ProductInfo mode="static" />
```

------

### `visible`

Activar cuando entra en viewport.

```tsx
<Comments activate="visible" />
```

------

### `idle`

Activar cuando el navegador esté libre.

```tsx
<Analytics activate="idle" />
```

------

### `interaction`

Activar cuando el usuario interactúe.

```tsx
<CartButton activate="interaction" />
```

Esto puede extenderse posteriormente:

```text
visible
idle
interaction
media
manual
load
```

Pero las cuatro primeras deberían ser estables.

------

# 6. Una decisión todavía más importante: no depender de hydration tradicional

Yo incluso evitaría llamar a esto simplemente "hydration".

En Nexa el concepto oficial sería:

> **Progressive Activation**

Porque no estamos intentando reconstruir una aplicación que ya existía en JavaScript.

El navegador ya tiene:

```html
<h1>
<p>
<button>
<form>
```

Nexa únicamente **conecta comportamiento donde sea necesario**.

Eso reduce muchísimo el coste de JavaScript.

------

# 7. Reactividad

Aquí necesitamos ser muy precisos.

Nexa tendrá:

```ts
signal()
computed()
effect()
```

Por ejemplo:

```ts
const price = signal(100);
const quantity = signal(2);

const total = computed(() =>
    price.value * quantity.value
);
```

HTML:

```tsx
<p>
    Total: {total.value}
</p>
```

El dependency graph será:

```text
price ─────┐
           ├──> total ───> <p>
quantity ──┘
```

Cuando cambia `quantity`:

```text
quantity
   ↓
total
   ↓
text node
```

No:

```text
application
 ↓
component
 ↓
virtual tree
 ↓
diff
```

------

# 8. Pero debemos permitir estado local y global

### Local

```ts
const count = signal(0);
```

### Compartido

```ts
export const cart = store({
    items: [],
    total: 0
});
```

### Computado

```ts
const total = computed(...)
```

### Efectos

```ts
effect(() => {
    localStorage.setItem(
        'cart',
        JSON.stringify(cart.value)
    );
});
```

------

# 9. El Runtime debe ser pequeño

Esto es otra regla de diseño.

No queremos:

```text
@nexa/runtime = 500 KB
```

El runtime debe dividirse:

```text
@nexa/runtime/core
@nexa/runtime/reactivity
@nexa/runtime/router
@nexa/runtime/forms
```

Y el compiler incluye solamente lo utilizado.

Idealmente una página sencilla podría necesitar:

```text
Runtime:
~3–10 KB
```

dependiendo de sus características.

No fijaría todavía un número exacto; fijaría el principio:

> **El runtime enviado al navegador debe ser proporcional a la interactividad utilizada.**

------

# 10. El componente no debería tener que saber dónde se ejecuta

Esto es fundamental para Tauri y Capacitor.

El mismo:

```ts
export component ProductCard(...)
```

puede ejecutarse en:

```text
Browser
Tauri
Capacitor
```

El código de aplicación no debería cambiar.

Cuando necesite una capacidad del dispositivo:

```ts
import { device } from '@nexa/platform';

await device.notifications.send(...)
```

El adapter decide:

```text
Web
 ↓
Web API

Tauri
 ↓
Rust

Capacitor
 ↓
Native Plugin
```

------

# 11. La estabilidad durante 10 años

Aquí quiero establecer algo mucho más fuerte que "semver".

## Nexa Developer Contract

Una aplicación Nexa 1.0 debería seguir funcionando en:

```text
Nexa 1.x
Nexa 2.x
Nexa 3.x
...
```

si utiliza solamente APIs estables.

------

# 12. API Stable vs Experimental

Tendremos:

```ts
@stable
@experimental
@internal
@deprecated
```

Por ejemplo:

```ts
export component Product() {}
```

es:

```text
STABLE
```

Mientras:

```ts
experimentalFeature()
```

puede cambiar.

Y:

```ts
__nexa_internal_x()
```

no es API pública.

------

# 13. Nunca romper una API estable sin necesidad

Supongamos que en Nexa 1 tenemos:

```ts
const count = signal(0);
```

En Nexa 10 puede existir internamente:

```text
SignalEngineV10
```

pero:

```ts
signal(0)
```

continúa funcionando.

Internamente:

```text
Old API
   ↓
Compatibility Layer
   ↓
New Engine
```

El usuario ni siquiera necesita saberlo.

------

# 14. Compatibilidad hacia adelante

Pero también debemos pensar en proyectos viejos.

Un proyecto Nexa 1 debería poder compilarse con Nexa 10.

Para ello:

```text
Nexa Compiler
       │
       ├── Legacy compatibility
       ├── Current compiler
       └── Optimizer
```

El compiler puede transformar sintaxis antigua internamente.

------

# 15. Codemods, pero opcionales

Si en Nexa 10 existe una API mejor:

```ts
oldSignal()
```

podemos proporcionar:

```bash
nexa migrate
```

que transforme:

```ts
oldSignal()
```

en:

```ts
signal()
```

Pero **el proyecto no queda obligado a migrar inmediatamente**.

Esto es exactamente lo contrario de:

> "Actualiza Angular y ahora tienes que modificar 200 archivos."

------

# 16. Long Term Support

Yo establecería desde el principio:

```text
Nexa LTS
```

Por ejemplo:

```text
Nexa 1 LTS
Nexa 2 LTS
Nexa 3 LTS
```

Una versión LTS recibe:

```text
security fixes
critical bug fixes
platform compatibility
```

pero no cambios arbitrarios de API.

------

# 17. Arquitectura reemplazable

Aquí está la clave para que Nexa pueda evolucionar dentro de 10 años.

No debemos hacer:

```text
Component → Compiler implementation
```

directamente.

Sino:

```text
Component
   ↓
Stable semantic model
   ↓
IR
   ↓
Compiler backend
```

Así podemos cambiar:

```text
Rust compiler v1
```

por:

```text
Rust compiler v2
```

o incluso:

```text
otro compiler
```

sin cambiar el modelo del desarrollador.

------

# 18. Y esto nos permite una evolución importante

Imagina que en 2036 aparece una tecnología mucho mejor que nuestra estrategia actual.

No queremos decir:

> "Nexa está muerto porque su arquitectura original quedó vieja."

Queremos:

```text
Nexa Developer API
        │
        ▼
Stable Semantic Layer
        │
 ┌──────┴─────────┐
 │                │
Engine 1       Engine 2
```

Podemos reemplazar el motor.

El código sigue siendo:

```tsx
<Product product={product} />
```

------

# 19. Versionado de la plataforma

Yo usaría algo parecido a:

```text
Nexa Platform API
```

con contratos versionados.

Por ejemplo:

```text
Nexa Platform API 1
Nexa Platform API 2
```

El compiler puede saber:

```text
project requires Platform API 1
```

y mantener compatibilidad.

------

# 20. Ahora podemos definir las 5 decisiones oficialmente

## DECISIÓN 01 — Component Model

```text
TypeScript
+
Nexa JSX-like syntax
+
typed props
+
signals
+
semantic HTML
```

Ejemplo:

```tsx
export component ProductCard(
    props: ProductProps
) {
    return (
        <article>
            <h2>{props.product.name}</h2>
        </article>
    );
}
```

------

## DECISIÓN 02 — Compiler

```text
TypeScript
    ↓
Nexa Parser
    ↓
AST
    ↓
Semantic Analyzer
    ↓
Nexa IR
    ↓
Optimizer
    ↓
HTML / JS / CSS / Manifest
```

Compiler inicialmente:

**Rust.**

AST/IR son internos y reemplazables.

------

## DECISIÓN 03 — HTML + Activation

Nexa siempre intenta producir:

```text
HTML real
```

primero.

Después:

```text
Activation Manifest
```

determina qué comportamiento necesita JavaScript.

No existe:

```text
<div id="app"></div>
```

como arquitectura obligatoria.

------

## DECISIÓN 04 — Progressive Activation

Cuatro estrategias estables:

```text
static
visible
idle
interaction
```

El HTML funciona primero.

JavaScript se descarga/activa solamente cuando hace falta.

------

## DECISIÓN 05 — Fine-Grained Reactivity

```text
signal
computed
effect
store
```

con dependency tracking.

Sin Virtual DOM como mecanismo obligatorio.

```text
Signal
 ↓
Dependency Graph
 ↓
DOM node
```

------

# 21. Y agregaría una sexta decisión transversal

Aunque originalmente dijimos cinco, **esta sexta es necesaria para cumplir tu objetivo de 10 años**:

## DECISIÓN 06 — Stable Developer Contract

```text
                    NEXA
                      │
              Stable API Layer
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
    Compiler        Runtime       Adapters
       │              │              │
    puede cambiar  puede cambiar  pueden cambiar
```

El desarrollador escribe contra:

```text
Component API
Router API
Signal API
Data API
UI API
Platform API
```

y no contra:

```text
compiler internals
runtime internals
DOM internals
IR
bundler
```

------

# 22. Esto también cambia cómo diseñaremos Nexa UI

Nexa UI debe tener el mismo principio:

```tsx
<Button variant="primary">
    Comprar
</Button>
```

Ese contrato debe permanecer.

En Nexa 1:

```text
Button Engine 1
```

En Nexa 10:

```text
Button Engine 10
```

pero:

```tsx
<Button variant="primary">
```

sigue funcionando.

La implementación interna puede cambiar completamente.

------

# 23. Una regla que pondría en el documento oficial

> **"Stable code should outlive the implementation that executes it."**

En español:

> **El código estable del desarrollador debe sobrevivir a la implementación que lo ejecuta.**

Esta debería ser una de las ideas fundamentales de Nexa.

------

# 24. Entonces el siguiente paso ya puede ser técnico

Con estas decisiones podemos comenzar a diseñar **Nexa Core Specification 0.1**.

Yo no empezaría todavía haciendo una UI bonita ni un CLI grande.

El primer prototipo debería demostrar únicamente esto:

```text
                    NEXA v0.1
                       │
                TypeScript/JSX
                       │
                       ▼
                    Compiler
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
         AST           IR        Analyzer
                       │
                       ▼
                    Renderer
                       │
                       ▼
                 HTML + Manifest
                       │
                       ▼
                    Browser
                       │
                ┌──────┴──────┐
                ▼             ▼
              HTML          Signal
             visible       activation
```

Y probar algo muy concreto:

```tsx
export component Counter() {
    const count = signal(0);

    return (
        <section>
            <h1>Contador</h1>

            <p>{count.value}</p>

            <button
                onClick={() => count.value++}
            >
                Incrementar
            </button>
        </section>
    );
}
```

Nexa debería producir **HTML que ya tenga sentido sin JavaScript**, un manifest que diga qué debe activarse y un runtime mínimo que conecte únicamente el botón y el texto con el `signal`.

Si conseguimos ese flujo, tenemos el **ADN real del framework**. Después podemos construir encima UI tipo Quasar, Forms, i18n, Router avanzado, SEO, PWA, Tauri, Capacitor, DevTools, etc., sin comprometer la arquitectura.

---------------

Sí. Y ahora sí estamos llegando al punto donde **conviene dejar de agregar funcionalidades y cerrar el contrato arquitectónico**.

Además, una precisión: **TypeScript 7.0 ya fue publicado oficialmente el 8 de julio de 2026**, así que podemos tomar TypeScript 7 como la base de desarrollo de Nexa. Es además una versión con un nuevo compilador nativo y mejoras importantes de rendimiento. ([Microsoft for Developers](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/?utm_source=chatgpt.com))

Pero hay una distinción importante:

> **Nexa puede estar escrito para el desarrollador en TypeScript 7, mientras que el compilador de Nexa puede estar implementado en Rust.**

El programador de Nexa **no necesita aprender Rust**.

------

# Lo que todavía debemos decidir antes de escribir código

Yo cerraría estas **10 decisiones finales**.

## 1. El contrato de componentes

Debe quedar congelado:

```tsx
export component ProductCard(props: ProductProps) {
    return (
        <article>
            <h2>{props.product.name}</h2>
        </article>
    );
}
```

Tenemos que decidir exactamente:

- props
- children
- eventos
- slots
- composición
- lifecycle
- componentes async
- componentes server/data
- componentes interactivos

Esto no debe cambiar después.

------

# 2. Sintaxis

Aquí tenemos que decidir si Nexa utilizará:

```tsx
<Product />
```

con JSX/TSX tradicional, o si crearemos una variante propia.

**Mi recomendación: TSX.**

Porque:

- TypeScript ya lo conoce.
- Los editores ya lo soportan.
- No inventamos otro lenguaje.
- Facilita migraciones.
- Reduce el coste de aprendizaje.

Podemos agregar semántica propia de Nexa sin inventar una sintaxis completamente nueva.

------

# 3. Modelo de reactividad

Lo congelaría como:

```ts
signal()
computed()
effect()
store()
```

Ejemplo:

```ts
const quantity = signal(1);

const total = computed(() =>
    quantity.value * product.price
);
```

Y:

```tsx
<span>{total.value}</span>
```

El compiler genera dependencias directas.

Esto es mucho más estable que hacer depender el framework de un Virtual DOM.

------

# 4. Modelo de rendering

Nexa tendría **cuatro estrategias**, pero una sola API:

```text
HTML
SSG
Streaming
Progressive Activation
```

El desarrollador no debería tener que aprender cuatro frameworks.

Por ejemplo:

```tsx
<Product />
```

puede producir HTML.

Y:

```tsx
<Product activate="interaction" />
```

puede activar comportamiento.

La API sigue siendo la misma.

------

# 5. Data Layer

También hay que congelar:

```ts
query()
mutation()
resource()
```

Por ejemplo:

```ts
const product = query({
    key: ['product', id],

    fetch: () =>
        api.get<Product>(`/products/${id}`)
});
```

Y que soporte:

```text
REST
JSON
GraphQL
RPC
FormData
Streams
WebSocket
SSE
```

mediante adapters.

------

# 6. Router

No debemos dejarlo para después.

Debe soportar desde el principio:

```text
/
/products
/products/:id
/blog/:slug
```

y:

```text
navigation
prefetch
loading
404
redirect
nested routes
route params
query params
layouts
guards
lazy loading
```

Y algo fundamental:

### La URL sigue siendo la fuente de verdad.

No queremos:

```text
estado SPA → URL secundaria
```

sino:

```text
URL
 ↓
Router
 ↓
Page
 ↓
Data
 ↓
HTML
```

Eso favorece SEO, accesibilidad y deep linking.

------

# 7. SEO como contrato del framework

No debe existir:

```ts
setTitle()
```

como única solución.

Nexa debe entender:

```text
title
description
canonical
robots
OpenGraph
Twitter
JSON-LD
hreflang
sitemap
RSS
HTTP status
redirect
structured data
```

Y además:

```text
<h1>
<h2>
<p>
<article>
<nav>
<main>
<section>
```

deben ser HTML real.

------

# 8. UI API

Aquí debemos congelar el contrato de Nexa UI:

```tsx
<Button />
Input
Select
Dialog
Drawer
Card
Table
Tabs
Menu
Tooltip
Toast
Form
DatePicker
Pagination
Avatar
```

Pero la implementación visual podrá evolucionar.

Por ejemplo:

```tsx
<Button variant="primary">
```

debe seguir funcionando en Nexa 1, 5 y 10.

Podremos cambiar internamente:

```text
CSS
DOM
accessibility implementation
animation engine
theme engine
```

sin romper al desarrollador.

------

# 9. Platform API

Esto también debe ser estable:

```ts
platform.storage
platform.notifications
platform.camera
platform.files
platform.share
platform.clipboard
```

Entonces:

```tsx
await platform.share({
    title: 'Producto',
    url: location.href
});
```

funciona independientemente de:

```text
Web
Tauri
Capacitor
```

Los adapters cambian; la API Nexa no.

------

# 10. El sistema de compatibilidad

Esta es la parte que más quiero que quede definida antes de programar.

Tenemos que establecer:

```text
Stable
Experimental
Internal
Deprecated
Removed
```

Por ejemplo:

```ts
signal()
```

es:

```text
STABLE
```

pero:

```ts
experimentalScheduler()
```

es:

```text
EXPERIMENTAL
```

Y:

```ts
__nexa_internal()
```

es:

```text
INTERNAL
```

------

# Una regla todavía más fuerte

Yo pondría en la especificación:

> **Una API marcada como Stable no se elimina en una versión mayor sin una ruta de compatibilidad.**

Eso significa que si dentro de 10 años descubrimos una arquitectura mejor:

```text
Nexa 1
   ↓
Nexa 10
```

el nuevo engine puede implementar la API antigua.

```text
                 Stable API
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
      Engine V1              Engine V10
```

El desarrollador no tiene que reescribir su aplicación.

------

# Y hay otra cosa MUY importante: no atarnos demasiado a TypeScript 7

Aquí haría una pequeña corrección a tu idea.

Sí:

> **Nexa será TypeScript-first.**

Pero no deberíamos decir:

> "Nexa depende eternamente de TypeScript 7."

Porque eso nos podría crear el mismo problema que estamos intentando evitar.

TypeScript 7 es nuestra **base actual de lenguaje y tooling**. Oficialmente, TypeScript 7 tiene un nuevo compilador nativo y la propia documentación indica que se seguirá evolucionando con nuevas versiones. ([Microsoft for Developers](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/?utm_source=chatgpt.com))

Nexa debería permitir:

```text
Nexa Language
       │
       ▼
TypeScript 7+
       │
       ▼
Nexa Compiler
```

El desarrollador escribe TypeScript.

Pero nosotros controlamos la semántica de Nexa.

------

# Esto nos da una arquitectura muy interesante

```text
              DESARROLLADOR
                    │
                    ▼
              TypeScript 7+
                    │
                    ▼
              NEXA LANGUAGE
                    │
                    ▼
             NEXA COMPILER
                    │
              ┌─────┴─────┐
              ▼           ▼
           Browser      Native
              │           │
             Web       Tauri/
                       Capacitor
```

Y el compilador podría estar escrito en Rust:

```text
TypeScript
    ↓
Nexa Compiler
    ↓
Rust
    ↓
HTML / CSS / JS / Assets
```

**Rust es una implementación, no una dependencia conceptual para el programador.**

------

# Hay una última decisión que recomiendo agregar

## Nexa Compatibility Layer

Desde la primera versión.

Por ejemplo:

```text
@nexa/compat
```

No porque vayamos a romper cosas, sino precisamente para **evitar romperlas**.

Cuando Nexa tenga una mejora arquitectónica:

```text
Old API
   ↓
Compatibility Layer
   ↓
New Engine
```

Esto nos permitirá hacer algo que considero fundamental:

### Evolución sin migración obligatoria.

Un desarrollador puede tener:

```text
proyecto creado en 2027
```

y en:

```text
2037
```

seguir ejecutando:

```bash
nexa build
```

sin tener que reescribir toda su aplicación.

Si quiere las nuevas capacidades:

```bash
nexa migrate
```

pero **migrar es una decisión del desarrollador, no una obligación para seguir funcionando**.

------

# Entonces yo considero cerrada la filosofía de Nexa

Tenemos:

```text
NEXA
│
├── TypeScript-first
│
├── TSX
│
├── Compiler en Rust
│
├── HTML-first
│
├── SEO-first
│
├── Accessibility-first
│
├── Progressive Activation
│
├── Fine-grained reactivity
│
├── Backend agnostic
│
├── No Node SSR obligatorio
│
├── API-driven
│
├── UI propia
│
├── Tauri
├── Capacitor
│
├── SSG
├── Streaming
├── Hybrid
│
├── Stable API
├── Compatibility Layer
├── LTS
└── Migration tools
```

Y la filosofía podría quedar resumida en una frase:

> **"Write once, evolve without rewriting."**

Es decir:

**Escribes tu aplicación una vez y Nexa evoluciona alrededor de ella, no obliga a tu aplicación a evolucionar cada vez que cambia Nexa.**

Con esto, yo **ya no agregaría más funcionalidades antes de comenzar**. Lo siguiente debería ser redactar la **Nexa Core Specification v0.1**: carpetas del repositorio, paquetes, arquitectura del compiler Rust, sintaxis TSX, AST, IR, runtime, manifest de activación y el primer programa mínimo de Nexa.

-------------

El programador **no tendría que compilar en Rust ni ejecutar Rust directamente**. La experiencia debería ser igual de sencilla que `npm run build`, pero con el motor de compilación de Nexa escrito en Rust.

Por ejemplo:

```bash
nexa dev
nexa build
nexa preview
```

## ¿Qué ocurre internamente?

El flujo sería:

```text
                    DESARROLLADOR
                         │
                         │
                   nexa build
                         │
                         ▼
                    Nexa CLI
                         │
                         ▼
                Nexa Compiler
                    escrito en Rust
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
            HTML        JS/CSS     Assets
              │          │          │
              └──────────┼──────────┘
                         ▼
                       dist/
```

El desarrollador solamente instala Nexa:

```bash
npm install -g nexa
```

o, idealmente, posteriormente:

```bash
nexa create my-app
```

y trabaja siempre con:

```bash
nexa dev
nexa build
nexa preview
```

------

# ¿Dónde entra Node?

Aquí hay una decisión importante.

**Nexa no debería necesitar Node como servidor de producción.**

Pero podemos permitir que Node sea solamente una herramienta de desarrollo si resulta conveniente.

Por ejemplo:

```text
DESARROLLO

TypeScript
   ↓
Nexa CLI
   ↓
Rust Compiler
   ↓
dist/
```

No:

```text
TypeScript
   ↓
Node SSR
   ↓
HTML
```

El servidor de producción podría ser:

```text
Nginx
Apache
Caddy
PHP
Go
Python
Rust
Cloudflare
S3/CDN
```

dependiendo de cómo se despliegue la aplicación.

------

# ¿Y el `npm build`?

Aquí Nexa puede funcionar de dos maneras.

### Opción A — Nexa CLI nativa

La que prefiero:

```bash
nexa build
```

El ejecutable:

```text
nexa
```

sería un binario nativo.

Por ejemplo:

```text
Linux
nexa
 ↓
Rust

Windows
nexa.exe
 ↓
Rust

macOS
nexa
 ↓
Rust
```

No necesitas Node instalado para compilar.

------

### Opción B — Compatibilidad con npm

Podríamos permitir:

```json
{
  "scripts": {
    "dev": "nexa dev",
    "build": "nexa build"
  }
}
```

Entonces alguien acostumbrado a:

```bash
npm run build
```

puede seguir usando exactamente eso.

Pero:

```text
npm
 ↓
nexa
 ↓
Rust Compiler
```

Node/npm solamente está ejecutando el comando; **no está haciendo el SSR ni el compilado principal**.

------

# Incluso podemos eliminar Node completamente

Una aplicación Nexa podría tener:

```text
my-app/
├── src/
├── public/
├── nexa.config.ts
├── package.json
└── ...
```

y:

```bash
nexa dev
```

sin Node.

El CLI descarga/resuelve las dependencias necesarias y ejecuta el compiler Rust.

Esto es especialmente interesante para tu objetivo porque tú quieres evitar que el ecosistema termine dependiendo de Node.

------

# Pero hay un detalle importante

No deberíamos intentar reemplazar npm desde el día uno.

TypeScript tiene un ecosistema enorme.

Por ejemplo:

```text
npm
pnpm
bun
```

ya resuelven:

```text
packages
dependencies
registry
versions
lockfiles
```

Nexa puede aprovechar eso.

La arquitectura podría ser:

```text
                  Nexa CLI
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
   Dependency     Compiler      Dev Server
    Manager         Rust           Rust
        │            │              │
        ▼            ▼              ▼
      npm/etc.    Nexa IR         Browser
```

Y posteriormente podríamos desarrollar:

```text
@nexa/package-manager
```

si realmente hace falta.

------

# Y esto nos da una ventaja enorme

Imagina que instalas Nexa en una máquina limpia.

Con un framework tradicional puedes terminar teniendo:

```text
Node
npm
npx
Vite
TypeScript
framework CLI
bundler
SSR runtime
```

Nexa podría llegar a:

```text
Nexa CLI
   │
   ├── Compiler
   ├── Bundler
   ├── Dev Server
   ├── Analyzer
   └── Runtime tooling
```

todo construido en Rust.

El desarrollador sigue escribiendo:

```text
TypeScript
TSX
CSS
HTML
```

------

# Y el servidor de producción queda completamente separado

Esto es justamente lo que diferencia la arquitectura que estamos diseñando.

Supongamos que haces una tienda:

```text
                    NEXA
                     │
              nexa build
                     │
                     ▼
                  dist/
              ┌──────┴──────┐
              │             │
             HTML          JS/CSS
              │             │
              └──────┬──────┘
                     ▼
                    CDN
                     │
                     ▼
                  Browser
                     │
                     │ API
                     ▼
              ┌──────────────┐
              │ Backend      │
              │              │
              │ PHP / Go     │
              │ Python / etc │
              └──────────────┘
```

El backend solamente entrega:

```json
{
  "id": 123,
  "name": "Producto",
  "price": 199
}
```

Nexa se encarga del:

```text
HTML
SEO
routing
reactividad
UI
interactividad
hydration/activation
```

------

## Mi recomendación definitiva

Para Nexa:

**Desarrollador:**

```text
TypeScript + TSX
```

**CLI:**

```text
Rust
```

**Compiler:**

```text
Rust
```

**Bundler:**

```text
Rust
```

**Dev server:**

```text
Rust
```

**Runtime del navegador:**

```text
JavaScript/TypeScript compilado
```

**Backend:**

```text
PHP / Go / Python / Rust / Java / etc.
```

**Node:**

```text
Opcional
```

Y el comando principal sería simplemente:

```bash
nexa build
```

No:

```bash
node server.js
```

ni:

```bash
npm run ssr
```

ni:

```text
Node SSR → HTML
```

Esto encaja perfectamente con la filosofía que estamos definiendo: **Rust queda debajo de Nexa como infraestructura; TypeScript queda arriba como lenguaje del desarrollador.**

--

Sí. **Esa sería la dirección que recomiendo para Nexa**, con una precisión importante: Rust no "reemplaza TypeScript"; Rust sería el **toolchain/engine de Nexa** que procesa el código TypeScript/TSX del desarrollador y genera los artefactos necesarios.

La arquitectura quedaría así:

```text
                    DESARROLLADOR
                         │
                    TypeScript
                       + TSX
                         │
                         ▼
                 ┌─────────────────┐
                 │    NEXA CLI     │
                 │     Rust        │
                 └────────┬────────┘
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
         Compiler      Bundler      Analyzer
           Rust          Rust          Rust
             │            │            │
             └────────────┼────────────┘
                          ▼
                   NEXA BUILD GRAPH
                          │
          ┌───────────────┼────────────────┐
          ▼               ▼                ▼
        Web             Mobile           Desktop
          │               │                │
        HTML            Tauri/           Tauri/
       JS/CSS          Capacitor          Rust
          │               │                │
          ▼               ▼                ▼
       Browser         Android/iOS      Linux/Win/macOS
```

## Los comandos

Tendríamos una CLI central:

```bash
nexa create my-app
nexa dev
nexa build
nexa preview
nexa add ui
nexa add forms
nexa add i18n
nexa add pwa
nexa add auth
nexa remove forms
nexa update
nexa migrate
nexa analyze
nexa test
```

Y algunos comandos especializados:

```bash
nexa generate component ProductCard
nexa generate page products
nexa generate api product
```

Aunque **no debemos decidir todavía todos los comandos**. Primero definimos el Core y después diseñamos la CLI alrededor de él.

------

# Lo más interesante: un solo proyecto

La idea sería que el desarrollador tenga:

```text
my-store/
├── src/
│   ├── components/
│   ├── pages/
│   ├── layouts/
│   ├── stores/
│   └── app.tsx
│
├── public/
├── nexa.config.ts
├── package.json
└── tsconfig.json
```

Y pueda ejecutar:

```bash
nexa dev
```

para web.

Después:

```bash
nexa build
```

para producción web.

Y si decide convertirlo en aplicación:

```bash
nexa add tauri
```

y posteriormente:

```bash
nexa build --target desktop
```

o:

```bash
nexa build --target android
```

según el adapter/plataforma instalada.

------

# Pero aquí hay una distinción MUY importante

No quiero que Nexa intente convertir:

```text
TypeScript → Rust → aplicación nativa
```

Eso no sería necesario.

El pipeline sería diferente según el target.

### Web

```text
TypeScript/TSX
      ↓
Nexa Compiler
      ↓
HTML + JS + CSS + assets
      ↓
Browser
```

### Tauri

```text
TypeScript/TSX
      ↓
Nexa Compiler
      ↓
Web application
      ↓
Tauri WebView
      +
Rust native backend
      ↓
Desktop
```

### Capacitor

```text
TypeScript/TSX
      ↓
Nexa Compiler
      ↓
Web application
      ↓
Capacitor
      ↓
Android / iOS
```

Por tanto, **Nexa no necesita reinventar Tauri ni Capacitor**.

Los adapters hacen esa integración.

------

# ¿Dónde queda Rust?

Rust sería el corazón de infraestructura:

```text
@nexa/cli
@nexa/compiler
@nexa/parser
@nexa/transformer
@nexa/bundler
@nexa/optimizer
@nexa/dev-server
@nexa/analyzer
@nexa/test-runner
```

Todos podrían formar parte del ecosistema Rust de Nexa.

El desarrollador no necesita saberlo.

------

# Y esto permite algo que me parece fundamental

Podemos tener:

```bash
nexa build
```

y Nexa puede determinar:

```text
¿Qué archivos existen?
¿Qué componentes son estáticos?
¿Qué necesitan JS?
¿Qué necesita activación?
¿Qué rutas existen?
¿Qué datos necesita cada ruta?
¿Qué HTML puede generarse?
¿Qué JavaScript debe enviarse?
¿Qué CSS se utiliza?
¿Qué assets necesita?
¿Qué imágenes deben optimizarse?
¿Qué páginas necesitan prerender?
¿Qué páginas son dinámicas?
¿Qué SEO debe generarse?
```

Y construir un **Build Graph**.

Por ejemplo:

```text
                    Application
                         │
                    Build Graph
                         │
       ┌─────────────────┼─────────────────┐
       ▼                 ▼                 ▼
     Routes           Components         Assets
       │                 │                 │
       ▼                 ▼                 ▼
     HTML              JS chunks         Images
       │                 │
       └─────────────────┼─────────────────┘
                         ▼
                       dist/
```

Esto es mucho más potente que simplemente "compilar TypeScript".

------

# Y ahí aparece una de las grandes diferencias de Nexa

Un framework tradicional puede pensar:

```text
"Necesito ejecutar la aplicación."
```

Nexa debería pensar:

> **"Necesito construir el producto final más eficiente posible."**

Por eso el compiler puede saber:

```text
Esta página no necesita JS.
Esta otra necesita 4 KB.
Este componente necesita activación al interactuar.
Esta imagen debe cargarse después.
Este contenido debe estar en HTML.
Este endpoint puede consultarse después.
Este componente nunca aparece en la primera pantalla.
```

------

# SEO quedaría integrado en el build

Por ejemplo:

```tsx
export component ProductPage() {

    const product = await query(...);

    return (
        <>
            <SEO
                title={product.name}
                description={product.description}
            />

            <main>
                <h1>{product.name}</h1>
                <p>{product.description}</p>
            </main>
        </>
    );
}
```

Nexa puede producir:

```html
<head>
    <title>...</title>
    <meta name="description" content="...">
    <link rel="canonical" href="...">
    ...
</head>

<body>
    <main>
        <h1>Producto</h1>
        <p>Descripción...</p>
    </main>
</body>
```

**sin que exista un Node SSR ejecutándose para servir esa página.**

Ese HTML puede terminar directamente en:

```text
CDN
Nginx
Apache
hosting estático
Cloudflare
S3
```

y el backend solamente proporciona los datos cuando corresponda.

------

# Y una consecuencia importante

Nexa no sería solamente:

> "otro framework frontend".

Sería más correcto definirlo como:

> **un application compiler y runtime multiplataforma basado en TypeScript.**

Eso explica mejor lo que estamos construyendo.

```text
                NEXA
                 │
       ┌─────────┼─────────┐
       │         │         │
      WEB      MOBILE    DESKTOP
       │         │         │
     HTML     Capacitor  Tauri
       │         │         │
       └─────────┼─────────┘
                 │
           TypeScript/TSX
```

Y el backend es completamente externo:

```text
PHP
Go
Python
Rust
Java
.NET
etc.
```

mediante:

```text
REST
JSON
GraphQL
RPC
SSE
WebSocket
etc.
```

------

## Una última decisión que sí debemos tomar ahora

Antes de empezar a escribir el primer código de Nexa, yo definiría **el nombre y la estructura oficial de los paquetes**.

Por ejemplo:

```text
@nexa/core
@nexa/runtime
@nexa/compiler
@nexa/router
@nexa/http
@nexa/ui
@nexa/forms
@nexa/i18n
@nexa/platform
@nexa/pwa
@nexa/tauri
@nexa/capacitor
@nexa/devtools
```

y, por otro lado, los componentes Rust:

```text
nexa-cli
nexa-parser
nexa-ast
nexa-ir
nexa-compiler
nexa-bundler
nexa-runtime
nexa-dev-server
nexa-analyzer
```

**Si cerramos esta nomenclatura y las responsabilidades de cada paquete, ya podemos pasar a diseñar el repositorio inicial y comenzar a construir Nexa 0.1.**

Sí. **Ya podemos comenzar.**

Tenemos definidas las decisiones fundamentales:

- **TypeScript + TSX** como lenguaje del desarrollador.
- **Rust** como base del CLI, compiler, bundler, optimizer y tooling.
- **Sin Node SSR obligatorio**.
- Backend completamente independiente: PHP, Go, Python, Rust, etc.
- APIs mediante JSON, REST, GraphQL, RPC, streams, etc.
- **HTML-first + SEO completo**.
- **Progressive Activation** en lugar de hydration tradicional obligatoria.
- **Fine-grained reactivity**, sin depender de Virtual DOM.
- Router, HTTP, data layer y platform API definidos como contratos.
- UI oficial estilo ecosistema Quasar, pero independiente.
- PWA, Tauri y Capacitor mediante adapters.
- i18n, Forms, Auth, Notifications, Storage, etc. como paquetes oficiales.
- Compiler interno reemplazable.
- **Stable Developer API**.
- Compatibility Layer.
- `nexa migrate` para mejoras arquitectónicas.
- LTS y compatibilidad a largo plazo.
- Una misma aplicación orientada a **Web + Mobile + Desktop**.

Y la regla principal queda establecida:

> **Nexa puede cambiar internamente; el código estable del desarrollador no debería tener que cambiar.**

Por tanto, **sí: podemos comenzar la construcción de Nexa 0.1**. El siguiente paso sería diseñar formalmente el **repositorio, paquetes Rust, `nexa-cli`, sintaxis inicial, AST, IR y el primer compilador funcional**, antes de añadir UI y funcionalidades secundarias.

Perfecto. Entonces entramos ya en la **construcción técnica de Nexa 0.1**.

El siguiente paso no es todavía programar componentes ni UI. Primero vamos a establecer la **estructura del repositorio y las fronteras entre paquetes**, porque eso determinará cómo podrá evolucionar Nexa durante muchos años sin romper el código existente.

# NEXA 0.1 — Arquitectura del repositorio

Propongo un **monorepo**, con Rust para el toolchain y TypeScript para las APIs que utiliza el desarrollador.

```text
nexa/
├── README.md
├── LICENSE
├── Cargo.toml
├── rust-toolchain.toml
├── package.json
├── pnpm-workspace.yaml
│
├── crates/
│   ├── nexa-cli/
│   ├── nexa-parser/
│   ├── nexa-ast/
│   ├── nexa-analyzer/
│   ├── nexa-ir/
│   ├── nexa-transformer/
│   ├── nexa-compiler/
│   ├── nexa-bundler/
│   ├── nexa-renderer/
│   ├── nexa-runtime-generator/
│   ├── nexa-dev-server/
│   ├── nexa-assets/
│   ├── nexa-seo/
│   ├── nexa-a11y/
│   ├── nexa-config/
│   └── nexa-errors/
│
├── packages/
│   ├── core/
│   ├── runtime/
│   ├── router/
│   ├── http/
│   ├── reactivity/
│   ├── platform/
│   └── types/
│
├── official/
│   ├── ui/
│   ├── forms/
│   ├── i18n/
│   ├── auth/
│   ├── notifications/
│   ├── storage/
│   ├── pwa/
│   ├── image/
│   └── realtime/
│
├── adapters/
│   ├── tauri/
│   ├── capacitor/
│   ├── node/
│   ├── deno/
│   └── custom/
│
├── templates/
│   ├── basic/
│   ├── web/
│   └── full/
│
├── tests/
│   ├── compiler/
│   ├── renderer/
│   ├── reactivity/
│   ├── router/
│   ├── seo/
│   └── integration/
│
└── docs/
    ├── architecture/
    ├── compiler/
    ├── runtime/
    ├── api/
    └── versioning/
```

La estructura puede cambiar durante 0.x, pero las **fronteras conceptuales** debemos tratarlas como parte de nuestra arquitectura.

------

# 1. `crates/`: el corazón Rust

Aquí vive todo lo que hace posible:

```bash
nexa dev
nexa build
nexa preview
```

## `nexa-cli`

Es el ejecutable:

```bash
nexa
```

Responsabilidades:

```text
create
dev
build
preview
add
remove
generate
analyze
test
update
migrate
```

No debe contener el compiler directamente.

Debe coordinar los demás crates.

------

# 2. `nexa-parser`

Se encarga de leer:

```tsx
export component Product() {
    return (
        <h1>Hello Nexa</h1>
    );
}
```

y producir nuestro AST.

No debería preocuparse por:

- HTML final
- bundling
- filesystem
- navegador
- SEO.

Solo:

```text
Source
 ↓
Parser
 ↓
AST
```

------

# 3. `nexa-ast`

Define las estructuras semánticas.

Por ejemplo conceptualmente:

```rust
Component
 ├── name
 ├── props
 ├── state
 └── template
```

y:

```rust
Element
 ├── tag
 ├── attributes
 └── children
```

Pero aquí aparece una regla importante:

### AST no será API pública.

Podremos modificar internamente:

```text
AST v1
AST v2
AST v3
```

sin obligar al desarrollador a cambiar su código.

------

# 4. `nexa-analyzer`

Este será muy importante.

Analizará:

```text
component dependencies
signals
events
routes
data dependencies
SEO
accessibility
client activation
assets
```

Por ejemplo:

```tsx
<p>{product.name}</p>
```

puede ser detectado como:

```text
product.name
     ↓
dynamic expression
     ↓
DOM dependency
```

------

# 5. `nexa-ir`

El **Intermediate Representation** será el puente entre análisis y generación.

```text
TSX
 ↓
AST
 ↓
Analyzer
 ↓
Nexa IR
 ↓
Optimizer
```

Esto nos permite cambiar el compiler en el futuro.

Por ejemplo:

```text
Nexa 1
TypeScript → AST → IR1 → output

Nexa 10
TypeScript → AST → IR10 → output
```

La aplicación sigue siendo:

```tsx
<Product />
```

------

# 6. `nexa-transformer`

Aquí hacemos transformaciones:

```text
signals
events
components
async expressions
imports
dynamic imports
activation
```

Por ejemplo:

```ts
count.value++
```

puede convertirse internamente en una operación optimizada del runtime.

------

# 7. `nexa-compiler`

Este coordina:

```text
Parser
Analyzer
Transformer
IR
Optimizer
Renderer
Bundler
```

Conceptualmente:

```text
                COMPILER
                   │
      ┌────────────┼────────────┐
      ▼            ▼            ▼
   Parser       Analyzer      Assets
      │            │
      └──────┬─────┘
             ▼
            IR
             │
             ▼
         Transformer
             │
             ▼
          Optimizer
             │
       ┌─────┴─────┐
       ▼           ▼
    Renderer     Bundler
```

------

# 8. `nexa-renderer`

Este será nuestro renderer HTML.

Debe ser capaz de producir:

```html
<html>
<head>
...
</head>
<body>
...
</body>
</html>
```

Pero además:

```text
HTML fragments
streaming
head
attributes
events
activation markers
```

No debería depender de Node.

------

# 9. `nexa-runtime-generator`

Este genera solamente el JavaScript necesario.

Por ejemplo:

```text
Página estática
→ 0 runtime JS

Página con signal
→ reactivity runtime

Página con router
→ router runtime

Página con dialog
→ dialog runtime
```

Esto será fundamental para mantener pequeño el bundle.

------

# 10. `nexa-bundler`

Se encargará de:

```text
dependency graph
tree shaking
code splitting
dynamic imports
hashing
minification
source maps
CSS
JS
assets
```

Y producirá:

```text
dist/
├── index.html
├── assets/
│   ├── app.xxxx.js
│   ├── product.xxxx.js
│   └── app.xxxx.css
└── manifest.json
```

------

# 11. `nexa-dev-server`

Cuando ejecutamos:

```bash
nexa dev
```

queremos un servidor Rust.

Debe ofrecer:

```text
HTTP
HMR
watcher
rebuild
error overlay
source maps
```

Pero aquí hay una decisión importante:

**HMR no debe convertirse en una dependencia arquitectónica del runtime de producción.**

Es solamente tooling.

------

# 12. `nexa-seo`

Aquí empezamos a diferenciar Nexa.

Debe analizar:

```text
title
description
canonical
robots
OpenGraph
JSON-LD
hreflang
headings
semantic HTML
structured data
```

Y generar warnings:

```text
SEO:
✓ title
✓ description
✓ canonical
⚠ Missing h1
⚠ Image without alt
```

------

# 13. `nexa-a11y`

Analizará:

```text
ARIA
labels
keyboard navigation
semantic HTML
contrast metadata
interactive elements
forms
images
dialogs
```

No reemplazará herramientas externas como Lighthouse, pero tendrá análisis durante build.

------

# 14. `packages/`

Aquí están las APIs TypeScript que consume la aplicación.

Por ejemplo:

```ts
import { signal } from '@nexa/reactivity';
```

o:

```ts
import { http } from '@nexa/http';
```

Aquí tenemos una separación muy importante:

```text
Rust
 ↓
construye Nexa

TypeScript
 ↓
construye aplicaciones Nexa
```

------

# 15. `@nexa/reactivity`

Nuestro sistema:

```ts
signal()
computed()
effect()
store()
```

Este paquete será pequeño.

------

# 16. `@nexa/router`

API:

```ts
route()
navigate()
redirect()
notFound()
```

y componentes:

```tsx
<Link />
<RouterView />
```

El router debe poder trabajar tanto con:

```text
HTML inicial
```

como con:

```text
client navigation
```

------

# 17. `@nexa/http`

No dependerá de Axios.

Tendrá una abstracción propia:

```ts
http.get()
http.post()
http.put()
http.patch()
http.delete()
```

Después podremos ofrecer:

```text
@nexa/http-axios
@nexa/http-fetch
```

si realmente hacen falta.

------

# 18. `@nexa/platform`

Será nuestra capa multiplataforma:

```ts
platform.storage
platform.camera
platform.notifications
platform.clipboard
platform.share
platform.files
```

Implementaciones:

```text
Web
Tauri
Capacitor
```

------

# 19. `official/`

Aquí empieza el ecosistema de alto nivel.

Por ejemplo:

```text
@nexa/ui
@nexa/forms
@nexa/i18n
@nexa/auth
@nexa/pwa
```

La aplicación podrá hacer:

```bash
nexa add ui
nexa add i18n
nexa add forms
```

------

# 20. `adapters/`

Aquí está una de nuestras decisiones arquitectónicas más importantes.

```text
Nexa Core
    │
    ├── Web
    ├── Tauri
    ├── Capacitor
    └── Custom
```

No queremos que:

```text
@nexa/core
```

importe:

```text
tauri
capacitor
node
```

El Core no conoce esas plataformas.

------

# 21. Primer objetivo: NO construir todo

Aunque el repositorio tenga esta arquitectura, **Nexa 0.1 no implementará todo**.

Primero construiremos:

```text
nexa-cli
     ↓
parser
     ↓
AST
     ↓
IR
     ↓
renderer
     ↓
HTML
```

Y después:

```text
reactivity
     ↓
activation
     ↓
runtime
```

Ese será nuestro primer milestone real.

------

# 22. El primer programa Nexa

Queremos llegar a esto:

```tsx
export component App() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>My first Nexa application.</p>
        </main>
    );
}
```

Ejecutar:

```bash
nexa build
```

y obtener:

```text
dist/
└── index.html
```

con:

```html
<main>
    <h1>Hello Nexa</h1>
    <p>My first Nexa application.</p>
</main>
```

**HTML real.**

Sin:

```text
Node SSR
React
Vue
Angular
Virtual DOM
```

------

# 23. Segundo milestone

Después:

```tsx
export component App() {

    const count = signal(0);

    return (
        <main>
            <h1>Counter</h1>

            <p>{count.value}</p>

            <button onClick={() => count.value++}>
                Increment
            </button>
        </main>
    );
}
```

Y Nexa debe producir:

```text
HTML
 +
activation manifest
 +
minimal runtime
```

El navegador recibe primero:

```html
<h1>Counter</h1>
<p>0</p>
<button>Increment</button>
```

y posteriormente activa solamente lo necesario.

------

# 24. Ese será nuestro primer "Hello World" real

No quiero empezar por:

```text
UI
Forms
Auth
PWA
Tauri
Capacitor
```

porque todavía no sabemos si el corazón funciona.

Primero debemos demostrar:

```text
                Nexa 0.1
                   │
               TypeScript
                   │
                  TSX
                   │
                Parser
                   │
                  AST
                   │
               Analyzer
                   │
                   IR
                   │
               Renderer
                   │
                   ▼
              HTML REAL
```

Después:

```text
HTML
 │
 ├── SEO
 ├── Reactivity
 ├── Activation
 └── Runtime
```

Y recién después construimos todo el ecosistema.

------

## Por tanto, el siguiente paso concreto

Ahora debemos diseñar **`nexa-parser` + `nexa-ast`**.

Ahí definiremos exactamente qué entiende Nexa de este código:

```tsx
export component App() {
    const count = signal(0);

    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>{count.value}</p>

            <button onClick={() => count.value++}>
                Increment
            </button>
        </main>
    );
}
```

y qué estructura interna Rust genera.

Ese será el **primer diseño de código real de Nexa**: el AST, sus nodos, sus tipos y el pipeline `source → AST → IR`.

Perfecto. Ahora sí entramos en el **primer núcleo real de Nexa**: definir cómo el código TypeScript/TSX del desarrollador se convierte en una estructura que el compilador Rust pueda entender.

# NEXA 0.1 — Paso 2: Parser + AST

Nuestro pipeline inicial será:

```text
                 Código Nexa
                     │
                     ▼
              TypeScript / TSX
                     │
                     ▼
                NEXA PARSER
                     │
                     ▼
                  NEXA AST
                     │
                     ▼
              SEMANTIC ANALYZER
                     │
                     ▼
                   NEXA IR
                     │
                     ▼
                  RENDERER
                     │
                     ▼
                    HTML
```

La primera regla es importante:

> **El AST representa lo que el desarrollador escribió; el IR representa lo que Nexa necesita hacer.**

No debemos mezclar ambas cosas.

------

# 1. ¿Qué tiene que entender Nexa?

Nuestro primer ejemplo será:

```tsx
export component App() {
    const count = signal(0);

    return (
        <main>
            <h1>Hello Nexa</h1>

            <p>{count.value}</p>

            <button onClick={() => count.value++}>
                Increment
            </button>
        </main>
    );
}
```

Nexa debe reconocer:

```text
export
component
App
function body
signal
return
JSX
elements
attributes
expressions
events
```

Pero también debemos preparar el parser para:

```tsx
import { signal } from '@nexa/reactivity';

interface Product {
    id: number;
    name: string;
}

export component ProductCard(props: Product) {
    ...
}
```

Por eso **no debemos crear un parser TypeScript completo desde cero**.

------

# 2. TypeScript debe seguir siendo TypeScript

Nexa debe reutilizar el ecosistema existente para comprender:

```text
TypeScript
TSX
interfaces
types
generics
enums
decorators
imports
exports
async
await
etc.
```

Nuestro parser debe añadir el concepto:

```text
component
```

y la semántica Nexa sobre TSX.

Conceptualmente:

```text
TypeScript Parser
       +
Nexa Syntax
       ↓
Nexa AST
```

Esto también ayuda muchísimo a la compatibilidad futura.

------

# 3. AST de Nexa

Propongo comenzar con estos nodos principales:

```text
NexaAst
│
├── Program
│
├── Import
├── Export
│
├── Component
│
├── Interface
├── Type
│
├── Variable
├── Function
│
├── Return
│
├── Element
├── Fragment
├── Text
├── Expression
│
├── EventHandler
├── Attribute
│
└── Comment
```

Pero no todos son exclusivos de Nexa.

Muchos provienen de TypeScript.

------

# 4. `Component`

Este será uno de nuestros nodos fundamentales.

Conceptualmente:

```rust
struct Component {
    name: Identifier,
    props: Option<Props>,
    body: Vec<Statement>,
    template: Template,
    exported: bool,
}
```

Para:

```tsx
export component App() {
    ...
}
```

tendríamos:

```text
Component
├── name: App
├── exported: true
├── props: none
├── body
└── template
```

------

# 5. Props

Para:

```tsx
interface ProductProps {
    product: Product;
    featured?: boolean;
}

export component ProductCard(
    props: ProductProps
) {
    ...
}
```

el AST debe conservar:

```text
Component
│
├── name: ProductCard
│
└── props
    ├── product: Product
    └── featured?: boolean
```

El tipo sigue siendo responsabilidad de TypeScript.

Nexa lo **analiza**, pero no debe inventar otro sistema de tipos.

------

# 6. Element

Para:

```tsx
<main>
```

tendríamos:

```text
Element
├── tag: main
├── attributes
└── children
```

Para:

```tsx
<h1>Hello Nexa</h1>
```

sería:

```text
Element
├── tag: h1
└── children
    └── Text("Hello Nexa")
```

------

# 7. HTML semántico

Esto es importante para nuestra filosofía SEO.

El AST debe conocer que:

```text
h1
h2
main
article
section
nav
header
footer
p
ul
ol
form
button
```

son elementos HTML.

Pero **no deberíamos crear un AST diferente para cada etiqueta**.

Todas son:

```text
Element(tag)
```

El analyzer será quien conozca las reglas semánticas.

Así podemos evolucionar el análisis sin cambiar el AST.

------

# 8. Expressions

Ahora:

```tsx
<p>{count.value}</p>
```

Tenemos:

```text
Element
└── p
    └── Expression
        └── MemberAccess
            ├── object: count
            └── property: value
```

Y para:

```tsx
<h1>{product.name}</h1>
```

igual:

```text
Expression
└── product.name
```

Esto será fundamental posteriormente para descubrir dependencias.

------

# 9. Events

Para:

```tsx
<button onClick={() => count.value++}>
```

no deberíamos tratar `onClick` simplemente como un atributo de texto.

El AST debe representar:

```text
Element
├── tag: button
├── events
│   └── click
│       └── ArrowFunction
│           └── count.value++
└── children
```

Esto permite que el analyzer determine:

```text
button
 ↓
requires activation
```

------

# 10. Static vs Dynamic

Aquí empieza una parte muy importante de Nexa.

El analyzer podrá clasificar:

```text
STATIC
DYNAMIC
INTERACTIVE
ASYNC
```

Por ejemplo:

```tsx
<h1>Hello Nexa</h1>
```

es:

```text
STATIC
```

Mientras:

```tsx
<p>{product.name}</p>
```

es:

```text
DYNAMIC
```

Y:

```tsx
<button onClick={...}>
```

es:

```text
INTERACTIVE
```

Esto posteriormente determinará qué termina en HTML y qué necesita runtime.

------

# 11. Signal

Para:

```ts
const count = signal(0);
```

el AST normal podría representar simplemente una llamada:

```text
VariableDeclaration
├── name: count
└── initializer
    └── CallExpression
        ├── callee: signal
        └── argument: 0
```

**No quiero que el parser decida que `signal()` es reactivo.**

Eso pertenece al analyzer.

¿Por qué?

Porque posteriormente podemos tener:

```ts
import { signal } from '@nexa/reactivity';
```

y el analyzer sabe que esa función representa un signal.

Así mantenemos separadas:

```text
Syntax
Semantic meaning
```

------

# 12. Dependency Graph

Después del AST viene nuestro analyzer.

Con:

```tsx
const count = signal(0);

<p>{count.value}</p>
```

podemos construir:

```text
count
 │
 ▼
signal
 │
 ▼
count.value
 │
 ▼
<p> text node
```

Y con:

```tsx
<button onClick={() => count.value++}>
```

también:

```text
click
 │
 ▼
count.value++
 │
 ▼
signal mutation
 │
 ▼
<p>
```

Esto es lo que permitirá la reactividad fina.

------

# 13. El AST NO debe saber cómo funciona el DOM

Otra regla que establecería ahora:

> **AST no conoce el navegador.**

El AST solamente representa:

```text
component
element
expression
event
function
data
```

No:

```text
HTMLElement
DOM node
addEventListener
querySelector
```

Eso pertenece al IR/runtime.

------

# 14. Nuestro IR será diferente

Por ejemplo:

```tsx
<p>{count.value}</p>
```

AST:

```text
Element
└── Expression
    └── count.value
```

IR podría convertirse en algo conceptualmente parecido a:

```text
CreateElement("p")

CreateTextNode(
    binding = count.value
)

BindText(
    signal = count,
    target = text_node
)
```

Aquí ya estamos describiendo **operaciones**.

Por eso:

```text
AST = qué escribió
IR  = qué debe hacer Nexa
```

------

# 15. Otro ejemplo

Código:

```tsx
<button onClick={() => count.value++}>
    Increment
</button>
```

AST:

```text
Element(button)
├── Event(click)
│   └── Function
│       └── Assignment
└── Text("Increment")
```

IR:

```text
CreateElement(button)

SetText("Increment")

RegisterEvent(
    event = click,
    handler = increment_count
)
```

Posteriormente el optimizer podrá decidir cómo implementar eso de la manera más pequeña posible.

------

# 16. Y aquí aparece nuestra gran ventaja

Podemos cambiar el runtime.

Hoy:

```text
IR
 ↓
Runtime V1
```

Mañana:

```text
IR
 ↓
Runtime V10
```

El código del desarrollador:

```tsx
count.value++
```

no cambia.

Esto conecta directamente con nuestra regla de estabilidad de 10 años.

------

# 17. Estructura Rust inicial

Yo organizaría `nexa-ast` así:

```text
crates/nexa-ast/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── program.rs
    ├── component.rs
    ├── expression.rs
    ├── statement.rs
    ├── element.rs
    ├── attribute.rs
    ├── event.rs
    ├── types.rs
    └── source.rs
```

Y:

```text
crates/nexa-parser/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── parser.rs
    ├── typescript.rs
    ├── tsx.rs
    └── nexasyntax.rs
```

------

# 18. Source Location

Esto parece pequeño pero debemos implementarlo **desde el día uno**.

Cada nodo debe poder saber dónde apareció:

```text
source file
line
column
offset
```

Por ejemplo:

```rust
struct Span {
    start: u32,
    end: u32,
}
```

Entonces Nexa puede mostrar:

```text
src/pages/product.tsx:18:9
```

y:

```text
Error NX1004:
Interactive component requires activation.

18 | <button onClick={...}>
              ^^^^^^^^^
```

Esto será esencial para un buen developer experience.

------

# 19. Errores Nexa

También los vamos a centralizar:

```text
crates/nexa-errors/
```

Con códigos:

```text
NX1000
NX1001
NX1002
...
```

Por ejemplo:

```text
NX1001 — Invalid component declaration
NX2001 — Invalid route
NX3001 — SEO issue
NX4001 — Accessibility issue
```

Los códigos no deberían depender de la implementación interna.

Así documentación y herramientas pueden referirse a:

```text
NX1001
```

durante años.

------

# 20. Primera especificación de sintaxis

Por ahora propongo que Nexa reconozca:

```tsx
export component Name(props: Props) {
    return (
        ...
    );
}
```

Y:

```tsx
component Name(props: Props) {
    ...
}
```

si es un componente local.

No agregaría todavía:

```text
class component
decorators Nexa
templates propios
archivos .nexa
otro lenguaje
```

Queremos que Nexa siga siendo TypeScript/TSX.

------

# 21. Extensiones de archivo

También debemos decidirlo ahora.

Yo usaría:

```text
.ts
.tsx
```

exactamente igual que TypeScript.

No:

```text
.nexa
.nex
```

¿Por qué?

Porque:

```text
VS Code
TypeScript
linters
formatters
IDEs
GitHub
herramientas existentes
```

ya entienden `.ts` y `.tsx`.

Nexa agrega semántica sobre ellos.

------

# 22. Primer objetivo técnico

Por tanto, nuestro primer milestone será:

```text
NEXA-CORE-001
```

### Input

```tsx
export component App() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>My first Nexa application.</p>
        </main>
    );
}
```

### Parser

```text
Source
 ↓
AST
```

### Analyzer

Debe determinar:

```text
Component: App
Static: true
Interactive: false
Reactive: false
```

### Renderer

Debe producir:

```html
<main>
    <h1>Hello Nexa</h1>
    <p>My first Nexa application.</p>
</main>
```

Sin runtime.

------

# 23. Segundo milestone

Después añadimos:

```tsx
export component App() {

    const count = signal(0);

    return (
        <main>
            <h1>Counter</h1>

            <p>{count.value}</p>

            <button onClick={() => count.value++}>
                Increment
            </button>
        </main>
    );
}
```

Y esperamos:

```text
AST
 ↓
Analyzer
 ↓
Dependency Graph
 ↓
IR
 ↓
HTML
 +
Activation Manifest
 +
minimal runtime
```

------

# 24. Una decisión adicional que queda cerrada

**No vamos a implementar un Virtual DOM.**

Nuestro pipeline será:

```text
TypeScript / TSX
       ↓
      AST
       ↓
    Analyzer
       ↓
Dependency Graph
       ↓
      IR
       ↓
    Optimizer
       ↓
 ┌─────┴─────────┐
 ▼               ▼
HTML          Runtime
```

Eso encaja perfectamente con nuestra meta:

```text
SEO completo
+
HTML nativo
+
JS mínimo
+
reactividad fina
+
multiplataforma
```

------

## Siguiente paso

Con esto ya tenemos definido el **modelo conceptual del AST**.

El siguiente paso ya puede ser todavía más concreto: **diseñar el IR de Nexa 0.1 y el Dependency Graph**, porque ahí decidiremos exactamente cómo Nexa transforma:

```tsx
<h1>{product.name}</h1>
```

en una relación:

```text
product.name
      ↓
binding
      ↓
DOM text node
```

y cómo:

```tsx
<button onClick={() => count.value++}>
```

se convierte en una activación mínima.

Ese es el punto donde empezaremos a construir la característica que realmente diferencia a Nexa de Vue/React/Angular: **HTML primero y comportamiento conectado posteriormente, sin reconstruir la aplicación completa.**

Sí. Ahora pasamos al **núcleo más importante de Nexa 0.1**: el **IR (Intermediate Representation) + Dependency Graph**.

Aquí es donde dejamos de pensar simplemente en "convertir TSX a HTML" y empezamos a definir cómo Nexa sabrá **qué debe convertirse en HTML, qué necesita JavaScript y qué debe activarse en el navegador**.

# NEXA 0.1 — Paso 3: IR + Dependency Graph

Nuestro pipeline queda:

```text
TypeScript / TSX
       │
       ▼
    Parser
       │
       ▼
      AST
       │
       ▼
   Analyzer
       │
       ├───────────────┐
       ▼               ▼
Dependency Graph      IR
       │               │
       └───────┬───────┘
               ▼
           Optimizer
               │
        ┌──────┴──────┐
        ▼             ▼
      HTML          Runtime
```

------

# 1. ¿Por qué necesitamos IR?

El AST representa lo que escribió el desarrollador.

Por ejemplo:

```tsx
<h1>{product.name}</h1>
```

El AST podría decir:

```text
Element
└── h1
    └── Expression
        └── MemberAccess
            ├── product
            └── name
```

Pero el renderer necesita saber:

> "Tengo que crear un `<h1>`, insertar texto y mantener ese texto conectado con `product.name`."

Eso es responsabilidad del IR.

------

# 2. Nuestro IR no debe representar HTML

Esto también es importante.

No queremos que el IR sea simplemente:

```text
IR = HTML intermedio
```

Debe representar **operaciones semánticas de Nexa**.

Por ejemplo:

```text
CreateElement
InsertText
SetAttribute
BindText
ListenEvent
CreateComponent
Conditional
Loop
```

Así podemos utilizar el mismo IR para diferentes targets.

------

# 3. Primer IR

Propongo estas operaciones iniciales:

```text
CreateElement
CreateText
SetAttribute
SetProperty
AppendChild
InsertText
BindText
ListenEvent
Component
Fragment
If
Each
```

Más adelante:

```text
AsyncBoundary
Suspense
Stream
Portal
Teleport
Lazy
Activation
```

------

# 4. Ejemplo estático

Código:

```tsx
<main>
    <h1>Hello Nexa</h1>
</main>
```

IR:

```text
CreateElement(main)

CreateElement(h1)
CreateText("Hello Nexa")

AppendChild(h1, text)
AppendChild(main, h1)
```

Pero podemos optimizarlo.

El optimizer puede reconocer:

```text
TODO es estático
```

y generar directamente:

```html
<main>
    <h1>Hello Nexa</h1>
</main>
```

sin runtime.

------

# 5. Ejemplo dinámico

Ahora:

```tsx
<h1>{product.name}</h1>
```

IR:

```text
CreateElement(h1)

CreateText(binding: product.name)

AppendChild(h1, text)
```

Pero necesitamos registrar la dependencia:

```text
product.name
      │
      ▼
 text node
```

------

# 6. Dependency Graph

Este será uno de los componentes fundamentales del compiler.

Para:

```tsx
const product = getProduct();

<h1>{product.name}</h1>
```

tenemos:

```text
             product
                │
                ▼
              .name
                │
                ▼
             <h1>
                │
                ▼
            text node
```

Pero en una aplicación real tendremos un grafo mucho mayor:

```text
App
 │
 ├── Header
 │    └── User
 │
 ├── ProductPage
 │    ├── Product
 │    ├── Price
 │    └── AddToCart
 │
 └── Footer
```

Nexa podrá analizar qué partes dependen de qué datos.

------

# 7. Clasificación de cada nodo

Cada elemento del IR podrá clasificarse.

## Static

```tsx
<h1>Hello Nexa</h1>
STATIC
```

No necesita runtime.

------

## Dynamic

```tsx
<h1>{product.name}</h1>
DYNAMIC
```

Necesita una fuente de datos.

------

## Reactive

```tsx
<p>{count.value}</p>
```

si `count` es un signal:

```text
REACTIVE
```

------

## Interactive

```tsx
<button onClick={...}>
INTERACTIVE
```

Necesita event handling.

------

## Async

```tsx
const product = await getProduct();
ASYNC
```

Puede requerir streaming o boundary.

------

# 8. Esto nos permite hacer algo muy importante

Supongamos:

```tsx
<main>

    <h1>Producto</h1>

    <p>{product.name}</p>

    <p>{product.description}</p>

    <button onClick={addToCart}>
        Comprar
    </button>

</main>
```

Nexa podría producir:

```text
main
│
├── h1               STATIC
│
├── p                DYNAMIC
│
├── p                DYNAMIC
│
└── button           INTERACTIVE
```

Por lo tanto:

```text
HTML
├── h1
├── product.name
├── product.description
└── button

Runtime
└── button handler
```

No necesitamos convertir todo `<main>` en una aplicación JavaScript.

------

# 9. Esto es Progressive Activation

Aquí tenemos que establecer una definición oficial.

**Progressive Activation**:

> Nexa entrega primero el HTML necesario para representar la interfaz y activa únicamente las partes que requieren comportamiento del cliente.

Por ejemplo:

```text
Página
│
├── contenido
│    └── HTML
│
├── navegación
│    └── activation
│
├── carrito
│    └── activation
│
└── analytics
     └── activation
```

No:

```text
Página completa
      ↓
JavaScript
      ↓
reconstruir DOM
```

------

# 10. Activation Manifest

Para lograrlo necesitamos un artefacto intermedio.

Por ejemplo:

```json
{
  "components": {
    "AddToCart": {
      "events": ["click"],
      "chunk": "/assets/add-to-cart.js"
    }
  }
}
```

Pero esto es solamente conceptual.

Nuestro formato final debe ser binario/compacto y optimizable.

El build podría producir:

```text
dist/
├── index.html
├── assets/
│   ├── app.js
│   ├── add-to-cart.js
│   └── styles.css
└── .nexa/
    └── activation.bin
```

Aunque todavía no debemos fijar que necesariamente sea JSON.

------

# 11. ¿Por qué no usar JSON para todo?

Porque el manifest puede crecer mucho.

En desarrollo:

```json
activation.json
```

es excelente.

En producción:

```text
binary/compact representation
```

puede ser mejor.

Nexa podría incluso eliminar completamente información que el runtime no necesita.

------

# 12. Reactividad

Ahora el caso importante:

```tsx
const count = signal(0);

<p>{count.value}</p>

<button onClick={() => count.value++}>
    +
</button>
```

Dependency Graph:

```text
signal(count)
      │
      ├──────────────┐
      ▼              ▼
<p> text          button
      ▲              │
      │              ▼
      └──── mutation count
```

Nexa sabe:

```text
count
 ↓
text node
```

Entonces cuando cambia:

```text
count = 1
```

no hace:

```text
render App()
```

ni:

```text
render component tree
```

sino:

```text
update text node
```

Eso es fine-grained reactivity.

------

# 13. IR de ese ejemplo

Conceptualmente:

```text
SignalCreate
    id = count
    initial = 0

CreateElement(p)

CreateText
    binding = count.value

BindText
    source = count
    target = text

CreateElement(button)

CreateText("+") 

ListenEvent
    target = button
    event = click
    handler = increment(count)
```

------

# 14. Conditional rendering

Tenemos que contemplarlo desde el principio.

```tsx
{loggedIn && (
    <Dashboard />
)}
```

IR:

```text
Conditional
├── condition: loggedIn
└── true:
    └── Component(Dashboard)
```

Y:

```tsx
{loggedIn ? <Dashboard /> : <Login />}
Conditional
├── condition: loggedIn
├── true: Dashboard
└── false: Login
```

------

# 15. Loops

Para:

```tsx
<ul>
    {products.map(product => (
        <li>{product.name}</li>
    ))}
</ul>
```

IR:

```text
CreateElement(ul)

Each
├── source: products
├── item: product
└── template:
    └── Element(li)
        └── Bind(product.name)
```

Esto permitirá optimizaciones posteriores.

------

# 16. Keys

También debemos soportar:

```tsx
products.map(product => (
    <Product key={product.id} product={product} />
))
```

El IR debe conocer:

```text
Each
├── source
├── key
└── template
```

Pero no debemos implementar un Virtual DOM.

La key sirve para que el runtime conozca la identidad de los elementos.

------

# 17. Async

Ahora llegamos a algo especialmente importante para SEO.

Supongamos:

```tsx
export component ProductPage() {

    const product = await api.get('/products/123');

    return (
        <article>
            <h1>{product.name}</h1>
            <p>{product.description}</p>
        </article>
    );
}
```

Nexa puede generar HTML directamente:

```text
API
 ↓
data
 ↓
Nexa renderer
 ↓
HTML
```

No necesitamos:

```text
Browser
 ↓
JavaScript
 ↓
API
 ↓
render
```

para obtener el contenido inicial.

------

# 18. Streaming

Posteriormente podremos tener:

```tsx
<Suspense>
    <Recommendations />
</Suspense>
```

Y generar:

```text
HTML inicial
     ↓
stream
     ↓
recommendations
     ↓
stream
```

Esto será importante para ecommerce, news y aplicaciones grandes.

------

# 19. SEO Analyzer conectado al IR

Aquí aparece una arquitectura muy potente.

El analyzer puede recorrer:

```text
IR
```

y encontrar:

```text
Page
│
├── title
├── description
├── h1
├── article
├── canonical
├── structured data
└── links
```

Entonces Nexa puede advertir:

```text
NX3001
Missing <h1>

NX3002
Missing page title

NX3003
Image missing alt

NX3004
Canonical URL missing
```

Esto ocurre durante:

```bash
nexa build
```

------

# 20. El HTML es un producto del IR

Esta es una regla arquitectónica que debemos congelar:

```text
AST
 ↓
IR
 ↓
HTML
```

Nunca:

```text
AST
 ↓
HTML
 ↓
reparsear HTML
 ↓
runtime
```

Eso sería desperdiciar información.

------

# 21. Arquitectura del compiler

Entonces ahora nuestro compiler queda:

```text
                     SOURCE
                        │
                        ▼
                    PARSER
                        │
                        ▼
                       AST
                        │
                        ▼
                    ANALYZER
                        │
             ┌──────────┴──────────┐
             ▼                     ▼
      Dependency Graph             │
             │                     │
             └──────────┬──────────┘
                        ▼
                       IR
                        │
                        ▼
                    OPTIMIZER
                        │
             ┌──────────┼──────────┐
             ▼          ▼          ▼
           HTML       Runtime     Assets
             │          │
             └────┬─────┘
                  ▼
                BUNDLE
                  │
                  ▼
                 dist/
```

------

# 22. El optimizador será importantísimo

Por ejemplo:

```tsx
<h1>Hello Nexa</h1>
```

El optimizer dice:

```text
STATIC
```

Resultado:

```html
<h1>Hello Nexa</h1>
```

Sin:

```text
runtime
binding
JavaScript
```

Mientras:

```tsx
<p>{count.value}</p>
```

dice:

```text
REACTIVE
```

y genera solamente la infraestructura necesaria.

------

# 23. Resultado esperado

Para una página típica:

```text
100 elementos HTML
10 componentes
3 signals
4 eventos
2 API calls
```

Nexa podría producir:

```text
HTML:
100 elementos

Runtime:
solo 3 bindings + 4 eventos

JS:
chunks mínimos

CSS:
solo CSS utilizado

SEO:
metadata + structured data

Activation:
solo zonas interactivas
```

Ese es exactamente el comportamiento que estamos buscando.

------

# 24. Estructura Rust del IR

Ahora podemos definir:

```text
crates/nexa-ir/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── node.rs
    ├── element.rs
    ├── text.rs
    ├── binding.rs
    ├── event.rs
    ├── component.rs
    ├── conditional.rs
    ├── each.rs
    ├── async.rs
    ├── graph.rs
    └── source.rs
```

Y:

```text
crates/nexa-analyzer/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── analyzer.rs
    ├── dependencies.rs
    ├── reactivity.rs
    ├── interactivity.rs
    ├── async.rs
    ├── seo.rs
    └── accessibility.rs
```

------

# 25. Primera versión del IR

Para Nexa 0.1 podemos limitarlo a:

```text
Element
Text
Expression
Binding
Event
Component
Fragment
Conditional
Each
```

No necesitamos implementar todavía:

```text
Streaming
Suspense
Portals
Workers
Server Actions
```

Pero la arquitectura debe poder agregarlos posteriormente **sin romper el IR existente**.

------

# 26. La decisión que acabamos de cerrar

El Core de Nexa ahora tiene esta filosofía:

> **El AST conserva la intención del código; el Dependency Graph descubre relaciones; el IR describe operaciones; el Optimizer elimina todo lo innecesario; el Renderer produce HTML; el Runtime solamente recibe lo que realmente necesita comportamiento.**

Esto es la base de:

**SEO + HTML-first + progressive activation + fine-grained reactivity.**

------

## Siguiente paso

Ahora ya podemos pasar al siguiente nivel: **diseñar el Runtime de Nexa y el mecanismo exacto de Progressive Activation**.

Ahí responderemos técnicamente preguntas como:

```text
¿Cómo encuentra el runtime un botón?
¿Cómo conecta un evento sin hidratar todo el árbol?
¿Cómo actualiza un signal?
¿Cómo se cargan chunks bajo demanda?
¿Cómo se conserva el estado durante una navegación?
¿Cómo funciona el router sin reconstruir la página?
¿Cómo se hace la activación después de recibir HTML?
```

Ese paso será crucial porque ahí definiremos **cómo funciona Nexa realmente en el navegador**, no solamente cómo compila.

Perfecto. Ahora entramos en **Nexa Runtime + Progressive Activation**, que es probablemente la parte más importante después del compiler.

Hasta ahora definimos cómo Nexa construye la aplicación. Ahora definiremos **cómo se comporta esa aplicación en el navegador**.

# NEXA 0.1 — Paso 4: Runtime + Progressive Activation

La arquitectura completa pasa a ser:

```text
                    TypeScript / TSX
                           │
                           ▼
                        Parser
                           │
                           ▼
                          AST
                           │
                           ▼
                       Analyzer
                           │
                 ┌─────────┴─────────┐
                 ▼                   ▼
          Dependency Graph           IR
                 │                   │
                 └─────────┬─────────┘
                           ▼
                        Optimizer
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
            HTML         Runtime       Assets
              │            │
              └──────┬─────┘
                     ▼
                   Browser
                     │
                     ▼
             Progressive Activation
```

La idea fundamental:

> **El navegador recibe primero una interfaz funcionalmente representada en HTML y solamente descarga/activa el JavaScript que realmente necesita.**

------

# 1. No queremos "hydration" tradicional

Aquí quiero hacer una distinción importante.

En frameworks tradicionales, generalmente tenemos:

```text
HTML
 ↓
JavaScript
 ↓
reconstruir/recorrer árbol
 ↓
comparar estado
 ↓
conectar eventos
 ↓
hidratar
```

Eso puede funcionar muy bien, pero para Nexa queremos otra estrategia.

Nuestra terminología será:

## Progressive Activation

```text
HTML
 ↓
DOM existente
 ↓
detectar puntos de activación
 ↓
activar solamente esos puntos
```

No queremos reconstruir la aplicación completa.

------

# 2. Ejemplo

Tenemos:

```tsx
export component ProductPage() {
    return (
        <main>
            <h1>MacBook</h1>

            <p>Computadora portátil...</p>

            <button onClick={addToCart}>
                Agregar al carrito
            </button>
        </main>
    );
}
```

El HTML inicial puede ser:

```html
<main>
    <h1>MacBook</h1>
    <p>Computadora portátil...</p>

    <button data-nexa-a="1">
        Agregar al carrito
    </button>
</main>
```

El navegador ya puede mostrar:

```text
MacBook
Computadora portátil...
[Agregar al carrito]
```

**antes de cargar el código interactivo.**

------

# 3. Activation Marker

Nexa necesita identificar qué parte del HTML requiere comportamiento.

Podríamos utilizar:

```html
<button data-nexa-a="1">
```

o:

```html
<div data-nexa="component:abc">
```

Pero no quiero llenar el HTML de atributos innecesarios.

Por eso el optimizer debe decidir cuándo realmente hace falta.

Una página estática:

```html
<h1>Hello Nexa</h1>
<p>Contenido...</p>
```

no necesita:

```html
data-nexa="..."
```

porque no existe nada que activar.

------

# 4. El Activation Manifest

Durante build, Nexa conoce:

```text
Button
 ↓
click
 ↓
addToCart
 ↓
chunk cart.js
```

Puede generar algo conceptualmente así:

```json
{
  "1": {
    "event": "click",
    "chunk": "/assets/cart.js",
    "handler": "addToCart"
  }
}
```

El runtime sabe:

```text
<button data-nexa-a="1">
```

y puede asociarlo con:

```text
cart.js
```

------

# 5. ¿Qué ocurre cuando el usuario hace click?

Flujo:

```text
Usuario
  │
  ▼
click
  │
  ▼
Nexa Runtime
  │
  ├── ¿está activado?
  │       │
  │       └── NO
  │
  ▼
cargar chunk
  │
  ▼
registrar handler
  │
  ▼
ejecutar addToCart()
```

Después de la primera activación:

```text
button
 ↓
handler ya cargado
 ↓
clicks posteriores
 ↓
ejecución directa
```

------

# 6. Pero podemos hacerlo todavía mejor

No deberíamos esperar necesariamente al click.

Podemos tener diferentes estrategias:

```text
activation="interaction"
activation="visible"
activation="idle"
activation="load"
activation="manual"
```

Por ejemplo:

```tsx
<ProductReviews activation="visible" />
```

Cuando entra en el viewport:

```text
IntersectionObserver
        ↓
cargar reviews.js
        ↓
activar Reviews
```

------

# 7. Estrategias de activación

## `interaction`

```tsx
<Button activation="interaction">
```

Carga cuando el usuario interactúa.

Ideal para:

```text
menus
dialogs
buttons
dropdowns
```

------

## `visible`

```tsx
<Comments activation="visible" />
```

Se activa cuando aparece en pantalla.

Ideal para:

```text
comments
recommendations
charts
widgets
```

------

## `idle`

```tsx
<Analytics activation="idle" />
```

Se ejecuta cuando el navegador está libre.

------

## `load`

Para elementos que realmente necesitan ejecutarse inmediatamente.

```tsx
<AppShell activation="load" />
```

------

## `manual`

El programador decide:

```ts
activate(component);
```

Esto sería más avanzado.

------

# 8. No todo componente necesita activación

Este concepto será fundamental.

Tenemos:

```tsx
export component Article() {
    return (
        <article>
            <h1>Noticias de hoy</h1>
            <p>Contenido...</p>
        </article>
    );
}
```

Nexa determina:

```text
Article
STATIC
```

Resultado:

```html
<article>
    <h1>Noticias de hoy</h1>
    <p>Contenido...</p>
</article>
```

JavaScript necesario:

```text
0 bytes
```

si no existen otras dependencias.

Esto es especialmente bueno para:

- noticias
- blogs
- documentación
- landing pages
- ecommerce
- contenido SEO.

------

# 9. Reactividad

Ahora:

```tsx
const count = signal(0);

<p>{count.value}</p>
```

Nexa genera una relación:

```text
signal(count)
       │
       ▼
text binding
       │
       ▼
<p>
```

Cuando:

```ts
count.value++
```

el runtime hace:

```text
signal changed
      ↓
binding notified
      ↓
text node updated
```

No:

```text
App()
 ↓
render()
 ↓
diff()
 ↓
patch entire tree
```

------

# 10. El Runtime será extremadamente pequeño

Queremos algo parecido conceptualmente a:

```text
@nexa/runtime
```

que contenga solamente:

```text
signals
bindings
events
activation
DOM operations
scheduler
router integration
```

Y cada aplicación puede utilizar solamente una parte.

El bundler hace tree-shaking:

```text
App
 ↓
usa signal
 ↓
runtime signal
 ↓
solo esa parte
```

------

# 11. Scheduler

Necesitamos un scheduler interno.

Por ejemplo:

```text
signal change
      ↓
queue update
      ↓
scheduler
      ↓
batch
      ↓
DOM update
```

Si ocurre:

```ts
count.value++;
name.value = "Hector";
price.value = 100;
```

no queremos:

```text
DOM update
DOM update
DOM update
```

Queremos:

```text
changes
   ↓
batch
   ↓
scheduler
   ↓
minimal DOM updates
```

------

# 12. Estado

Aquí debemos separar tres tipos de estado.

## Local state

```ts
const open = signal(false);
```

Pertenece al componente.

------

## Application state

```ts
const user = store(...);
```

Puede compartirse.

------

## Server state

```ts
const product = query(...);
```

Proviene del backend.

No debemos mezclarlos.

La arquitectura será:

```text
              State
                │
       ┌────────┼────────┐
       ▼        ▼        ▼
     Local      App     Server
     Signal    Store    Query
```

Esto evitará una gran cantidad de problemas que aparecen cuando todo termina metido en un único store.

------

# 13. Server Data

Por ejemplo:

```ts
const product = query({
    key: ['product', id],

    fetch: () =>
        http.get(`/products/${id}`)
});
```

Durante generación inicial:

```text
Route
 ↓
query
 ↓
API
 ↓
data
 ↓
HTML
```

El HTML puede contener:

```html
<h1>Producto XYZ</h1>
<p>Descripción...</p>
```

El navegador no necesita volver a pedir inmediatamente lo mismo.

------

# 14. Transfer State

Para evitar una segunda petición:

```text
Server/build
   ↓
data
   ↓
HTML
```

y también:

```text
serialized state
```

El runtime puede recuperar ese estado.

Conceptualmente:

```html
<script type="application/nexa-state">
...
</script>
```

Pero aquí debemos ser muy cuidadosos.

No debemos introducir:

```text
JSON enorme
```

en todas las páginas.

El optimizer debe serializar solamente los datos que el cliente realmente necesita.

------

# 15. Router + Runtime

Ahora viene algo importante.

Cuando el usuario navega:

```text
/products
   ↓
/products/123
```

Nexa puede hacer:

```text
click
 ↓
router
 ↓
prefetch route
 ↓
load required data
 ↓
update DOM
 ↓
update URL
```

sin recargar toda la página.

Pero si Google entra directamente:

```text
/products/123
```

el servidor/CDN entrega:

```html
<h1>Producto 123</h1>
```

directamente.

Esto nos da:

```text
SEO
+
SPA-like navigation
```

------

# 16. Prefetch

Podemos tener:

```tsx
<Link href="/products/123">
```

y Nexa puede detectar:

```text
usuario mueve mouse
       ↓
prefetch
       ↓
route chunk
       +
data
```

Entonces cuando hace click:

```text
navigation
 ↓
instantánea
```

Pero nuevamente:

**prefetch debe ser inteligente**, no descargar toda la aplicación.

------

# 17. Router Architecture

El router tendrá tres niveles:

```text
URL
 │
 ▼
Route Matcher
 │
 ▼
Route Module
 │
 ▼
Page
```

Y una navegación:

```text
Navigation
   │
   ├── URL update
   ├── route loading
   ├── data loading
   ├── component loading
   └── DOM transition
```

------

# 18. Navegación sin perder estado

Supongamos:

```text
Layout
├── Header
├── Sidebar
└── Main
```

Cambias:

```text
/products
```

a:

```text
/products/123
```

Nexa no tiene por qué destruir:

```text
Header
Sidebar
```

Puede actualizar solamente:

```text
Main
```

Esto se desprende del Dependency Graph + Router Tree.

------

# 19. Eventos

No queremos hacer:

```js
button.addEventListener(...)
```

para cada elemento inmediatamente.

Podemos usar **event delegation** cuando sea beneficioso.

Por ejemplo:

```text
document
   │
   ▼
Nexa Event Dispatcher
   │
   ├── click
   ├── input
   ├── submit
   └── change
```

Y el manifest determina dónde debe ir el evento.

Esto puede reducir listeners.

Pero el compiler debe elegir entre:

```text
delegation
```

y:

```text
direct listener
```

según el caso.

------

# 20. Forms

Esto conecta con nuestro paquete oficial `@nexa/forms`.

Un formulario:

```tsx
<form onSubmit={submit}>
    <input name="email" />
    <button>Enviar</button>
</form>
```

debe seguir siendo:

**HTML nativo.**

No queremos:

```text
div
 ↓
JS
 ↓
simular formulario
```

El navegador debe entender:

```html
<form>
<input>
<button>
```

Incluso sin JavaScript cuando sea posible.

------

# 21. Progressive Enhancement

Esta será otra regla de Nexa:

> **Si una funcionalidad puede funcionar correctamente con HTML nativo, Nexa no debe obligar a utilizar JavaScript.**

Ejemplo:

```html
<form method="post" action="/login">
```

es válido.

Después Nexa puede activar:

```text
AJAX
validation
loading state
optimistic UI
```

pero el HTML sigue siendo válido.

------

# 22. Accessibility

Esto también afecta al runtime.

Por ejemplo:

```tsx
<Dialog>
```

Nexa UI deberá generar correctamente:

```text
role
aria-modal
focus management
keyboard navigation
focus restoration
```

Pero la base sigue siendo HTML semántico.

------

# 23. Error boundaries

Necesitamos definir cómo se comporta Nexa cuando falla:

```text
component
API
route
lazy chunk
```

Por ejemplo:

```tsx
<ErrorBoundary>
    <Product />
</ErrorBoundary>
```

IR:

```text
ErrorBoundary
└── Product
```

Y runtime:

```text
error
 ↓
boundary
 ↓
fallback
```

Esto debemos incorporarlo desde temprano aunque su implementación completa pueda esperar.

------

# 24. Loading boundaries

Igualmente:

```tsx
<Suspense>
    <Recommendations />
</Suspense>
```

podrá representar:

```text
loading
   ↓
component
   ↓
resolved
```

Pero el objetivo será que esto funcione también con streaming posteriormente.

------

# 25. El Runtime completo

Conceptualmente:

```text
@nexa/runtime
│
├── activation
│
├── reactivity
│   ├── signal
│   ├── computed
│   ├── effect
│   └── scheduler
│
├── dom
│   ├── create
│   ├── update
│   ├── remove
│   └── events
│
├── router
│
├── state
│
├── async
│
└── errors
```

Pero **el usuario no tendrá que importar todo esto manualmente**.

El compiler determina qué necesita.

------

# 26. Ejemplo completo

Código Nexa:

```tsx
export component Counter() {

    const count = signal(0);

    return (
        <section>
            <h1>Counter</h1>

            <p>{count.value}</p>

            <button onClick={() => count.value++}>
                Increment
            </button>
        </section>
    );
}
```

Build:

```text
AST
 ↓
Analyzer
 ↓
Dependency Graph

h1 → STATIC
p → REACTIVE
button → INTERACTIVE

 ↓
IR
 ↓
Optimizer
```

Resultado conceptual:

```text
HTML
├── <section>
├── <h1>Counter</h1>
├── <p>0</p>
└── <button>Increment</button>

Runtime
├── signal
├── binding
└── click handler
```

No:

```text
Virtual DOM
```

No:

```text
full hydration
```

No:

```text
re-render entire component
```

------

# 27. El navegador

Carga:

```text
index.html
```

Primero ve:

```text
Counter
0
[Increment]
```

Luego Nexa Runtime encuentra:

```text
button → activation
```

Carga el código necesario.

Click:

```text
count.value++
```

El runtime sabe:

```text
count
 ↓
<p>
```

Actualiza:

```html
<p>1</p>
```

Y solamente eso.

------

# 28. Esto también funciona para ecommerce

Por ejemplo:

```text
Producto
├── nombre       → HTML
├── descripción  → HTML
├── precio       → HTML
├── imagen       → HTML
├── reviews      → HTML
├── carrito      → interactive
├── favoritos    → interactive
└── filtros      → interactive
```

Podemos tener:

```text
SEO:
████████████████████

JS:
       ██
       ██
       ██
```

En lugar de:

```text
SEO:
████████████████████

JS:
████████████████████
```

------

# 29. Arquitectura final hasta este punto

Ahora tenemos:

```text
                         NEXA
                          │
                ┌─────────┴─────────┐
                │                   │
             Compiler             Runtime
                │                   │
        ┌───────┼───────┐     ┌─────┼─────┐
        ▼       ▼       ▼     ▼     ▼     ▼
       AST      IR    Graph   DOM  State Router
        │       │       │
        └───────┼───────┘
                ▼
             Optimizer
                │
       ┌────────┼────────┐
       ▼        ▼        ▼
     HTML      JS       CSS
       │        │
       └────┬───┘
            ▼
         Browser
            │
            ▼
   Progressive Activation
```

Y la regla fundamental queda:

> **HTML primero. JavaScript cuando sea necesario. Activación por unidad de comportamiento.**

------

# 30. Lo que NO vamos a hacer todavía

No vamos a saltar directamente a:

- UI completa.
- Forms completos.
- i18n.
- Tauri.
- Capacitor.
- Auth.
- WebSocket.
- PWA.

Primero debemos demostrar que el mecanismo central funciona.

El siguiente paso lógico será **Nexa Router + Data Layer**, porque necesitamos resolver una cuestión fundamental:

```text
URL
 ↓
Route
 ↓
API
 ↓
Data
 ↓
HTML SEO
 ↓
Progressive Activation
```

Ahí construiremos el flujo que permitirá que una página como:

```text
/productos/iphone-17
```

sea **SEO completa desde la primera respuesta**, pero después se comporte como una aplicación moderna sin recargar toda la página.

Perfecto. El siguiente paso será **Nexa Router + Data Layer**. Aquí vamos a resolver una de las piezas que hacen que Nexa sea realmente diferente: **la URL y los datos del backend deben poder producir HTML SEO completo sin convertir el backend en un servidor SSR de Nexa**.

# NEXA 0.1 — Paso 5: Router + Data Layer

La arquitectura queda:

```text
                    REQUEST
                       │
                       ▼
                  /products/123
                       │
                       ▼
                  Nexa Router
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
         Route Match          Route Data
             │                   │
             │                   ▼
             │                  API
             │                   │
             │                   ▼
             │                 JSON
             │                   │
             └──────────┬────────┘
                        ▼
                   Page Model
                        │
                        ▼
                       IR
                        │
              ┌─────────┴─────────┐
              ▼                   ▼
             HTML              Activation
              │                   │
              └─────────┬─────────┘
                        ▼
                     Browser
```

Pero hay una distinción fundamental.

## El backend NO renderiza Nexa

El backend puede ser:

```text
PHP
Go
Python
Rust
Java
.NET
etc.
```

y simplemente proporcionar:

```http
GET /api/products/123
{
  "id": 123,
  "name": "Producto XYZ",
  "price": 199
}
```

Nexa se encarga de transformar esos datos en HTML.

------

# 1. Router de Nexa

El desarrollador podrá definir:

```text
src/
├── pages/
│   ├── index.tsx
│   ├── about.tsx
│   └── products/
│       ├── index.tsx
│       └── [id].tsx
```

Y Nexa generará:

```text
/
 /about
 /products
 /products/:id
```

No quiero que el programador tenga que configurar manualmente cada ruta en un archivo gigante.

------

# 2. File-based routing

Usaremos una convención similar a:

```text
pages/
```

pero con reglas propias de Nexa.

Ejemplo:

```text
pages/
├── index.tsx
├── about.tsx
├── products/
│   ├── index.tsx
│   └── [id].tsx
└── news/
    └── [slug].tsx
```

Resultado:

```text
/
 /about
 /products
 /products/:id
 /news/:slug
```

------

# 3. Dynamic routes

Archivo:

```text
products/[id].tsx
```

representa:

```text
/products/:id
```

Por ejemplo:

```text
/products/10
/products/25
/products/500
```

El componente puede recibir:

```ts
type Props = {
    id: string;
};
```

y Nexa resolverá:

```text
URL
 ↓
id = 123
```

------

# 4. Route Object

Internamente:

```rust
struct Route {
    path: String,
    component: ComponentId,
    loader: Option<LoaderId>,
    metadata: Option<MetadataId>,
}
```

Pero no debemos exponer esta estructura al desarrollador.

El developer trabaja con:

```tsx
export component ProductPage({ id }) {
    ...
}
```

------

# 5. Route Data

Aquí introducimos una API oficial:

```ts
export async function load({ params, api }) {
    return api.get(`/products/${params.id}`);
}
```

Entonces:

```text
/products/123
       │
       ▼
params.id = 123
       │
       ▼
load()
       │
       ▼
GET /api/products/123
       │
       ▼
JSON
```

Y el resultado se entrega al componente.

------

# 6. API Client oficial

En lugar de que cada desarrollador implemente Axios manualmente, Nexa tendrá:

```ts
import { api } from '@nexa/http';
```

Pero además quiero que Nexa permita utilizar:

```ts
fetch()
```

directamente.

Y opcionalmente:

```ts
axios
```

si el desarrollador lo desea.

No debemos crear una dependencia obligatoria.

------

# 7. `@nexa/http`

El adapter oficial podría proporcionar:

```ts
api.get()
api.post()
api.put()
api.patch()
api.delete()
```

Además:

```ts
api.request()
```

Con:

```ts
api.interceptors
```

si posteriormente necesitamos:

```text
auth
logging
retry
headers
refresh token
```

------

# 8. Data Layer

Quiero separar claramente:

```text
Route Data
Component Data
Client State
```

### Route Data

Datos necesarios para construir la página.

```text
Product
Article
News
Category
```

### Component Data

Datos que un componente puede solicitar posteriormente.

```text
Reviews
Recommendations
Notifications
```

### Client State

Estado local:

```text
cart
modal
filters
theme
```

No deben mezclarse.

------

# 9. Ejemplo ecommerce

```tsx
export component ProductPage({ data }) {

    return (
        <article>
            <h1>{data.name}</h1>

            <p>{data.description}</p>

            <strong>
                {data.price}
            </strong>

            <AddToCart product={data} />
        </article>
    );
}
```

Loader:

```ts
export async function load({ params, api }) {

    return api.get(
        `/products/${params.id}`
    );
}
```

Pipeline:

```text
URL
 │
 ▼
/products/123
 │
 ▼
Route
 │
 ▼
load()
 │
 ▼
API
 │
 ▼
JSON
 │
 ▼
ProductPage
 │
 ▼
IR
 │
 ▼
HTML
```

Resultado:

```html
<article>
    <h1>Producto XYZ</h1>
    <p>Descripción...</p>
    <strong>199</strong>
</article>
```

------

# 10. Esto resuelve nuestro problema SEO

Google no necesita ejecutar:

```text
JavaScript
 ↓
fetch
 ↓
render
```

para descubrir:

```html
<h1>Producto XYZ</h1>
<p>Descripción...</p>
```

El HTML inicial ya contiene esos elementos.

Y además tenemos:

```html
<title>Producto XYZ</title>
<meta name="description" ...>

<h1>Producto XYZ</h1>
```

------

# 11. ¿Pero quién ejecuta `load()`?

Aquí hay una cuestión arquitectónica muy importante.

**No queremos convertir esto en SSR Node.**

El proceso Nexa puede ejecutarse de distintas formas.

## Modo producción con servidor compatible

```text
Browser
   │
   ▼
Nexa runtime/server adapter
   │
   ├── route
   ├── API
   └── renderer
   │
   ▼
HTML
```

Ese servidor puede ser:

```text
Nexa server
```

compilado en Rust.

No Node.

------

# 12. Pero también podemos tener Static Generation

Para contenido que no cambia continuamente:

```text
nexa build
```

puede hacer:

```text
Route
 ↓
load()
 ↓
API
 ↓
JSON
 ↓
HTML
```

y generar:

```text
dist/
├── index.html
├── about/
│   └── index.html
├── products/
│   └── 123/
│       └── index.html
└── news/
    └── article/
        └── index.html
```

Entonces podemos desplegarlo incluso en:

```text
Nginx
Apache
CDN
GitHub Pages
Cloudflare Pages
servidor estático
```

sin ejecutar Nexa en producción.

------

# 13. Tres modos de renderizado

Esto debemos definirlo ahora.

## SSG

```text
Build
 ↓
API
 ↓
HTML
```

Ideal para:

```text
documentation
blog
marketing
catalog
news
```

------

## ISR / Regeneration

Más adelante:

```text
HTML cacheado
 ↓
revalidate
 ↓
API
 ↓
nuevo HTML
```

No necesitamos implementarlo en 0.1, pero la arquitectura debe permitirlo.

------

## Dynamic Rendering

Para páginas que dependen de información en tiempo real:

```text
Request
 ↓
Route
 ↓
API
 ↓
HTML
```

Aquí entra nuestro runtime/server en Rust.

------

# 14. Y aquí aparece una diferencia importante

No estamos diciendo:

> "Nexa necesita un servidor."

Estamos diciendo:

> "Nexa puede producir HTML de varias maneras."

```text
                  Nexa Compiler
                       │
             ┌─────────┼──────────┐
             ▼         ▼          ▼
            SSG       Dynamic    Client
             │         │          │
             ▼         ▼          ▼
           HTML      HTML       HTML
```

Por eso un proyecto Nexa puede ser:

```text
100% estático
```

o:

```text
dinámico
```

o:

```text
híbrido
```

------

# 15. Navegación del lado cliente

Una vez cargada la aplicación:

```text
/products
```

usuario hace click:

```text
/products/123
```

El runtime puede hacer:

```text
Router
 ↓
fetch route data
 ↓
fetch component chunk
 ↓
update DOM
 ↓
history.pushState()
```

Sin recarga completa.

------

# 16. Pero debemos conservar una regla

Si JavaScript está desactivado:

```text
/products
```

debe seguir funcionando cuando el servidor tenga HTML generado.

Entonces:

```html
<a href="/products/123">
```

sigue siendo un enlace HTML real.

No:

```html
<div onclick="navigate(...)">
```

------

# 17. Prefetch del router

Nexa puede hacer:

```text
hover / visible
       ↓
prefetch route
       ↓
component chunk
       +
data
```

Pero tendremos una política:

```text
prefetch = optional
```

El navegador nunca debe necesitarlo para funcionar.

------

# 18. Metadata de rutas

Cada página podrá declarar:

```ts
export const meta = ({ data }) => ({
    title: data.name,
    description: data.description,
    canonical: `/products/${data.id}`
});
```

Nexa genera:

```html
<title>Producto XYZ</title>

<meta
    name="description"
    content="Descripción..."
>

<link
    rel="canonical"
    href="/products/123"
>
```

------

# 19. Open Graph

También:

```ts
export const meta = ({ data }) => ({
    title: data.name,

    openGraph: {
        title: data.name,
        description: data.description,
        image: data.image
    }
});
```

Genera:

```html
<meta property="og:title">
<meta property="og:description">
<meta property="og:image">
```

------

# 20. JSON-LD

Y Nexa podrá tener una API SEO oficial:

```ts
export const structuredData = ({ data }) => ({
    "@type": "Product",
    name: data.name,
    description: data.description,
    offers: {
        price: data.price
    }
});
```

Nexa genera:

```html
<script type="application/ld+json">
...
</script>
```

Así el SEO no queda reducido a:

```text
title
description
```

sino que podemos construir:

```text
HTML semántico
+
Metadata
+
OpenGraph
+
Canonical
+
JSON-LD
+
Links
+
sitemap
+
robots
```

------

# 21. SEO Analyzer

El compiler puede revisar:

```text
NX3001 Missing title
NX3002 Missing description
NX3003 Missing H1
NX3004 Multiple H1
NX3005 Missing canonical
NX3006 Image without alt
NX3007 Invalid structured data
NX3008 Broken internal link
```

Al hacer:

```bash
nexa build
```

podríamos obtener:

```text
Nexa SEO Analysis

/products/123

✓ title
✓ description
✓ canonical
✓ h1
✓ structured data
✓ image alt

SEO score: OK
```

Esto será una herramienta del compiler, no una dependencia externa.

------

# 22. API Adapter

Aquí retomamos algo que habíamos definido antes.

Nexa no debe asumir:

```text
REST
```

solamente.

El Data Layer debe soportar adapters:

```text
@nexa/http
@nexa/graphql
@nexa/rpc
@nexa/custom
```

Conceptualmente:

```text
               Data Layer
                    │
       ┌────────────┼─────────────┐
       ▼            ▼             ▼
      REST       GraphQL         RPC
       │            │             │
       └────────────┼─────────────┘
                    ▼
                Page Data
```

Así el backend puede ser:

```text
PHP REST API
Go REST API
Python GraphQL
Rust RPC
```

y Nexa no depende de ninguno.

------

# 23. Cache

También necesitamos definir esto desde ahora.

El Data Layer debe distinguir:

```text
no-cache
cache
revalidate
immutable
```

Ejemplo:

```ts
const product = query({
    key: ['product', id],
    staleTime: 60_000
});
```

Pero la API exacta todavía no la congelaría.

La arquitectura sí:

```text
Request
 ↓
Cache
 ↓
Data source
 ↓
Response
```

------

# 24. Error de API

Debe existir un modelo estándar:

```text
Route
 │
 ▼
Data loader
 │
 ├── success
 │
 ├── 404
 │
 ├── 401
 │
 ├── 403
 │
 └── 500
```

Por ejemplo:

```ts
throw notFound();
```

y Nexa puede producir:

```text
HTTP 404
+
404 HTML
+
SEO metadata
```

No una pantalla JavaScript de error.

------

# 25. Redirects

Igualmente:

```ts
redirect('/login');
```

debe poder producir:

```text
HTTP 302/303
```

en rendering dinámico.

Mientras que navegación cliente puede utilizar:

```text
history.pushState()
```

según el caso.

------

# 26. Una decisión especialmente importante

**La URL siempre es una fuente de verdad.**

No:

```text
state
 ↓
decide URL
```

sino:

```text
URL
 ↓
route
 ↓
page
 ↓
state
```

Esto facilita:

- SEO
- bookmarks
- navegación
- SSR/SSG
- accesibilidad
- refresh
- deep links

------

# 27. Arquitectura del Router

En Rust:

```text
crates/nexa-router/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── matcher.rs
    ├── route.rs
    ├── params.rs
    ├── tree.rs
    ├── navigation.rs
    ├── prefetch.rs
    └── history.rs
```

Data Layer:

```text
crates/nexa-data/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── loader.rs
    ├── query.rs
    ├── cache.rs
    ├── request.rs
    ├── response.rs
    ├── errors.rs
    └── adapters.rs
```

SEO:

```text
crates/nexa-seo/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── metadata.rs
    ├── structured_data.rs
    ├── analyzer.rs
    ├── sitemap.rs
    └── robots.rs
```

------

# 28. Nuestro flujo completo ahora

Tenemos:

```text
                         NEXA
                          │
              ┌───────────┴───────────┐
              │                       │
           COMPILER                 RUNTIME
              │                       │
              ▼                       ▼
             AST                    Router
              │                       │
              ▼                       ▼
           Analyzer                Data Layer
              │                       │
         ┌────┴────┐                  │
         ▼         ▼                  ▼
       Graph      IR                 API
         │         │                  │
         └────┬────┘                  │
              ▼                       │
          Optimizer                  │
              │                       │
       ┌──────┼──────┐               │
       ▼      ▼      ▼               │
      HTML    JS    CSS              │
       │      │                       │
       └──────┼───────────────────────┘
              ▼
            Browser
```

------

# 29. Lo que acabamos de conseguir

Nexa ahora tiene una arquitectura donde:

### Backend

Puede ser:

```text
PHP
Go
Python
Rust
Java
etc.
```

### Comunicación

Puede ser:

```text
REST
JSON
GraphQL
RPC
custom adapter
```

### Rendering

Puede ser:

```text
SSG
Dynamic
Hybrid
Client
```

### Frontend

Es:

```text
TypeScript
TSX
Nexa Runtime
```

### Compiler

Es:

```text
Rust
```

### Node

**No es necesario para ejecutar ni construir una aplicación Nexa.**

Node puede existir opcionalmente para herramientas externas, pero Nexa no depende de él.

------

# 30. Y esto nos lleva al siguiente paso

Ya tenemos las cuatro piezas críticas:

```text
1. AST
2. IR + Dependency Graph
3. Runtime + Progressive Activation
4. Router + Data Layer
```

Ahora debemos entrar en una parte que tú ya habías señalado anteriormente y que conviene diseñar **antes de escribir código serio**:

# NEXA 0.1 — Paso 6: UI Core + CSS + Forms + i18n + Plugins/Adapters

Aquí vamos a definir:

```text
@nexa/ui
@nexa/forms
@nexa/i18n
@nexa/router
@nexa/http
@nexa/runtime
@nexa/seo
```

y, especialmente, cómo hacer que Nexa tenga una experiencia tipo **Quasar**:

```text
nexa.config.ts
        │
        ├── UI
        ├── CSS
        ├── fonts
        ├── icons
        ├── boot
        ├── plugins
        ├── aliases
        ├── i18n
        └── adapters
```

sin convertir todo eso en dependencias obligatorias ni crear un framework monolítico difícil de mantener.

Ese será el siguiente paso antes de entrar finalmente en la **estructura física del repositorio y comenzar a escribir el primer código Rust de Nexa**.

Perfecto. Ahora definimos el **Paso 6: UI Core + CSS + Forms + i18n + Plugins + Adapters**. Esta parte es importante porque aquí Nexa deja de ser solamente un compiler/runtime y empieza a convertirse en un **framework completo**.

La regla será:

> **Nexa Core debe ser pequeño y estable; las capacidades adicionales deben ser modulares, oficiales y versionadas con compatibilidad a largo plazo.**

------

# NEXA 0.1 — Paso 6: UI y sistema de extensiones

La arquitectura propuesta:

```text
NEXA
│
├── Core
│   ├── Compiler
│   ├── Runtime
│   ├── Router
│   ├── Data
│   └── Reactivity
│
├── Official
│   ├── UI
│   ├── Forms
│   ├── i18n
│   ├── HTTP
│   ├── SEO
│   ├── PWA
│   └── Testing
│
└── Adapters
    ├── Capacitor
    ├── Tauri
    ├── REST
    ├── GraphQL
    └── Custom
```

------

# 1. Primero: UI no debe estar dentro del Compiler

No quiero esto:

```text
Nexa Compiler
 ├── Button
 ├── Dialog
 ├── Input
 ├── Table
 └── ...
```

Porque terminaríamos con un framework monolítico.

En cambio:

```text
Compiler
   │
   └── understands components
                 ▲
                 │
              @nexa/ui
```

El compiler sabe qué es un componente.

No necesita saber qué es un `NButton`.

------

# 2. `@nexa/ui`

Este será nuestro equivalente conceptual a la experiencia UI de Quasar.

Tendremos componentes:

```text
@nexa/ui
│
├── Layout
├── Container
├── Button
├── Input
├── Select
├── Checkbox
├── Radio
├── Switch
├── Dialog
├── Drawer
├── Menu
├── Dropdown
├── Tabs
├── Card
├── Table
├── Pagination
├── Tooltip
├── Notification
├── Loading
├── Avatar
├── Badge
├── Breadcrumb
└── ...
```

Pero habrá una diferencia fundamental:

> Los componentes deben producir **HTML semántico y accesible**, no simplemente divs estilizados.

------

# 3. UI sin dependencia obligatoria de un CSS framework

El desarrollador no debería necesitar:

```text
Bootstrap
Tailwind
DaisyUI
Material
```

para utilizar Nexa UI.

Por ejemplo:

```tsx
import { Button } from '@nexa/ui';

<Button>
    Guardar
</Button>
```

Nexa proporciona:

```text
HTML
CSS
behavior
accessibility
```

------

# 4. Pero CSS debe seguir siendo reemplazable

Esto es importante para no encerrar al desarrollador.

Debe poder utilizar:

```text
Nexa UI
```

o:

```text
Tailwind
```

o:

```text
CSS propio
```

o:

```text
otro sistema UI
```

Por tanto:

```text
Nexa
│
├── UI oficial
│
└── CSS externo
```

son compatibles.

------

# 5. Diseño visual configurable

Queremos una experiencia parecida a:

```ts
nexa.config.ts
```

Por ejemplo:

```ts
export default defineConfig({
    ui: {
        theme: 'default',
        darkMode: true
    }
});
```

Pero quiero llevarlo más lejos.

Podríamos tener:

```ts
ui: {
    primary: '#1976d2',
    secondary: '#26a69a',
    accent: '#9c27b0',
    dark: true
}
```

Nexa genera las variables CSS.

Por ejemplo:

```css
:root {
    --nexa-primary: ...;
    --nexa-secondary: ...;
}
```

Los componentes utilizan tokens, no colores rígidos.

------

# 6. Design Tokens

Esto debería estar en el Core UI.

```text
@nexa/ui
   │
   ▼
Design Tokens
```

Categorías:

```text
colors
spacing
typography
radius
elevation
breakpoints
transitions
z-index
```

Por ejemplo:

```ts
theme: {
    colors: {
        primary: '#...',
        secondary: '#...'
    },

    spacing: {
        sm: '8px',
        md: '16px',
        lg: '24px'
    }
}
```

Esto hace que el UI sea personalizable sin modificar los componentes.

------

# 7. CSS Architecture

No quiero que Nexa dependa de:

```text
runtime CSS-in-JS
```

para cada componente.

Preferimos CSS estático generado durante build.

```text
Component
 ↓
compiler
 ↓
CSS extraction
 ↓
optimizer
 ↓
styles.css
```

Así el navegador recibe:

```text
HTML
+
CSS
```

sin necesidad de ejecutar JavaScript para pintar la UI.

------

# 8. CSS Tree Shaking

Si utilizas:

```tsx
<Button />
<Card />
```

pero nunca:

```tsx
<Calendar />
<Drawer />
<Table />
```

el build debería poder eliminar CSS innecesario.

```text
@nexa/ui
   │
   ├── Button.css ✓
   ├── Card.css ✓
   ├── Calendar.css ✗
   ├── Drawer.css ✗
   └── Table.css ✗
```

Resultado:

```text
CSS mínimo.
```

------

# 9. Fonts

Esto también debe entrar en la configuración.

```ts
export default defineConfig({
    fonts: {
        families: [
            'Inter',
            'Roboto'
        ]
    }
});
```

Nexa puede gestionar:

```text
font loading
preload
fallback
subset
local fonts
web fonts
```

Pero no debe obligar a usar una fuente específica.

------

# 10. Icons

Igual:

```ts
icons: {
    provider: 'nexa'
}
```

Pero también:

```text
Lucide
Material Icons
Font Awesome
SVG propios
```

mediante adapters.

Idealmente:

```tsx
<Icon name="search" />
```

y el compiler puede convertirlo en SVG optimizado.

------

# 11. Notifications

Esto sí debería ser una capacidad oficial.

```ts
import { notify } from '@nexa/ui';

notify({
    message: 'Guardado correctamente'
});
```

Pero internamente:

```text
notify()
   ↓
Notification service
   ↓
UI component
```

No queremos que cada componente tenga que implementar su propio sistema.

------

# 12. Loading

También:

```ts
loading.show();
```

y:

```ts
loading.hide();
```

Pero debe existir una versión declarativa:

```tsx
<Button loading={saving}>
    Guardar
</Button>
```

Entonces Nexa UI sabe:

```text
saving = true
 ↓
button disabled
 ↓
loading indicator
```

------

# 13. Boot System

Aquí sí podemos adoptar una idea muy buena de Quasar.

Nexa tendrá:

```text
src/boot/
```

Ejemplo:

```text
src/
├── boot/
│   ├── api.ts
│   ├── auth.ts
│   └── analytics.ts
```

Y configuración:

```ts
boot: [
    'api',
    'auth'
]
```

Nexa genera el orden de inicialización.

Pero:

> Boot no debe ser obligatorio.

Una aplicación pequeña podría no tener ningún boot.

------

# 14. ¿Qué es Boot realmente?

Es simplemente:

```text
Application startup hooks
```

Por ejemplo:

```ts
export default defineBoot(({ app }) => {
    app.use(api);
});
```

Pero internamente Nexa podría compilarlo directamente.

No necesitamos un gigantesco sistema de plugins para algo tan simple.

------

# 15. Plugins

Aquí tenemos que ser muy cuidadosos.

No quiero:

```text
plugin
plugin
plugin
plugin
plugin
```

modificando arbitrariamente el compiler.

Eso destruiría la estabilidad.

Por eso habrá dos niveles.

### Application Plugin

Puede añadir:

```text
services
components
routes
commands
```

### Compiler Plugin

Será mucho más restringido.

Solo podrá utilizar APIs oficiales.

------

# 16. API de plugins estable

Algo como:

```ts
export default definePlugin({
    name: 'my-plugin',

    setup(app) {
        ...
    }
});
```

Nexa garantiza:

```text
Plugin API v1
```

durante toda la generación mayor compatible.

Si internamente cambiamos Rust:

```text
Rust compiler V1
       ↓
Rust compiler V2
       ↓
Rust compiler V3
```

el plugin no debería enterarse.

------

# 17. Adapters

Aquí está una de nuestras decisiones más importantes.

Los adapters sirven para integrar sistemas externos sin contaminar Core.

```text
@nexa/adapters
```

Ejemplos:

```text
REST
GraphQL
Firebase
Tauri
Capacitor
Web APIs
Storage
Authentication
Maps
```

------

# 18. Tauri

Como queremos utilizar Rust:

```text
Nexa
 ↓
Tauri
 ↓
Desktop
```

Esto encaja muy bien.

Una aplicación Nexa puede generar:

```text
Web
Desktop
Mobile
```

usando el mismo código UI.

------

# 19. Capacitor

También:

```text
Nexa
 ↓
Capacitor
 ↓
Android / iOS
```

No necesitamos crear:

```text
Vue project
React project
Android project
iOS project
```

para cada plataforma.

El frontend Nexa sigue siendo:

```text
TypeScript + Nexa
```

------

# 20. Pero Nexa no debe depender de Tauri

Esto es importante.

El Core:

```text
@nexa/core
```

no debe importar:

```text
tauri
capacitor
electron
```

La relación será:

```text
             Nexa
               │
       ┌───────┼────────┐
       ▼       ▼        ▼
      Web    Tauri   Capacitor
```

------

# 21. Storage

Necesitamos una abstracción estable:

```ts
storage.get('theme');
storage.set('theme', 'dark');
storage.remove('theme');
```

El adapter decide dónde guardar.

Web:

```text
localStorage
IndexedDB
```

Tauri:

```text
filesystem/database
```

Capacitor:

```text
native storage
```

El código de la aplicación no cambia.

------

# 22. Forms

Ahora sí llegamos a otro módulo oficial.

```text
@nexa/forms
```

Debe manejar:

```text
fields
validation
errors
submission
state
async validation
accessibility
```

Ejemplo conceptual:

```tsx
<Form onSubmit={submit}>

    <Field
        name="email"
        rules={[required(), email()]}
    />

    <Button type="submit">
        Enviar
    </Button>

</Form>
```

Pero nuevamente:

**HTML real debajo.**

------

# 23. Forms + progressive enhancement

Esto es muy importante para Nexa.

El formulario:

```html
<form method="post" action="/api/register">
```

debe ser funcional.

Nexa puede mejorar:

```text
validation
loading
AJAX
errors
optimistic UI
```

cuando JavaScript está disponible.

------

# 24. i18n

Módulo:

```text
@nexa/i18n
```

Debe soportar:

```text
translations
pluralization
dates
numbers
currency
locale detection
fallback
lazy loading
```

Ejemplo:

```ts
t('welcome');
```

o:

```tsx
<h1>{t('home.title')}</h1>
```

------

# 25. SEO + i18n

Y aquí hay una conexión importante.

Nexa debe poder generar:

```html
<html lang="es">
```

y:

```html
<link rel="alternate" hreflang="en">
<link rel="alternate" hreflang="es">
```

por ruta.

Por ejemplo:

```text
/es/productos/123
/en/products/123
```

Esto será gestionado por Router + i18n + SEO.

------

# 26. Plugins vs Adapters

Vamos a establecer una regla clara:

### Plugin

Extiende Nexa.

```text
new functionality
```

### Adapter

Conecta Nexa con algo externo.

```text
Nexa ←→ external system
```

Ejemplo:

```text
Stripe adapter
Tauri adapter
GraphQL adapter
Firebase adapter
```

No debemos llamar plugin a todo.

------

# 27. `nexa.config.ts`

Ahora podemos empezar a visualizarlo:

```ts
import { defineConfig } from 'nexa';

export default defineConfig({

    app: {
        name: 'My App'
    },

    ui: {
        theme: 'default',
        darkMode: true
    },

    css: [
        'src/css/app.css'
    ],

    fonts: [
        'Inter'
    ],

    boot: [
        'api'
    ],

    i18n: {
        defaultLocale: 'es'
    },

    router: {
        mode: 'history'
    },

    plugins: [],

    adapters: []
});
```

Pero esta API debe ser **muy estable**.

No quiero que en Nexa 2:

```ts
ui:
```

se convierta en:

```ts
framework.ui.config:
```

sin una razón arquitectónica extremadamente fuerte.

------

# 28. Configuración declarativa

Además, Nexa debería detectar automáticamente muchas cosas.

Por ejemplo:

```text
src/pages
src/components
src/assets
src/styles
src/boot
src/layouts
```

No queremos un `nexa.config.ts` de 500 líneas.

La filosofía será:

> **Convention over configuration, configuration when necessary.**

------

# 29. Estructura de un proyecto Nexa

Podríamos terminar con algo así:

```text
my-app/
│
├── src/
│   ├── pages/
│   ├── layouts/
│   ├── components/
│   ├── composables/
│   ├── services/
│   ├── stores/
│   ├── boot/
│   ├── i18n/
│   ├── css/
│   ├── assets/
│   └── app.ts
│
├── public/
│
├── nexa.config.ts
├── tsconfig.json
└── package.json
```

Aunque hay una cuestión importante:

## ¿Necesitamos `package.json`?

No necesariamente.

Si Nexa será completamente Rust:

```bash
nexa add @nexa/ui
```

podría mantener su propio:

```text
nexa.lock
nexa.toml
```

o algún formato propio.

Pero aquí todavía no debemos decidirlo.

------

# 30. El gran objetivo: eliminar Node

Si queremos realmente cumplir nuestra filosofía:

```bash
nexa dev
nexa build
nexa preview
nexa add
nexa remove
nexa update
nexa test
nexa lint
```

todo debe ser ejecutado por:

```text
nexa
 ↓
Rust
```

El desarrollador no necesita:

```bash
npm
npx
node
vite
webpack
```

para trabajar con Nexa.

------

# 31. Pero TypeScript sigue siendo el lenguaje

Esto es importante.

El programador escribe:

```ts
.ts
.tsx
```

No Rust.

Rust es:

```text
compiler
bundler
runtime engine cuando corresponda
CLI
tooling
```

Por tanto:

```text
Developer
    │
    │ TypeScript
    ▼
  Nexa CLI
    │
    │ Rust
    ▼
Compiler
```

------

# 32. Compatibilidad a 10 años

Aquí establecería una regla muy fuerte:

### Stable APIs

```text
@nexa/runtime
@nexa/router
@nexa/forms
@nexa/i18n
@nexa/ui
@nexa/http
```

tendrán contratos estables.

### Internal APIs

```text
AST internals
IR internals
optimizer internals
compiler internals
```

pueden evolucionar.

Esto permite:

```text
Nexa 1.x
Nexa 2.x
Nexa 3.x
```

sin obligar al desarrollador a reescribir toda su aplicación.

------

# 33. Compatibilidad de proyectos

Nuestro objetivo debería ser:

```text
Proyecto Nexa 1.0
       │
       ▼
Nexa 5.0
       │
       ▼
funciona
```

Quizás aparezcan:

```text
warnings
```

pero no:

```text
"reescribe toda tu aplicación"
```

Las migraciones serán:

```bash
nexa migrate
```

cuando realmente sean necesarias.

------

# 34. Sistema de migraciones

Cuando exista una mejora arquitectónica real:

```text
Nexa 2
```

puede introducir:

```text
NX migration
```

El CLI puede analizar:

```text
AST
```

y transformar automáticamente código antiguo.

Por ejemplo:

```text
old API
 ↓
AST migration
 ↓
new API
```

Esto es muchísimo mejor que decir:

> "A partir de ahora cambia 400 archivos manualmente."

------

# 35. Nuestra arquitectura modular queda así

```text
@nexa/
│
├── compiler
├── ast
├── ir
├── analyzer
├── optimizer
│
├── runtime
├── reactivity
├── router
├── data
│
├── ui
├── forms
├── i18n
├── http
├── seo
│
├── testing
├── devtools
│
└── adapters
    ├── tauri
    ├── capacitor
    ├── graphql
    └── ...
```

Y todo ello puede compilarse desde:

```bash
nexa build
```

------

# 36. Ahora ya tenemos prácticamente definido el Core conceptual

Hasta ahora:

### Paso 1

**AST + Parser**

### Paso 2

**IR + Dependency Graph**

### Paso 3

**Runtime + Progressive Activation**

### Paso 4

**Router + Data Layer**

### Paso 5

**UI + Forms + i18n + Plugins + Adapters**

Y el siguiente paso ya debería ser diferente.

Ya no debemos seguir agregando funcionalidades indefinidamente.

# Paso 7 — Arquitectura física del repositorio Nexa

Aquí vamos a definir exactamente cómo construiremos el proyecto:

```text
nexa/
├── crates/
│   ├── nexa-cli/
│   ├── nexa-parser/
│   ├── nexa-ast/
│   ├── nexa-analyzer/
│   ├── nexa-ir/
│   ├── nexa-optimizer/
│   ├── nexa-renderer/
│   ├── nexa-runtime/
│   ├── nexa-router/
│   ├── nexa-data/
│   ├── nexa-seo/
│   └── ...
│
├── packages/
│   ├── runtime/
│   ├── ui/
│   ├── forms/
│   └── i18n/
│
└── examples/
```

Y definiremos **qué parte será Rust, qué parte TypeScript, cómo se compilan ambas, cómo `nexa dev` observa archivos, cómo `nexa build` produce `dist/`, cómo se genera el runtime y cómo se instala Nexa en una máquina sin Node**.

Ese será el punto en el que ya podremos dejar de diseñar abstractamente y comenzar a construir el **primer workspace Rust real de Nexa**.

Sí. Ahora pasamos al **Paso 7: arquitectura física del repositorio y del toolchain de Nexa**.

Aquí ya no vamos a definir funcionalidades nuevas; vamos a convertir todo lo anterior en una estructura técnicamente construible y que pueda evolucionar durante muchos años.

# NEXA 0.1 — Paso 7: Workspace y arquitectura física

La primera decisión importante:

> **Nexa será un monorepo Rust, con el compilador/toolchain escrito en Rust y los paquetes que ejecutará el navegador escritos en TypeScript.**

Eso significa que hay que distinguir **dos mundos**:

```text
                    NEXA
                     │
          ┌──────────┴──────────┐
          │                     │
       TOOLCHAIN              APP
          │                     │
        Rust                TypeScript
          │                     │
   compiler / CLI          runtime / UI
   bundler / analyzer      router / forms
   optimizer / dev server  i18n / etc.
```

------

# 1. Repositorio principal

Propongo comenzar así:

```text
nexa/
│
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md
│
├── crates/
│
├── packages/
│
├── tools/
│
├── examples/
│
├── tests/
│
└── docs/
```

No vamos a crear 50 crates desde el primer día.

Los módulos aparecerán cuando realmente exista código que los justifique.

------

# 2. `crates/` — corazón Rust

Aquí estará el toolchain.

Inicialmente:

```text
crates/
│
├── nexa-cli/
├── nexa-parser/
├── nexa-ast/
├── nexa-ir/
├── nexa-analyzer/
├── nexa-optimizer/
├── nexa-renderer/
├── nexa-bundler/
└── nexa-runtime-builder/
```

Más adelante:

```text
nexa-router/
nexa-data/
nexa-seo/
nexa-dev-server/
nexa-plugin/
nexa-migrate/
nexa-test/
```

No necesitamos crearlos todos ahora.

------

# 3. `nexa-cli`

Será el ejecutable:

```bash
nexa
```

Su responsabilidad será coordinar.

Por ejemplo:

```bash
nexa create
nexa dev
nexa build
nexa preview
nexa add
nexa remove
nexa update
nexa test
nexa lint
nexa migrate
```

Pero el CLI **no debe contener toda la lógica**.

Esto sería un error:

```text
nexa-cli
 ├── parser
 ├── compiler
 ├── router
 ├── bundler
 └── everything
```

El CLI solamente orquesta.

```text
nexa-cli
   │
   ├── parser
   ├── analyzer
   ├── optimizer
   ├── renderer
   └── bundler
```

------

# 4. `nexa-parser`

Aquí comienza realmente el compilador.

Entrada:

```text
.ts
.tsx
```

Salida:

```text
AST
```

Conceptualmente:

```text
Source
  │
  ▼
Lexer
  │
  ▼
Parser
  │
  ▼
AST
```

Pero aquí tenemos una decisión técnica importante:

**No vamos a inventar TypeScript desde cero.**

TypeScript es un lenguaje enorme.

Necesitamos compatibilidad real con el TypeScript moderno que decidamos soportar.

------

# 5. TypeScript 7

Nuestra intención será:

```text
Developer
    │
    ▼
TypeScript
    │
    ▼
Nexa compiler
```

El desarrollador no aprende:

```text
NexaScript
```

ni:

```text
Nexa language
```

Es:

```text
TypeScript + Nexa
```

Esto es crucial para adopción.

------

# 6. Parser y compatibilidad

El parser de Nexa debe entender:

```ts
const x = 10;
<div>Hello</div>
<Component />
interface User {
    name: string;
}
```

etc.

Y Nexa añade sus propias construcciones cuando sean necesarias.

No queremos romper TypeScript innecesariamente.

------

# 7. `nexa-ast`

Aquí definimos nuestro árbol interno.

Por ejemplo:

```rust
enum Node {
    Element(ElementNode),
    Text(TextNode),
    Expression(ExpressionNode),
    Component(ComponentNode),
    Fragment(FragmentNode),
}
```

Esto es **nuestro IR semántico**, no simplemente una copia del AST original de TypeScript.

La separación será:

```text
TypeScript AST
       │
       ▼
Nexa AST
       │
       ▼
Nexa IR
```

------

# 8. ¿Por qué tres niveles?

Porque necesitamos poder evolucionar.

```text
Source AST
     ↓
Semantic AST
     ↓
IR
```

El TypeScript AST puede cambiar.

Nuestro IR no debería cambiar constantemente.

Así podemos tener:

```text
TypeScript 7
      ↓
Nexa parser
      ↓
Nexa IR v1
```

y posteriormente:

```text
TypeScript 8
      ↓
Nexa parser
      ↓
Nexa IR v1
```

siempre que sea posible.

------

# 9. `nexa-analyzer`

Aquí Nexa comienza a hacer algo que será fundamental para nuestra arquitectura.

Analiza:

```text
components
dependencies
signals
events
routes
data
static content
dynamic content
CSS
assets
```

Por ejemplo:

```tsx
<h1>{product.name}</h1>

<button onClick={addToCart}>
    Add
</button>
```

El analyzer determina:

```text
h1
 └── depends on product.name

button
 └── event: click
 └── handler: addToCart
```

------

# 10. Dependency Graph

El analyzer genera:

```text
              Product
                 │
          ┌──────┴──────┐
          ▼             ▼
       <h1>          <button>
          │             │
      product.name   addToCart
```

Esto alimentará:

```text
optimizer
runtime
code splitting
progressive activation
```

Es uno de los componentes más importantes de todo Nexa.

------

# 11. `nexa-ir`

IR significa:

**Intermediate Representation.**

Será el lenguaje intermedio de Nexa.

Por ejemplo:

```text
PAGE
 │
 ├── STATIC_ELEMENT
 │    └── h1
 │
 ├── BINDING
 │    └── product.name
 │
 └── EVENT
      └── click → addToCart
```

No es HTML.

No es JavaScript.

No es TypeScript.

Es una representación optimizable.

------

# 12. Optimizer

Después:

```text
Nexa IR
   │
   ▼
Optimizer
```

puede detectar:

```text
static nodes
unused code
dead branches
unused CSS
unused components
shared dependencies
activation boundaries
```

Por ejemplo:

```text
Component A
 ↓
no interaction
 ↓
STATIC
```

Entonces:

```text
JavaScript = 0
```

para esa parte.

------

# 13. Renderer

El renderer convierte IR en HTML.

```text
IR
 ↓
Renderer
 ↓
HTML
```

Ejemplo:

```text
IR

ELEMENT article
ELEMENT h1
TEXT "Producto"
```

produce:

```html
<article>
    <h1>Producto</h1>
</article>
```

------

# 14. Renderer no debe depender del navegador

Esto es importante.

`nexa-renderer` debe poder funcionar en Rust puro.

No debería necesitar:

```text
Chrome
Chromium
DOM browser
Node
```

Debe poder hacer:

```text
IR → HTML
```

directamente.

Esto permitirá:

```text
SSG
pre-render
testing
SEO analysis
server rendering
```

------

# 15. `nexa-bundler`

Este será nuestro sustituto conceptual de:

```text
webpack
Vite
Rollup
esbuild
```

No necesariamente debemos competir con ellos en cada característica inmediatamente.

Pero Nexa debe tener su propio pipeline:

```text
TS
 ↓
Nexa compiler
 ↓
IR
 ↓
Optimizer
 ↓
JS chunks
 ↓
CSS
 ↓
assets
```

------

# 16. Code splitting

El bundler debe entender:

```text
route
component
activation
dependency
```

Por ejemplo:

```text
Home
 ├── runtime
 └── home.js

Product
 ├── runtime
 ├── product.js
 └── cart.js

Admin
 ├── runtime
 └── admin.js
```

Resultado:

```text
dist/assets/
├── runtime.x.js
├── home.x.js
├── product.x.js
├── cart.x.js
└── admin.x.js
```

------

# 17. `nexa-runtime-builder`

El runtime TypeScript será compilado/optimizado por Nexa.

Aquí hay una distinción importante:

**Rust no tiene que convertir TypeScript directamente en código máquina.**

El navegador necesita JavaScript.

Por tanto:

```text
TypeScript
   ↓
Nexa compiler
   ↓
JavaScript optimizado
```

El ejecutable Nexa puede coordinar todo esto.

------

# 18. ¿Dónde entra Rust?

Rust será el cerebro:

```text
nexa
 │
 ├── parse
 ├── analyze
 ├── transform
 ├── optimize
 ├── bundle
 ├── generate
 └── serve
```

El producto final sigue siendo:

```text
HTML
CSS
JavaScript
assets
```

porque eso es lo que entiende el navegador.

------

# 19. `packages/`

Aquí estarán los paquetes que forman parte del ecosistema que utiliza el desarrollador.

```text
packages/
│
├── runtime/
├── ui/
├── forms/
├── i18n/
├── http/
├── router/
└── seo/
```

Estos son principalmente TypeScript.

------

# 20. ¿Por qué no meterlos en `crates/`?

Porque:

```text
crates/
```

es código Rust.

Mientras:

```text
packages/
```

es código que terminará en la aplicación.

Esto mantiene una separación limpia:

```text
Rust
 └── toolchain

TypeScript
 └── application runtime
```

------

# 21. Runtime

```text
packages/runtime/
│
├── src/
│   ├── signal.ts
│   ├── effect.ts
│   ├── scheduler.ts
│   ├── activation.ts
│   ├── events.ts
│   └── dom.ts
│
└── package metadata
```

Este paquete será especialmente importante.

------

# 22. UI

```text
packages/ui/
│
├── src/
│   ├── components/
│   │   ├── Button/
│   │   ├── Input/
│   │   ├── Dialog/
│   │   ├── Card/
│   │   └── ...
│   │
│   ├── services/
│   │   ├── notification/
│   │   ├── loading/
│   │   └── dialog/
│   │
│   └── theme/
```

Cada componente debería tener:

```text
component
styles
types
accessibility behavior
```

------

# 23. Forms

```text
packages/forms/
```

con:

```text
Field
Form
validation
schema
submission
errors
```

------

# 24. i18n

```text
packages/i18n/
```

con:

```text
locale
translation
pluralization
formatting
fallback
lazy loading
```

------

# 25. Router

```text
packages/router/
```

será el runtime del router.

Pero recuerda:

```text
Rust compiler
```

también analiza las rutas.

Por eso:

```text
Compiler
    │
    ▼
route manifest
    │
    ▼
@nexa/router
```

------

# 26. Data Layer

```text
packages/http/
```

y posiblemente:

```text
packages/data/
```

Separaremos:

```text
HTTP transport
```

de:

```text
server-state management
```

Así no terminamos mezclando:

```text
fetch()
cache
query
mutation
state
```

en una sola API.

------

# 27. SEO

```text
packages/seo/
```

Aunque parte del SEO será responsabilidad del compiler.

Tenemos:

```text
Compiler SEO Analyzer
```

y:

```text
@nexa/seo
```

para las APIs del desarrollador.

------

# 28. `examples/`

Esto será extremadamente importante.

Desde el principio tendremos:

```text
examples/
├── hello-world/
├── counter/
├── routing/
├── ecommerce/
├── blog/
├── forms/
└── i18n/
```

Cada ejemplo debe compilar con:

```bash
nexa build
```

Esto funcionará como una especie de prueba de integración permanente.

------

# 29. `tests/`

Aquí tendremos:

```text
tests/
├── parser/
├── ast/
├── ir/
├── analyzer/
├── optimizer/
├── renderer/
├── router/
├── seo/
└── integration/
```

Nexa debe ser **test-first en el Core**.

Especialmente:

```text
compiler
renderer
optimizer
router
```

porque son componentes donde una regresión puede romper miles de proyectos.

------

# 30. `docs/`

La documentación también tendrá una arquitectura estable:

```text
docs/
├── getting-started/
├── concepts/
├── compiler/
├── runtime/
├── router/
├── ui/
├── forms/
├── i18n/
├── adapters/
├── deployment/
└── migration/
```

Y desde el principio documentaremos:

```text
Stable API
Experimental API
Internal API
```

------

# 31. Cargo Workspace

El `Cargo.toml` raíz será algo conceptualmente así:

```toml
[workspace]
resolver = "3"

members = [
    "crates/nexa-cli",
    "crates/nexa-parser",
    "crates/nexa-ast",
    "crates/nexa-ir",
    "crates/nexa-analyzer",
    "crates/nexa-optimizer",
    "crates/nexa-renderer",
    "crates/nexa-bundler"
]
```

No necesitamos agregar todavía los crates que aún no implementamos.

------

# 32. Arquitectura de dependencias

Aquí quiero imponer una regla desde el comienzo.

```text
CLI
 │
 ├── Compiler
 │    ├── Parser
 │    ├── AST
 │    ├── Analyzer
 │    ├── IR
 │    ├── Optimizer
 │    └── Renderer
 │
 └── Bundler
```

Pero:

```text
Parser
  X
  ↓
CLI
```

El parser no debe conocer el CLI.

Igualmente:

```text
Renderer
  X
  ↓
UI
```

El renderer no debe conocer componentes específicos.

Esto evita dependencias circulares.

------

# 33. Arquitectura de capas

Podemos formalizar:

```text
                  APPLICATION
                       │
              ┌────────┴────────┐
              │                 │
           Runtime             UI
              │                 │
              └────────┬────────┘
                       │
                    Compiler
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
           Parser     IR      Analyzer
             │         │         │
             └─────────┼─────────┘
                       ▼
                    Optimizer
                       │
                       ▼
                    Renderer
                       │
                       ▼
                     HTML
```

------

# 34. `nexa dev`

El flujo será:

```bash
nexa dev
```

Internamente:

```text
CLI
 ↓
Project Scanner
 ↓
Compiler
 ↓
Dev Server
 ↓
File Watcher
```

Cuando cambias:

```text
src/pages/index.tsx
```

Nexa detecta:

```text
file changed
 ↓
dependency graph
 ↓
affected modules
 ↓
recompile only affected parts
 ↓
browser update
```

Esto nos permitirá posteriormente implementar HMR.

------

# 35. `nexa build`

```bash
nexa build
```

Pipeline:

```text
Project
  ↓
Config
  ↓
Scan
  ↓
Parse
  ↓
Analyze
  ↓
IR
  ↓
Optimize
  ↓
Render
  ↓
Bundle
  ↓
SEO analysis
  ↓
dist/
```

------

# 36. `nexa preview`

No debe recompilar.

```text
nexa preview
```

simplemente toma:

```text
dist/
```

y lo sirve.

------

# 37. `nexa add`

Aquí todavía tenemos una decisión pendiente.

No quiero copiar automáticamente el comportamiento de npm.

Nuestro sistema de paquetes tendrá que resolver:

```text
package discovery
versions
integrity
lockfile
registry
signatures
cache
offline installation
```

Eso merece su propio diseño.

Por ahora podemos dejar:

```bash
nexa add @nexa/ui
```

como interfaz pública.

La implementación del Package Manager será el **Paso 8**.

------

# 38. Algo muy importante: no debemos depender de npm

Si Nexa realmente quiere cumplir:

> "No Node."

entonces esto:

```bash
nexa add @nexa/ui
```

no puede terminar haciendo:

```bash
npm install
```

por debajo.

Nuestro CLI deberá tener:

```text
Nexa Package Manager
```

propio.

Podrá descargar:

```text
TypeScript packages
```

desde un registry.

Pero el proceso lo controla:

```text
Rust
```

------

# 39. Entonces la arquitectura empieza a quedar muy clara

```text
                 NEXA TOOLCHAIN
                       │
             ┌─────────┴─────────┐
             │                   │
          Rust CLI            Package Manager
             │                   │
             ▼                   ▼
          Compiler            Registry
             │
       ┌─────┼──────┐
       ▼     ▼      ▼
     AST     IR    Graph
       │     │      │
       └─────┼──────┘
             ▼
         Optimizer
             │
      ┌──────┼───────┐
      ▼      ▼       ▼
     HTML    JS      CSS
      │      │       │
      └──────┼───────┘
             ▼
         Application
             │
        TypeScript
```

------

# 40. Una última decisión antes de escribir código

Ahora estamos en un punto importante.

**No empezaría todavía escribiendo el compiler completo.**

Antes debemos definir el **Nexa Package Manager + formato del proyecto + `nexa.config.ts` + lockfile**.

Porque si hacemos el compiler primero y después descubrimos que nuestro modelo de paquetes no funciona, tendremos que reorganizar gran parte del toolchain.

Así que el siguiente paso será:

# NEXA 0.1 — Paso 8: Package Manager + Project Manifest

Definiremos exactamente:

```text
nexa create
nexa add
nexa remove
nexa update
nexa install
nexa lock
```

y especialmente:

```text
nexa.toml
nexa.lock
nexa.config.ts
```

Además resolveremos una cuestión que será fundamental para el objetivo de **10+ años de estabilidad**:

> **Cómo versionar Nexa, sus paquetes y sus APIs sin repetir el problema de que cada actualización importante obligue al desarrollador a reconstruir su proyecto.**

Después de ese paso ya podremos entrar al **primer código real de Nexa en Rust**, comenzando por `nexa-cli` + parser/AST mínimo y una primera aplicación que haga:

```bash
nexa create hello
cd hello
nexa dev
nexa build
```

y produzca HTML real.

Perfecto. Entramos al **Paso 8: Package Manager + Project Manifest**. Esta parte merece especial cuidado porque queremos que Nexa pueda mantenerse estable durante 10+ años y, al mismo tiempo, evolucionar internamente.

# NEXA 0.1 — Paso 8

La estructura que vamos a definir será:

```text
nexa/
├── nexa.toml
├── nexa.lock
├── nexa.config.ts
├── src/
└── public/
```

Y tendremos dos conceptos diferentes:

```text
nexa.toml       → identidad y dependencias del proyecto
nexa.lock       → versiones exactas e integridad
nexa.config.ts  → comportamiento/configuración de Nexa
```

No debemos mezclar los tres.

------

# 1. `nexa.toml`

Será el equivalente conceptual a un manifest del proyecto.

Ejemplo:

```toml
[project]
name = "my-store"
version = "0.1.0"
nexa = "1"

[dependencies]
"@nexa/ui" = "^1.0"
"@nexa/forms" = "^1.0"
"@nexa/i18n" = "^1.0"
```

Y podría existir:

```toml
[dev-dependencies]
"@nexa/testing" = "^1.0"
```

La versión:

```toml
nexa = "1"
```

no significa "usar exactamente Nexa 1.0".

Significa:

> Este proyecto pertenece al contrato de compatibilidad de Nexa 1.

------

# 2. `nexa.lock`

Este archivo sí congela las dependencias.

Por ejemplo, conceptualmente:

```toml
[[package]]
name = "@nexa/ui"
version = "1.4.2"
source = "registry"
integrity = "sha256-..."

[[package]]
name = "@nexa/forms"
version = "1.2.0"
source = "registry"
integrity = "sha256-..."
```

Así:

```text
nexa.toml
    │
    │ rangos permitidos
    ▼
resolver
    │
    ▼
nexa.lock
    │
    │ versiones exactas
    ▼
instalación reproducible
```

------

# 3. ¿Por qué necesitamos lockfile?

Porque:

```text
Proyecto A
```

compilado hoy y:

```text
Proyecto A
```

compilado dentro de dos años deberían poder producir el mismo resultado si el código y el lockfile son iguales.

Esto es importantísimo para:

- CI/CD
- producción
- equipos
- reproducibilidad
- debugging
- seguridad

------

# 4. `nexa.config.ts`

Aquí dejamos la configuración de comportamiento.

Ejemplo:

```ts
import { defineConfig } from 'nexa';

export default defineConfig({
    app: {
        name: 'My Store'
    },

    ui: {
        theme: 'default',
        darkMode: true
    },

    i18n: {
        defaultLocale: 'es'
    },

    router: {
        mode: 'history'
    }
});
```

La separación queda:

```text
nexa.toml
   ↓
¿Qué tiene mi proyecto?

nexa.lock
   ↓
¿Qué versiones exactas uso?

nexa.config.ts
   ↓
¿Cómo quiero que funcione Nexa?
```

------

# 5. `nexa create`

Queremos que empezar un proyecto sea muy sencillo:

```bash
nexa create my-store
```

Resultado:

```text
my-store/
├── nexa.toml
├── nexa.config.ts
├── src/
│   ├── pages/
│   │   └── index.tsx
│   ├── components/
│   ├── layouts/
│   ├── assets/
│   └── css/
├── public/
└── .gitignore
```

Inicialmente podemos generar una aplicación mínima sin instalar 200 paquetes.

------

# 6. `nexa add`

Ejemplo:

```bash
nexa add @nexa/ui
```

Proceso:

```text
nexa add
   │
   ▼
Registry
   │
   ▼
Resolve dependency
   │
   ▼
Download
   │
   ▼
Verify integrity
   │
   ▼
Update nexa.toml
   │
   ▼
Update nexa.lock
```

------

# 7. Versionado

Aquí propongo utilizar **Semantic Versioning**, pero con reglas más estrictas que las habituales.

```text
MAJOR.MINOR.PATCH
```

Por ejemplo:

```text
1.4.2
```

significa:

```text
1 → contrato principal
4 → nuevas capacidades compatibles
2 → corrección
```

------

# 8. Pero Nexa tendrá una regla adicional

Un cambio:

```text
1.4 → 1.5
```

**no puede romper APIs existentes.**

Y:

```text
1.5 → 1.6
```

tampoco.

Incluso:

```text
1.x
```

debe mantener una compatibilidad muy fuerte.

------

# 9. ¿Cuándo se permite romper?

Solamente en:

```text
2.0
```

pero incluso entonces queremos:

```bash
nexa migrate
```

para automatizar la transición.

Por ejemplo:

```text
Nexa 1.x
   │
   ▼
nexa migrate
   │
   ▼
Nexa 2.x
```

El objetivo es:

```text
manual migration
≈ mínimo
```

------

# 10. Stable / Experimental / Internal

Cada API tendrá uno de tres estados.

### Stable

```ts
import { Button } from '@nexa/ui';
```

Garantía de compatibilidad.

### Experimental

```ts
import { X } from '@nexa/experimental';
```

Puede cambiar.

### Internal

```text
@nexa/internal/*
```

No debe utilizarse directamente.

Esto es fundamental.

------

# 11. El desarrollador nunca debería importar internals

No:

```ts
import { CompilerNode } from '@nexa/compiler/internal';
```

Porque si lo hace, queda atado a la implementación.

En cambio:

```ts
import { signal } from '@nexa/runtime';
```

si `signal` es Stable, debe permanecer compatible.

------

# 12. API de compatibilidad

Podemos tener:

```text
Nexa API Contract
```

que defina:

```text
@nexa/runtime v1
@nexa/router v1
@nexa/forms v1
@nexa/ui v1
@nexa/i18n v1
```

Internamente podemos cambiar:

```text
Rust compiler
IR
optimizer
bundler
```

sin cambiar esos contratos.

------

# 13. Ejemplo

Hoy:

```ts
const count = signal(0);
```

En el futuro podemos cambiar completamente:

```text
runtime implementation
```

pero esto:

```ts
signal(0)
```

sigue funcionando.

El desarrollador no necesita saber cómo está implementado.

------

# 14. Package Registry

Necesitamos un registry.

Conceptualmente:

```text
registry.nexa.dev
```

pero el nombre del dominio todavía **no lo debemos asumir ni registrar como decisión definitiva**.

El registry tendrá:

```text
packages
versions
metadata
checksums
signatures
dependencies
```

------

# 15. Package format

No quiero que Nexa dependa de:

```text
node_modules/
```

Nuestro sistema será:

```text
.nexa/
```

Por ejemplo:

```text
.nexa/
├── cache/
├── packages/
└── registry/
```

El formato exacto lo definiremos durante la implementación.

------

# 16. Una diferencia importante con npm

No queremos:

```text
cada proyecto
└── node_modules
    └── cientos/miles de copias
```

Preferimos un cache global:

```text
Nexa Cache
     │
     ├── @nexa/ui@1.4.2
     ├── @nexa/forms@1.2.0
     └── @nexa/i18n@1.1.3
             ▲
             │
       ┌─────┴─────┐
       │           │
   proyecto A   proyecto B
```

Esto puede ahorrar muchísimo espacio.

------

# 17. Offline mode

Esto también debería estar desde el principio.

```bash
nexa add @nexa/ui --offline
```

Si está en cache:

```text
✓ package found locally
```

Y:

```bash
nexa build --offline
```

debería funcionar si todas las dependencias están disponibles.

------

# 18. Seguridad

Nuestro package manager debe verificar:

```text
checksum
```

y posteriormente:

```text
digital signature
```

La cadena será:

```text
Registry
   ↓
Package
   ↓
Hash
   ↓
Signature
   ↓
Nexa
   ↓
Install
```

Esto será muy importante para un ecosistema de muchos años.

------

# 19. Dependency resolution

Supongamos:

```text
App
 ├── @nexa/ui ^1.2
 └── shop-ui ^2.0

shop-ui
 └── @nexa/ui ^1.3
```

El resolver debe intentar:

```text
@nexa/ui 1.x
```

compatible con ambas.

Si no existe:

```text
CONFLICT
```

y Nexa debe explicarlo claramente.

No mensajes crípticos.

Algo como:

```text
Dependency conflict

Your project requires:
@nexa/ui ^1.2

shop-ui requires:
@nexa/ui ^2.0

These requirements cannot be satisfied together.
```

------

# 20. Dependencias opcionales

También:

```toml
[dependencies]
"@nexa/ui" = "^1.0"

[optional-dependencies]
"@nexa/maps" = "^1.0"
```

Esto permite mantener proyectos pequeños.

------

# 21. Peer dependencies

Necesitaremos algo equivalente a peer dependencies para ciertos paquetes.

Por ejemplo:

```text
@nexa/plugin-auth
```

puede requerir:

```text
@nexa/runtime >=1.2
```

pero no instalar otra copia del runtime.

Esto será responsabilidad del resolver.

------

# 22. Monorepo

Nexa también debe permitir:

```text
company-project/
├── apps/
│   ├── web/
│   ├── desktop/
│   └── mobile/
│
├── packages/
│   ├── shared/
│   └── components/
│
└── nexa.toml
```

Podremos soportar workspaces:

```toml
[workspace]
members = [
    "apps/web",
    "apps/desktop",
    "packages/shared"
]
```

Esto nos permitirá compartir:

```text
TypeScript
UI
models
services
```

entre plataformas.

------

# 23. Web + Tauri + Capacitor

Por ejemplo:

```text
my-product/
│
├── apps/
│   ├── web/
│   ├── desktop/
│   └── mobile/
│
└── packages/
    ├── core/
    └── ui/
```

La lógica compartida:

```text
packages/core
```

y cada plataforma:

```text
apps/web
apps/desktop
apps/mobile
```

puede tener su adapter.

------

# 24. Pero no debemos convertir Nexa en un monorepo obligatorio

Una aplicación simple:

```text
hello/
├── nexa.toml
└── src/
```

debe funcionar perfectamente.

El workspace será opcional.

------

# 25. Comandos oficiales

Con lo definido hasta ahora:

```bash
nexa create
nexa dev
nexa build
nexa preview

nexa add
nexa remove
nexa install
nexa update

nexa test
nexa lint

nexa migrate

nexa info
nexa doctor
```

Y posiblemente:

```bash
nexa clean
nexa cache
```

------

# 26. `nexa doctor`

Será muy útil.

```bash
nexa doctor
```

podría mostrar:

```text
Nexa Doctor

✓ Rust toolchain
✓ Nexa compiler
✓ Project manifest
✓ Lockfile
✓ Dependencies
✓ TypeScript
✓ Configuration
✓ Build environment

Project is healthy.
```

Si existe un problema:

```text
✗ Missing dependency
✗ Invalid configuration
```

y explicar exactamente cómo solucionarlo.

------

# 27. `nexa info`

```bash
nexa info
```

mostraría:

```text
Nexa 1.0.0

Compiler:
Rust 1.xx

Project:
my-store

Nexa API:
1

Target:
web

Renderer:
static

Dependencies:
12
```

------

# 28. Una decisión clave: TypeScript no será una dependencia global

No queremos decirle al usuario:

```bash
npm install -g typescript
```

Nexa debe controlar la versión del compilador TypeScript que necesita.

Conceptualmente:

```text
Nexa
 │
 ├── TypeScript frontend compatibility
 │
 └── Rust toolchain
```

Esto ayuda a conseguir builds reproducibles.

------

# 29. La distribución de Nexa

El usuario instala un único ejecutable:

```text
nexa
```

Por ejemplo:

```bash
nexa create my-app
```

No:

```bash
node
npm
npx
vite
typescript global
```

El usuario básicamente ve:

```text
Nexa
```

------

# 30. Instalación en diferentes sistemas

Queremos soportar:

```text
Linux
Windows
macOS
```

y posteriormente:

```text
FreeBSD
```

si resulta viable.

El CLI es Rust, por lo que tenemos una buena base para distribución multiplataforma.

------

# 31. Y ahora aparece una decisión arquitectónica muy importante

Tenemos que separar:

```text
Nexa SDK
```

de:

```text
Nexa CLI
```

El usuario instala:

```text
nexa
```

pero internamente el CLI puede contener:

```text
compiler
bundler
package manager
dev server
```

Mientras que los proyectos utilizan:

```text
@nexa/runtime
@nexa/ui
...
```

------

# 32. Arquitectura completa hasta ahora

```text
                         NEXA
                          │
              ┌───────────┴───────────┐
              │                       │
          NEXA TOOLCHAIN          NEXA PACKAGES
              │                       │
             Rust                 TypeScript
              │                       │
       ┌──────┼──────┐          ┌─────┼──────┐
       ▼      ▼      ▼          ▼     ▼      ▼
    Parser   IR   Optimizer   Runtime UI    Forms
       │      │      │          │     │      │
       └──────┼──────┘          └─────┼──────┘
              │                       │
              ▼                       ▼
           Renderer               Application
              │                       │
              └──────────┬────────────┘
                         ▼
                  HTML / CSS / JS
```

------

# 33. Ya podemos congelar una primera especificación

Hasta este momento tenemos definido:

| Área              | Decisión               |
| ----------------- | ---------------------- |
| Lenguaje          | TypeScript             |
| Toolchain         | Rust                   |
| CLI               | Rust                   |
| Compiler          | Rust                   |
| Renderer          | Rust                   |
| Bundler           | Rust                   |
| Runtime           | TypeScript             |
| UI                | TypeScript             |
| Router            | TS + soporte compiler  |
| Data              | TS + compiler          |
| SEO               | Compiler + runtime API |
| CSS               | Nexa UI + CSS propio   |
| Forms             | Oficial                |
| i18n              | Oficial                |
| Plugins           | API estable            |
| Adapters          | API estable            |
| Web               | Primera plataforma     |
| Desktop           | Tauri adapter          |
| Mobile            | Capacitor adapter      |
| Node backend      | No requerido           |
| Node build        | No requerido           |
| SSG               | Sí                     |
| Dynamic rendering | Sí                     |
| Hybrid            | Sí                     |
| Package manager   | Propio                 |
| Lockfile          | Sí                     |
| Migration system  | Sí                     |
| Long-term API     | Sí                     |

------

# 34. Lo que NO debemos decidir todavía

Hay varias cosas que sería un error congelar prematuramente:

```text
❌ sintaxis definitiva de componentes
❌ API definitiva de signals
❌ formato binario del cache
❌ protocolo definitivo del registry
❌ implementación definitiva del bundler
❌ API completa de plugins
```

Primero debemos hacer un **prototipo funcional**.

Después podremos validar esas decisiones.

------

# 35. Siguiente paso: el contrato de componentes

Ahora llegamos a una de las decisiones más importantes de Nexa.

Tenemos que definir exactamente qué significa:

```tsx
<Component />
```

en Nexa.

Y especialmente cómo se comportará:

```tsx
export component Product({ product }) {
    return (
        <article>
            <h1>{product.name}</h1>

            <Button onClick={addToCart}>
                Comprar
            </Button>
        </article>
    );
}
```

Tenemos que especificar:

```text
props
children
events
state
signals
effects
refs
slots
conditional rendering
loops
async data
components
server/client boundary
activation
```

y cómo el compiler transforma todo eso en:

```text
HTML estático
+
mínimo JavaScript
```

Este será el **Paso 9: Component Model + Reactividad de Nexa**.

Y aquí conviene tomarnos bastante tiempo, porque **este será el verdadero ADN de Nexa**. No queremos copiar JSX de React ni el modelo de Vue; tenemos que diseñar el modelo pensando desde el inicio en nuestro objetivo principal: **HTML completo primero, JavaScript únicamente donde sea necesario y activación progresiva sin sacrificar DX**.

Sí. Ya tenemos suficiente diseño para **dejar de definir arquitectura y empezar a construir**.

El siguiente paso ya no será otra discusión conceptual:

# Paso 9 — Crear el repositorio real de NEXA

Vamos a construir primero el **mínimo vertical funcional** (*vertical slice*), en lugar de intentar implementar todo el framework de una vez.

El primer objetivo será conseguir esto:

```bash
nexa create hello
cd hello
nexa dev
```

y que podamos escribir:

```tsx
export default function Home() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>Mi primera aplicación Nexa</p>
        </main>
    );
}
```

para obtener HTML real:

```html
<main>
    <h1>Hello Nexa</h1>
    <p>Mi primera aplicación Nexa</p>
</main>
```

Y después:

```bash
nexa build
```

debe generar:

```text
dist/
├── index.html
├── assets/
│   └── ...
└── ...
```

------

## La primera versión que vamos a construir

No intentaremos hacer todavía:

```text
❌ UI completa
❌ Forms completos
❌ i18n
❌ Tauri
❌ Capacitor
❌ GraphQL
❌ Package Registry
❌ HMR avanzado
❌ optimizador complejo
```

Primero construiremos:

```text
                    NEXA 0.1
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
       CLI Rust      Parser       Renderer
          │            │            │
          └────────────┼────────────┘
                       ▼
                    HTML
```

Con solamente:

```text
nexa create
nexa dev
nexa build
nexa preview
```

------

# Estructura inicial

Crearemos:

```text
nexa/
├── Cargo.toml
│
├── crates/
│   ├── nexa-cli/
│   ├── nexa-parser/
│   ├── nexa-ast/
│   └── nexa-renderer/
│
├── packages/
│   └── runtime/
│
├── examples/
│
└── tests/
```

### Primera responsabilidad de cada crate

```text
nexa-cli
    ↓
comandos

nexa-parser
    ↓
TypeScript/TSX → AST

nexa-ast
    ↓
estructura interna

nexa-renderer
    ↓
AST → HTML
```

------

# Pero hay una decisión técnica que debemos resolver ahora

Para que el primer prototipo sea realmente útil, **no vamos a intentar implementar un compilador TypeScript completo desde cero**.

El pipeline inicial será:

```text
                    archivo .tsx
                         │
                         ▼
                 TypeScript parser
                         │
                         ▼
                    Nexa AST
                         │
                         ▼
                     Renderer
                         │
                         ▼
                       HTML
```

Después sustituiremos/integraremos las piezas necesarias con nuestro propio pipeline Rust.

La arquitectura final seguirá siendo:

```text
TypeScript
     ↓
Nexa Parser
     ↓
Nexa AST
     ↓
Nexa IR
     ↓
Analyzer
     ↓
Optimizer
     ↓
Renderer
     ↓
HTML + JS + CSS
```

------

# Y el primer hito será muy concreto

Quiero que lleguemos a:

```bash
cargo build
```

para compilar Nexa.

Luego:

```bash
nexa create hello
```

que genere:

```text
hello/
├── nexa.toml
├── nexa.config.ts
├── src/
│   └── pages/
│       └── index.tsx
└── public/
```

Después:

```bash
cd hello
nexa build
```

y finalmente:

```text
dist/index.html
```

con:

```html
<!doctype html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width">
</head>
<body>
    <main>
        <h1>Hello Nexa</h1>
    </main>
</body>
</html>
```

**Eso será nuestro primer Nexa funcionando.**

A partir de ahí iremos incorporando, en orden:

```text
1. CLI
2. Project scanner
3. TSX parser
4. AST
5. HTML renderer
6. Build pipeline
7. Router
8. Data/API
9. SEO
10. Reactividad
11. Progressive activation
12. CSS
13. UI
14. Forms
15. i18n
16. Package manager
17. Dev server/HMR
18. Tauri
19. Capacitor
```

Y cada etapa deberá dejar **algo ejecutable**, no solamente código teórico.

## Entonces sí: estamos listos.

El **siguiente mensaje ya puede ser directamente la construcción del repositorio**, empezando por `Cargo.toml`, `nexa-cli` y el primer comando:

```bash
nexa create
```

Ese será el punto cero real del proyecto.

Ahora sí: **el siguiente paso es crear físicamente el proyecto NEXA**. Ya no necesitamos seguir diseñando más arquitectura antes de tener un prototipo ejecutándose.

Vamos a hacerlo por **Hito 0: Bootstrap del Toolchain**.

## 1. Objetivo inmediato

Al terminar este paso debemos tener:

```text
nexa/
├── Cargo.toml
├── crates/
│   └── nexa-cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── README.md
```

Y poder ejecutar:

```bash
cargo build
```

para obtener:

```text
target/debug/nexa
```

Después:

```bash
./target/debug/nexa
```

y que responda:

```text
Nexa CLI
Version: 0.1.0
```

------

# 2. Primer comando real de Nexa

Nuestro CLI tendrá desde el principio la estructura:

```text
nexa
├── create
├── dev
├── build
├── preview
├── add
├── remove
├── install
├── update
├── test
├── lint
├── migrate
├── doctor
└── info
```

Pero inicialmente solamente implementaremos:

```bash
nexa create
nexa build
nexa dev
nexa preview
nexa info
```

Los demás comandos existirán posteriormente.

------

# 3. Primer `nexa create`

Queremos que:

```bash
nexa create hello
```

genere:

```text
hello/
├── nexa.toml
├── nexa.config.ts
├── src/
│   └── pages/
│       └── index.tsx
├── public/
└── .gitignore
```

Y `index.tsx` inicialmente será:

```tsx
export default function Home() {
    return (
        <main>
            <h1>Hello Nexa</h1>
            <p>Welcome to Nexa.</p>
        </main>
    );
}
```

------

# 4. Primer `nexa build`

Luego:

```bash
cd hello
nexa build
```

deberá producir:

```text
dist/
└── index.html
```

con HTML real.

No queremos todavía JavaScript, reactividad ni hidratación.

Primero:

```text
TSX
 ↓
AST
 ↓
HTML
```

Esto es deliberado.

------

# 5. Primer gran test de Nexa

El test inicial será:

```text
src/pages/index.tsx
        │
        ▼
      Parser
        │
        ▼
       AST
        │
        ▼
    Renderer
        │
        ▼
   dist/index.html
```

Si conseguimos esto, tenemos el **primer compilador Nexa funcional**.

------

# 6. Después agregaremos la pieza crítica

Una vez que:

```text
TSX → HTML
```

funcione, pasaremos a:

```text
TSX
 ↓
AST
 ↓
IR
 ↓
Dependency Graph
 ↓
HTML
+
minimal JS
```

Ahí empezaremos a implementar nuestra característica diferencial:

### Progressive Activation

Por ejemplo:

```tsx
<h1>Producto</h1>

<Counter />
```

deberá convertirse conceptualmente en:

```html
<h1>Producto</h1>

<div data-nexa="counter">
    ...
</div>
```

y solamente `Counter` recibirá JavaScript.

------

# 7. No construiremos todavía el framework completo

La estrategia será:

```text
HITO 0
Bootstrap
   ↓
HITO 1
TSX → HTML
   ↓
HITO 2
IR
   ↓
HITO 3
Reactividad
   ↓
HITO 4
Progressive Activation
   ↓
HITO 5
Router
   ↓
HITO 6
API/Data
   ↓
HITO 7
SEO
   ↓
HITO 8
CSS/UI
   ↓
HITO 9
Forms/i18n
   ↓
HITO 10
Package Manager
   ↓
HITO 11
Tauri/Capacitor
```

Cada hito será ejecutable.

------

## Por tanto, el siguiente paso concreto es este:

**Crear el workspace Rust de NEXA y el primer `nexa-cli`.**

No necesitamos discutir otra arquitectura ahora.

En el próximo paso podemos escribir directamente los archivos iniciales:

```text
Cargo.toml
crates/nexa-cli/Cargo.toml
crates/nexa-cli/src/main.rs
README.md
```

y hacer que:

```bash
cargo build
./target/debug/nexa info
```

sea nuestro **primer NEXA funcionando**.