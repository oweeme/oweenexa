//! Fase 3-7: IR (+ datos y parámetros reales, si los hay) → HTML.
//!
//! El renderer consume el IR ya clasificado (`nexa_ir`), no el AST crudo:
//! "el HTML es un producto del IR" (docs/Arquitectura SEO Completo
//! Framework.md). Eso es lo que permite que `data-nexa="<id>"` salga
//! directamente de `node.classification` — ningún manifiesto externo
//! decide qué marcar; `nexa-activation` (Fase 5) se limita a recorrer
//! este mismo IR y construir el manifiesto a partir de esos mismos ids.
//!
//! Desde la Fase 7, el renderer acepta un [`RenderContext`] con el JSON
//! del `load()` de la página y los parámetros de ruta que capturó
//! `nexa-router`. Una expresión dinámica se resuelve a su valor real si
//! su raíz es `data` o `params`; cualquier otra referencia — o `data`/
//! `params` cuando no hay nada que resolver — sigue renderizándose como
//! el comentario inerte `<!--nexa:product.name-->` de siempre.

mod context;
mod html;

#[cfg(test)]
mod tests;

pub use context::RenderContext;

use nexa_ir::{IrComponent, IrNode, IrNodeKind};

/// Renderiza un componente como documento HTML completo.
pub fn render_html_document(component: &IrComponent, ctx: &RenderContext) -> String {
    let body = render_node(&component.root, ctx);

    format!(
        "<!doctype html>\n\
<html lang=\"es\">\n\
<head>\n\
<meta charset=\"UTF-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>Nexa</title>\n\
</head>\n\
<body>\n\
{body}\n\
</body>\n\
</html>\n"
    )
}

/// Renderiza un nodo individual.
pub fn render_node(node: &IrNode, ctx: &RenderContext) -> String {
    match &node.kind {
        IrNodeKind::Text(text) => html::escape_html(text),
        IrNodeKind::Expression(expr) => match context::resolve(expr, ctx) {
            Some(resolved) => html::escape_html(&resolved),
            None => format!("<!--nexa:{}-->", expr.path()),
        },
        IrNodeKind::Translate(translate) => match context::resolve_translation(translate, ctx) {
            Some(resolved) => html::escape_html(&resolved),
            None => format!("<!--nexa:t({})-->", translate.key),
        },
        IrNodeKind::Element { .. } => html::render_element(node, ctx),
        IrNodeKind::For { each, item_name, body } => render_for(each, item_name, body, ctx),
    }
}

/// `<For each={data.items}>{(item) => (...)}</For>` (Fase 30): el `body`
/// es una plantilla clasificada una sola vez (un solo `NodeId` por nodo
/// interactivo adentro, ver `nexa-activation`) — acá se renderiza una
/// copia de su HTML por cada elemento real del array, cada una con
/// `item_name` apuntando a ese elemento. Todas las copias comparten el
/// mismo `data-nexa="<id>"` si el cuerpo tiene algo interactivo; el
/// runtime de activación (`packages/runtime`) activa cada copia por
/// separado, no solo la primera.
fn render_for(each: &nexa_ast::Expr, item_name: &str, body: &IrNode, ctx: &RenderContext) -> String {
    context::resolve_each(each, ctx)
        .iter()
        .map(|item| {
            let item_ctx = RenderContext {
                data: ctx.data,
                params: ctx.params,
                translations: ctx.translations,
                loop_binding: Some((item_name, item)),
                current_path: ctx.current_path,
            };
            render_node(body, &item_ctx)
        })
        .collect()
}
