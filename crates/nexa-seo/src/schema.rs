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
    // `</script>` dentro de un valor de texto no debe poder cerrar el
    // script antes de tiempo — es la misma clase de escape que necesita
    // cualquier JSON incrustado a mano en HTML.
    let json = json.replace("</script>", "<\\/script>");

    Some(format!("<script type=\"application/ld+json\">{json}</script>"))
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
