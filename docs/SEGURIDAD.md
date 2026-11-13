# Seguridad: qué protege Nexa y qué sigue siendo tuyo

Las afirmaciones de esta página están **probadas** (tests en `crates/nexa-renderer`, `nexa-seo`,
`nexa-ast`, `nexa-loader`, `nexa-cli`, y un ataque de punta a punta con una API maliciosa).

## Lo que Nexa protege por ti

| Ataque | Cómo se evita |
|---|---|
| **XSS por texto** (`<script>` en un título, comentario o nombre que viene de tu API) | Todo texto y todo valor de atributo se escapa (`&lt;`, `&quot;`). Un valor no puede abrir una etiqueta ni salirse de un atributo. |
| **XSS por URL** (`href="javascript:…"`, `data:text/html`, `vbscript:`) | Una URL que viene de **datos** (API, `params`) en `href`, `src`, `action`, `formaction`, `poster`… con un esquema ejecutable se **omite**. Detecta los disfraces: mayúsculas, espacios, `java\tscript:`. `data:image/` solo vale como `src` de una `<img>`. |
| **XSS por atributos de evento** (`onclick={data.x}`, `srcdoc={data.x}`) | Un dato nunca llega a un atributo `on*` ni a `srcdoc`. |
| **Romper el JSON-LD** (`</script>`, `<!--<script>`) | El JSON-LD escapa todo `<`, `>`, `&` y U+2028/9 como `\uXXXX`: sigue siendo JSON válido y el valor hostil queda como dato. |
| **`canonical` / `og:image` falsos** | Solo se emiten con `http(s)` o rutas relativas. |
| **HTML/Markdown de tu API** (cuerpo de un post) | `<SafeHtml>` y `<Markdown>` lo pasan por un sanitizador con lista de etiquetas permitidas: nada de `<script>`, `<iframe>`, `<svg>`, `<form>`, `on*`, `style`, `id`, `javascript:`/`data:`. Ver abajo. |
| **Props de una isla** | Van en un atributo, escapados. |
| **Inyección en la llamada a tu API** (`/blog/x/a?admin=1`, `../`) | Los parámetros de la URL se **codifican** (`%XX`) antes de ir a tu API; `.`/`..` se rechazan. |
| **Path traversal** (`/../etc/passwd`) | Rechazado (400) en el ejecutable, y bloqueado en `nexa preview`/`dev`. |
| **XSS reflejado en el 404** | La ruta se escapa en las páginas de error de `preview`/`dev`; el ejecutable responde texto plano. |
| **Fugas de errores internos** | Si tu API falla, el visitante ve un `502` genérico; el detalle (con tu URL interna) va solo al log. |
| **API lenta** | Plazo de 10 s por llamada. |
| **Cabeceras** | `X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`, `Referrer-Policy` en todo. |
| **Directorio de trabajo del ejecutable** | `0700`, dueño verificado; un paquete manipulado no puede escribir fuera. |

Lo que escribes tú a mano en el TSX (`<a href="javascript:void(0)">`) se considera de confianza y no se toca;
el mismo texto **desde datos** sí se bloquea.

## Contenido enriquecido: `<SafeHtml>` y `<Markdown>`

```tsx
<article>
    <h1>{data.title}</h1>
    <SafeHtml html={data.bodyHtml} />        {/* HTML de un editor (TinyMCE, Quill…) */}
    <Markdown text={data.bodyMarkdown} />    {/* Markdown (CommonMark + tablas + tachado) */}
</article>
```

El valor se sanea **al renderizar** con `ammonia` (basado en el analizador de HTML5 de Servo; re-serializa el
DOM, así que no se deja engañar por trucos de mutación):

- **Permitido**: párrafos, títulos, listas, citas, tablas, `<pre><code class="language-x">`, `<strong>`,
  `<em>`, `<img>` (https), enlaces (`http`, `https`, `mailto`, `tel`, o relativos).
- **Siempre quitado**: `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`, `<form>`, `<input>`,
  `<svg>`, `<math>`, `<base>`, `<meta>`, `<link>`, todo atributo `on*`, `style`, `id` (evita el DOM
  clobbering), `srcdoc`, y cualquier URL `javascript:`/`data:`/`vbscript:`.
- **Enlaces**: reciben `rel="noopener noreferrer nofollow ugc"`; **imágenes**: `loading="lazy"`; `class`
  solo en `<code>`/`<pre>` (resaltado de sintaxis).
- **Límites**: un valor de más de 2 MB, o con más de 128 niveles de etiquetas anidadas, no se muestra
  (protege al servidor; un post real no se acerca). Un valor que no es texto no muestra nada.
- El atributo solo admite una referencia a datos (`data.x`, `params.x`, el item de un `<For>` o un prop).
  Un literal escrito a mano no necesita saneado: escríbelo directamente en el TSX.
- No hay `<RawHtml>` sin sanear, a propósito.

**Qué cubre y qué no:** el contenido llega seguro al HTML de la página. No valida que el autor "diga la
verdad" ni evita el spam de enlaces (por eso `nofollow ugc`).

## Lo que sigue siendo tu responsabilidad

- **SQL / NoSQL injection, autenticación y autorización → tu backend.** Nexa nunca construye consultas ni
  guarda usuarios. Recibe tus datos como JSON y manda los parámetros ya codificados, pero tu API debe usar
  consultas parametrizadas y comprobar permisos. El `guard` del Modo App **solo protege la interfaz**: lo
  que protege los datos es que tu API responda `401`/`403`.
- **CSRF.** Usa cookies `SameSite=Lax`/`Strict` + `HttpOnly` en tu backend (ver "Backend, proxy y sesión").
- **CSP (`Content-Security-Policy`).** No se envía por defecto: una CSP genérica rompería los imports de
  terceros (`[imports]`) y la cámara de `platform`. Si quieres una, añádela en tu nginx/proxy con los
  orígenes que usas.
- **Contenido enriquecido.** `{data.body}` **escapa todo**: si tu API devuelve `"<p>Hola</p>"` se ve el
  texto literal, no un párrafo. Para un post con formato usa `<SafeHtml html={data.body} />` o
  `<Markdown text={data.body} />` (ver abajo): son la forma explícita y saneada.
- **Dependencias del navegador** (React, Phaser…) y el contenido de `public/`: las que tú elijas.
- **HTTPS.** `nexa bundle` no termina TLS: ponlo detrás de un proxy inverso o balanceador.
- **Parámetros en el *host* de `load.url`**: no pongas un `:parámetro` donde iría el dominio
  (`https://:sub.miapi.com`).

## Cómo comprobarlo tú mismo

`cargo test -p nexa-renderer --test composition` (sección "Seguridad"), `cargo test -p nexa-ast url_safety`,
y el patrón del ataque de punta a punta: una API que devuelva `<script>…`, `javascript:…`,
`"><img onerror=…>` y `</script><!--<script>` en cada campo, `nexa build`, y analizar el HTML con un parser
(no con búsquedas de texto: `&lt;script&gt;` contiene la palabra "script" y es inofensivo).
