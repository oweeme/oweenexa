# Política de LTS y migraciones entre versiones mayores

Introducida en la Fase 15, antes de considerar Nexa 1.0. Complementa
`docs/POLITICA-DE-VERSIONES.md` (Fase 12), que cubre los **módulos**
individuales (`ui`, `forms`, `platform`, ...) — este documento cubre el
**framework como un todo**: el compilador (`nexa-cli` y las crates que lo
componen) y el contrato que expone (`nexa.toml`, la forma del HTML/JS que
genera, las convenciones de `src/pages`).

## Qué cubre esta política

- El formato de `nexa.toml` (secciones, nombres de campo).
- Las convenciones de archivo que reconoce el compilador (`load`, `seo`,
  `schema`, `t()`, `[locale]`, `data-nexa-*`).
- La forma del HTML/manifiesto que produce `nexa build` (lo que
  `packages/runtime`/`packages/router` en el navegador esperan leer).

**No** cubre: el contenido exacto de un mensaje de error, el orden de
las líneas que imprime `nexa build` en la terminal, ni detalles internos
de qué crate de Rust hace qué (eso puede cambiar libremente).

## Versionado

Nexa como un todo sigue SemVer: `MAYOR.MENOR.PARCHE`.

- **PARCHE**: correcciones de bugs, sin cambios de comportamiento
  observable intencionales.
- **MENOR**: nuevas capacidades compatibles hacia atrás — un proyecto en
  `1.2` sigue compilando sin cambios en `1.3`.
- **MAYOR**: el único momento en que algo de lo listado arriba puede
  romperse.

## Ciclo de vida de cada versión mayor

- Cada versión mayor (`1.x`, `2.x`, ...) recibe correcciones de bugs y de
  seguridad durante **al menos los 12 meses** posteriores al lanzamiento
  de la siguiente versión mayor (ej. si `2.0` sale, `1.x` sigue recibiendo
  parches un año más).
- Un cambio que rompe compatibilidad en `nexa.toml`/las convenciones de
  archivo se anuncia con **al menos una versión menor de anticipación**:
  la versión menor anterior a la mayor que rompe algo emite un aviso
  (`nexa build`/`nexa add`) señalando qué va a cambiar y en qué versión.

## Codemods

Ahora mismo (Nexa `0.1.0`, camino a `1.0`) **no existe ningún codemod
todavía** — no tendría sentido: no hay ninguna versión mayor anterior de
la que migrar. Esto no es un vacío sin resolver, es simplemente
prematuro.

Lo que sí queda comprometido desde ya, para cuando llegue la primera
versión mayor que rompa algo (`2.0`, lo que sea que eso termine
significando):

- Todo cambio incompatible en `nexa.toml` o en una convención de archivo
  viene acompañado de un codemod real (`nexa migrate`, o un subcomando
  equivalente) que reescribe el proyecto del usuario automáticamente —
  no una guía en prosa que el usuario tiene que aplicar a mano.
- El codemod se prueba contra el proyecto de referencia de la Fase 15
  (`examples/oweeme-shop`) antes de publicarse: si no migra ese proyecto
  limpiamente, no está listo.
- Si un codemod no puede migrar algo automáticamente (ambigüedad real,
  no solo pereza), debe decirlo explícitamente por archivo/línea — nunca
  fallar en silencio ni migrar a medias.

## Por qué esto importa ahora, y no solo cuando exista una v2

Un tercero que evalúe si adoptar Nexa necesita saber, hoy, qué tan caro
es quedarse atrás — esa es la pregunta real detrás de "pre-producción
real" (Fase 15). Esta política es la respuesta, incluso mientras la
respuesta a "¿qué codemods existen?" siga siendo, honestamente,
"ninguno todavía, porque no hace falta ninguno todavía".
