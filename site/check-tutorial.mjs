// El tutorial no puede mentir: cada bloque de código con `title="ruta"` de docs/TUTORIAL-PASO-A-PASO.md tiene que
// ser IDÉNTICO al archivo correspondiente de examples/tutorial-blog (que se prueba en un Chrome real: e2e/tutorial.mjs).
//
//   node check-tutorial.mjs
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const md = readFileSync(join(root, "docs/TUTORIAL-PASO-A-PASO.md"), "utf8");
const example = join(root, "examples/tutorial-blog");

const blocks = [...md.matchAll(/^```[a-z]*\s+title="([^"]+)"\n([\s\S]*?)\n```$/gm)];
let bad = 0;
for (const [, path, body] of blocks) {
    const file = join(example, path);
    if (!existsSync(file)) { console.error(`✘ ${path}: el tutorial lo muestra pero no existe en examples/tutorial-blog`); bad++; continue; }
    if (readFileSync(file, "utf8").trimEnd() !== body.trimEnd()) { console.error(`✘ ${path}: el bloque del tutorial NO coincide con el archivo probado`); bad++; }
}
// y al revés: los archivos que el tutorial pide crear tienen que estar todos mostrados
const needed = ["api.mjs", "nexa.toml", "package.json", "src/layout.tsx", "src/styles/site.css", "islands/search.island.ts", "src/components/PostCard.tsx",
    "src/locales/es.json", "src/locales/en.json", "src/pages/[locale]/index.tsx", "src/pages/[locale]/blog/index.tsx", "src/pages/[locale]/blog/[slug].tsx",
    "src/app/session.ts", "src/app/config.ts", "src/app/pages/login.tsx", "src/app/pages/index.tsx"];
const shown = new Set(blocks.map((b) => b[1]));
for (const f of needed) if (!shown.has(f)) { console.error(`✘ ${f}: el proyecto lo necesita pero el tutorial no lo muestra`); bad++; }

console.log(`${blocks.length} bloques con título comprobados contra examples/tutorial-blog.`);
if (bad) { console.error(`\n${bad} problema(s): regenera el tutorial desde los archivos o corrige el ejemplo.`); process.exit(1); }
console.log("El tutorial coincide con el código probado.");
