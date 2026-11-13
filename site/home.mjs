// La portada del sitio (bilingüe ES/EN). Recibe `shell` y `highlight` de build.mjs.

/** Un texto en los dos idiomas: el alternador de la barra superior muestra uno u otro. */
const t = (es, en) => `<span class="es">${es}</span><span class="en">${en}</span>`;

export function home({ shell, SITE_URL, REPO, highlight }) {
    const window = (title, code, lang) =>
        `<div class="ventana"><div class="barra"><span class="escamas"><i></i><i></i><i></i></span><span class="t">${title}</span></div><div class="cuerpo">${code}</div></div>`;
    const codeWin = (title, source, lang) => `<div class="code"><div class="bar"><i></i><i></i><i></i><span>${title}</span><button class="copy" type="button">Copiar</button></div><pre><code class="hljs language-${lang}">${highlight(source, lang)}</code></pre></div>`;

    const cards = [
        ["HTML real, SEO de serie", "Real HTML, SEO built in",
            "Cada página se compila a HTML completo, con <code>title</code>, <code>meta</code>, Open Graph, JSON-LD y <code>hreflang</code>. Google ve el contenido sin ejecutar nada.",
            "Every page compiles to complete HTML with <code>title</code>, <code>meta</code>, Open Graph, JSON-LD and <code>hreflang</code>. Search engines see your content without running anything."],
        ["JavaScript mínimo", "Minimal JavaScript",
            "Una página sin interacción no manda ni un byte de JavaScript. Lo interactivo se activa por nodo: al hacer clic, al verse o cuando el navegador está libre.",
            "A page with no interaction ships zero bytes of JavaScript. Interactivity activates per node: on click, when visible, or when the browser is idle."],
        ["Cualquier backend", "Any backend",
            "Lee tus datos con un <code>GET</code> HTTP: PHP, Go, Node, Python… Sin ORM ni servidor propio. <code>[proxy]</code> elimina CORS y deja funcionar las cookies.",
            "Reads your data with a plain HTTP <code>GET</code>: PHP, Go, Node, Python… No ORM, no bundled server. <code>[proxy]</code> removes CORS and keeps cookies working."],
        ["Islas", "Islands",
            "Un pedazo interactivo dentro de HTML estático, en TypeScript puro, React, Vue o lo que prefieras. Si el JavaScript falla, la página sigue funcionando.",
            "An interactive piece inside static HTML, in plain TypeScript, React, Vue or whatever you prefer. If the JavaScript fails, the page still works."],
        ["Modo App (SPA)", "App Mode (SPA)",
            "Una SPA en TSX con señales, router, <code>guard</code> y cliente de API, en el mismo proyecto. Para tu panel de administración o tu dashboard.",
            "A TSX SPA with signals, router, <code>guard</code> and API client, in the same project. For your admin panel or dashboard."],
        ["Componentes sin runtime", "Components with no runtime",
            "<code>&lt;Card title={data.x}&gt;</code> se expande al compilar: reutilizas HTML sin ningún coste en el navegador. También <code>&lt;For&gt;</code>, <code>&lt;Show&gt;</code>, <code>&lt;Markdown&gt;</code>.",
            "<code>&lt;Card title={data.x}&gt;</code> expands at build time: reuse HTML at zero browser cost. Plus <code>&lt;For&gt;</code>, <code>&lt;Show&gt;</code>, <code>&lt;Markdown&gt;</code>."],
        ["Idiomas que crecen contigo", "Languages that grow with you",
            "Empieza con <code>es.json</code>; añade <code>en.json</code> y <code>de.json</code> cuando quieras: aparecen <code>/en</code> y <code>/de</code>, con <code>hreflang</code>, fallback y puerta de idioma.",
            "Start with <code>es.json</code>; add <code>en.json</code> and <code>de.json</code> later: <code>/en</code> and <code>/de</code> appear, with <code>hreflang</code>, fallback and a language gate."],
        ["Un solo ejecutable", "A single executable",
            "<code>nexa bundle</code> empaqueta el sitio con su servidor: gzip, caché, SPA y proxy. Y renderiza bajo demanda lo que publicas <em>después</em> de compilar.",
            "<code>nexa bundle</code> packs the site with its server: gzip, cache, SPA and proxy. And it renders on demand whatever you publish <em>after</em> building."],
        ["Seguridad probada", "Tested security",
            "Todo lo que viene de tu API se escapa; <code>javascript:</code> y <code>onclick</code> desde datos se omiten; el HTML y Markdown se sanean. Probado con ataques reales en Chrome.",
            "Everything from your API is escaped; <code>javascript:</code> and <code>onclick</code> from data are dropped; HTML and Markdown are sanitized. Tested with real attacks in Chrome."],
        ["Sin backend, sin pasos manuales", "No backend, no manual steps",
            "Blog y documentación en archivos Markdown (<code>content:/posts</code>), islas y CSS empaquetados solos con esbuild, ruta comodín, <code>404.tsx</code>, redirecciones, feed RSS y una CSP sin <code>unsafe-inline</code>.",
            "Blog and docs from Markdown files (<code>content:/posts</code>), islands and CSS bundled automatically with esbuild, catch-all routes, <code>404.tsx</code>, redirects, an RSS feed and a CSP without <code>unsafe-inline</code>."],
        ["Escritorio con Tauri", "Desktop with Tauri",
            "El mismo proyecto, convertido en una app nativa de unos 11 MB con <code>nexa build --target desktop</code>, con un puente a funciones de Rust.",
            "The same project, turned into a ~11 MB native app with <code>nexa build --target desktop</code>, with a bridge to Rust functions."],
    ].map(([tes, ten, des, den]) => `<div class="card"><h3>${t(tes, ten)}</h3><p>${t(des, den)}</p></div>`).join("\n");

    const coreCode = `export const load = {\n  url: "/api/posts/:slug",\n};\n\nexport default function Post() {\n  return (\n    <article>\n      <h1>{data.title}</h1>\n      <Markdown text={data.body} />\n    </article>\n  );\n}`;
    const islandCode = `<div\n  data-nexa-island="search"\n  data-nexa-strategy="visible"\n/>\n\n// search.island.ts\nexport default function mount(el) {\n  /* tu código */\n}`;
    const appCode = `import { state } from "@nexa/app";\n\nexport default function C() {\n  const n = state(0);\n  return (\n    <button onClick={() => n.value++}>\n      Clics: {n}\n    </button>\n  );\n}`;
    const pre = (src, lang) => `<pre><code class="hljs language-${lang}">${highlight(src, lang)}</code></pre>`;

    const pieces = `<div class="tres">
${window("src/pages/blog/[slug].tsx", `<h3>Nexa Core</h3><p>${t("HTML real para lo que debe indexarse: blog, tienda, catálogo, landing.", "Real HTML for whatever must be indexed: blog, shop, catalog, landing.")}</p>${pre(coreCode, "typescript")}`)}
${window("islands/search.island.ts", `<h3>${t("Isla", "Island")}</h3><p>${t("Un pedazo interactivo dentro de una página estática. Se carga solo cuando hace falta.", "An interactive piece inside a static page. Loads only when needed.")}</p>${pre(islandCode, "typescript")}`)}
${window("src/app/pages/index.tsx", `<h3>${t("Modo App", "App Mode")}</h3><p>${t("Una SPA para lo privado: panel, dashboard, apps con login.", "An SPA for the private side: panel, dashboard, apps with login.")}</p>${pre(appCode, "typescript")}`)}
</div>`;

    const slugExample = `// src/pages/[locale]/blog/[slug].tsx  →  /es/blog/hola-nexa  ·  /en/blog/hola-nexa
export const load  = { url: "/api/posts/:slug?lang=:locale" };  // los datos de ESTA página
export const paths = { url: "/api/posts" };                     // qué páginas pre-renderizar
export const seo   = { title: \`\${data.title} — \${t("site.name")}\`, description: data.excerpt };

export default function Post() {
  return (
    <article>
      <h1>{data.title}</h1>
      <Markdown text={data.body} />   {/* Markdown de tu API, saneado */}
    </article>
  );
}`;

    const main = `<main id="contenido">
<div class="agua">
<i class="anillo"></i><i class="anillo"></i><i class="anillo"></i>
<div class="ventana hero"><div class="barra"><span class="escamas"><i></i><i></i><i></i></span><span class="t">~/mi-sitio — nexa dev</span></div>
<div class="cuerpo">
<img class="logo" src="./assets/logo.svg" alt="Nexa" width="84" height="84">
<h1>Nexa</h1>
<p class="ver"><span class="badge">v0.1.0 · MIT</span></p>
<p class="lead">${t("HTML real para los buscadores. JavaScript solo donde hace falta. Y el backend que tú quieras.", "Real HTML for search engines. JavaScript only where you need it. And any backend you like.")}</p>
<div class="btns"><a class="btn main" href="./tutorial/">${t("Empezar el tutorial", "Start the tutorial")}</a><a class="btn" href="./guia/">${t("Instalación", "Install")}</a><a class="btn" href="${REPO}" rel="noopener">GitHub</a></div>
</div></div></div>

<div class="wrap">
<section class="band"><div class="stats">
<div class="ventana stat"><b>0 B</b><span>${t("de JavaScript en una página sin interacción", "of JavaScript on a page with no interaction")}</span></div>
<div class="ventana stat"><b>+1000</b><span>${t("pruebas automáticas, incluidas las de Chrome real", "automated tests, including real-Chrome ones")}</span></div>
<div class="ventana stat"><b>≈17 MB</b><span>${t("el sitio y su servidor, en un solo ejecutable", "the site and its server, in one executable")}</span></div>
</div></section>

<section class="band" id="por-que"><h2>${t("Por qué Nexa", "Why Nexa")}</h2>
<p class="sub">${t("Casi todos los frameworks parten de «toda la página es una app de JavaScript» y luego recuperan el SEO. Nexa parte al revés: toda página es HTML real desde el primer byte, y el JavaScript es un añadido explícito.", "Most frameworks start from “the whole page is a JavaScript app” and then recover SEO. Nexa starts the other way: every page is real HTML from the first byte, and JavaScript is an explicit add-on.")}</p>
<div class="cards">${cards}</div></section>

<section class="band" id="piezas"><h2>${t("Tres piezas, un proyecto", "Three pieces, one project")}</h2>
<p class="sub">${t("Una web real tiene contenido que debe indexarse, partes interactivas y un panel privado. Nexa te da una herramienta para cada una, y las combinas en el mismo proyecto y dominio.", "A real site has content that must be indexed, interactive parts and a private panel. Nexa gives you one tool for each, combined in the same project and domain.")}</p>
${pieces}</section>

<section class="band" id="codigo"><h2>${t("Un vistazo al código", "A look at the code")}</h2>
<p class="sub">${t("Una ruta dinámica de blog, con idiomas, SEO, datos estructurados y Markdown, en una sola página.", "A dynamic blog route, with languages, SEO, structured data and Markdown, in a single page.")}</p>
${codeWin("src/pages/[locale]/blog/[slug].tsx", slugExample, "typescript")}</section>

<section class="band" id="empezar"><h2>${t("Empieza en tres pasos", "Start in three steps")}</h2>
<p class="sub">${t("Necesitas Rust (por ahora Nexa se compila desde el código fuente) y Node.", "You need Rust (for now Nexa is built from source) and Node.")}</p>
<ol class="steps">
<li><h3>${t("Instala Nexa", "Install Nexa")}</h3>${codeWin("bash", "git clone https://github.com/oweeme/oweenexa.git\ncd oweenexa\ncargo install --path crates/nexa-cli --root ~/.local", "bash")}</li>
<li><h3>${t("Crea un proyecto", "Create a project")}</h3>${codeWin("bash", "nexa create mi-sitio --template app\ncd mi-sitio", "bash")}</li>
<li><h3>${t("Ábrelo", "Run it")}</h3>${codeWin("bash", "nexa dev\n# http://127.0.0.1:4321", "bash")}<p>${t("Sigue el <a href=\"./tutorial/\">tutorial paso a paso</a>: construyes un blog bilingüe con un buscador y un panel, y lo despliegas como un solo ejecutable.", "Follow the <a href=\"./tutorial/\">step-by-step tutorial</a>: you build a bilingual blog with a search box and a panel, and ship it as a single executable.")}</p></li>
</ol></section>

<section class="band" id="cuando"><h2>${t("Cuándo usarlo, y cuándo no", "When to use it, and when not")}</h2>
<div class="cards">
<div class="card"><h3>${t("Ideal para", "Great for")}</h3><p>${t("Blogs, tiendas y catálogos con SEO · sitios multi-idioma · landings y documentación · paneles de administración · apps medianas con un backend propio · apps de escritorio ligeras.", "Blogs, shops and catalogs with SEO · multilingual sites · landings and docs · admin panels · mid-size apps with your own backend · lightweight desktop apps.")}</p></div>
<div class="card"><h3>${t("No es para", "Not for")}</h3><p>${t("Redes sociales enormes, editores gráficos tipo Figma o juegos 3D pesados. Si dependes del ecosistema de React, úsalo dentro de una isla, pero no esperes que Nexa lo reemplace.", "Huge social networks, Figma-like graphic editors or heavy 3D games. If you rely on React’s ecosystem, use it inside an island, but don’t expect Nexa to replace it.")}</p></div>
<div class="card"><h3>${t("Estado del proyecto", "Project status")}</h3><p>${t("Joven y de un solo autor: las APIs pueden cambiar. Probado a fondo en Linux; en Windows y macOS pasan los tests de Rust y una prueba de humo en cada integración continua, y la app de escritorio compila en los tres sistemas (las pruebas de navegador, solo en Linux).", "Young and single-author: APIs may change. Thoroughly tested on Linux; on Windows and macOS the Rust tests and a smoke test run in every CI build, and the desktop app builds on all three (browser tests, Linux only).")}</p></div>
</div></section>

<section class="band" id="docs"><h2>${t("Documentación", "Documentation")}</h2>
<p class="sub">${t("En español. La referencia cubre cada función y cada opción.", "In Spanish. The reference covers every function and option.")}</p>
<div class="cards">
<a class="card" href="./tutorial/" style="text-decoration:none;color:inherit"><h3>${t("Tutorial paso a paso", "Step-by-step tutorial")}</h3><p>${t("Un blog bilingüe, una isla y un panel, desde cero.", "A bilingual blog, an island and a panel, from scratch.")}</p></a>
<a class="card" href="./conceptos/" style="text-decoration:none;color:inherit"><h3>${t("Core, isla o Modo App", "Core, island or App Mode")}</h3><p>${t("Qué usar para qué, con un ejemplo real.", "What to use for what, with a real example.")}</p></a>
<a class="card" href="./blog-y-tienda/" style="text-decoration:none;color:inherit"><h3>${t("Blog y tienda con API", "Blog and shop with an API")}</h3><p>${t("Rutas dinámicas, idiomas y publicar sin recompilar.", "Dynamic routes, languages and publishing without rebuilding.")}</p></a>
<a class="card" href="./seguridad/" style="text-decoration:none;color:inherit"><h3>${t("Seguridad", "Security")}</h3><p>${t("Qué protege Nexa y qué sigue siendo tuyo.", "What Nexa protects and what is still yours.")}</p></a>
<a class="card" href="./librerias/" style="text-decoration:none;color:inherit"><h3>React, Vue, Phaser…</h3><p>${t("Cómo usar librerías de terceros en islas y en la SPA.", "How to use third-party libraries in islands and the SPA.")}</p></a>
<a class="card" href="./referencia/" style="text-decoration:none;color:inherit"><h3>${t("Referencia", "Reference")}</h3><p>${t("Cada función, atributo y opción de <code>nexa.toml</code>.", "Every function, attribute and <code>nexa.toml</code> option.")}</p></a>
</div></section>
</div>
</main>`;

    return shell({
        rootP: "./", url: "", nav: "home",
        title: "Nexa — HTML real, JavaScript solo donde hace falta",
        description: "Framework frontend HTML-first, SEO-first y backend-agnóstico. Compilador en Rust, aplicación en TypeScript: páginas con SEO, islas interactivas, una SPA para tu panel y un solo ejecutable para producción.",
        main,
    });
}
