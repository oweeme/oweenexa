# Elige tu camino: ¿qué uso para qué?

Nexa tiene **tres piezas** y la confusión más común es no saber cuál usar. Esta página es la guía
de decisión. Para construir algo de punta a punta, ver
[TUTORIAL-BLOG-TIENDA.md](TUTORIAL-BLOG-TIENDA.md).

## Las tres piezas

| Pieza | Dónde vive | Qué es | Cuándo |
|---|---|---|---|
| **Nexa Core** | `src/pages/` | HTML real generado por el compilador, con SEO. Sin JavaScript salvo lo que pidas. | Todo contenido que un buscador debe ver: blog, tienda, catálogo, landing, documentación. |
| **Isla** | un `<div data-nexa-island>` dentro de una página de Core | Un pedazo interactivo (cualquier módulo JS, un framework incluido) que se activa cuando hace falta. | Filtros, carrito, comentarios, mapas, editores, gráficos, chat. |
| **Modo App** | `src/app/` | Una SPA: TSX completo que corre en el navegador, con router propio. | Paneles, dashboards, apps con login. **No tiene SEO.** |

Una sola regla: **lo que debe indexarse va en Core; lo que es interactivo va en una isla o en el Modo App.**

## Un ejemplo real: un sitio como oweeme.com

Un sitio con una parte pública (que cambia de contenido) y un panel privado usa **las tres piezas a la vez**,
en un solo proyecto y un solo dominio:

```
oweeme.com/                      ┐
oweeme.com/es  /en  /de          │  NEXA CORE  (src/pages)
oweeme.com/blog/autor/titulo     │  HTML real para Google. Lo que no cambia se pre-renderiza;
oweeme.com/es/shop/zapatos/runner│  lo que se publica después se renderiza bajo demanda (nexa bundle).
                                 ┘
   …y DENTRO de esas páginas,    ──►  ISLAS: el buscador/filtro, el botón de carrito, los comentarios,
   solo lo interactivo                  el menú "Hola, Héctor" (que consulta /api/me desde el navegador).

oweeme.com/dashboard             ──►  MODO APP (src/app, [app] base = "/dashboard"): login, tablas,
                                      formularios. Una SPA: sin SEO, con `guard`, hablando con tu API.
```

Cómo encajan:

- **Mismo dominio, mismas cookies.** La sesión (cookie `HttpOnly` que pone tu backend, vía `[proxy]`)
  sirve para las islas de la web pública y para el dashboard.
- **La web pública es HTML compartido por todos.** El servidor no sabe quién eres al renderizar (el HTML
  se cachea y se sirve igual a todos): lo personal ("Hola, Héctor", el contador del carrito) lo pone una
  **isla** que consulta tu API desde el navegador. Se puede montar en el `layout.tsx` y sobrevive a la navegación.
- **Del sitio al dashboard** es una carga completa (`<a href="/dashboard" data-nexa-reload>`): son dos routers distintos.
- **Que Google no indexe el dashboard:** pon `Disallow: /dashboard` en `public/robots.txt`.
- **Idiomas:** `t()` y `[locale]` son de Core. El Modo App **no tiene un sistema de traducción propio**:
  el dashboard gestiona sus textos con un diccionario y una señal (ver "Límites conocidos").

Si solo tuvieras la parte pública, usarías Core (+ islas). Si solo tuvieras un panel, el Modo App.

## Preguntas frecuentes

**¿Un blog, una tienda o un catálogo que consume una API tiene que ir dentro de una isla?**
No. Va **directo en Core**: `load` pide los datos a tu API y `<For>` pinta la lista como HTML real.
Si metieras la lista dentro de una isla, el buscador no la vería. La isla es solo para lo
interactivo *dentro* de esa página (un filtro, un botón de "añadir al carrito").

**¿El multi-idioma va en una isla?**
No: es nativo de Core. `src/pages/[locale]/...` + `src/locales/<idioma>.json` + `t("clave")`. Nexa
genera `hreflang` y pre-renderiza cada idioma (incluso sin ningún endpoint: lee `src/locales/`).

**¿Puedo usar React / Vue?**
Sí, pero no en las páginas de Core (esas son TSX de Nexa, sin hooks): en una **isla**, o montado con
`ref` dentro del Modo App. Ver `examples/react-islands-demo`. TypeScript puro funciona en cualquiera de las tres.

**¿Es seguro contra inyecciones?**
Lo que viene de tu API se escapa, las URLs ejecutables (`javascript:`) y los `onclick` desde datos se
omiten, y los parámetros se codifican antes de llamar a tu API. Lo que sigue siendo del backend (SQL,
autenticación, CSP) está en [SEGURIDAD.md](SEGURIDAD.md).

**¿Hay router?**
Sí, en tres niveles: rutas por archivos en Core (`[slug]`, `[locale]`, varios segmentos), navegación
sin recarga entre páginas de Core (con prefetch y layout persistente), y el router de cliente del
Modo App (parámetros, `guard`, carga lazy). Un router propio de React o Vue dentro de una isla convive
sin conflicto.

**¿Una ruta como `/blog/hectorarchitect/desarrollando-oweelinux-conlinux` es dinámica?**
Sí: `src/pages/blog/[author]/[slug].tsx`. Ver el tutorial.

## ¿Cómo se sirve el contenido que cambia? (el punto clave)

Tienes cuatro formas de que un post o producto nuevo aparezca:

| Modo | Cómo | Cuándo se actualiza | Cuándo usarlo |
|---|---|---|---|
| **Pre-renderizado** | `export const paths` → `nexa build` genera un HTML por entrada | Al volver a correr `nexa build` | Contenido que cambia poco; máxima velocidad y simplicidad. |
| **Bajo demanda** (`nexa bundle`) | Un ejecutable con las fuentes; renderiza al recibir la ruta y la guarda en caché | Al vencer `--revalidate` (60 s por defecto) | **Blog/tienda que se publica sin recompilar.** |
| **Al vuelo** (`nexa preview`) | El servidor de desarrollo/vista previa | En cada petición | Solo para probar; no es un servidor de producción. |
| **En el navegador** | Modo App pide los datos desde el cliente | Al instante | Zonas privadas; sin SEO. |

Lo habitual en un blog o tienda: **pre-renderizar lo que ya existe (`paths`) y dejar que
`nexa bundle` renderice bajo demanda lo nuevo.** Tienes velocidad estática para lo conocido y no
recompilas para lo que publicas después.

## ¿Dónde lo despliego?

| Opción | Comando | Notas |
|---|---|---|
| Un ejecutable | `nexa bundle` | Sin Node ni nginx; gzip, caché, SPA, proxy y renderizado bajo demanda incluidos. Sin HTTPS propio: ponlo detrás de un proxy/balanceador. |
| Estáticos + nginx | `nexa build` + `nexa add nginx` | Solo lo pre-renderizado; para renderizado bajo demanda hace falta el ejecutable. |
| Escritorio | `nexa add tauri` + `nexa build --target desktop` | Binario nativo con el webview del sistema. |
| Móvil | `nexa add capacitor` | Sin verificar en este repositorio. |

## ¿En qué lenguaje se programa?

**TypeScript con TSX** (`.tsx`). La sintaxis recuerda a React, pero:
- **Core**: un subconjunto fijo (`data.x`, `params.x`, `t()`, `<For>`, `<Show>`, componentes que se
  expanden al compilar). Sin hooks ni lógica libre.
- **Modo App**: TSX con señales (`state`, `computed`), al estilo Solid. No es React.
- **Islas**: lo que quieras (React, Vue, TypeScript puro), empaquetado con esbuild.
- **Backend**: cualquier lenguaje que hable HTTP/JSON.
