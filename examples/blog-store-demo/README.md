# blog-store-demo — blog y tienda con API, idiomas y renderizado bajo demanda

Las rutas son las de un sitio real:

| Archivo | URL |
|---|---|
| `src/pages/blog/[author]/[slug].tsx` | `/blog/hectorarchitect/desarrollando-oweelinux-conlinux` |
| `src/pages/[locale]/index.tsx` | `/es`, `/en` (idiomas de `src/locales/*.json`, **sin endpoint**) |
| `src/pages/[locale]/shop/[category]/[product].tsx` | `/en/shop/zapatos/runner` |

```bash
python3 api.py &                                   # API de ejemplo en :9300 (registra cada llamada)
export NEXA_API_URL=http://127.0.0.1:9300
nexa build                                         # pre-renderiza lo que `paths` lista (y es/en solos)
nexa bundle --no-build --out site                  # UN ejecutable, con las fuentes para renderizar bajo demanda
./site --port 4500 --api-url $NEXA_API_URL --revalidate 60
```

- Un post o producto **publicado después del build** se renderiza al recibirlo y queda en caché `--revalidate` segundos.
- Si la API cae, se sigue sirviendo la última copia; una ruta nunca vista da 502.
- `../../e2e/ssr-on-demand.sh` comprueba todo esto (25 comprobaciones).
