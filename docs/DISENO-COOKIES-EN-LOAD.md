# [Diseño] Lectura de cookies en `load()` para sesión server-side

Documento de decisión para el issue #18 — evalúa el mecanismo, no lo
implementa. El criterio de aceptación del issue pide explícitamente
esto: una decisión de diseño primero, un issue de implementación
después.

## El problema, en una frase

`export const load = { url: "..." }` es un objeto literal que
`nexa-parser` extrae con análisis sintáctico puro (Fase 7) — nunca hay
un motor de JS ejecutando nada. Eso es lo que permite que Nexa sepa,
sin ejecutar una sola línea del proyecto, exactamente qué URL va a
pedir cada página. Leer una cookie de sesión real (HttpOnly, invisible
para JS del cliente) para decidir qué pedirle al backend rompe esa
premisa si se hace mal: "leer una cookie" suena inocente hasta que se
pregunta *qué* cookie, *quién* la declara, y si eso abre la puerta a
que `load` empiece a tomar decisiones (`if cookie == x, redirigir`) —
en ese punto ya no es una plantilla literal, es un programa.

## Restricción de fondo que cambia todo: cookies y `nexa build` no se llevan

Esto no está en el issue original, pero es la primera conclusión real
de este análisis y condiciona todo lo demás: **una cookie es
información de una petición concreta, y `nexa build` no tiene ninguna
petición** — genera archivos estáticos una sola vez, sin visitante
todavía. Una página cuyo `load` depende de `cookies.*` no puede
pre-renderizarse nunca como HTML estático; solo puede existir como una
ruta dinámica servida en el momento por `nexa preview`/`nexa dev`, o
por un futuro servidor de producción "siempre corriendo" (el mismo
bucket que ya existe desde la Fase 34 para rutas sin `paths`
declarado). Esto no es un obstáculo — es una aclaración necesaria del
alcance: **cookies en `load()` es, por diseño, una capacidad exclusiva
del modo de despliegue dinámico, nunca del build estático.**
`nexa build` debería tratar una página así exactamente como ya trata
una ruta dinámica sin `paths`: la salta, con el mismo aviso.

## Decisión 1 — qué se expone: un objeto `cookies.*` con allowlist explícita en `nexa.toml`

```toml
# nexa.toml
[cookies]
allow = ["session_id"]
```

```tsx
export const load = {
    url: "/api/profile",
    headers: { Authorization: cookies.session_id },
};
```

- `cookies.session_id` es una referencia literal, exactamente la misma
  forma que `data.x`/`params.x` — nunca `cookies[variable]` ni
  `document.cookie` crudo.
- Solo los nombres declarados en `[cookies] allow` son válidos. Un
  `cookies.algo` no declarado se trata igual que un specifier de import
  no declarado (Fase 31, `NEXA-PKG-004`): un aviso de `nexa lint`, y
  potencialmente un error duro en `nexa build`/`nexa preview` — a
  decidir en el issue de implementación, pero el principio ya es
  consistente con un mecanismo que Nexa ya tiene.
- **Nunca "todas las cookies"**. Una allowlist es lo único que mantiene
  esto auditable: leer `nexa.toml` alcanza para saber exactamente qué
  información de sesión toca cada proyecto, sin tener que leer cada
  `load()` de cada página.

### Por qué no una función `cookies.get(nombre)`

Se evaluó y se descarta: una llamada a función abre la puerta a "¿y si
el argumento es una variable calculada?", exactamente el tipo de
ambigüedad que el resto de Nexa evita a propósito (`t()` tampoco acepta
`t(variable)`, ver `docs/REFERENCIA.md`). Un acceso de propiedad
(`cookies.session_id`) es sintácticamente imposible de parametrizar,
igual que `data.x`.

## Decisión 2 — dónde puede aparecer `cookies.*`: solo dentro de `load`, nunca en el cuerpo de la página

Esta es la restricción de seguridad central del diseño: **`cookies.*`
no se resuelve en JSX, `seo`, ni `schema`** — a diferencia de `data.*`/
`params.*`, que sí pueden imprimirse en el HTML. Si `cookies.*`
funcionara en cualquier lado como `data.*`, un desarrollador podría por
accidente escribir `<p>{cookies.session_id}</p>` y filtrar un token de
sesión al HTML servido — exactamente el tipo de bug silencioso que
Nexa evita en todo lo demás (ver la disciplina de "nunca inventar un
valor" del renderer). La única salida legítima para un valor de
`cookies.*` es como **header saliente de la petición HTTP que hace
`nexa-loader` contra el backend** — nunca como texto en el documento.
Si el backend necesita devolver algo derivado de la sesión (el nombre
del usuario logueado, por ejemplo), lo hace en el JSON de respuesta
como cualquier otro campo de `data.*` — el flujo ya existente, sin
ningún mecanismo nuevo.

## Decisión 3 — `load` sigue sin poder decidir nada: sin redirects condicionales

Explícitamente **no** — habilitar `if (sin sesión) redirigir a
/login` dentro de `load` es una superficie completamente distinta
(control de flujo condicional, la primera vez que Nexa tendría eso en
cualquier parte del compilador) y, si alguna vez se construye, merece
su propio issue de diseño, no colgarse de este. La única salida
observable de `load` sigue siendo: datos (JSON, éxito) o "no encontrado"
(404 -> `PageError::NotFound`, ya existente). Un backend que decide que
la sesión es inválida ya tiene una salida perfectamente literal para
eso: devolver 401/403, y que la página lo trate igual que hoy trata un
404 — extender `PageError`/`LoaderError` con un caso más para
"no autorizado" es una pieza pequeña y consistente con lo que ya existe,
sin abrir la puerta a redirects arbitrarios.

## Cómo se mantiene "nunca código arbitrario" (la pregunta central del issue)

Tres líneas, ninguna nueva de verdad — cada una ya tiene precedente
exacto en Nexa hoy:

1. **Superficie declarada de antemano, no descubierta en runtime**
   (`[cookies] allow`) — mismo principio que `[imports]`.
2. **Referencia sintáctica, nunca una llamada ni una variable como
   clave** (`cookies.nombre`) — mismo principio que `data.x`/`t()`.
3. **Sin control de flujo: la forma de `load` no cambia, solo gana un
   campo más (`headers`) cuyos valores son literales o referencias
   limitadas** — mismo principio que `data-nexa-props={{...}}` (Fase 16,
   `JsonTemplate`) o el segundo argumento de `t()` (Fase 45): un objeto
   literal cuyos valores son texto o una referencia `data`/`params`
   conocida, nunca una expresión arbitraria.

Nada de esto convierte a `load` en un programa — sigue siendo, letra
por letra, "un patrón sintáctico que el compilador resuelve él mismo".

## Bosquejo de implementación (fuera de alcance de este documento — para el issue de seguimiento)

Solo para que el issue de implementación no arranque de cero:

- `nexa.toml`: sección `[cookies] allow = [...]`, parseada junto al
  resto de `manifest.rs`.
- `nexa_ast::Loader`: gana `pub headers: Vec<(String, HeaderValue)>`
  opcional, con `enum HeaderValue { Static(String), Cookie(String), Param(String) }`
  — mismo patrón "todo o nada" de reconocimiento que ya usa
  `nexa-parser::translate` para el segundo argumento de `t()`.
- `nexa-cli` (preview/dev): leer el header `Cookie` real de la petición
  entrante (`tiny_http::Request`), parsear `nombre=valor; nombre2=valor2`
  a un `BTreeMap<String, String>`, y pasarlo a `compile_page` igual que
  ya se pasan `params`/`api_base`.
- `nexa_loader::load`: nuevo parámetro `cookies: &BTreeMap<String, String>`;
  cada `HeaderValue::Cookie(nombre)` en `loader.headers` se resuelve
  contra ese mapa y se agrega como header real a la petición `ureq`
  saliente — si el nombre no está en `[cookies] allow`, error de build
  claro (mismo canal `PageError::Other` que ya usa la validación de
  interpolación de `t()`, Fase 45).
- `nexa build`: una página cuyo `load.headers` referencia `cookies.*`
  se trata como ruta dinámica no pre-renderizable — mismo aviso que ya
  existe para rutas sin `paths` (Fase 34).
- `PageError`/`LoaderError`: un caso nuevo para 401/403 del backend,
  tratado por la página igual que hoy trata un 404.

## Resumen de la decisión

| Pregunta del issue | Decisión |
| --- | --- |
| ¿Qué se expone? | `cookies.*`, solo nombres declarados en `nexa.toml [cookies] allow` — nunca "todas". |
| ¿Habilita redirects/control de flujo? | No. `load` sigue devolviendo solo datos o "no encontrado"/"no autorizado". |
| ¿Cómo se mantiene "nunca código arbitrario"? | Allowlist declarada + referencia sintáctica limitada (`cookies.nombre`) + sin nuevas formas de control de flujo — los tres mismos principios que ya usa el resto de Nexa. |
| ¿Dónde puede usarse el valor? | Solo como header saliente dentro de `load.headers` — nunca en JSX/`seo`/`schema`, para no poder filtrar un token de sesión al HTML. |
| ¿Compatible con `nexa build` estático? | No — es, por diseño, una capacidad exclusiva de rutas dinámicas servidas en el momento (mismo bucket que la Fase 34). |

**Próximo paso:** si el usuario confirma esta dirección, abrir un issue
de implementación nuevo con el bosquejo de arriba como punto de
partida — este documento no cambia ningún código todavía, tal como
pide el propio issue #18.
