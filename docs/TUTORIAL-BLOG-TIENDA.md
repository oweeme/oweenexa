# Tutorial: un blog y una tienda con API, idiomas y publicación sin recompilar

Construimos las rutas de un sitio real:

```
/blog/hectorarchitect/desarrollando-oweelinux-conlinux      ← blog: autor + título (dos segmentos dinámicos)
/es          /en                                            ← portada por idioma
/en/shop/zapatos/runner                                     ← tienda: idioma + categoría + producto
```

El proyecto terminado está en `examples/blog-store-demo` (con una API de ejemplo en Python). Antes,
si no lo has hecho, ver [ELIGE-TU-CAMINO.md](ELIGE-TU-CAMINO.md).

## 1. Qué espera Nexa de tu API

Nexa solo necesita JSON por HTTP (cualquier lenguaje). Para este ejemplo:

| Endpoint | Responde |
|---|---|
| `GET /api/blog` | `[{"author":"hectorarchitect","slug":"desarrollando-oweelinux-conlinux"}, …]` — **qué páginas existen** |
| `GET /api/blog/:author/:slug` | `{"title":"…","body":"…"}` — **los datos de una página** (404 si no existe) |
| `GET /api/shop` | `[{"category":"zapatos","product":"runner"}, …]` |
| `GET /api/shop/:category/:product` | `{"name":"Runner","price":99}` |

## 2. La ruta del blog

El nombre del archivo es la URL; cada `[segmento]` es un parámetro:

```tsx
// src/pages/blog/[author]/[slug].tsx
export const load  = { url: "/api/blog/:author/:slug" };   // datos de ESTA página
export const paths = { url: "/api/blog" };                 // qué páginas pre-renderizar
export const seo = {
    title: `${data.title} — ${params.author}`,
    description: data.body,
    canonical: `https://oweeme.com/blog/${params.author}/${params.slug}`,
};

export default function Post() {
    return (
        <article>
            <h1>{data.title}</h1>
            <p>por {params.author}</p>
            <p>{data.body}</p>
        </article>
    );
}
```

`nexa build` (con `NEXA_API_URL=https://tu-api`) pide `/api/blog`, y por cada entrada genera
`dist/blog/<autor>/<slug>/index.html`: HTML real, indexable, sin JavaScript.

El **contenido va directo en la página**, no dentro de una isla: así Google lo ve.

Si el cuerpo del post es **HTML de un editor o Markdown**, no lo pongas con `{data.body}` (se vería el
texto con etiquetas): usa `<SafeHtml html={data.body} />` o `<Markdown text={data.body} />`. Se sanean al
renderizar (sin `<script>`, `on*`, `javascript:`…); ver [SEGURIDAD.md](SEGURIDAD.md).

```tsx
<article>
    <h1>{data.title}</h1>
    <Markdown text={data.body} />
</article>
```

## 3. Idiomas, sin endpoint

```
src/locales/es.json   { "home": { "title": "Bienvenido" }, "common": { "tagline": "…" } }
src/locales/en.json   { "home": { "title": "Welcome" } }
```

```tsx
// src/pages/[locale]/index.tsx
export const seo = { title: t("home.title"), description: t("common.tagline"), canonical: `https://oweeme.com/${params.locale}` };
export default function Home() { return <main><h1>{t("home.title")}</h1></main>; }
```

- `/es` y `/en` se generan **solos** a partir de `src/locales/*.json` (no hace falta `paths`).
- En una ruta como `[locale]/shop/[category]/[product].tsx`, si tu `paths` solo devuelve
  `{category, product}`, Nexa la pre-renderiza **una vez por idioma**.
- Una clave que falta en un idioma **cae al idioma por defecto** (`[i18n] default = "es"` en
  `nexa.toml`; `es` si no lo pones). Un idioma que no existe (`/xx`) es un 404 real.
- `hreflang` se añade automáticamente.

### Empezar en español y añadir otros idiomas después

Lo normal es empezar solo con `src/locales/es.json`: todo vive bajo `src/pages/[locale]/` y salen
`/es/...`. Cuando quieras alemán e inglés **basta con crear `de.json` y `en.json`** y recompilar:
aparecen `/de/...` y `/en/...`, con `hreflang` en todas las páginas, y las claves que aún no traduces
**caen al español** (puedes traducir poco a poco). Ver `examples/i18n-demo`.

- **Usa `[locale]` desde el primer día.** Cambiar la estructura de URLs más tarde (de `/blog/x` a
  `/es/blog/x`) obliga a redirecciones y daña el SEO.
- **La portada `/`**: si no escribes tu propia `src/pages/index.tsx`, `nexa build` genera una **puerta de
  idioma**: lleva al idioma del navegador si el sitio lo tiene (alemán → `/de`), y si no, al idioma por
  defecto (`[i18n] default`, `es`); sin JavaScript hay enlaces a cada idioma y `hreflang="x-default"`.
- **El contenido que viene de tu API** también cambia por idioma: pásale el idioma, p. ej.
  `export const load = { url: "/api/blog/:author/:slug?lang=:locale" };` (o `/api/:locale/blog/...`).

## 4. Componentes compartidos

```tsx
// src/components/Price.tsx
export default function Price(props) { return <p class="price">{t("shop.price")}: {props.amount}</p>; }
```
```tsx
import Price from "../../../../components/Price";
…<Price amount={data.price} />…
```

Los componentes se expanden **al compilar**: el resultado es el mismo HTML que si lo escribieras a mano.

## 5. Publicar sin recompilar

```bash
export NEXA_API_URL=https://tu-api
nexa build                         # pre-renderiza lo que existe hoy
nexa bundle --no-build --out site  # UN ejecutable que además sabe renderizar lo nuevo
./site --port 8080 --host 0.0.0.0 --api-url $NEXA_API_URL --revalidate 60
```

Qué pasa cuando alguien visita un post que **se publicó después del build**:

1. El ejecutable lo renderiza al recibirlo (pide `/api/blog/…` a tu API) y lo guarda en caché.
2. Durante `--revalidate` segundos (60 por defecto) se sirve desde la caché: **una sola llamada a tu
   API por página y ventana**, aunque lleguen mil visitas a la vez.
3. Pasada la ventana, la próxima visita lo vuelve a pedir: una edición del post aparece sola.
4. **Si tu API cae**, el sitio sigue sirviendo la última copia de cada página que ya vio
   (`X-Nexa-Cache: stale`). Una página nunca vista devuelve `502`.
5. Un post que no existe (404 de tu API) es un 404 real, recordado solo 10 s para que un post nuevo
   aparezca pronto sin que un bot insistente golpee tu API.

Lo que **no** cuesta ninguna llamada a tu API: lo pre-renderizado, los archivos estáticos y las rutas
que no parecen páginas (`/wp-login.php`, `/.env`…).

### Seguridad que ya está resuelta

- Los parámetros de la URL se **codifican** antes de ir a tu API: `/blog/x/a%3Fadmin%3D1` no puede
  inyectar `?admin=1` ni segmentos (`..`) en tu endpoint.
- El mensaje de error de una API caída (que puede traer su URL interna) va al log del servidor,
  nunca al visitante.
- Las llamadas a tu API tienen un plazo de 10 s: una API lenta no cuelga el servidor.

### Cuándo recompilar

Un post nuevo **no** necesita `nexa build`. Recompila (o relanza) cuando cambies el **código**:
plantillas, componentes, traducciones. Si no quieres renderizado bajo demanda (solo lo pre-renderizado):
`nexa bundle --static`.

## 6. Interactividad: las islas

Con la lista ya en HTML (para SEO), añade lo interactivo como una isla:

```tsx
<div data-nexa-island="productFilter" data-nexa-strategy="visible" data-nexa-props={{ groups: data.groups }}>
    <p>Cargando filtro…</p>   {/* HTML real mientras tanto */}
</div>
```

```toml
# nexa.toml
[imports]
productFilter = "/vendor/filter.js"     # el módulo de la isla (React, Vue o TypeScript puro)
```

Ver `examples/react-islands-demo` (React 19 y TypeScript puro).

## 7. Seguridad

Nexa escapa todo lo que viene de tu API, bloquea `href="javascript:…"` desde datos y codifica los
parámetros de la URL antes de llamar a tu API. Lo que **no** hace (SQL injection, autenticación, CSP,
contenido HTML enriquecido) está en [SEGURIDAD.md](SEGURIDAD.md).

## 8. Verificarlo

```bash
nexa lint                 # SEO, accesibilidad, errores de TSX y del Modo App (para CI)
```

`e2e/ssr-on-demand.sh` automatiza el escenario completo de este tutorial (25 comprobaciones).
