// Prueba de humo MULTIPLATAFORMA (Windows, macOS, Linux): sin Python, sin Chrome, solo Node + el binario de `nexa`.
//
//   node e2e/smoke.mjs <ruta-al-binario-nexa>
//
// Crea un proyecto, lo compila, lo empaqueta en UN ejecutable y lo sirve: comprueba lo más sensible a cada
// sistema operativo (pegar el sitio al final del .exe/binario y leerlo al arrancar, rutas, cabeceras).
import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, existsSync, statSync } from "node:fs";
import net from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const nexa = resolve(process.argv[2] ?? "");
if (!process.argv[2] || !existsSync(nexa)) {
    console.error("uso: node e2e/smoke.mjs <ruta-al-binario-nexa>");
    process.exit(2);
}

let pass = 0, fail = 0;
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); cond ? pass++ : fail++; };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function run(args, cwd) {
    const r = spawnSync(nexa, args, { cwd, encoding: "utf8" });
    return { code: r.status, out: (r.stdout ?? "") + (r.stderr ?? "") };
}

async function freePort() {
    return new Promise((resolvePort) => {
        const s = net.createServer().listen(0, "127.0.0.1", () => {
            const { port } = s.address();
            s.close(() => resolvePort(port));
        });
    });
}

const work = mkdtempSync(join(tmpdir(), "nexa-smoke-"));
console.log(`nexa: ${nexa}\ncarpeta de trabajo: ${work}\nplataforma: ${process.platform}/${process.arch}\n`);

// 1. create + lint + build
let r = run(["create", "smoke", "--template", "app"], work);
ok("nexa create --template app", r.code === 0 && existsSync(join(work, "smoke", "nexa.toml")));
const project = join(work, "smoke");
ok("create deja tsconfig.json y nexa-env.d.ts", existsSync(join(project, "tsconfig.json")) && existsSync(join(project, "nexa-env.d.ts")));

r = run(["lint"], project);
ok("nexa lint: sin avisos", r.code === 0);

r = run(["build"], project);
ok("nexa build", r.code === 0 && existsSync(join(project, "dist", "index.html")) && existsSync(join(project, "dist", "app", "index.html")));
if (r.code !== 0) console.log(r.out);

// 2. bundle (copia el binario y le pega el sitio al final)
const exeName = process.platform === "win32" ? "smoke-server.exe" : "smoke-server";
const out = join(work, exeName);
r = run(["bundle", "--no-build", "--out", out], project);
ok("nexa bundle: genera el ejecutable", r.code === 0 && existsSync(out));
if (r.code !== 0) console.log(r.out);
ok("el ejecutable pesa más que `nexa` (lleva el sitio pegado)", existsSync(out) && statSync(out).size > statSync(nexa).size);

// 3. arrancarlo FUERA del proyecto (carpeta de trabajo distinta) y probarlo
const port = await freePort();
const child = spawn(out, ["--port", String(port), "--quiet"], { cwd: work, stdio: ["ignore", "pipe", "pipe"] });
let childLog = "";
child.stdout.on("data", (d) => (childLog += d));
child.stderr.on("data", (d) => (childLog += d));
const base = `http://127.0.0.1:${port}`;

let up = false;
for (let i = 0; i < 100 && !up; i++) {
    try { up = (await fetch(base + "/")).status === 200; } catch { await sleep(100); }
}
ok("el ejecutable arranca y responde (sin proyecto ni Node en su carpeta)", up);

if (up) {
    const get = (p, init) => fetch(base + p, { redirect: "manual", ...init });
    const home = await get("/");
    const homeText = await home.text();
    ok("/ : 200 HTML con el contenido de la portada", home.status === 200 && homeText.includes("Mi app"));
    ok("cabeceras de seguridad presentes", home.headers.get("x-content-type-options") === "nosniff" && !!home.headers.get("x-frame-options"));

    const gz = await get("/", { headers: { "accept-encoding": "gzip" } });
    ok("gzip cuando el cliente lo acepta", gz.headers.get("content-encoding") === "gzip");
    const etag = home.headers.get("etag");
    const again = await get("/", { headers: { "if-none-match": etag } });
    ok("ETag: If-None-Match -> 304 sin cuerpo", etag && again.status === 304 && (await again.text()) === "");

    ok("/app : 200 (shell de la SPA)", (await (await get("/app")).text()).includes('id="app"'));
    ok("/app/ruta/profunda : 200 (fallback de SPA)", (await get("/app/ruta/profunda")).status === 200);
    ok("/app/falta.js : 404 (un asset que falta NO devuelve HTML)", (await get("/app/falta.js")).status === 404);
    ok("/nope : 404", (await get("/nope")).status === 404);
    ok("POST / : 405", (await get("/", { method: "POST" })).status === 405);

    const runtime = (await (await get("/app")).text()).match(/\/assets\/nexa-app\.[0-9a-f]{8}\.js/)?.[0];
    const asset = runtime ? await get(runtime) : null;
    ok("asset con hash: Cache-Control immutable y tipo JavaScript", !!asset && /immutable/.test(asset.headers.get("cache-control") ?? "") && /javascript/.test(asset.headers.get("content-type") ?? ""));

    // path traversal con una petición cruda (fetch normalizaría los `..`)
    const raw = await new Promise((res) => {
        const s = net.connect(port, "127.0.0.1", () => s.write("GET /../../etc/passwd HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n"));
        let o = "";
        s.on("data", (d) => (o += d));
        s.on("close", () => res(o));
    });
    ok("path traversal rechazado (400)", raw.startsWith("HTTP/1.1 400"));
} else {
    console.log("--- salida del servidor ---\n" + childLog);
}

child.kill();
await sleep(200);
console.log(`\nRESULTADO: ${pass} OK, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
