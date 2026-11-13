# Tutorial: tu primer sitio con Nexa, paso a paso

En este tutorial construyes un sitio real, desde cero, y en cada paso compruebas que funciona:

- un **blog bilingüe** (español e inglés) cuyos artículos vienen de una API, con HTML real para buscadores;
- una **isla**: un buscador interactivo que mejora la lista sin romperla si falla el JavaScript;
- un **panel privado** (una SPA) donde inicias sesión y publicas un artículo;
- y lo despliegas como **un solo ejecutable**, que además muestra los artículos que publiques *después* de compilar.

Tiempo: unos 40 minutos. El proyecto terminado está en
[`examples/tutorial-blog`](https://github.com/oweeme/oweenexa/tree/main/examples/tutorial-blog) y se comprueba
automáticamente en un Chrome real (23 comprobaciones), así que si sigues los pasos, funciona.

> **Antes de empezar, una idea.** Nexa tiene tres piezas. **Core** (`src/pages`) genera HTML real para
> lo que debe indexarse: el blog. Una **isla** es un pedazo interactivo dentro de una página. El **Modo App**
> (`src/app`) es una SPA para lo privado: el panel. Un sitio real usa las tres a la vez, y este tutorial también.

## Requisitos

- **Node.js 20 o superior** (para la API de ejemplo y para empaquetar la isla).
- **Rust** ([rustup.rs](https://rustup.rs/)), solo para instalar Nexa — todavía no hay binarios publicados.
- Un navegador, y una terminal.

## Paso 0 · Instalar Nexa

```bash
git clone https://github.com/oweeme/oweenexa.git
cd oweenexa
cargo install --path crates/nexa-cli --root ~/.local
export PATH="$HOME/.local/bin:$PATH"     # añádelo a tu ~/.bashrc o ~/.zshrc
```

En Windows (PowerShell): `cargo install --path crates/nexa-cli` instala en `%USERPROFILE%\.cargo\bin`, que ya está en el `PATH`.

Comprueba que funciona:

```bash
nexa info
```

Verás la versión y los enlaces a la documentación.

## Paso 1 · Crear el proyecto

```bash
nexa create tutorial-blog --template app
cd tutorial-blog
```

Nexa crea esta estructura:

```text
tutorial-blog/
├── nexa.toml            ← la configuración del proyecto
├── src/
│   ├── pages/           ← NEXA CORE: cada archivo es una página HTML (con SEO)
│   └── app/pages/       ← MODO APP: las páginas de la SPA
├── public/              ← archivos que se copian tal cual (imágenes, CSS...)
├── tsconfig.json        ← solo para tu editor
└── nexa-env.d.ts        ← tipos para el editor (autocompletado)
```

Borra la portada de ejemplo: usaremos una portada por idioma.

```bash
rm src/pages/index.tsx          # en PowerShell: Remove-Item src/pages/index.tsx
```

## Paso 2 · La API

Nexa no trae base de datos ni backend: **lee los datos de tu API** (en cualquier lenguaje) por HTTP. Para el tutorial,
una API de juguete en Node sin dependencias, con artículos en español e inglés, un login con cookie y un endpoint para publicar:

```js title="api.mjs"
// API de ejemplo del tutorial: un backend cualquiera (aquí Node sin dependencias). Nexa solo necesita JSON por HTTP.
import { createServer } from "node:http";

const PORT = Number(process.env.PORT ?? 9500);

// "Base de datos" en memoria. Cada post tiene su texto en español y, si existe, en inglés.
const posts = [
    {
        slug: "hola-nexa",
        author: "Héctor",
        date: "2026-01-10",
        es: { title: "Hola, Nexa", body: "## Un sitio con HTML real\n\nEste post está escrito en **Markdown**. Nexa lo convierte a HTML al compilar, y lo sanea.\n\n- Páginas de contenido con SEO\n- Islas para lo interactivo\n- Un panel como SPA" },
        en: { title: "Hello, Nexa", body: "## A site with real HTML\n\nThis post is written in **Markdown**. Nexa turns it into HTML at build time, and sanitizes it." },
    },
    {
        slug: "islas",
        author: "Héctor",
        date: "2026-02-02",
        es: { title: "Qué es una isla", body: "Una **isla** es un pedazo interactivo dentro de una página que, por lo demás, es HTML estático." },
    },
];

const sessions = new Map(); // sid -> nombre

const json = (res, status, body, headers = {}) => {
    res.writeHead(status, { "Content-Type": "application/json", ...headers });
    res.end(JSON.stringify(body));
};
const pick = (post, lang) => ({ slug: post.slug, author: post.author, date: post.date, lang: post[lang] ? lang : "es", ...(post[lang] ?? post.es) });
const summary = (p) => ({ slug: p.slug, author: p.author, date: p.date, lang: p.lang, title: p.title, excerpt: p.body.replace(/[#*`>-]/g, "").trim().slice(0, 110) });
const readBody = (req) => new Promise((ok) => { let d = ""; req.on("data", (c) => (d += c)); req.on("end", () => ok(d ? JSON.parse(d) : {})); });
const who = (req) => {
    const sid = /(?:^|;\s*)sid=([^;]+)/.exec(req.headers.cookie ?? "")?.[1];
    return sid ? sessions.get(sid) : undefined;
};
const slugify = (s) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");

createServer(async (req, res) => {
    const url = new URL(req.url, "http://x");
    const lang = url.searchParams.get("lang") ?? "es";

    if (req.method === "GET" && url.pathname === "/api/posts") {
        return json(res, 200, posts.map((p) => summary(pick(p, lang))));
    }
    const one = /^\/api\/posts\/([^/]+)$/.exec(url.pathname);
    if (req.method === "GET" && one) {
        const post = posts.find((p) => p.slug === decodeURIComponent(one[1]));
        return post ? json(res, 200, pick(post, lang)) : json(res, 404, { error: "no existe" });
    }

    if (req.method === "POST" && url.pathname === "/api/login") {
        const { name } = await readBody(req);
        const sid = Math.random().toString(36).slice(2);
        sessions.set(sid, name || "anónimo");
        return json(res, 200, { name: sessions.get(sid) }, { "Set-Cookie": `sid=${sid}; HttpOnly; Path=/; SameSite=Lax` });
    }
    if (req.method === "POST" && url.pathname === "/api/logout") {
        return json(res, 200, { ok: true }, { "Set-Cookie": "sid=; HttpOnly; Path=/; Max-Age=0" });
    }
    if (req.method === "GET" && url.pathname === "/api/me") {
        const name = who(req);
        return name ? json(res, 200, { name }) : json(res, 401, { error: "sin sesión" });
    }
    if (req.method === "POST" && url.pathname === "/api/admin/posts") {
        const name = who(req);
        if (!name) return json(res, 401, { error: "sin sesión" });
        const { title, body } = await readBody(req);
        if (!title) return json(res, 400, { error: "falta el título" });
        const post = { slug: slugify(title), author: name, date: new Date().toISOString().slice(0, 10), es: { title, body: body || "" } };
        posts.unshift(post);
        return json(res, 201, summary(pick(post, "es")));
    }
    json(res, 404, { error: "no existe" });
}).listen(PORT, "127.0.0.1", () => console.log(`API de ejemplo en http://127.0.0.1:${PORT}`));
```

Crea el archivo `api.mjs` en la raíz del proyecto con ese contenido. Arráncala **en una segunda terminal** y déjala abierta:

```bash
node api.mjs
# API de ejemplo en http://127.0.0.1:9500
```

Comprueba que responde (en Windows, abre la URL en el navegador):

```bash
curl http://127.0.0.1:9500/api/posts
# [{"slug":"hola-nexa","author":"Héctor","date":"2026-01-10","lang":"es","title":"Hola, Nexa",...},...]
```

## Paso 3 · Configurar el proyecto

Reemplaza el contenido de `nexa.toml` por este:

```toml title="nexa.toml"
[project]
name = "tutorial-blog"
version = "0.1.0"

[dependencies]

# Idioma por defecto (y de respaldo: una clave sin traducir cae a este idioma).
[i18n]
default = "es"

# La SPA (el panel) vive en /panel
[app]
base = "/panel"
title = "Panel"

# `nexa dev`, `nexa preview` y el ejecutable reenvían /api/* a tu backend: mismo origen, sin CORS.
[proxy]
"/api" = "http://127.0.0.1:9500"

# El módulo de la isla del buscador (lo genera `npm run vendor`).
[imports]
search = "/vendor/search.js"
```

Qué hace cada sección:

- **`[i18n] default`**: el idioma por defecto. Una clave sin traducir en otro idioma cae a este.
- **`[app] base`**: la SPA vivirá en `/panel`.
- **`[proxy]`**: reenvía `/api/*` a tu backend. El navegador cree que la API es del mismo origen: **no hay CORS** y las cookies funcionan.
- **`[imports]`**: el nombre `search` apunta al módulo de la isla (lo generamos en el paso 8).

## Paso 4 · Textos y diseño general

Los textos de cada idioma viven en `src/locales/<idioma>.json`:

```json title="src/locales/es.json"
{
  "site": { "name": "Mi blog", "tagline": "Un sitio hecho con Nexa" },
  "nav": { "home": "Inicio", "blog": "Blog" },
  "home": { "title": "Bienvenido", "latest": "Últimos artículos" },
  "blog": { "title": "Todos los artículos", "search": "Buscar un artículo…", "by": "por", "back": "← Volver al blog", "empty": "Todavía no hay artículos." }
}
```

El inglés solo traduce una parte — a propósito:

```json title="src/locales/en.json"
{
  "site": { "tagline": "A site built with Nexa" },
  "nav": { "home": "Home", "blog": "Blog" },
  "home": { "title": "Welcome", "latest": "Latest articles" },
  "blog": { "title": "All articles", "search": "Search an article…", "by": "by", "back": "← Back to the blog" }
}
```

> `site.name` no está en `en.json`: la página inglesa mostrará «Mi blog». Es el **fallback** al idioma por defecto,
> y te permite traducir poco a poco.

El **layout** envuelve *todas* las páginas (cabecera y pie). El único elemento `data-nexa-slot`, vacío, es donde se inserta cada página:

```tsx title="src/layout.tsx"
// Se envuelve TODA página con esto. El único elemento `data-nexa-slot` (vacío) es donde va la página.
export const head = {
    stylesheets: ["/site.css"],
};

export default function Layout() {
    return (
        <div class="shell">
            <header>
                <a class="brand" href={`/${params.locale}`}>{t("site.name")}</a>
                <nav>
                    <a href={`/${params.locale}`}>{t("nav.home")}</a>
                    <a href={`/${params.locale}/blog`}>{t("nav.blog")}</a>
                    <a href="/es" hreflang="es">ES</a>
                    <a href="/en" hreflang="en">EN</a>
                </nav>
            </header>
            <main data-nexa-slot></main>
            <footer>{t("site.tagline")}</footer>
        </div>
    );
}
```

Y un poco de estilo, que se copia tal cual al sitio:

```css title="public/site.css"
:root { --fg: #252223; --bg: #faf7ee; --muted: #5f5a55; --accent: #488c54; --line: #d6cfbb; }
@media (prefers-color-scheme: dark) { :root { --fg: #ece6d6; --bg: #262325; --muted: #b8b0a1; --accent: #93cf9b; --line: #55514f; } }
body { margin: 0; font: 16px/1.6 system-ui, sans-serif; color: var(--fg); background: var(--bg); }
.shell { max-width: 720px; margin: 0 auto; padding: 0 16px; }
header { display: flex; align-items: center; justify-content: space-between; padding: 16px 0; border-bottom: 1px solid var(--line); }
.brand { font-weight: 700; text-decoration: none; color: var(--fg); }
nav { display: flex; gap: 14px; }
a { color: var(--accent); }
.card { border: 1px solid var(--line); border-radius: 10px; padding: 4px 16px; margin: 12px 0; }
.card h3 { margin: 10px 0 0; }
.meta { color: var(--muted); font-size: .9rem; margin: 4px 0; }
.search { width: 100%; padding: 8px 12px; border: 1px solid var(--line); border-radius: 8px; font: inherit; margin: 8px 0; background: transparent; color: inherit; }
footer { margin: 32px 0; color: var(--muted); font-size: .9rem; }
```

## Paso 5 · Un componente y la portada

Un **componente** de Nexa Core se *expande al compilar*: no existe en el navegador, y por eso no cuesta ni un byte de JavaScript.

```tsx title="src/components/PostCard.tsx"
// Un componente de Nexa Core: NO existe en ejecución. Al compilar, `<PostCard post={p} />` se sustituye
// por este HTML con los props ya puestos (el resultado es el mismo que si lo escribieras a mano).
export default function PostCard(props) {
    return (
        <article class="card">
            <h3><a href={`/${props.locale}/blog/${props.post.slug}`}>{props.post.title}</a></h3>
            <p class="meta">{props.post.date} · {t("blog.by")} {props.post.author}</p>
            <p>{props.post.excerpt}…</p>
        </article>
    );
}
```

La portada pide los artículos a tu API con `load`. En una ruta `[locale]`, `:locale` se sustituye por el idioma de la página:

```tsx title="src/pages/[locale]/index.tsx"
import PostCard from "../../components/PostCard";

// Los datos vienen de TU API, al compilar. `:locale` se sustituye por el idioma de la página.
export const load = { url: "/api/posts?lang=:locale" };

export const seo = {
    title: `${t("site.name")} — ${t("home.title")}`,
    description: t("site.tagline"),
    canonical: `https://example.com/${params.locale}`,
};

export default function Home() {
    return (
        <section>
            <h1>{t("home.title")}</h1>
            <h2>{t("home.latest")}</h2>
            <For each={data}>{(post) => <PostCard post={post} locale={params.locale} />}</For>
        </section>
    );
}
```

Crea la carpeta con corchetes tal cual: `src/pages/[locale]/`. El nombre de cada archivo **es** la URL, y `[locale]` es un parámetro.

**Compruébalo.** Arranca el servidor de desarrollo (en la terminal del proyecto; la API del paso 2 debe seguir abierta):

```bash
NEXA_API_URL=http://127.0.0.1:9500 nexa dev
# PowerShell:  $env:NEXA_API_URL="http://127.0.0.1:9500"; nexa dev
```

Abre <http://127.0.0.1:4321/es>: verás «Bienvenido» y dos tarjetas con los artículos de la API.
Y <http://127.0.0.1:4321/en>: los mismos artículos con los textos en inglés. Edita cualquier archivo y el navegador se recarga solo.

## Paso 6 · El blog y la página de cada artículo

La lista de artículos:

```tsx title="src/pages/[locale]/blog/index.tsx"
import PostCard from "../../../components/PostCard";

export const load = { url: "/api/posts?lang=:locale" };

export const seo = {
    title: `${t("blog.title")} — ${t("site.name")}`,
    description: t("blog.title"),
    canonical: `https://example.com/${params.locale}/blog`,
};

export default function Blog() {
    return (
        <section>
            <h1>{t("blog.title")}</h1>

            {/* La ISLA: mejora progresiva. Sin JavaScript la lista de abajo sigue siendo HTML completo. */}
            <div data-nexa-island="search" data-nexa-strategy="load" data-nexa-props={{ placeholder: t("blog.search") }}></div>

            <div id="post-list">
                <For each={data}>{(post) => <PostCard post={post} locale={params.locale} />}</For>
            </div>
        </section>
    );
}
```

(El `<div data-nexa-island>` lo explicamos en el paso 8. Hasta entonces verás un aviso 404 de `/vendor/search.js` en la consola del navegador: es normal, la isla aún no existe.)

Y la página de **un** artículo: una ruta **dinámica**. `[slug]` es el parámetro, y estas tres líneas hacen el trabajo:

- `load` pide a tu API los datos de *esta* página;
- `paths` le dice a `nexa build` **qué páginas existen** (una por cada artículo y por cada idioma, sin escribir nada más);
- `seo` y `schema` generan `<title>`, `<meta>`, `canonical` y los datos estructurados (JSON-LD) para buscadores.

```tsx title="src/pages/[locale]/blog/[slug].tsx"
// Una ruta DINÁMICA: /es/blog/hola-nexa, /en/blog/hola-nexa, ...
export const load = { url: "/api/posts/:slug?lang=:locale" }; // los datos de ESTA página
export const paths = { url: "/api/posts" };                    // qué páginas pre-renderizar (los idiomas se añaden solos)

export const seo = {
    title: `${data.title} — ${t("site.name")}`,
    description: data.excerpt,
    canonical: `https://example.com/${params.locale}/blog/${params.slug}`,
};

// Datos estructurados para buscadores (JSON-LD).
export const schema = {
    type: "Article",
    headline: data.title,
    datePublished: data.date,
    author: { type: "Person", name: data.author },
};

export default function Post() {
    return (
        <article>
            <p><a href={`/${params.locale}/blog`}>{t("blog.back")}</a></p>
            <h1>{data.title}</h1>
            <p class="meta">{data.date} · {t("blog.by")} {data.author}</p>
            {/* El cuerpo es Markdown de la API: <Markdown> lo convierte a HTML y lo SANEA (sin <script>, sin javascript:). */}
            <Markdown text={data.body} />
        </article>
    );
}
```

El cuerpo de los artículos es **Markdown** que viene de la API. `<Markdown text={...} />` lo convierte a HTML al compilar y lo **sanea**:
sin `<script>`, sin `javascript:`, sin atributos `on*`. Puedes publicar contenido de usuarios sin miedo a inyecciones.

**Compruébalo.** Abre <http://127.0.0.1:4321/es/blog/hola-nexa>. Después pulsa **Ctrl+U** (ver código fuente): ahí está el `<h1>`,
el texto del artículo, el JSON-LD y los `<link rel="alternate" hreflang>` — **HTML real**, que es lo que ve Google, sin ejecutar JavaScript.

## Paso 7 · Idiomas

Ya lo tienes: cada página existe en `/es/...` y `/en/...`, con `hreflang` automático.

- `/en/blog/islas`: la API no tiene esa traducción y devuelve el español. El texto de la **interfaz** (en `en.json`) sí está en inglés.
- **La raíz `/`**: como no hay `src/pages/index.tsx`, Nexa genera una **puerta de idioma**: lleva al visitante a `/en` si su navegador está en inglés,
  y a `/es` en cualquier otro caso. Sin JavaScript hay enlaces a cada idioma.
- Añadir un idioma después es crear `src/locales/de.json`. Aparecen `/de/...` solos y lo que no traduzcas cae al español.

## Paso 8 · Una isla: el buscador

Una **isla** es un módulo JavaScript con una función `mount(elemento, props)`. Aquí, en TypeScript puro:

```ts title="islands/search.island.ts"
// Una ISLA escrita en TypeScript puro. Nexa la llama con `mount(elemento, props)`.
// Mejora una lista que YA existe en el HTML: filtra los artículos mientras escribes.
export default function mount(el: Element, props: Record<string, unknown>) {
    const input = document.createElement("input");
    input.type = "search";
    input.placeholder = String(props.placeholder ?? "Buscar…");
    input.className = "search";
    el.replaceChildren(input);

    const onInput = () => {
        const q = input.value.trim().toLowerCase();
        document.querySelectorAll<HTMLElement>("#post-list .card").forEach((card) => {
            card.hidden = q !== "" && !card.textContent!.toLowerCase().includes(q);
        });
    };
    input.addEventListener("input", onInput);

    // Lo que devuelves se llama al desmontar la isla.
    return () => input.removeEventListener("input", onInput);
}
```

Fíjate en que **mejora algo que ya existe**: la lista del blog es HTML completo. Si el JavaScript falla, el blog sigue funcionando.

Hay que empaquetarla a un único archivo con [esbuild](https://esbuild.github.io/). Crea el `package.json`:

```json title="package.json"
{
  "name": "tutorial-blog",
  "private": true,
  "type": "module",
  "scripts": {
    "api": "node api.mjs",
    "vendor": "esbuild islands/search.island.ts --bundle --format=esm --target=es2022 --minify --outfile=public/vendor/search.js"
  },
  "devDependencies": {
    "esbuild": "^0.28.2"
  }
}
```

```bash
npm install
npm run vendor
# public/vendor/search.js  425b
```

Eso genera `public/vendor/search.js`, el módulo que declaramos en `[imports]` (paso 3). En la página, el `<div data-nexa-island="search">`
(paso 6) lo carga cuando hace falta (`data-nexa-strategy="load"`) y le pasa `props` ya resueltos en el servidor, incluido el texto traducido.

**Compruébalo.** En <http://127.0.0.1:4321/es/blog> aparece un campo de búsqueda; escribe «isla» y solo queda un artículo.

> Dentro de una isla puedes usar **React, Vue o lo que prefieras**: Nexa solo exige la función `mount`. Mira `examples/react-islands-demo`.

## Paso 9 · El panel privado (Modo App)

El panel es una **SPA**: TSX completo que corre en el navegador, con su propio router. Cuatro archivos.

Primero, el estado de sesión y el cliente de la API:

```ts title="src/app/session.ts"
import { createApi, persisted, navigate } from "@nexa/app";

// El estado de la UI (el nombre) sobrevive a recargar la página. La credencial real es la cookie `sid`
// (HttpOnly) que pone tu backend: este código nunca la ve.
export const user = persisted<string | null>("tutorial.user", null);

// `/api` es del mismo origen gracias a `[proxy]` en nexa.toml: sin CORS, y la cookie viaja sola.
export const api = createApi({
    baseURL: "/api",
    // Si la sesión caduca en el backend (401), limpiamos la UI y volvemos al login.
    onUnauthorized: () => {
        user.value = null;
        navigate("/login", { replace: true });
    },
});
```

El **guard** protege las rutas del panel (sin sesión → `/login`):

```ts title="src/app/config.ts"
import { user } from "./session";

// Rutas protegidas: sin sesión, todo menos /login lleva a /login (y al revés).
// OJO: esto protege la INTERFAZ. Lo que protege los datos es que tu API responda 401.
export function guard(to: { path: string }) {
    if (!user.value && to.path !== "/login") return "/login";
    if (user.value && to.path === "/login") return "/";
}
```

> **Importante:** el guard protege la *interfaz*. Los *datos* los protege tu API respondiendo `401`. Nunca confíes solo en el navegador.

La página de login. Una **señal** (`state`) y `value={name}` enlazan el campo con el estado en los dos sentidos:

```tsx title="src/app/pages/login.tsx"
import { state, navigate } from "@nexa/app";
import { user, api } from "../session";

export default function Login() {
    const name = state("");
    const error = state("");

    async function enter() {
        try {
            const me = await api.post<{ name: string }>("/login", { name: name.value });
            user.value = me.name;
            navigate("/");
        } catch (e) {
            error.value = String(e);
        }
    }

    return (
        <main>
            <h1>Panel</h1>
            {/* `value={name}`: enlace en los dos sentidos con una señal. */}
            <input id="name" placeholder="Tu nombre" value={name} />
            <button id="enter" onClick={enter}>Entrar</button>
            <p id="error">{error}</p>
        </main>
    );
}
```

Y el panel (reemplaza el contador de ejemplo): lista los artículos y publica uno nuevo.

```tsx title="src/app/pages/index.tsx"
import { state, For, Show, navigate } from "@nexa/app";
import { user, api } from "../session";

interface Post { slug: string; title: string; date: string }

export default function Dashboard() {
    const posts = state<Post[]>([]);
    const title = state("");
    const body = state("");
    const saved = state("");

    const load = () => api.get<Post[]>("/posts").then((list) => (posts.value = list)).catch(() => {});
    load();

    async function create() {
        if (!title.value.trim()) return;
        const post = await api.post<Post>("/admin/posts", { title: title.value, body: body.value });
        saved.value = post.slug;
        title.value = "";
        body.value = "";
        await load();
    }

    async function logout() {
        await api.post("/logout");
        user.value = null;
        navigate("/login");
    }

    return (
        <main>
            <h1>Hola, {user}</h1>
            <button id="logout" onClick={logout}>Salir</button>

            <h2>Nuevo artículo</h2>
            <input id="title" placeholder="Título" value={title} />
            <textarea id="body" placeholder="Contenido en Markdown" value={body}></textarea>
            <button id="create" onClick={create}>Publicar</button>
            {/* `Show`: se muestra solo cuando hay algo guardado. */}
            <Show when={saved}>
                <p id="saved">Publicado: <a href={() => `/es/blog/${saved.value}`} data-nexa-reload>{saved}</a></p>
            </Show>

            <h2>Artículos ({() => posts.value.length})</h2>
            <ul id="list">
                <For each={posts}>{(p) => <li>{p.title} <small>({p.date})</small></li>}</For>
            </ul>
        </main>
    );
}
```

Las señales hacen que **solo cambie lo que depende de ellas**: `{user}`, `{() => posts.value.length}`, la lista con `<For>`.
No hay Virtual DOM ni "re-render" de componentes.

**Compruébalo.** Abre <http://127.0.0.1:4321/panel>: te lleva a `/panel/login`. Escribe tu nombre, entra, publica un artículo
y verás que aparece en la lista.

## Paso 10 · Revisar y compilar

```bash
NEXA_API_URL=http://127.0.0.1:9500 nexa lint
# Modo App: 2 ruta(s) y 4 módulo(s) sin errores.
# 10 página(s) revisada(s), sin avisos.

NEXA_API_URL=http://127.0.0.1:9500 nexa build
# Modo App: 2 ruta(s) en /panel (4 módulo(s) compilado(s) a dist/assets/app/) ...
# Compilado /es -> dist/es/index.html
# Compilado /en/blog/hola-nexa -> dist/en/blog/hola-nexa/index.html
# ...
# Generado dist/index.html: puerta de idioma ...
```

`nexa lint` revisa SEO, accesibilidad y errores de la SPA (úsalo en tu CI). `nexa build` genera `dist/`: HTML estático
para cada página, más los módulos de la SPA. **Todo lo que existía al compilar ya es HTML.**

## Paso 11 · Producción: un solo ejecutable

```bash
NEXA_API_URL=http://127.0.0.1:9500 nexa bundle --no-build --out mi-sitio
./mi-sitio --port 8080 --api-url http://127.0.0.1:9500
```

(En Windows usa `--out mi-sitio.exe` y ejecuta `.\mi-sitio.exe`.)

`mi-sitio` es **un único archivo**: el sitio dentro de una copia de `nexa`. Cópialo a un servidor y ejecútalo; no necesita Node, nginx ni nada más
(la API sí, claro, es tu backend). Incluye compresión, caché, cabeceras de seguridad, la SPA y el proxy a tu API.

Y ahora lo importante: **publica un artículo desde el panel** (<http://127.0.0.1:8080/panel>), pulsa el enlace «Publicado: …» y verás la página.
Ese artículo **no existía cuando compilaste**. El ejecutable lo renderizó al recibir la visita, lo guardó en caché y volverá a pedirlo a tu API
pasados `--revalidate` segundos (60 por defecto). Si tu API se cae, seguirá sirviendo la última copia.

## Lo que has construido

| Pieza | Dónde | Qué ganas |
|---|---|---|
| Páginas de contenido | `src/pages/` | HTML real, SEO, cero JavaScript |
| Idiomas | `src/locales/` + `[locale]` | `/es`, `/en`, `hreflang`, fallback, puerta de idioma |
| Componentes | `src/components/` | Reutilizas HTML sin coste en el navegador |
| Una isla | `islands/` | Interactividad solo donde hace falta |
| El panel | `src/app/` | Una SPA con login, estado y llamadas a tu API |
| Un ejecutable | `nexa bundle` | Despliegue sin dependencias y contenido que se publica sin recompilar |

## Siguientes pasos

- [Elige tu camino](ELIGE-TU-CAMINO.md): cuándo usar Core, una isla o el Modo App.
- [Tutorial: blog y tienda con API e idiomas](TUTORIAL-BLOG-TIENDA.md): más detalle sobre rutas dinámicas y publicación sin recompilar.
- [Seguridad](SEGURIDAD.md): qué protege Nexa y qué sigue siendo tuyo.
- [Referencia](REFERENCIA.md): cada función y cada opción.
