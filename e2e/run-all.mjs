// Corre TODA la suite de punta a punta con un solo comando:
//
//   node e2e/run-all.mjs [--nexa <ruta>] [--only smoke,blog] [--keep-going]
//
// Por cada ejemplo: levanta su API (Python), compila, lo sirve con `nexa preview` y con el ejecutable de
// `nexa bundle`, ejecuta las pruebas en un Chrome real y limpia todos los procesos. Sale con código ≠ 0 si
// algo falla. Requiere: Node, Python 3, Chrome/Chromium (CHROME=/ruta si no está en el sitio habitual) y `npm`.
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const args = process.argv.slice(2);
const flag = (name) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : undefined; };
const only = flag("--only")?.split(",");
const keepGoing = args.includes("--keep-going");

const exe = process.platform === "win32" ? ".exe" : "";
const nexa = resolve(
    flag("--nexa") ?? process.env.NEXA_BIN ??
        [join(root, "target/release/nexa" + exe), join(root, "target/debug/nexa" + exe)].find(existsSync) ?? "",
);
if (!existsSync(nexa)) {
    console.error("No encuentro el binario de nexa: compila con `cargo build` o pasa --nexa <ruta>.");
    process.exit(2);
}
const python = ["python3", "python"].find((p) => spawnSync(p, ["--version"]).status === 0);
if (!python) { console.error("Hace falta Python 3 (las APIs de ejemplo)."); process.exit(2); }

// ---------------------------------------------------------------- utilidades
const children = new Set();
function cleanup() { for (const c of children) { try { c.kill(); } catch {} } children.clear(); }
process.on("exit", cleanup);
for (const sig of ["SIGINT", "SIGTERM"]) process.on(sig, () => { cleanup(); process.exit(130); });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
function start(cmd, argv, opts = {}) {
    const child = spawn(cmd, argv, { stdio: "ignore", ...opts });
    children.add(child);
    child.on("exit", () => children.delete(child));
    return child;
}
async function waitFor(url, what, tries = 150) {
    for (let i = 0; i < tries; i++) {
        try { if ((await fetch(url)).status < 500) return; } catch {}
        await sleep(100);
    }
    throw new Error(`${what} no respondió en ${url}`);
}
function run(cmd, argv, opts = {}) {
    const r = spawnSync(cmd, argv, { encoding: "utf8", shell: process.platform === "win32" && /^npm/.test(cmd), ...opts });
    if (r.status !== 0) throw new Error(`falló: ${cmd} ${argv.join(" ")}\n${(r.stdout ?? "") + (r.stderr ?? "")}`);
    return r.stdout;
}
const npm = process.platform === "win32" ? "npm.cmd" : "npm";

const results = [];
/** Ejecuta un script de pruebas de Node y cuenta sus líneas OK/FAIL. */
async function suite(label, script, base, extraEnv = {}) {
    process.stdout.write(`\n▶ ${label}\n`);
    const r = spawnSync(process.execPath, [join(here, script), base], {
        encoding: "utf8",
        env: { ...process.env, ...extraEnv },
        timeout: 180_000,
    });
    const out = (r.stdout ?? "") + (r.stderr ?? "");
    const okN = (out.match(/^OK /gm) ?? []).length;
    const failN = (out.match(/^FAIL /gm) ?? []).length + (r.status !== 0 && !/^FAIL /m.test(out) ? 1 : 0);
    for (const line of out.split("\n")) if (/^FAIL |RESULTADO|Error/.test(line)) console.log("   " + line);
    console.log(`   ${failN === 0 ? "✔" : "✘"} ${okN} OK, ${failN} FAIL`);
    results.push({ label, ok: okN, fail: failN });
    if (failN && !keepGoing) throw new Error(`falló: ${label}`);
}

const want = (name) => !only || only.includes(name);
const work = mkdtempSync(join(tmpdir(), "nexa-e2e-"));
const ex = (name) => join(root, "examples", name);
const env = (extra) => ({ ...process.env, ...extra });
const nexaIn = (dir, argv, extra = {}) => run(nexa, argv, { cwd: dir, env: env(extra) });

async function serve(dir, port, extra = {}) {
    const c = start(nexa, ["preview", "--port", String(port)], { cwd: dir, env: env(extra) });
    await waitFor(`http://127.0.0.1:${port}/`, `nexa preview (${port})`);
    return c;
}
async function serveBundle(dir, name, port, extraArgs, extra = {}) {
    const out = join(work, name + exe);
    nexaIn(dir, ["bundle", "--no-build", "--out", out], extra);
    const c = start(out, ["--port", String(port), "--quiet", ...extraArgs], { cwd: work });
    await waitFor(`http://127.0.0.1:${port}/`, `${name} (${port})`);
    return c;
}
function install(dir) { if (!existsSync(join(dir, "node_modules"))) run(npm, ["install", "--no-audit", "--no-fund"], { cwd: dir }); }

// ---------------------------------------------------------------- las pruebas
try {
    install(here); // puppeteer-core

    if (want("smoke")) await suite("smoke (crear, compilar, empaquetar, servir)", "smoke.mjs", nexa);

    // app-demo: SPA con login por cookie a través del proxy + Core con componentes
    if (want("app")) {
        const api = start(python, [join(ex("app-demo"), "backend.py")], { cwd: ex("app-demo") });
        await waitFor("http://127.0.0.1:9100/api/catalog", "API de app-demo");
        const e = { NEXA_API_URL: "http://127.0.0.1:9100" };
        nexaIn(ex("app-demo"), ["build"], e);
        nexaIn(ex("app-demo"), ["lint"], e);
        const p = await serve(ex("app-demo"), 4321, e);
        await suite("app-demo · SPA con cookies + proxy (nexa preview)", "spa-auth.mjs", "http://127.0.0.1:4321");
        await suite("app-demo · Core con componentes y <For> anidado (nexa preview)", "core-components.mjs", "http://127.0.0.1:4321");
        p.kill();
        const b = await serveBundle(ex("app-demo"), "app-demo-site", 4400, ["--api-url", "http://127.0.0.1:9100"], e);
        await suite("app-demo · SPA con cookies + proxy (ejecutable único)", "spa-auth.mjs", "http://127.0.0.1:4400");
        await suite("app-demo · Core con componentes (ejecutable único)", "core-components.mjs", "http://127.0.0.1:4400");
        b.kill();
        api.kill();
    }

    // phaser-demo
    if (want("phaser")) {
        install(ex("phaser-demo"));
        nexaIn(ex("phaser-demo"), ["build"]);
        const p = await serve(ex("phaser-demo"), 4331);
        await suite("phaser-demo · juego dentro del Modo App", "phaser.mjs", "http://127.0.0.1:4331");
        p.kill();
    }

    // react-islands-demo (usa la API de app-demo: /api/catalog)
    if (want("react")) {
        const api = start(python, [join(ex("app-demo"), "backend.py")], { cwd: ex("app-demo") });
        await waitFor("http://127.0.0.1:9100/api/catalog", "API de app-demo");
        install(ex("react-islands-demo"));
        const e = { NEXA_API_URL: "http://127.0.0.1:9100" };
        nexaIn(ex("react-islands-demo"), ["build"], e);
        const p = await serve(ex("react-islands-demo"), 4341, e);
        await suite("react-islands-demo · React y TypeScript puro en islas", "react-islands.mjs", "http://127.0.0.1:4341");
        p.kill();
        api.kill();
    }

    // i18n-demo: la puerta de idioma
    if (want("i18n")) {
        nexaIn(ex("i18n-demo"), ["build"]);
        const p = await serve(ex("i18n-demo"), 4361);
        await suite("i18n-demo · puerta de idioma (nexa preview)", "locale-gate.mjs", "http://127.0.0.1:4361");
        p.kill();
        const b = await serveBundle(ex("i18n-demo"), "i18n-site", 4362, []);
        await suite("i18n-demo · puerta de idioma (ejecutable único)", "locale-gate.mjs", "http://127.0.0.1:4362");
        b.kill();
    }

    // tutorial-blog: el proyecto del tutorial (SEO, idiomas, isla, panel SPA, publicar bajo demanda)
    if (want("tutorial")) {
        const api = start(process.execPath, [join(ex("tutorial-blog"), "api.mjs")], { cwd: ex("tutorial-blog") });
        await waitFor("http://127.0.0.1:9500/api/posts", "API del tutorial");
        install(ex("tutorial-blog"));
        const e = { NEXA_API_URL: "http://127.0.0.1:9500" };
        nexaIn(ex("tutorial-blog"), ["build"], e);
        nexaIn(ex("tutorial-blog"), ["lint"], e);
        const b = await serveBundle(ex("tutorial-blog"), "tutorial-site", 4600, ["--api-url", "http://127.0.0.1:9500"], e);
        await suite("tutorial-blog · el proyecto del tutorial, de punta a punta", "tutorial.mjs", "http://127.0.0.1:4600");
        b.kill();
        api.kill();
    }

    // content-demo: sitio sin backend (Markdown local, comodín, 404, redirecciones, cabeceras, esbuild integrado)
    if (want("content")) {
        install(ex("content-demo"));
        nexaIn(ex("content-demo"), ["build"]);
        const b = await serveBundle(ex("content-demo"), "content-site", 4700, []);
        await suite("content-demo · sitio sin backend, de punta a punta (ejecutable único)", "content-site.mjs", "http://127.0.0.1:4700");
        b.kill();
    }

    // blog-store-demo: contenido enriquecido saneado + renderizado bajo demanda
    if (want("blog")) {
        const api = start(python, [join(ex("blog-store-demo"), "api.py")], { cwd: ex("blog-store-demo") });
        await waitFor("http://127.0.0.1:9300/api/blog", "API de blog-store-demo");
        const e = { NEXA_API_URL: "http://127.0.0.1:9300" };
        nexaIn(ex("blog-store-demo"), ["build"], e);
        nexaIn(ex("blog-store-demo"), ["lint"], e);
        const b = await serveBundle(ex("blog-store-demo"), "blog-site", 4500, ["--api-url", "http://127.0.0.1:9300", "--revalidate", "2"], e);
        await suite("blog-store-demo · contenido enriquecido con ataques (ejecutable único)", "rich-content.mjs", "http://127.0.0.1:4500");
        // la última: simula que la API se cae
        await suite("blog-store-demo · renderizado bajo demanda, caché, i18n, seguridad", "ssr-on-demand.mjs", "http://127.0.0.1:4500", { API_PID: String(api.pid) });
        b.kill();
        api.kill();
    }
} catch (error) {
    console.error("\n✘ " + error.message);
    cleanup();
    process.exitCode = 1;
}

cleanup();
const okTotal = results.reduce((n, r) => n + r.ok, 0);
const failTotal = results.reduce((n, r) => n + r.fail, 0);
console.log("\n──────────────── resumen ────────────────");
for (const r of results) console.log(`${r.fail ? "✘" : "✔"} ${String(r.ok).padStart(3)} OK  ${String(r.fail).padStart(2)} FAIL  ${r.label}`);
console.log(`\nTOTAL: ${okTotal} OK, ${failTotal} FAIL en ${results.length} suites`);
process.exit(failTotal || process.exitCode ? 1 : 0);
