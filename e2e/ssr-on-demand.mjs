// Renderizado bajo demanda del ejecutable de `nexa bundle` (examples/blog-store-demo), de punta a punta.
//
// Requiere: api.py en :9300 y el ejecutable en :4500 con `--api-url http://127.0.0.1:9300 --revalidate 2`
// (lo levanta `run-all.mjs`; a mano, ver el README del ejemplo). Variable opcional: API_PID — el proceso de la
// API, para simular que "se cae" (la prueba 7). Multiplataforma: solo usa `fetch`.
const B = process.argv[2] ?? process.env.BASE ?? "http://127.0.0.1:4500";
const API = process.env.API ?? "http://127.0.0.1:9300";

let pass = 0, fail = 0;
const ok = (name, cond) => { console.log((cond ? "OK   " : "FAIL ") + name); cond ? pass++ : fail++; };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const get = (url, init) => fetch(url, { redirect: "manual", ...init });
const code = async (url, init) => (await get(url, init)).status;
const text = async (url) => (await get(url)).text();
const hdr = async (url, name) => (await get(url)).headers.get(name);
const log = async () => (await (await fetch(API + "/_log")).json());
const count = async (path) => (await log()).filter((p) => p === path).length;

// 1. Pre-renderizado: la ruta del usuario existe sin tocar la API
const P = "/blog/hectorarchitect/desarrollando-oweelinux-conlinux";
ok("ruta del usuario pre-renderizada: 200", (await code(B + P)) === 200);
ok("lo pre-renderizado no pasa por el renderizador (sin X-Nexa-Cache)", (await hdr(B + P, "x-nexa-cache")) === null);
ok("contenido real de la API en el HTML", (await text(B + P)).includes("Desarrollando OweeLinux con Linux"));

// 2. Post publicado DESPUÉS del build
const N = "/blog/hectorarchitect/nuevo-post";
ok("post nuevo antes de publicar: 404", (await code(B + N)) === 404);
await fetch(API + "/_publish");
await sleep(11_000); // un 404 se recuerda como máximo 10 s
ok("post publicado tras el build: 200 sin recompilar", (await code(B + N)) === 200);
ok("título del post nuevo en el HTML", (await text(B + N)).includes("Post recién publicado"));

// 3. Caché
await get(B + N);
ok("segunda visita: X-Nexa-Cache hit", (await hdr(B + N, "x-nexa-cache")) === "hit");
const before = await count("/api/blog/hectorarchitect/nuevo-post");
for (let i = 0; i < 5; i++) await get(B + N);
ok("5 visitas dentro de la ventana: 0 llamadas nuevas a la API", before === (await count("/api/blog/hectorarchitect/nuevo-post")));
await fetch(API + "/_edit");
await sleep(3_000);
ok("tras revalidar (2 s) se ve el post EDITADO", (await text(B + N)).includes("Post EDITADO"));

// 4. i18n + tienda dinámica
ok("idioma por defecto sin endpoint: /es", (await code(B + "/es")) === 200);
ok("idioma inexistente /xx: 404", (await code(B + "/xx")) === 404);
ok("idioma inexistente en una ruta profunda: 404", (await code(B + "/xx/shop/zapatos/runner")) === 404);
ok("producto nuevo (en inglés) renderizado bajo demanda", (await code(B + "/en/shop/zapatos/trail")) === 200);
const trail = await text(B + "/en/shop/zapatos/trail");
ok("...con textos en inglés ('Price')", trail.includes("Price: 120"));
ok("...y la clave que falta en inglés cae al español (fallback)", trail.includes("Software libre"));
ok("hreflang presente en la página dinámica", trail.includes('hreflang="es"'));

// 5. Seguridad
await get(B + "/blog/hectorarchitect/a%3Fx%3D1%26admin%3D1");
const seen = (await log()).join("\n");
ok("inyección de query en el slug: la API recibió el valor CODIFICADO", seen.includes("/api/blog/hectorarchitect/a%3Fx%3D1%26admin%3D1"));
ok("...y nunca recibió un query inyectado", !seen.includes("admin=1"));
// `fetch` normaliza `..`: se manda la ruta cruda con un socket, como haría un atacante.
const net = await import("node:net");
const raw = await new Promise((resolve) => {
    const u = new URL(B);
    const s = net.connect(Number(u.port), u.hostname, () => s.write("GET /blog/../../etc/passwd HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n"));
    let out = "";
    s.on("data", (d) => (out += d));
    s.on("close", () => resolve(out));
});
ok("path traversal: 400", raw.startsWith("HTTP/1.1 400"));
const callsBefore = (await log()).length;
for (const u of ["/wp-login.php", "/.env", "/xmlrpc.php", "/favicon.ico"]) await get(B + u);
ok("rutas que no parecen páginas (.php, .env...) no cuestan ninguna llamada a la API", callsBefore === (await log()).length);

// 6. Avalancha: 60 visitas simultáneas, dos veces (con una ventana de caché entre medias)
await fetch(API + "/_publish");
const burst = () => Promise.all(Array.from({ length: 60 }, () => get(B + "/en/shop/zapatos/trail").then((r) => r.arrayBuffer())));
await burst();
await sleep(3_000);
await burst();
const n = await count("/api/shop/zapatos/trail");
console.log(`     (llamadas a la API para ese producto tras 120 visitas con 2 ventanas de caché: ${n})`);
ok("avalancha: 120 visitas = unas pocas llamadas (<=4), no 120", n <= 4);

// 7. La API cae: se sirve la última copia
if (process.env.API_PID) {
    try { process.kill(Number(process.env.API_PID)); } catch {}
    await sleep(3_000);
    ok("API caída: el post ya visto se sigue sirviendo (stale)", (await code(B + N)) === 200 && (await hdr(B + N, "x-nexa-cache")) === "stale");
    ok("API caída y ruta nunca vista: 502 (no cuelga)", (await code(B + "/blog/otro/jamas-visto")) === 502);
    ok("API caída: lo pre-renderizado sigue perfecto", (await code(B + P)) === 200);
    ok("502 sin filtrar la URL interna de la API", !(await text(B + "/blog/otro/jamas-visto2")).includes("9300"));
} else {
    console.log("     (sin API_PID: se omite la prueba de 'la API cae')");
}

console.log(`\nRESULTADO: ${pass} OK, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
