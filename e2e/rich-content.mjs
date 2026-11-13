// Contenido enriquecido de una API (HTML y Markdown) con ataques: se ve bien y NADA se ejecuta.
// Requiere examples/blog-store-demo (api.py en :9300) compilado y servido (preview/bundle).
import puppeteer from "puppeteer-core";
const base = process.argv[2];
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const page = await browser.newPage();
const dialogs = [];
page.on("dialog", async (d) => { dialogs.push(d.message()); await d.dismiss(); });
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };

await page.goto(base + "/articles/demo", { waitUntil: "networkidle0" });
await new Promise((r) => setTimeout(r, 500)); // por si algún onerror/onload tardara

ok("NADA se ejecutó: window.__pwned sin definir", (await page.evaluate(() => window.__pwned)) === undefined);
ok("ningún diálogo (alert/confirm) se abrió", dialogs.length === 0);
const count = (sel) => page.$$eval(sel, (e) => e.length);

// HTML enriquecido
ok("HTML: subtítulo, negrita y lista se ven como etiquetas reales", (await count("#html h2")) === 1 && (await count("#html strong")) === 1 && (await count("#html li")) === 2);
ok("HTML: el bloque de código conserva su clase de resaltado", (await count("#html pre code.language-rust")) === 1);
ok("HTML: la imagen https se queda, perezosa", (await page.$eval("#html img[alt=foto]", (i) => i.getAttribute("loading"))) === "lazy");
ok("HTML: el enlace https se queda con rel seguro", await page.$eval("#html a[href^=https]", (a) => a.rel.includes("noopener") && a.rel.includes("nofollow")));
ok("HTML: <script>, <svg>, <iframe> no existen en el artículo", (await count("#html script, #html svg, #html iframe")) === 0);
ok("HTML: ninguna etiqueta conserva un atributo on* ni style", (await page.$$eval("#html *", (els) => els.every((e) => [...e.attributes].every((a) => !a.name.startsWith("on") && a.name !== "style")))));
ok("HTML: el enlace javascript: perdió su href (queda como texto)", (await page.$eval("#html #evil, #html a:not([href])", (a) => a.getAttribute("href"))) === null);
ok("HTML: la imagen con onerror desapareció o quedó sin manejador", (await page.$$eval("#html img", (imgs) => imgs.every((i) => !i.hasAttribute("onerror")))));

// Markdown
ok("Markdown: título, negrita, código, lista y tabla", (await count("#md h1")) === 1 && (await count("#md strong")) === 1 && (await count("#md code")) >= 1 && (await count("#md li")) === 2 && (await count("#md table td")) === 2);
ok("Markdown: enlace https ok; el de javascript: sin href", (await count("#md a[href^='https://oweeme.com']")) === 1 && (await count("#md a[href^='javascript']")) === 0);
ok("Markdown: <script> y <img onerror> incrustados no existen", (await count("#md script")) === 0 && (await page.$$eval("#md img", (i) => i.every((x) => !x.hasAttribute("onerror")))));

// El mismo valor por la expresión normal sigue escapado (se ve como TEXTO)
const asText = await page.$eval("#as-text", (p) => p.textContent);
ok("{data.html} sin <SafeHtml> sigue escapado: se ve el texto con las etiquetas", asText.includes("<h2>Subtítulo</h2>") && (await count("#as-text *")) === 0);

// El HTML crudo (lo que ve el buscador, sin JavaScript) tampoco trae nada activo
const raw = await (await fetch(base + "/articles/demo")).text();
// Solo las dos secciones saneadas: la tercera (#escaped) muestra a propósito el ataque como TEXTO escapado.
const article = raw.slice(raw.indexOf("<article"), raw.indexOf('<section id="escaped"'));
ok("HTML crudo (sin JavaScript): ni <script>, <iframe>, <svg>, on*= ni href=javascript en el contenido saneado", !/<script|<iframe|<svg|\son\w+=|href="javascript/i.test(article));

await browser.close();
