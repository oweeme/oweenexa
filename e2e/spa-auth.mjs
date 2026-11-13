import puppeteer from "puppeteer-core";
const base = process.argv[2];
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const page = await browser.newPage();
const errors = [];
const allowed401 = new Set(["401 /api/me", "404 /api/users/999", "401 /api/users/1", "401 /api/users"]);
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("response", (r) => {
    if (r.status() >= 400 && !r.url().endsWith("/favicon.ico") && !allowed401.has(r.status() + " " + new URL(r.url()).pathname))
        errors.push("HTTP " + r.status() + " " + r.url());
});
const t = (s) => page.$eval(s, (e) => e.textContent);
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const path = () => new URL(page.url()).pathname;

// 1. API vía proxy, mismo origen, sin CORS
await page.goto(base + "/", { waitUntil: "networkidle0" });
const viaProxy = await page.evaluate(async () => {
    const r = await fetch("/api/me");
    return { status: r.status, body: await r.json() };
});
ok("proxy reenvía el 401 del backend tal cual", viaProxy.status === 401 && viaProxy.body.error === "sin sesión");
allowed401.add("401 /api/me");

// 2. login real: POST con cuerpo JSON, Set-Cookie HttpOnly
await page.goto(base + "/app", { waitUntil: "networkidle0" });
await page.waitForSelector("#name");
ok("sin sesión -> /app/login", path() === "/app/login");
await page.type("#name", "Hector");
await page.click("#enter");
await page.waitForSelector("#list li a");
ok("login por POST a través del proxy", path() === "/app");
ok("la lista usa la cookie de sesión (backend la exige)", (await page.$$("#list li")).length === 3);
ok("la cookie es HttpOnly: JS no la ve", !(await page.evaluate(() => document.cookie)).includes("sid"));
const cookies = await page.cookies();
ok("el navegador SÍ guardó la cookie sid HttpOnly", cookies.some((c) => c.name === "sid" && c.httpOnly));

// 3. sobrevive a recargar (estado de UI persistido + cookie)
await page.goto(base + "/app/users/2", { waitUntil: "networkidle0" });
await page.waitForFunction(() => document.querySelector("#title")?.textContent === "Beto");
ok("deep link tras recargar: sigue autenticado", (await t("#email")) === "beto@x.io");
ok("el nombre de la sesión se restauró (persisted)", (await page.evaluate(() => localStorage.getItem("demo.user"))) === '"Hector"');

// 4. un 404 del backend llega como 404 (no se enmascara)
const notFound = await page.evaluate(async () => (await fetch("/api/users/999")).status);
allowed401.add("404 /api/users/999");
ok("el 404 del backend se conserva", notFound === 404);

// 5. 401 con sesión vencida en el backend -> onUnauthorized limpia la UI y va a login
await page.goto(base + "/app", { waitUntil: "networkidle0" });
await page.waitForSelector("#list");
await page.evaluate(() => fetch("/api/logout", { method: "POST" })); // el backend pierde la sesión
allowed401.add("401 /api/users");
await page.click("#list li a"); // navega a un detalle: la API responde 401
await page.waitForSelector("#name");
ok("401 -> onUnauthorized -> vuelve a /app/login", path() === "/app/login");
ok("la UI limpió el usuario persistido", (await page.evaluate(() => localStorage.getItem("demo.user"))) === "null");

// 6. logout explícito
await page.type("#name", "Otra");
await page.click("#enter");
await page.waitForSelector("#logout");
await page.click("#logout");
await page.waitForSelector("#name");
ok("logout -> /app/login", path() === "/app/login");

// 7. backend caído -> 502 legible, no un cuelgue
await page.goto(base + "/", { waitUntil: "networkidle0" });
ok("sin errores inesperados", errors.length === 0);
if (errors.length) console.log(errors);
await browser.close();
