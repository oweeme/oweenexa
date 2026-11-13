// API de ejemplo del tutorial: un backend cualquiera (aquí Node sin dependencias). Nexa solo necesita JSON por HTTP.
import { createServer } from "node:http";

const PORT = Number(process.env.PORT ?? 9500);

// "Base de datos" en memoria. Cada post tiene su texto en español y, si existe, en inglés.
const posts = [
    {
        slug: "hola-nexa",
        author: "Héctor",
        date: "2026-01-10",
        es: { title: "Hola, Nexa", body: "## Un sitio con HTML real\n\nEste post está escrito en **Markdown**. Nexa lo convierte a HTML al compilar, y lo sanea.\n\n- Páginas de contenido con SEO\n- Islas para lo interactivo\n- Un panel como SPA" },
        en: { title: "Hello, Nexa", body: "## A site with real HTML\n\nThis post is written in **Markdown**. Nexa turns it into HTML at build time, and sanitizes it." },
    },
    {
        slug: "islas",
        author: "Héctor",
        date: "2026-02-02",
        es: { title: "Qué es una isla", body: "Una **isla** es un pedazo interactivo dentro de una página que, por lo demás, es HTML estático." },
    },
];

const sessions = new Map(); // sid -> nombre

const json = (res, status, body, headers = {}) => {
    res.writeHead(status, { "Content-Type": "application/json", ...headers });
    res.end(JSON.stringify(body));
};
const pick = (post, lang) => ({ slug: post.slug, author: post.author, date: post.date, lang: post[lang] ? lang : "es", ...(post[lang] ?? post.es) });
const summary = (p) => ({ slug: p.slug, author: p.author, date: p.date, lang: p.lang, title: p.title, excerpt: p.body.replace(/[#*`>-]/g, "").trim().slice(0, 110) });
const readBody = (req) => new Promise((ok) => { let d = ""; req.on("data", (c) => (d += c)); req.on("end", () => ok(d ? JSON.parse(d) : {})); });
const who = (req) => {
    const sid = /(?:^|;\s*)sid=([^;]+)/.exec(req.headers.cookie ?? "")?.[1];
    return sid ? sessions.get(sid) : undefined;
};
const slugify = (s) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");

createServer(async (req, res) => {
    const url = new URL(req.url, "http://x");
    const lang = url.searchParams.get("lang") ?? "es";

    if (req.method === "GET" && url.pathname === "/api/posts") {
        return json(res, 200, posts.map((p) => summary(pick(p, lang))));
    }
    const one = /^\/api\/posts\/([^/]+)$/.exec(url.pathname);
    if (req.method === "GET" && one) {
        const post = posts.find((p) => p.slug === decodeURIComponent(one[1]));
        return post ? json(res, 200, pick(post, lang)) : json(res, 404, { error: "no existe" });
    }

    if (req.method === "POST" && url.pathname === "/api/login") {
        const { name } = await readBody(req);
        const sid = Math.random().toString(36).slice(2);
        sessions.set(sid, name || "anónimo");
        return json(res, 200, { name: sessions.get(sid) }, { "Set-Cookie": `sid=${sid}; HttpOnly; Path=/; SameSite=Lax` });
    }
    if (req.method === "POST" && url.pathname === "/api/logout") {
        return json(res, 200, { ok: true }, { "Set-Cookie": "sid=; HttpOnly; Path=/; Max-Age=0" });
    }
    if (req.method === "GET" && url.pathname === "/api/me") {
        const name = who(req);
        return name ? json(res, 200, { name }) : json(res, 401, { error: "sin sesión" });
    }
    if (req.method === "POST" && url.pathname === "/api/admin/posts") {
        const name = who(req);
        if (!name) return json(res, 401, { error: "sin sesión" });
        const { title, body } = await readBody(req);
        if (!title) return json(res, 400, { error: "falta el título" });
        const post = { slug: slugify(title), author: name, date: new Date().toISOString().slice(0, 10), es: { title, body: body || "" } };
        posts.unshift(post);
        return json(res, 201, summary(pick(post, "es")));
    }
    json(res, 404, { error: "no existe" });
}).listen(PORT, "127.0.0.1", () => console.log(`API de ejemplo en http://127.0.0.1:${PORT}`));
