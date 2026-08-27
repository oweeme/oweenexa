use crate::Expr;

/// Un fragmento de una plantilla de texto: literal, o una referencia
/// simbólica (`data.name`, `params.slug`) a resolver más tarde. Se usa
/// para los valores de `seo` (`title: \`${data.name} | Oweeme\``) y,
/// desde la Fase 15, para valores de atributos JSX dinámicos
/// (`href={\`/${params.locale}/productos/${params.slug}\`}`) — antes de
/// la Fase 15, un atributo así se perdía en silencio (el atributo
/// aparecía en el HTML sin ningún valor, un bug real encontrado
/// construyendo el proyecto de referencia de esa fase).
#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Expr(Expr),
    /// `t("home.title")` (Fase 10): la clave de traducción, sin resolver.
    Translate(String),
}

/// Una secuencia de fragmentos que, concatenados, forman el valor final
/// (ej. el `<title>` de la página, o un `href` dinámico).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Template(pub Vec<TemplatePart>);

impl Template {
    pub fn literal(text: impl Into<String>) -> Self {
        Template(vec![TemplatePart::Text(text.into())])
    }

    /// Una plantilla de un único fragmento dinámico — el caso común de
    /// `href={data.image}` (una sola referencia, sin texto alrededor).
    pub fn from_expr(expr: Expr) -> Self {
        Template(vec![TemplatePart::Expr(expr)])
    }
}
