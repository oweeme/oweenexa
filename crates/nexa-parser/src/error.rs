#[derive(Debug)]
pub enum ParseError {
    Syntax(String),
    NoDefaultExport,
    NoJsxReturned,
    Unsupported(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Syntax(msg) => write!(f, "syntax error: {msg}"),
            ParseError::NoDefaultExport => {
                write!(f, "no se encontró un `export default function` en el archivo")
            }
            ParseError::NoJsxReturned => {
                write!(f, "el export default no retorna JSX (`return (<...>);`)")
            }
            ParseError::Unsupported(what) => write!(f, "no soportado todavía: {what}"),
        }
    }
}

impl std::error::Error for ParseError {}
