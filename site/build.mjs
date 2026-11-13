// Genera el sitio de documentación (GitHub Pages) a partir de ../docs/*.md.
//
//   node build.mjs            ->  site/dist/
//   SITE_URL=https://oweeme.github.io/oweenexa/ node build.mjs
//
// Una única fuente de verdad: los .md de docs/ (que también se leen en GitHub). Este script los convierte en
// páginas con barra lateral, índice, buscador y tema claro/oscuro, y FALLA si algún enlace interno está roto.
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, posix } from "node:path";
import { fileURLToPath } from "node:url";
import { marked } from "marked";
import GithubSlugger from "github-slugger";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import css from "highlight.js/lib/languages/css";
import diff from "highlight.js/lib/languages/diff";
import ini from "highlight.js/lib/languages/ini";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";
import powershell from "highlight.js/lib/languages/powershell";
import { home } from "./home.mjs";

for (const [name, lang] of Object.entries({ bash, css, diff, ini, javascript, json, python, rust, typescript, xml, yaml, powershell })) hljs.registerLanguage(name, lang);
const ALIAS = { sh: "bash", shell: "bash", console: "bash", zsh: "bash", ts: "typescript", tsx: "typescript", jsx: "typescript", js: "javascript", mjs: "javascript", toml: "ini", html: "xml", svg: "xml", yml: "yaml", ps1: "powershell", rs: "rust", py: "python" };

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const docsDir = join(root, "docs");
const out = join(here, "dist");
const SITE_URL = (process.env.SITE_URL ?? "https://oweeme.github.io/oweenexa/").replace(/\/?$/, "/");
const BASE_PATH = new URL(SITE_URL).pathname;
const REPO = "https://github.com/oweeme/oweenexa";

// ------------------------------------------------------------------ navegación
const GROUPS = [
    { title: "Empezar", items: [
        { file: "TUTORIAL-PASO-A-PASO.md", slug: "tutorial", title: "Tutorial paso a paso" },
        { file: "GUIA-DE-INICIO.md", slug: "guia", title: "Instalación y primeros pasos" },
        { file: "ELIGE-TU-CAMINO.md", slug: "conceptos", title: "Core, isla o Modo App" },
    ] },
    { title: "Guías", items: [
        { file: "TUTORIAL-BLOG-TIENDA.md", slug: "blog-y-tienda", title: "Blog y tienda con API" },
        { file: "SEGURIDAD.md", slug: "seguridad", title: "Seguridad" },
        { file: "VENDORING.md", slug: "librerias", title: "Librerías de terceros (React, Phaser…)" },
        { file: "CI-Y-PRUEBAS.md", slug: "ci", title: "CI y pruebas" },
    ] },
    { title: "Referencia", split: { file: "REFERENCIA.md", slug: "referencia", title: "Referencia completa" } },
    { title: "Proyecto", items: [
        { file: "FASES-DE-CONSTRUCCION.md", slug: "historial", title: "Historial de desarrollo" },
        { file: "POLITICA-DE-VERSIONES.md", slug: "versiones", title: "Política de versiones" },
        { file: "POLITICA-LTS.md", slug: "lts", title: "Política LTS" },
        { file: "DISENO-COOKIES-EN-LOAD.md", slug: "cookies-en-load", title: "Diseño: cookies en load" },
    ] },
];

// ------------------------------------------------------------------ utilidades
// Quita el formato de un título conservando el contenido de los códigos en línea tal cual (`src/pages/**/*.tsx`
// no debe perder sus asteriscos por parecerse a una negrita).
const stripMd = (s) => {
    const codes = [];
    const t = s.replace(/`([^`]*)`/g, (_, c) => `\u0000${codes.push(c) - 1}\u0000`)
        .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1").replace(/[*_]{1,3}([^*_]+)[*_]{1,3}/g, "$1").trim();
    return t.replace(/\u0000(\d+)\u0000/g, (_, i) => codes[Number(i)]);
};
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const depth = (url) => (url === "" ? 0 : url.split("/").filter(Boolean).length);
const rootPrefix = (url) => (depth(url) === 0 ? "./" : "../".repeat(depth(url)));
const relUrl = (from, to) => rootPrefix(from) + to;
const warn = (msg) => { warnings.push(msg); };
const warnings = [];

function slugPart(heading) {
    const s = new GithubSlugger().slug(stripMd(heading)).replace(/-{2,}/g, "-").replace(/^-|-$/g, "");
    return s || "seccion";
}

/** Parte un markdown en secciones `## ...` ignorando los `##` que están dentro de bloques de código. */
function splitSections(md) {
    const lines = md.split("\n");
    const sections = [{ heading: null, lines: [] }];
    let fence = null;
    for (const line of lines) {
        const f = /^\s*(```+|~~~+)/.exec(line);
        if (f) {
            if (fence === null) fence = f[1][0]; // abre un bloque
            else if (f[1][0] === fence) fence = null; // lo cierra (mismo tipo de valla)
        }
        const h = !fence && /^## (.+)$/.exec(line);
        if (h) sections.push({ heading: h[1].trim(), lines: [line] });
        else sections[sections.length - 1].lines.push(line);
    }
    return sections.map((s) => ({ heading: s.heading, md: s.lines.join("\n") }));
}

// ------------------------------------------------------------------ 1) páginas
/** @type {Array<{url:string,title:string,group:string,file:string,md:string,shift:number,idsFrom:number}>} */
const pages = [];
for (const group of GROUPS) {
    if (group.split) {
        const { file, slug, title } = group.split;
        const sections = splitSections(readFileSync(join(docsDir, file), "utf8"));
        // Intro (el # título + lo anterior al primer ##) y el «Índice» van juntos en la portada de la referencia.
        const intro = sections[0];
        const indexSection = sections.find((s) => s.heading === "Índice");
        pages.push({ url: `${slug}/`, title, group: group.title, file, md: intro.md + (indexSection ? "\n" + indexSection.md : ""), shift: 0, sectionOf: file });
        const used = new Set();
        for (const s of sections.slice(1)) {
            if (s.heading === "Índice") continue;
            let part = slugPart(s.heading);
            while (used.has(part)) part += "-2";
            used.add(part);
            pages.push({ url: `${slug}/${part}/`, title: stripMd(s.heading), group: group.title, file, md: s.md, shift: 1, sectionOf: file });
        }
    } else {
        for (const item of group.items) {
            if (!existsSync(join(docsDir, item.file))) { warn(`falta docs/${item.file}`); continue; }
            pages.push({ url: `${item.slug}/`, title: item.title, group: group.title, file: item.file, md: readFileSync(join(docsDir, item.file), "utf8"), shift: 0 });
        }
    }
}
const docUrl = {}; // docs/X.md -> url de su página principal
for (const p of pages) if (!(p.file in docUrl)) docUrl[p.file] = p.url;

// ------------------------------------------------------------------ 2) ids de encabezados (como GitHub) y mapa de anclas
const slugger = {}; // un slugger por documento fuente, para numerar duplicados igual que GitHub
const anchorMap = {}; // archivo -> { id -> url de página }
for (const p of pages) {
    slugger[p.file] ??= new GithubSlugger();
    anchorMap[p.file] ??= {};
    p.tokens = marked.lexer(p.md);
    p.headings = [];
    for (const t of p.tokens) {
        if (t.type !== "heading") continue;
        t.id = slugger[p.file].slug(stripMd(t.text));
        const level = Math.max(1, t.depth - p.shift);
        p.headings.push({ id: t.id, text: stripMd(t.text), level });
        anchorMap[p.file][t.id] = p.url;
        t.shiftedDepth = level;
    }
}

// ------------------------------------------------------------------ 3) enlaces
const ownDirs = /^(examples|crates|packages|e2e|site|docs|scripts|\.github)\//;
function rewriteHref(href, page) {
    if (/^(https?:|mailto:|tel:)/.test(href)) return { href, external: true };
    if (href.startsWith("#")) {
        const id = decodeURIComponent(href.slice(1));
        const target = anchorMap[page.file]?.[id];
        if (!target) { warn(`${page.file}: ancla desconocida ${href}`); return { href }; }
        return { href: target === page.url ? href : `${relUrl(page.url, target)}#${id}` };
    }
    const [pathPart, frag] = href.split("#");
    const name = posix.basename(pathPart);
    if (pathPart.endsWith(".md") && docUrl[name] && !pathPart.startsWith("../") && !ownDirs.test(pathPart.replace(/^\.\//, ""))) {
        const target = frag && anchorMap[name]?.[decodeURIComponent(frag)] ? anchorMap[name][decodeURIComponent(frag)] : docUrl[name];
        return { href: relUrl(page.url, target) + (frag ? `#${frag}` : "") };
    }
    // un archivo del repositorio: se enlaza a GitHub
    const rel = ownDirs.test(pathPart) || pathPart === "README.md" || pathPart === "LICENSE" ? pathPart : posix.normalize(posix.join("docs", pathPart));
    const isDir = !/\.[a-z0-9]+$/i.test(rel) || rel.endsWith("/");
    return { href: `${REPO}/${isDir ? "tree" : "blob"}/main/${rel.replace(/\/$/, "")}${frag ? `#${frag}` : ""}`, external: true };
}

// ------------------------------------------------------------------ 4) render
function highlight(code, lang) {
    const name = ALIAS[lang] ?? lang;
    if (name && hljs.getLanguage(name)) return hljs.highlight(code, { language: name, ignoreIllegals: true }).value;
    return esc(code);
}

function renderPage(page) {
    const renderer = new marked.Renderer();
    renderer.heading = function ({ tokens, id, shiftedDepth, depth: d }) {
        const level = Math.min(4, shiftedDepth ?? d);
        const inner = this.parser.parseInline(tokens);
        const anchor = level > 1 ? `<a class="anc" href="#${id}" aria-label="Enlace a esta sección">#</a>` : "";
        return `<h${level} id="${id}">${inner}${anchor}</h${level}>\n`;
    };
    renderer.code = ({ text, lang }) => {
        const info = (lang ?? "").trim();
        const language = info.split(/\s+/)[0] ?? "";
        const title = /title="([^"]+)"/.exec(info)?.[1];
        const label = title ?? (language && language !== "text" ? language : "");
        const cls = language && language !== "text" ? ` language-${language}` : "";
        return `<div class="code"><div class="bar"><i></i><i></i><i></i><span>${esc(label)}</span><button class="copy" type="button">Copiar</button></div><pre><code class="hljs${cls}">${highlight(text, language)}</code></pre></div>\n`;
    };
    renderer.link = function ({ href, title, tokens }) {
        const r = rewriteHref(href, page);
        const rel = r.external ? ' rel="noopener"' : "";
        let label = this.parser.parseInline(tokens);
        // [ELIGE-TU-CAMINO.md](ELIGE-TU-CAMINO.md): en el sitio se muestra el título de la página, no el nombre del archivo
        const only = tokens.length === 1 && tokens[0].type === "text" ? tokens[0].text : "";
        const target = /\.md$/.test(only) ? pages.find((p) => p.url === docUrl[only]) : null;
        if (target && !r.external) label = esc(target.title);
        return `<a href="${esc(r.href)}"${title ? ` title="${esc(title)}"` : ""}${rel}>${label}</a>`;
    };
    renderer.table = function (token) {
        const header = token.header.map((c, i) => `<th${token.align[i] ? ` align="${token.align[i]}"` : ""}>${this.parser.parseInline(c.tokens)}</th>`).join("");
        const rows = token.rows.map((r) => `<tr>${r.map((c, i) => `<td${token.align[i] ? ` align="${token.align[i]}"` : ""}>${this.parser.parseInline(c.tokens)}</td>`).join("")}</tr>`).join("\n");
        return `<div class="table-wrap"><table><thead><tr>${header}</tr></thead><tbody>${rows}</tbody></table></div>\n`;
    };
    renderer.codespan = ({ text }) => `<code>${esc(text)}</code>`; // OJO: escapar — `<main>` en un texto no debe crear una etiqueta real
    return marked.parser(page.tokens, { renderer, gfm: true });
}

function plainText(tokens) {
    let text = "";
    const walk = (list) => {
        for (const t of list) {
            if (t.type === "code") text += " " + t.text;
            else if (t.type === "space" || t.type === "hr") continue;
            else if (t.tokens) walk(t.tokens);
            else if (t.items) walk(t.items);
            else if (t.rows) { walk(t.header ?? []); for (const r of t.rows) for (const c of r) walk(c.tokens); }
            else if (t.text) text += " " + t.text;
        }
        text += " ";
    };
    walk(tokens);
    return text.replace(/\s+/g, " ").trim();
}

// ------------------------------------------------------------------ 5) plantilla de documentación
const flat = pages;
function layout(page, bodyHtml) {
    const rootP = rootPrefix(page.url);
    const idx = flat.indexOf(page);
    const prev = flat[idx - 1], next = flat[idx + 1];
    const side = GROUPS.map((g) => {
        const items = flat.filter((p) => p.group === g.title);
        return `<h4>${esc(g.title)}</h4>` + items.map((p) => `<a href="${rootP}${p.url}"${p === page ? ' aria-current="page"' : ""}>${esc(p.title)}</a>`).join("");
    }).join("\n");
    const toc = page.headings.filter((h) => h.level >= 2 && h.level <= 3).map((h) => `<a class="h${h.level}" href="#${h.id}">${esc(h.text)}</a>`).join("");
    const desc = page.description;
    const pager = `<nav class="pager" aria-label="Páginas">${prev ? `<a class="prev" href="${rootP}${prev.url}"><small>← Anterior</small>${esc(prev.title)}</a>` : "<span></span>"}${next ? `<a class="next" href="${rootP}${next.url}"><small>Siguiente →</small>${esc(next.title)}</a>` : ""}</nav>`;
    const edit = `${REPO}/edit/main/docs/${page.file}`;
    return shell({
        rootP, url: page.url, title: `${page.title} · Nexa`, description: desc, nav: "docs",
        main: `<div class="docs">
<aside class="side" aria-label="Navegación">${side}</aside>
<article class="prose" id="contenido">${bodyHtml}${pager}<p class="editar"><a href="${edit}" rel="noopener">Editar esta página en GitHub</a></p></article>
<aside class="toc" aria-label="En esta página">${toc ? `<h4>En esta página</h4>${toc}` : ""}</aside>
</div>`,
    });
}

/** El marco común: <head>, barra superior, buscador, pie. */
export function shell({ rootP, url, title, description, nav, main, lang = "es", extraHead = "" }) {
    const canonical = SITE_URL + url;
    const link = (href, label, key, cls = "") => `<a${cls ? ` class="${cls}"` : ""} href="${href}"${nav === key ? " aria-current" : ""}>${label}</a>`;
    return `<!doctype html>
<html lang="${lang}" data-base="${rootP}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<meta name="description" content="${esc(description)}">
<link rel="canonical" href="${canonical}">
<meta property="og:type" content="website"><meta property="og:title" content="${esc(title)}"><meta property="og:description" content="${esc(description)}"><meta property="og:url" content="${canonical}">
<meta name="theme-color" content="#488c54">
<link rel="icon" type="image/svg+xml" href="${rootP}assets/favicon.svg">
<link rel="stylesheet" href="${rootP}assets/style.css">
<script>try{var t=localStorage.getItem("nexa-theme");if(t==="light"||t==="dark")document.documentElement.setAttribute("data-theme",t)}catch(e){}</script>
${extraHead}</head>
<body>
<a class="skip" href="#contenido">Saltar al contenido</a>
<header class="top"><div class="wrap">
${nav === "docs" ? '<button class="pill menu-btn" id="menu" type="button" aria-label="Menú">☰</button>' : ""}
<a class="brand" href="${rootP}"><img src="${rootP}assets/logo.svg" alt="" width="30" height="30"><strong>Nexa</strong><small>v0.1</small></a>
<nav aria-label="Principal">
${link(rootP + "tutorial/", "Tutorial", "tutorial")}
${link(rootP + "guia/", '<span class="es">Guía</span><span class="en">Guide</span>', "guia", "hide-m")}
${link(rootP + "referencia/", '<span class="es">Referencia</span><span class="en">Reference</span>', "ref", "hide-m")}
<a class="hide-m" href="${REPO}" rel="noopener">GitHub</a>
<button class="pill" type="button" data-search aria-label="Buscar"><svg width="15" height="15" viewBox="0 0 16 16" aria-hidden="true"><circle cx="7" cy="7" r="5" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg><span class="lbl"><span class="es">Buscar</span><span class="en">Search</span></span> <kbd>/</kbd></button>
${lang === "es" && nav === "home" ? '<button class="pill" id="lang" type="button" aria-label="Idioma">EN</button>' : ""}
<button class="pill" id="theme" type="button" aria-label="Tema claro/oscuro"><svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="6.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M8 1.5a6.5 6.5 0 0 1 0 13z" fill="currentColor"/></svg></button>
</nav></div></header>
${main}
<footer class="pie"><div class="wrap">
<span>© 2026 Hector Martinez · Licencia MIT</span>
<span><a href="${REPO}" rel="noopener">GitHub</a> · <a href="${rootP}tutorial/">Tutorial</a> · <a href="${rootP}referencia/">Referencia</a> · <a href="${rootP}seguridad/">Seguridad</a></span>
</div></footer>
<div class="buscar" id="buscar" role="dialog" aria-label="Buscar en la documentación"><div class="ventana"><div class="barra"><span class="escamas"><i></i><i></i><i></i></span><span class="t">Buscar</span></div><input type="search" placeholder="Buscar en la documentación…" aria-label="Buscar"><ul></ul></div></div>
<script src="${rootP}assets/site.js" defer></script>
</body>
</html>
`;
}

// ------------------------------------------------------------------ 6) generar
rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

const searchIndex = [];
const ids = {}; // url -> Set de ids (para comprobar enlaces)
const pageOf = (p) => `${p.group} › ${p.title}`;
for (const page of pages) {
    const body = renderPage(page);
    const firstP = page.tokens.find((t) => t.type === "paragraph");
    page.description = firstP ? stripMd(firstP.text).slice(0, 170) : page.title;
    page.text = plainText(page.tokens);
    const html = layout(page, body);
    mkdirSync(join(out, page.url), { recursive: true });
    writeFileSync(join(out, page.url, "index.html"), html);
    ids[page.url] = new Set(["contenido", ...page.headings.map((h) => h.id)]);

    searchIndex.push({ u: page.url, t: page.title, p: page.group, x: page.text.slice(0, 600), h: 0 });
    // cada encabezado de nivel 2-3 es un resultado propio, con el texto que le sigue
    const idxs = page.tokens.map((t, i) => (t.type === "heading" && t.shiftedDepth >= 2 && t.shiftedDepth <= 3 ? i : -1)).filter((i) => i >= 0);
    for (const i of idxs) {
        const t = page.tokens[i];
        const rest = [];
        for (let j = i + 1; j < page.tokens.length && page.tokens[j].type !== "heading"; j++) rest.push(page.tokens[j]);
        searchIndex.push({ u: `${page.url}#${t.id}`, t: stripMd(t.text), p: pageOf(page), x: plainText(rest).slice(0, 400), h: 1 });
    }
}

// portada
const homeHtml = home({ shell, SITE_URL, REPO, highlight });
writeFileSync(join(out, "index.html"), homeHtml);
ids[""] = new Set([...homeHtml.matchAll(/ id="([^"]+)"/g)].map((m) => m[1]));

// 404 (GitHub Pages lo sirve a cualquier profundidad: rutas absolutas desde la base)
const notFound = shell({ rootP: BASE_PATH, url: "404.html", title: "No encontrado · Nexa", description: "Esta página no existe.", nav: "", main: `<div class="wrap" style="padding:80px 16px;text-align:center"><h1 style="font-size:3rem;margin:0">404</h1><p style="color:var(--suave)">Esta página no existe.</p><p><a class="btn main" href="${BASE_PATH}">Ir al inicio</a> <a class="btn" href="${BASE_PATH}tutorial/">Tutorial</a></p></div>` })
    .replace(/data-base="[^"]*"/, `data-base="${BASE_PATH}"`);
writeFileSync(join(out, "404.html"), notFound);

// recursos
mkdirSync(join(out, "assets/fonts"), { recursive: true });
for (const f of ["style.css", "site.js", "logo.svg", "favicon.svg"]) cpSync(join(here, "assets", f), join(out, "assets", f));
const fontDir = join(here, "node_modules/@fontsource/inter/files");
for (const f of ["inter-latin-400-normal.woff2", "inter-latin-600-normal.woff2", "inter-latin-700-normal.woff2", "inter-latin-ext-400-normal.woff2"]) cpSync(join(fontDir, f), join(out, "assets/fonts", f));
writeFileSync(join(out, "search-index.json"), JSON.stringify(searchIndex));
writeFileSync(join(out, "sitemap.xml"), `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${["", ...pages.map((p) => p.url)].map((u) => `  <url><loc>${SITE_URL}${u}</loc></url>`).join("\n")}\n</urlset>\n`);
writeFileSync(join(out, "robots.txt"), `User-agent: *\nAllow: /\n\nSitemap: ${SITE_URL}sitemap.xml\n`);
writeFileSync(join(out, ".nojekyll"), "");

// ------------------------------------------------------------------ 7) comprobar estructura y enlaces internos
const broken = [];
// Estructura: un HTML de ejemplo mal escapado puede abrir etiquetas reales y romper la página (pasó con `<main>` en un
// código en línea). Cada página de documentación debe tener exactamente un <article> y un índice, y estar equilibrada.
for (const page of pages) {
    const html = readFileSync(join(out, page.url, "index.html"), "utf8");
    const count = (re) => (html.match(re) ?? []).length;
    const checks = [
        ["<article", count(/<article\b/g), 1], ["</article>", count(/<\/article>/g), 1],
        ["<aside", count(/<aside\b/g), 2], ["</aside>", count(/<\/aside>/g), 2],
        ["<main", count(/<main\b/g), 0], ["<h1", count(/<h1\b/g), 1],
    ];
    for (const [what, got, want] of checks) if (got !== want) broken.push(`${page.url}: estructura rota — ${what} aparece ${got} veces (esperado ${want}); ¿HTML de ejemplo sin escapar?`);
}
function walk(dir) { return readdirSync(dir, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(join(dir, e.name)) : [join(dir, e.name)])); }
for (const file of walk(out).filter((f) => f.endsWith(".html") && !f.endsWith("404.html"))) {
    // Lo que hay dentro de <pre>/<code> es TEXTO de ejemplo (`href="/algo.jpg"`), no enlaces de la página.
    const html = readFileSync(file, "utf8").replace(/<pre[\s\S]*?<\/pre>/g, "").replace(/<code[\s\S]*?<\/code>/g, "");
    const rel = file.slice(out.length + 1).replace(/index\.html$/, "");
    for (const m of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
        const href = m[1];
        if (/^(https?:|mailto:|tel:|data:|javascript:)/.test(href)) continue;
        const [p, frag] = href.split("#");
        const target = p === "" ? rel : posix.normalize(posix.join(rel, p));
        const full = join(out, target);
        const isDir = !/\.[a-z0-9]+$/i.test(target);
        const exists = isDir ? existsSync(join(full, "index.html")) || existsSync(full) : existsSync(full);
        if (!exists) { broken.push(`${rel || "/"}: ${href} (no existe ${target})`); continue; }
        const key = target === "." ? "" : target.endsWith("/") ? target : isDir ? target + "/" : target;
        if (frag && ids[key] && !ids[key].has(decodeURIComponent(frag))) broken.push(`${rel || "/"}: ${href} (falta el ancla #${frag} en ${key || "/"})`);
    }
}

console.log(`Sitio generado en site/dist/: ${pages.length + 1} páginas, ${searchIndex.length} entradas de búsqueda.`);
if (warnings.length) console.log(`\nAvisos (${warnings.length}):\n  ` + [...new Set(warnings)].join("\n  "));
if (broken.length) {
    console.error(`\nENLACES ROTOS (${broken.length}):\n  ` + broken.slice(0, 40).join("\n  "));
    process.exit(1);
}
console.log("Sin enlaces internos rotos.");
