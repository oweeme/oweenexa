use crate::{Expr, JsonTemplate, Template, Translate};

/// Un nodo del árbol de renderizado.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element(Element),
    /// Texto conocido en tiempo de parseo (no depende de ninguna variable).
    Text(String),
    /// Contenido dinámico (`{product.name}`): depende de datos que todavía
    /// no existen (Fase 7). El analyzer lo clasifica como `Dynamic`.
    Expression(Expr),
    /// `{t("home.title")}` / `{t("profile.donateTo", { name: data.x })}`
    /// (Fase 10, interpolación en la Fase 45): la clave y los argumentos
    /// de traducción, sin resolver todavía.
    Translate(Translate),
    /// `<For each={data.items}>{(item) => (...)}</For>` (Fase 30):
    /// iteración literal, resuelta por el compilador igual que `t()` o
    /// `data.*` — nunca un `.map()` ni código arbitrario del
    /// desarrollador.
    For(ForLoop),
}

/// Cuerpo de un `<For>`. `each` solo puede tener `data`/`params` como raíz
/// (mismo `Expr` limitado que el resto de Nexa); `item_name` es el único
/// identificador del parámetro del callback (sin destructuring); `body`
/// es el único nodo JSX que devuelve el callback — un arrow function de
/// cuerpo conciso solo puede devolver una expresión, así que esto nunca
/// admite múltiples hermanos en la raíz (fragmentos ya están fuera de
/// alcance en todo Nexa, ver `jsx.rs`).
#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    pub each: Expr,
    pub item_name: String,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub tag: String,
    pub attrs: Vec<Attr>,
    /// Manejadores de eventos (`onClick={buy}`). Separados de `attrs`
    /// porque no son atributos HTML: son lo que hace que un nodo sea
    /// `Interactive` en vez de `Static`.
    pub events: Vec<Event>,
    /// `data-nexa-island="..."` (Fase 16): marca este elemento como punto
    /// de montaje de una isla interactiva. Separado de `attrs` por la
    /// misma razón que `events` — no es un atributo HTML, es lo que hace
    /// que un nodo sea `Island` en vez de `Static`/`Interactive`.
    pub island: Option<Island>,
    pub children: Vec<Node>,
}

/// Punto de montaje de una isla interactiva (Fase 16). El `specifier` es
/// un nombre del import map (`nexa.toml [imports]`) que resuelve a un
/// módulo JS con un `export default function mount(el, props)` — Nexa
/// nunca abre ni interpreta ese módulo, solo genera el punto de montaje.
#[derive(Debug, Clone, PartialEq)]
pub struct Island {
    /// `data-nexa-island="productFilter"` — string literal únicamente,
    /// igual que `Event::strategy` no acepta expresiones dinámicas.
    pub specifier: String,
    /// `data-nexa-props={{ products: data.products }}`: se resuelve a
    /// JSON en tiempo de render, igual que `seo`/`schema`.
    pub props: Option<JsonTemplate>,
    /// `data-nexa-strategy` (`"visible"` por defecto si se omite — a
    /// diferencia de eventos, que por defecto usan `"interaction"`; una
    /// isla no tiene "el" evento obvio de disparo).
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attr {
    pub name: String,
    /// `None` representa un atributo booleano (`<button disabled>`).
    pub value: Option<AttrValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    /// `src="/logo.png"`: se conoce en tiempo de parseo.
    Static(String),
    /// `src={data.image}` o `href={\`/${params.locale}/x/${params.slug}\`}`
    /// (Fase 15: antes solo se soportaba una única referencia, no una
    /// plantilla con varias partes): se resuelve en tiempo de render,
    /// igual que el contenido dinámico de texto de `seo`. Si no se puede
    /// resolver del todo, el atributo entero se omite — un `src=""` roto
    /// es peor que ningún `src`.
    Dynamic(Template),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// Nombre del evento DOM, sin el prefijo `on` (`onClick` -> `click`).
    pub name: String,
    pub handler: Expr,
    /// Estrategia de activación pedida explícitamente vía el atributo
    /// `data-nexa-strategy` (`"interaction"`, `"visible"`, `"idle"`,
    /// `"load"`, `"manual"`). Sin validar todavía a este nivel — eso lo
    /// hace `nexa-activation` (Fase 5), que sabe cuál es el valor por
    /// defecto y qué hacer con un valor desconocido.
    pub strategy: Option<String>,
}
