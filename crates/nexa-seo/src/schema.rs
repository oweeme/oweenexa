use nexa_ast::JsonTemplate;

use crate::resolve::{resolve_json_template, SeoContext};

/// Resuelve `schema` a JSON real y lo envuelve en un
/// `<script type="application/ld+json">`. Añade `@context` en la raíz si
/// falta, y renombra `type` -> `@type` en cualquier nivel de anidamiento
/// (azúcar: `{ type: "Product" }` es más natural en JS que `{ "@type":
/// "Product" }`) a menos que ya se haya puesto `@type` explícitamente —
/// hace falta a cualquier profundidad porque schema.org anida tipos
/// (`Product` con un `offers` de tipo `Offer`, etc.).
pub fn render_schema_script(schema: Option<&JsonTemplate>, ctx: &SeoContext) -> Option<String> {
    let mut value = resolve_json_template(schema?, ctx);

    rename_type_to_at_type(&mut value);
    if let serde_json::Value::Object(map) = &mut value {
        map.entry("@context".to_string())
            .or_insert_with(|| serde_json::Value::String("https://schema.org".to_string()));
    }

    let json = serde_json::to_string(&value).ok()?;
    // Un valor de texto no debe poder cerrar el `<script>` (`</script>`) ni
    // desviar el analizador de HTML (`<!--<script>` lo deja en el estado
    // "doble escape", donde el `</script>` real ya no cierra y el resto de la
    // página se traga como código). Se escapa TODO `<`, `>` y `&` como
    // `\uXXXX` — JSON válido que cualquier lector decodifica igual — y los
    // separadores U+2028/2029.
    let json = escape_json_for_html(&json);

    Some(format!("<script type=\"application/ld+json\">{json}</script>"))
}

/// JSON seguro para incrustar dentro de `<script>`: ningún `<`, `>`, `&` ni
/// separador de línea Unicode queda sin escapar.
pub(crate) fn escape_json_for_html(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for c in json.chars() {
        match c {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out
}

fn rename_type_to_at_type(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if !map.contains_key("@type") {
                if let Some(type_value) = map.remove("type") {
                    map.insert("@type".to_string(), type_value);
                }
            }
            for nested in map.values_mut() {
                rename_type_to_at_type(nested);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                rename_type_to_at_type(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn json_ld_cannot_be_broken_out_of_by_a_hostile_value() {
        for hostile in [
            "</script><script>alert(1)</script>",
            "<!--<script>alert(1)",
            "<script>alert(1)</script>",
            "a & b \u{2028} c",
        ] {
            let escaped = escape_json_for_html(&serde_json::to_string(&serde_json::json!({ "name": hostile })).unwrap());
            assert!(!escaped.contains('<') && !escaped.contains('>') && !escaped.contains('&'), "{escaped}");
            assert!(!escaped.contains('\u{2028}'));
            // y sigue siendo JSON válido que devuelve el valor original
            let back: serde_json::Value = serde_json::from_str(&escaped).unwrap();
            assert_eq!(back["name"], hostile);
        }
    }
}
