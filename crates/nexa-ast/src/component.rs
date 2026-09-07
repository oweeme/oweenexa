use std::collections::BTreeMap;

use crate::{JsonTemplate, Loader, Node, SeoConfig};

/// Un componente Nexa: por ahora, un único árbol raíz sin props ni estado.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub root: Node,
    /// `export const load = { url: "..." }`, si la página lo declara.
    pub loader: Option<Loader>,
    /// `export const paths = { url: "..." }` (Fase 17): igual forma que
    /// `load`, pero el backend responde con un array de sets de
    /// parámetros (`[{"slug":"iphone-17"}, ...]`) en vez de un solo
    /// objeto — es lo que le permite a `nexa build` enumerar y
    /// pre-renderizar una ruta dinámica a HTML estático real, en vez de
    /// dejarla solo para `nexa preview`.
    pub paths: Option<Loader>,
    /// Código fuente exacto de cada `function nombre() {...}` o `const
    /// nombre = () => {...}` de nivel superior, indexado por nombre. Es
    /// lo que `nexa-activation` usa para generar el chunk real de un
    /// evento (`onClick={buy}`) en vez de un placeholder — sin que Rust
    /// llegue nunca a *ejecutar* ese código, solo a copiarlo.
    pub handlers: BTreeMap<String, String>,
    /// `export const seo = { title: ..., ... }`, si la página lo declara.
    pub seo: Option<SeoConfig>,
    /// `export const schema = { type: "Product", ... }` (JSON-LD), si la
    /// página lo declara.
    pub schema: Option<JsonTemplate>,
}
