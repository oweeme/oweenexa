// examples/content-demo: un sitio SIN backend. Contenido en src/content/ (Markdown), ruta comodín
// `[...path]`, página 404 propia, redirecciones y cabeceras de nexa.toml, e islas/CSS empaquetados por nexa.
import puppeteer from "puppeteer-core";
const base = process.argv[2];
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };
const get = (url, opts = {}) => fetch(base + url, { redirect: "manual", ...opts });
const text = async (url) => (await get(url)).text();

// 1. contenido local → HTML real
const blog = await text("/blog");
ok("la lista del blog sale de src/content/posts (más reciente primero)", blog.indexOf("Un segundo artículo") > -1 && blog.indexOf("Un segundo artículo") < blog.indexOf("Hola desde un archivo"));
ok("un borrador (draft: true) no se publica ni en la lista ni como página", !blog.includes("Aún no") && (await get("/blog/borrador")).status === 404);
const post = await text("/blog/hola");
ok("el Markdown del archivo llega convertido y SANEADO (sin <script> ni javascript:)", post.includes("<h2>Sin backend</h2>") && !post.includes("<script>alert") && !post.includes("javascript:"));
ok("SEO sale de la cabecera del archivo", post.includes("<title>Hola desde un archivo — Content demo</title>"));

const p1 = await text("/blog/page/1"), p2 = await text("/blog/page/2");
ok("paginación local: /blog/page/1 y /blog/page/2 traen un artículo cada una (más reciente primero)", p1.includes("Un segundo artículo") && !p1.includes("Hola desde") && p2.includes("Hola desde") && !p2.includes("Un segundo"));
ok("una página que no existe (/blog/page/3) es 404", (await get("/blog/page/3")).status === 404);

// 2. ruta comodín [...path]
ok("`[...path]` sirve /docs/guia/instalar (varios segmentos)", (await text("/docs/guia/instalar")).includes("nexa create mi-sitio"));
ok("una subcarpeta con index.md es su propia página (/docs/guia)", (await text("/docs/guia")).includes("Índice de la guía"));
ok("un camino que no existe bajo el comodín es 404, no un error", (await get("/docs/a/b/c")).status === 404);
ok("el contenido no se sale de src/content (../ y %2e%2e)", [(await get("/docs/%2e%2e/%2e%2e/nexa.toml")).status, (await get("/docs/..%2f..%2fnexa.toml")).status].every((s) => s === 404 || s === 400));

// 2b. layout anidado: src/pages/docs/_layout.tsx envuelve SOLO las páginas de /docs, dentro del layout global
const docPage = await text("/docs/guia/instalar");
ok("layout anidado: la barra lateral de /docs envuelve la página, dentro del layout global",
    docPage.includes('class="docs-nav"') && docPage.indexOf('<nav><a href="/">') < docPage.indexOf('class="docs-nav"') && docPage.indexOf('class="docs-nav"') < docPage.indexOf("nexa create mi-sitio"));
ok("fuera de /docs no hay barra lateral, y `_layout.tsx` no es una página", !(await text("/blog")).includes("docs-nav") && (await get("/docs/_layout")).status === 404);

// 3. página 404 propia, con estado 404
const nf = await get("/no-existe");
ok("/no-existe responde 404 con TU página (src/pages/404.tsx)", nf.status === 404 && (await nf.text()).includes("404 — No encontrada"));

// 3b. feed RSS generado a partir de la colección
const feedRes = await get("/feed.xml");
const feedXml = await feedRes.text();
ok("[feed]: /feed.xml es RSS con las entradas del blog (sin el borrador), más reciente primero",
    (feedRes.headers.get("content-type") ?? "").includes("xml") && feedXml.includes("<rss") && feedXml.includes("/blog/segundo") && feedXml.includes("/blog/hola") && !feedXml.includes("borrador") && feedXml.indexOf("/blog/segundo") < feedXml.indexOf("/blog/hola"));
ok("[feed]: cada página lo anuncia con <link rel=\"alternate\" type=\"application/rss+xml\">", (await text("/blog")).includes('type="application/rss+xml"'));

// 4. redirecciones y cabeceras de nexa.toml
const r1 = await get("/articulos");
ok("[redirects] simple: /articulos → /blog (301)", r1.status === 301 && new URL(r1.headers.get("location"), base).pathname === "/blog");
const r2 = await get("/guia/instalar");
ok("[redirects] con :param y estado propio: /guia/instalar → /docs/guia/instalar (302)", r2.status === 302 && new URL(r2.headers.get("location"), base).pathname === "/docs/guia/instalar");
const open = await get("/guia/%2Fevil.com");
ok("una redirección nunca se convierte en //otro-sitio (redirección abierta)", !(open.headers.get("location") ?? "").startsWith("//"));
const home = await get("/");
ok("[headers]: cabecera propia en todas las respuestas", home.headers.get("x-demo") === "content-demo");
const homeHtml = await home.text();
const csp = homeHtml.match(/http-equiv="Content-Security-Policy" data-nexa-csp content="([^"]+)"/)?.[1] ?? "";
ok("[security] csp: <meta> con tu política y un hash por cada script en línea (sin 'unsafe-inline')", csp.includes("default-src 'self'") && /script-src 'self' 'sha256-/.test(csp) && !csp.includes("unsafe-inline"));
ok("el 404 y las páginas de contenido también llevan su CSP", (await text("/blog/hola")).includes("data-nexa-csp") && (await text("/no-existe")).includes("data-nexa-csp"));

// 5. recursos empaquetados por nexa (esbuild): isla con import propio + CSS, y hoja de estilos
const html = await text("/");
const island = html.match(/"counter":"([^"]+)"/)?.[1];
const css = html.match(/href="(\/assets\/styles\/main\.[0-9a-f]+\.css)"/)?.[1];
ok("la isla `counter` entra sola al import map y la hoja de src/styles/ al <head>", !!island && !!css);
ok("los recursos son inmutables (nombre con hash → caché larga)", (await get(island)).headers.get("cache-control")?.includes("immutable") && (await get(css)).headers.get("cache-control")?.includes("immutable"));
ok("@import del CSS quedó empaquetado (la variable de _vars.css está dentro)", (await text(css)).includes("--accent"));

const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const page = await browser.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
page.on("console", (m) => { if (m.type() === "error" && !/favicon/.test(m.location()?.url ?? "")) errors.push("consola: " + m.text()); });
page.on("requestfailed", (r) => errors.push("falló " + r.url()));
await page.goto(base + "/", { waitUntil: "networkidle0" });
await page.waitForSelector("button.counter");
await page.click("button.counter"); await page.click("button.counter");
ok("la isla (con su módulo y su CSS importados) funciona en el navegador: 2 clics = 2", (await page.$eval("button.counter", (b) => b.textContent)) === "Clics: 2");
ok("el CSS de la isla se aplicó (borde del botón)", (await page.$eval("button.counter", (b) => getComputedStyle(b).borderTopWidth)) === "2px");
ok("la hoja global se aplicó (variable CSS importada con @import)", (await page.$eval("body", (b) => getComputedStyle(b).backgroundColor)) === "rgb(251, 250, 247)");
ok("la CSP no bloquea nada: sin errores en la consola del navegador", errors.length === 0);
if (errors.length) console.log("   ", errors);
await browser.close();
