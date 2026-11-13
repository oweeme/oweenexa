import puppeteer from "puppeteer-core";
const base = process.argv[2];
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };
async function visit(langs, path = "/") {
    const page = await browser.newPage();
    await page.evaluateOnNewDocument((l) => Object.defineProperty(navigator, "languages", { get: () => l }), langs);
    await page.goto(base + path, { waitUntil: "networkidle0" });
    const url = new URL(page.url());
    const h1 = await page.$eval("h1", (e) => e.textContent).catch(() => null);
    await page.close();
    return { path: url.pathname + url.search + url.hash, h1 };
}
const de = await visit(["de-DE", "de", "en"]);
ok(`navegador alemán -> /de (${de.path}, "${de.h1}")`, de.path === "/de" && de.h1 === "Willkommen");
const en = await visit(["en-US"]);
ok(`navegador inglés -> /en (${en.path}, "${en.h1}")`, en.path === "/en" && en.h1 === "Welcome");
const fr = await visit(["fr-FR", "fr"]);
ok(`idioma que el sitio NO tiene (francés) -> idioma por defecto /es (${fr.path})`, fr.path === "/es");
const fr_en = await visit(["fr-FR", "en-GB"]);
ok(`francés y luego inglés -> /en (${fr_en.path})`, fr_en.path === "/en");
const q = await visit(["de"], "/?utm_source=x#top");
ok(`conserva ?query y #hash al redirigir (${q.path})`, q.path === "/de?utm_source=x#top");
const raw = await (await fetch(base + "/")).text();
ok("sin JavaScript: enlaces a cada idioma + canonical + x-default", raw.includes('href="/de"') && raw.includes('hreflang="x-default" href="/es"') && raw.includes('rel="canonical" href="/es"'));
const deep = await fetch(base + "/blog");
ok("/blog sin prefijo de idioma sigue siendo 404 (no se adivina)", deep.status === 404);
await browser.close();
