# Política de versionado de Nexa

Introducida en la Fase 12 junto con `nexa.toml`/`nexa.lock`/`nexa add`.
Todo módulo oficial de Nexa (los que aparecen en el registro interno de
`nexa-cli`, `crates/nexa-cli/src/modules.rs`) tiene exactamente una de
estas tres etiquetas. La etiqueta decide qué se le permite romper, y
cuándo.

## Stable

- Sigue SemVer real: un cambio que rompe compatibilidad hacia atrás solo
  puede ocurrir en un salto de versión **mayor**.
- Cualquier cambio de comportamiento observable (nombres de clases CSS,
  atributos `data-nexa-*`, forma del manifiesto de activación, etc.)
  necesita, como mínimo, un ciclo de aviso de deprecación antes de
  eliminarse.
- Se puede declarar con `nexa add <módulo>` sin advertencias adicionales.
- Módulos actuales en este nivel: **`ui`** (Fase 9), **`forms`** (Fase
  10).

## Experimental

- Puede romper compatibilidad en **cualquier versión menor**, sin previo
  aviso — todavía se está decidiendo su forma final.
- `nexa add <módulo>` lo instala igual, pero imprime un aviso explícito
  señalando que es Experimental y remite a este documento.
- Un módulo pasa de Experimental a Stable cuando su API deja de cambiar
  entre fases consecutivas del roadmap — no antes.
- Ningún módulo ocupa este nivel todavía (agosto 2026); es el nivel por
  defecto para lo próximo que se añada al registro mientras se decide su
  forma.

## Internal

- No tiene ninguna garantía de compatibilidad: puede cambiar de forma
  arbitraria entre cualquier par de versiones, incluso dentro de un
  patch.
- **No se declara con `nexa add`** — `nexa add router`, por ejemplo,
  falla explícitamente explicando que `router` es interno, en vez de
  fingir que se instaló algo.
- Son piezas que o bien son parte del core y siempre están activas
  (`router`, `runtime` — inyectadas por `bootstrap::inject` en cada
  página, Fase 6), o son herramientas de un solo comando (`dev-client`,
  solo la usa `nexa dev`, Fase 11).

## Por qué esto importa ahora, y no solo en la Fase 15

Hasta la Fase 11, "instalar" un módulo de Nexa no significaba nada
formal: `@nexa/ui`/`@nexa/forms` ya funcionan con solo usarlos (clases
`nx-*`, `<form data-nexa-form>`), sin declarar nada en ningún archivo.
Esa disciplina de cero-config **no cambia** — `nexa add` no es un
requisito para que algo funcione, y `nexa build`/`nexa dev` avisan
(`NEXA-PKG-001`/`002`) pero nunca bloquean si falta la declaración.

Lo que sí cambia es que `nexa.toml`/`nexa.lock` ahora reflejan la
realidad del proyecto de verdad, sentando la forma exacta que va a
necesitar un registro real de paquetes de terceros en la Fase 15
(`@nexa/stripe`, `@nexa/firebase`, ...): ahí "instalar" un paquete sí
implicará descargar algo, y ahí es donde la diferencia entre Stable
(puedes confiar en él en producción) y Experimental (puede cambiar bajo
tus pies) empieza a importar de verdad.
