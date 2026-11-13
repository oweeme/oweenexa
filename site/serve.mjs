// Servidor estático mínimo para ver el sitio: node serve.mjs [puerto] [prefijo]
// Con prefijo (p. ej. /oweenexa/) simula GitHub Pages bajo un subdirectorio.
import { createServer } from "node:http";
import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "dist");
const port = Number(process.argv[2] ?? 4700);
const prefix = (process.argv[3] ?? "/").replace(/\/?$/, "/");
const types = { ".html": "text/html; charset=utf-8", ".css": "text/css", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".woff2": "font/woff2", ".xml": "application/xml", ".txt": "text/plain" };

createServer((req, res) => {
    let path = decodeURIComponent(new URL(req.url, "http://x").pathname);
    if (!path.startsWith(prefix)) { res.writeHead(404); return res.end("fuera del prefijo " + prefix); }
    path = normalize(path.slice(prefix.length)).replace(/^(\.\.[/\\])+/, "");
    let file = join(root, path);
    if (existsSync(file) && statSync(file).isDirectory()) file = join(file, "index.html");
    if (!existsSync(file)) { res.writeHead(404, { "Content-Type": types[".html"] }); return res.end(readFileSync(join(root, "404.html"))); }
    res.writeHead(200, { "Content-Type": types[extname(file)] ?? "application/octet-stream" });
    res.end(readFileSync(file));
}).listen(port, "127.0.0.1", () => console.log(`http://127.0.0.1:${port}${prefix}`));
