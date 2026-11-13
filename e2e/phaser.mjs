import puppeteer from "puppeteer-core";
const base = process.argv[2];
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? "/usr/bin/google-chrome", headless: "new", args: ["--no-sandbox"] });
const page = await browser.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("response", (r) => { if (r.status() >= 400 && !r.url().endsWith("/favicon.ico")) errors.push("HTTP " + r.status() + " " + r.url()); });
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); if (!cond) process.exitCode = 1; };
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

await page.goto(base + "/app", { waitUntil: "networkidle0" });
await page.waitForSelector("#game canvas", { timeout: 15000 });
ok("Phaser creó su <canvas> dentro del div de Nexa", true);
const size = await page.$eval("#game canvas", (c) => [c.width, c.height]);
ok("canvas de 800x400", size[0] === 800 && size[1] === 400);

await wait(1200);
const f1 = await page.$eval("#stats", (e) => e.textContent);
await wait(600);
const f2 = await page.$eval("#stats", (e) => e.textContent);
const n = (s) => Number(s.match(/Frames: (\d+)/)[1]);
ok(`el bucle del juego corre y actualiza HTML reactivo de Nexa (${n(f1)} -> ${n(f2)} frames)`, n(f2) > n(f1) && n(f1) > 10);

// el cuadrado se mueve: pintado real (no canvas en blanco)
const pixels = await page.$eval("#game canvas", (c) => {
    const ctx = c.getContext("2d");
    const d = ctx.getImageData(0, 0, c.width, c.height).data;
    let red = 0;
    for (let i = 0; i < d.length; i += 4) if (d[i] > 200 && d[i + 1] < 120) red++;
    return red;
});
ok(`el canvas tiene el cuadrado rojo dibujado (${pixels} px)`, pixels > 1000);

// entrada del juego -> señal de Nexa: clic sobre el cuadrado
let clicked = false;
for (let tries = 0; tries < 40 && !clicked; tries++) {
    const box = await page.$eval("#game canvas", (c) => { const r = c.getBoundingClientRect(); return { x: r.x, y: r.y }; });
    const bx = await page.evaluate(() => window.__game.scene.scenes[0].box.x);
    await page.mouse.click(box.x + bx, box.y + 200);
    await wait(40);
    const t = await page.$eval("#stats", (e) => e.textContent);
    clicked = /Clics en el cuadrado: [1-9]/.test(t);
}
ok("un clic sobre el sprite de Phaser actualiza una señal de Nexa", clicked);

// al salir de la página el juego se destruye (onCleanup)
await page.evaluate(() => history.pushState({}, "", "/app/otra"));
await page.evaluate(() => window.dispatchEvent(new PopStateEvent("popstate")));
await wait(300);
ok("al navegar, el juego se destruyó y no queda canvas", (await page.$("#game canvas")) === null);

ok("sin errores de JS ni 404", errors.length === 0);
if (errors.length) console.log(errors);
await browser.close();
