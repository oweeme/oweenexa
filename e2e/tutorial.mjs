// El proyecto del tutorial (examples/tutorial-blog), de punta a punta en un Chrome real:
// páginas con SEO, idiomas, puerta de idioma, la isla del buscador y el panel (SPA) que publica un artículo
// que aparece bajo demanda. Requiere la API del ejemplo (`node api.mjs`, :9500) y el sitio servido
// (ejecutable de `nexa bundle`, que es quien renderiza bajo demanda).
import puppeteer from "puppeteer-core";
const base = process.argv[2];
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [];
async function newPage(langs) {
    const page = await browser.newPage();
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("dialog", (d) => { errors.push("diálogo: " + d.message()); d.dismiss(); });
    if (langs) await page.evaluateOnNewDocument((l) => Object.defineProperty(navigator, "languages", { get: () => l }), langs);
    return page;
}
const text = async (url) => (await fetch(base + url)).text();

// 1. HTML real con SEO (lo que ve Google, sin JavaScript)
const post = await text("/es/blog/hola-nexa");
ok("el post ya es HTML real: el Markdown de la API llegó convertido", post.includes("<h2>Un sitio con HTML real</h2>") && post.includes("<strong>Markdown</strong>"));
ok("SEO: <title>, JSON-LD (Article) y hreflang con x-default", post.includes("<title>Hola, Nexa — Mi blog</title>") && post.includes('"@type":"Article"') && post.includes('hreflang="x-default"'));
ok("el layout envuelve la página (cabecera, pie y hoja de estilo)", post.includes('class="brand"') && post.includes("Un sitio hecho con Nexa") && post.includes('href="/site.css"'));
const blogList = await text("/es/blog");
ok("la lista del blog ya está en el HTML (sin JavaScript)", blogList.includes("Qué es una isla") && blogList.includes("Hola, Nexa"));

// 2. Idiomas: textos del idioma, y la clave sin traducir cae al español
const en = await text("/en");
ok("/en: textos en inglés", en.includes("Welcome") && en.includes("Latest articles"));
ok("/en/blog/hola-nexa: título traducido por la API", (await text("/en/blog/hola-nexa")).includes("<h1>Hello, Nexa</h1>"));
ok("/en/blog/islas: sin traducción en la API, cae al español", (await text("/en/blog/islas")).includes("<h1>Qué es una isla</h1>"));
ok("/en: 'site.name' no existe en en.json y cae al español", en.includes("Mi blog"));

// 3. La puerta de idioma
for (const [langs, expected] of [[["en-US", "en"], "/en"], [["es-ES"], "/es"], [["fr-FR"], "/es"]]) {
    const page = await newPage(langs);
    await page.goto(base + "/", { waitUntil: "networkidle0" });
    ok(`puerta de idioma: ${langs[0]} -> ${expected}`, new URL(page.url()).pathname === expected);
    await page.close();
}

// 4. La isla del buscador: mejora progresiva
{
    const page = await newPage();
    await page.goto(base + "/es/blog", { waitUntil: "networkidle0" });
    await page.waitForSelector("input.search", { timeout: 8000 });
    ok("la isla se montó (apareció el buscador con el texto del idioma)", (await page.$eval("input.search", (i) => i.placeholder)) === "Buscar un artículo…");
    ok("antes de buscar se ven todos los artículos", (await page.$$eval("#post-list .card:not([hidden])", (c) => c.length)) === 2);
    await page.type("input.search", "isla");
    await sleep(100);
    const visible = await page.$$eval("#post-list .card:not([hidden])", (c) => c.map((x) => x.querySelector("h3").textContent));
    ok("escribir 'isla' filtra: queda solo «Qué es una isla»", visible.length === 1 && visible[0] === "Qué es una isla");
    await page.close();
}

// 5. El panel (Modo App): login, publicar y ver el artículo nuevo bajo demanda
const page = await newPage();
await page.goto(base + "/panel", { waitUntil: "networkidle0" });
await page.waitForSelector("#name");
ok("sin sesión, el guard manda al login", new URL(page.url()).pathname === "/panel/login");
await page.type("#name", "Héctor");
await page.click("#enter");
await page.waitForSelector("#create");
ok("tras entrar: saludo con la señal del usuario y la lista de la API", (await page.$eval("h1", (h) => h.textContent)) === "Hola, Héctor" && (await page.$$("#list li")).length === 2);

await page.type("#title", "Mi primer artículo");
await page.type("#body", "## Publicado desde el panel\n\nTexto con **negrita**.\n\n<script>window.__pwned=1</script>\n\n[malo](javascript:window.__pwned=1)");
await page.click("#create");
await page.waitForSelector("#saved a");
ok("publicar muestra el enlace al artículo y actualiza la lista", (await page.$eval("#saved a", (a) => a.getAttribute("href"))) === "/es/blog/mi-primer-articulo" && (await page.$$("#list li")).length === 3);

await page.goto(base + "/es/blog/mi-primer-articulo", { waitUntil: "networkidle0" });
const html = await page.content();
ok("el artículo publicado DESPUÉS del build existe (renderizado bajo demanda)", html.includes("<h1>Mi primer artículo</h1>") && html.includes("<h2>Publicado desde el panel</h2>"));
ok("su Markdown se convirtió a HTML", (await page.$$eval("article strong", (s) => s.length)) === 1);
ok("el <script> y el javascript: del Markdown NO se ejecutaron ni existen", (await page.evaluate(() => window.__pwned)) === undefined && (await page.$$("article script")).length === 0 && !html.includes('href="javascript'));

// 6. Cerrar sesión y proteger la API
await page.goto(base + "/panel", { waitUntil: "networkidle0" });
await page.waitForSelector("#logout");
await page.click("#logout");
await page.waitForSelector("#name");
ok("cerrar sesión vuelve al login", new URL(page.url()).pathname === "/panel/login");
const status = await page.evaluate(async () => (await fetch("/api/admin/posts", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ title: "x" }) })).status);
ok("sin cookie, la API rechaza publicar (401): lo que protege los datos es el backend", status === 401);

ok("sin errores de JavaScript ni diálogos", errors.length === 0);
if (errors.length) console.log(errors);
await browser.close();
