// Copia el CSS de @nexa/ui a crates/nexa-ui/assets/ (multiplataforma: reemplaza a un `cp` de bash).
import { copyFileSync, mkdirSync, readdirSync } from "node:fs";

const from = "packages/ui/src";
const to = "crates/nexa-ui/assets";
mkdirSync(to, { recursive: true });
const files = readdirSync(from).filter((f) => f.endsWith(".css"));
for (const f of files) copyFileSync(`${from}/${f}`, `${to}/${f}`);
console.log(`Copiados ${files.length} archivos CSS a ${to}/`);
