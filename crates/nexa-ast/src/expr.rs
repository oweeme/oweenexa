/// Una expresión simbólica: una referencia a una variable o a una
/// propiedad de una variable (`product`, `product.name`).
///
/// No se evalúa aquí en el sentido de ejecutar JS — este tipo solo
/// captura *a qué depende* un nodo. El analyzer (Fase 3) lo usa para el
/// dependency graph; el renderer (Fase 7) lo usa para, cuando la raíz es
/// `data`, resolver la ruta de propiedades contra el JSON que devolvió el
/// `load()` de la página.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Identifier(String),
    Member { object: Box<Expr>, property: String },
}

impl Expr {
    /// Representación legible de la expresión: `product.name`.
    pub fn path(&self) -> String {
        match self {
            Expr::Identifier(name) => name.clone(),
            Expr::Member { object, property } => format!("{}.{}", object.path(), property),
        }
    }

    /// El identificador raíz del que depende esta expresión. En
    /// `product.name` es `product` — es lo que el dependency graph
    /// necesita para saber qué variable, si cambia, afecta a este nodo.
    pub fn root_identifier(&self) -> &str {
        match self {
            Expr::Identifier(name) => name,
            Expr::Member { object, .. } => object.root_identifier(),
        }
    }

    /// Los segmentos de propiedad *después* del identificador raíz:
    /// `data.name` -> `["name"]`, `data.price.amount` -> `["price",
    /// "amount"]`, `data` -> `[]`.
    pub fn property_path(&self) -> Vec<&str> {
        match self {
            Expr::Identifier(_) => Vec::new(),
            Expr::Member { object, property } => {
                let mut path = object.property_path();
                path.push(property.as_str());
                path
            }
        }
    }
}
