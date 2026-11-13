// Empaqueta las islas (React y TypeScript puro) a public/vendor/ (multiplataforma).
import { build } from "esbuild";

const common = { bundle: true, format: "esm", target: "es2022" };
const react = { ...common, jsx: "automatic", minify: true, define: { "process.env.NODE_ENV": '"production"' } };

await build({ ...react, entryPoints: ["islands/filter.island.tsx"], outfile: "public/vendor/filter.js" });
await build({ ...common, entryPoints: ["islands/ts.island.ts"], outfile: "public/vendor/ts-island.js" });
await build({ ...react, entryPoints: ["islands/react-app-mount.tsx"], outfile: "public/vendor/react-mount.js" });
