// Empaqueta Phaser a public/vendor/phaser.js (multiplataforma). Ver docs/VENDORING.md.
import { build } from "esbuild";

await build({
    entryPoints: ["vendor-entry.js"],
    bundle: true,
    format: "esm",
    target: "es2022",
    minify: true,
    outfile: "public/vendor/phaser.js", // esbuild crea la carpeta si no existe
});
