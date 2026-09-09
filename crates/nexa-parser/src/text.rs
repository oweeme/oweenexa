//! Normalización de texto JSX y de expresiones literales simples
//! (`{"texto"}`, `{42}`). Sin evaluación de variables — eso llega con el IR.

use oxc_ast::ast::{JSXExpression, JSXExpressionContainer};

/// Colapsa espacios/indentación de la misma forma que JSX de verdad
/// (mismo algoritmo que `cleanJSXElementLiteralChild` de Babel, no una
/// aproximación): el espacio/tab del **borde** de una línea se recorta
/// solo si esa línea NO es la primera o NO es la última del bloque de
/// texto — nunca en ambos casos a la vez. Un bloque de texto de una
/// sola línea es simultáneamente su primera y su última línea, así que
/// ningún borde se toca (Fase 36, bug real encontrado escribiendo
/// ejemplos legítimos: `{a} — {b}` perdía el espacio pegado a cada
/// expresión, porque la versión anterior recortaba las dos puntas de
/// *cualquier* línea sin importar su posición). Las líneas
/// intermedias (ni primera ni última) sí se recortan de los dos lados
/// — así es como colapsa la indentación entre elementos hermanos en
/// líneas separadas.
pub(crate) fn normalize_jsx_text(raw: &str) -> Option<String> {
    let lines: Vec<&str> = raw.lines().collect();
    let last_non_empty = lines.iter().rposition(|line| !line.trim().is_empty())?;

    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == lines.len() - 1;

        let mut segment = *line;
        if !is_first {
            segment = segment.trim_start_matches([' ', '\t']);
        }
        if !is_last {
            segment = segment.trim_end_matches([' ', '\t']);
        }
        if segment.is_empty() {
            continue;
        }

        out.push_str(segment);
        if i != last_non_empty {
            out.push(' ');
        }
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

pub(crate) fn expression_container_to_string(container: &JSXExpressionContainer) -> Option<String> {
    match &container.expression {
        JSXExpression::StringLiteral(s) => Some(s.value.as_str().to_string()),
        JSXExpression::NumericLiteral(n) => Some(format_number(n.value)),
        _ => None,
    }
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_jsx_text;

    #[test]
    fn a_pure_whitespace_block_normalizes_to_nothing() {
        // Indentación entre elementos hermanos en líneas separadas —
        // debe colapsar a nada, no a un espacio suelto.
        assert_eq!(normalize_jsx_text("\n    "), None);
        assert_eq!(normalize_jsx_text("   \n  \n   "), None);
        assert_eq!(normalize_jsx_text(""), None);
    }

    #[test]
    fn a_single_line_of_text_keeps_both_of_its_edge_spaces() {
        // Bug real (Fase 36): texto de una sola línea, pegado a una
        // expresión de cada lado (`{a} — {b}`) — antes perdía el
        // espacio de AMBOS bordes, porque se trataba como "primera y
        // última línea" dos veces en vez de una sola vez cada borde.
        assert_eq!(normalize_jsx_text(" — $"), Some(" — $".to_string()));
        assert_eq!(normalize_jsx_text(" hola "), Some(" hola ".to_string()));
    }

    #[test]
    fn multiline_indentation_collapses_but_real_content_keeps_a_single_separating_space() {
        // Texto real en varias líneas (con indentación real de código,
        // no forzado a mano) — el resultado debe leerse como una
        // oración normal, con un solo espacio entre palabras.
        let raw = "\n                    Hola\n                    mundo\n                ";
        assert_eq!(normalize_jsx_text(raw), Some("Hola mundo".to_string()));
    }

    #[test]
    fn leading_content_on_the_first_line_keeps_its_own_leading_edge() {
        // La primera línea nunca se recorta por el lado izquierdo —
        // importa cuando el texto arranca pegado a una expresión previa
        // en la misma línea en la que después sigue en una línea nueva.
        let raw = " pegado\n    después";
        assert_eq!(normalize_jsx_text(raw), Some(" pegado después".to_string()));
    }

    #[test]
    fn trailing_content_on_the_last_line_keeps_its_own_trailing_edge() {
        let raw = "antes    \n pegado ";
        assert_eq!(normalize_jsx_text(raw), Some("antes pegado ".to_string()));
    }
}
