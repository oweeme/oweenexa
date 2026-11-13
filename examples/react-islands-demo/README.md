# react-islands-demo — React y TypeScript puro en Nexa

Una tienda con HTML real (SEO) cuyo catálogo viene de una API, más:

- una **isla de React** (`islands/filter.island.tsx`): `useState`, filtro y contador, con los datos de la
  API como props;
- una **isla en TypeScript puro** (`islands/ts.island.ts`): ningún framework;
- **React dentro de la SPA** (`src/app/pages/index.tsx`): montado con `ref` + `createRoot`.

```bash
npm install && npm run vendor           # empaqueta las islas a public/vendor/ (React 19: ~219 KB)
python3 ../app-demo/backend.py &        # la API de ejemplo (/api/catalog) en :9100
NEXA_API_URL=http://127.0.0.1:9100 nexa build && nexa preview --port 4341
node ../../e2e/react-islands.mjs http://127.0.0.1:4341     # 12 comprobaciones
```

Regla: las páginas de `src/pages` son TSX de Nexa (no React). React vive en las islas (cualquier módulo con
`export default function mount(el, props)`), que se empaquetan **fuera** de `src/` con esbuild.
