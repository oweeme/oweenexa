//! `.env` (Fase 76): `nexa` lee `.env` y `.env.local` del proyecto y los pone en el entorno del
//! proceso (`NEXA_API_URL`, `NEXA_SITE_URL`…). Lo que ya está en el entorno real manda siempre;
//! `.env.local` (que no se sube a git) manda sobre `.env`.

use std::collections::BTreeMap;
use std::path::Path;

pub fn load(dir: &Path) {
    let mut vars = BTreeMap::new();
    for file in [".env", ".env.local"] {
        if let Ok(text) = std::fs::read_to_string(dir.join(file)) {
            vars.extend(parse(&text));
        }
    }
    for (k, v) in vars {
        if std::env::var_os(&k).is_none() {
            std::env::set_var(k, v);
        }
    }
}

pub fn parse(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim();
        if key.is_empty() || key.starts_with(|c: char| c.is_ascii_digit()) || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let value = value.trim();
        let value = if let Some(q) = value.chars().next().filter(|c| *c == '"' || *c == '\'') {
            match value[1..].find(q) {
                Some(end) => {
                    let inner = &value[1..1 + end];
                    if q == '"' { inner.replace("\\n", "\n").replace("\\\"", "\"") } else { inner.to_string() }
                }
                None => value[1..].to_string(),
            }
        } else {
            // sin comillas: un ` #` abre un comentario
            value.split(" #").next().unwrap_or("").trim().to_string()
        };
        out.push((key.to_string(), value));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comments_quotes_and_export() {
        let v = parse("# c\nNEXA_API_URL=http://localhost:9000\nexport A=\"x y\" \nB='1 # no'\nC=2 # sí es comentario\n\nmal linea\n1X=3\n");
        assert_eq!(
            v,
            vec![
                ("NEXA_API_URL".into(), "http://localhost:9000".into()),
                ("A".into(), "x y".into()),
                ("B".into(), "1 # no".into()),
                ("C".into(), "2".into()),
            ]
        );
    }
}
