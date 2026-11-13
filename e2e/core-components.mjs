import puppeteer from "puppeteer-core";
const base = process.argv[2];
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const page = await browser.newPage();
const logs = [], errors = [];
page.on("console", (m) => { if (m.type() === "log") logs.push(m.text()); });
page.on("pageerror", (e) => errors.push(e.message));
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };

const res = await page.goto(base + "/catalog", { waitUntil: "networkidle0" });
ok("HTTP 200", res.status() === 200);
const raw = await (await fetch(base + "/catalog")).text();
ok("el HTML crudo (sin JS) ya trae el contenido compuesto", raw.includes('<section class="card"><h2>Diseño</h2>') && raw.includes("Carla"));
ok("<title> de SEO presente", raw.includes("<title>Catálogo</title>"));
ok("2 tarjetas (For externo)", (await page.$$("section.card")).length === 2);
ok("3 miembros (For anidado)", (await page.$$("section.card li")).length === 3);
ok("el espacio entre <Tag/> y el nombre se conserva", (await page.$eval("section.card li", (e) => e.textContent)) === "lead Ana");
ok("0 errores de JS al cargar", errors.length === 0);

const buttons = await page.$$("section.card button");
await buttons[1].click();
await new Promise((r) => setTimeout(r, 400));
ok("el onClick pasado por prop a un componente se activa (2º botón)", logs.includes("saludo desde un componente"));
await buttons[0].click();
await new Promise((r) => setTimeout(r, 300));
ok("y también el del 1er botón (copias del <For>)", logs.filter((l) => l === "saludo desde un componente").length === 2);
ok("sin errores tras los clics", errors.length === 0);
await browser.close();
