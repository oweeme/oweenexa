import puppeteer from "puppeteer-core";
const base = process.argv[2];
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const page = await browser.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("response", (r) => { if (r.status() >= 400 && !r.url().endsWith("/favicon.ico")) errors.push("HTTP " + r.status() + " " + r.url()); });
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

// 1. El HTML crudo (lo que ve Google, sin ejecutar JS) ya trae el catálogo de la API
const raw = await (await fetch(base + "/shop")).text();
ok("HTML crudo: título y grupos de la API ya están (SEO, sin JS)", raw.includes("<h1>Equipos</h1>") && raw.includes("Diseño") && raw.includes("Carla"));
ok("HTML crudo: el fallback de la isla es HTML real", raw.includes("React se está cargando"));
ok("HTML crudo: las props de la isla vienen de la API", raw.includes("data-nexa-props") && raw.includes("Carla"));

// 2. La página en el navegador: islas hidratadas
await page.goto(base + "/shop", { waitUntil: "networkidle0" });
await page.waitForSelector("#react-root", { timeout: 8000 });
ok("isla de React montada (el fallback se reemplazó)", !(await page.content()).includes("React se está cargando"));
ok("React recibió los datos de la API como props (3 miembros)", (await page.$$("#react-list li")).length === 3);
await page.type("#react-q", "ar");
await wait(100);
ok("estado de React: el filtro deja 1 (Carla)", (await page.$$("#react-list li")).length === 1);
await page.click("#react-btn");
await page.click("#react-btn");
ok("useState de React: 2 clics = 2", (await page.$eval("#react-btn", (e) => e.textContent)) === "clics: 2");

ok("isla en TypeScript puro montada con los mismos datos", (await page.$eval("#ts-btn", (e) => e.textContent)) === "TS puro: 2 grupos");
await page.click("#ts-btn");
ok("TS puro: el clic funciona", (await page.$eval("#ts-btn", (e) => e.textContent)).endsWith("clic 1"));
ok("el HTML estático de SEO sigue en la página junto a las islas", (await page.$$("#static-list article")).length === 2);

// 3. React dentro del Modo App (SPA), sin isla: montado con ref
await page.goto(base + "/app", { waitUntil: "networkidle0" });
await page.waitForSelector("#app-react-btn", { timeout: 8000 });
await page.click("#app-react-btn");
ok("React dentro de la SPA de Nexa (ref + createRoot)", (await page.$eval("#app-react-btn", (e) => e.textContent)) === "React en la SPA: 1");

ok("sin errores de JS ni 404", errors.length === 0);
if (errors.length) console.log(errors);
await browser.close();
