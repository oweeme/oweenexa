# app-demo (Fase 60 — Modo App)

Un proyecto con las dos mitades de Nexa:

- `src/pages/` — landing con SEO, HTML real (Nexa Core).
- `src/app/` — una SPA completa bajo `/app`: login con guard, lista con
  filtro reactivo (`For`/`Show`), detalle con parámetros de ruta.

```bash
python3 backend.py &      # backend de ejemplo en :9100 (cualquier backend sirve)
nexa dev --port 4321      # o: nexa build && nexa preview --port 4321
# abrir http://127.0.0.1:4321/
```
