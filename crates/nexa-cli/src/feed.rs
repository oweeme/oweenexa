//! Feed RSS (Fase 81): `nexa build` genera `dist/feed.xml` a partir de una colección (`content:`) o de
//! tu API, y cada página lleva su `<link rel="alternate" type="application/rss+xml">`.
//!
//! ```toml
//! [feed]
//! title = "Mi blog"
//! description = "Artículos sobre Nexa"
//! source = "content:/posts"        # o "/api/posts" (tu API)
//! link = "/blog/:slug"             # `:campo` se sustituye por el campo de cada entrada
//! ```
//!
//! Cada entrada usa sus campos `title`, `date` (`2025-03-10` o RFC 3339) y `summary` (o `excerpt`/`description`).
//! Para URLs absolutas (los lectores de feeds las exigen) define `NEXA_SITE_URL`.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::manifest::FeedSection;

pub fn head_link(feed: &FeedSection) -> String {
    let title = escape(&feed.title);
    format!("<link rel=\"alternate\" type=\"application/rss+xml\" title=\"{title}\" href=\"/{}\">", feed.output_name())
}

/// El XML del feed. `site` (sin `/` final) vuelve absolutos los enlaces.
pub fn render(feed: &FeedSection, items: &[Value], site: Option<&str>) -> String {
    let base = site.unwrap_or("").trim_end_matches('/');
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n");
    out.push_str(&format!("<title>{}</title>\n", escape(&feed.title)));
    out.push_str(&format!("<link>{}/</link>\n", escape(base)));
    out.push_str(&format!("<description>{}</description>\n", escape(feed.description.as_deref().unwrap_or(&feed.title))));
    out.push_str(&format!("<atom:link href=\"{}/{}\" rel=\"self\" type=\"application/rss+xml\"/>\n", escape(base), feed.output_name()));
    for item in items.iter().take(feed.limit.unwrap_or(20)) {
        let Some(title) = item.get("title").and_then(Value::as_str) else { continue };
        let link = fill(&feed.link, item);
        let url = format!("{base}{link}");
        out.push_str("<item>\n");
        out.push_str(&format!("<title>{}</title>\n<link>{}</link>\n<guid isPermaLink=\"true\">{}</guid>\n", escape(title), escape(&url), escape(&url)));
        if let Some(date) = item.get("date").and_then(Value::as_str).and_then(rfc822) {
            out.push_str(&format!("<pubDate>{date}</pubDate>\n"));
        }
        let summary = ["summary", "excerpt", "description"].iter().find_map(|k| item.get(*k).and_then(Value::as_str));
        if let Some(s) = summary {
            out.push_str(&format!("<description>{}</description>\n", escape(s)));
        }
        out.push_str("</item>\n");
    }
    out.push_str("</channel>\n</rss>\n");
    out
}

/// `/blog/:slug` → `/blog/hola` con los campos de la entrada (codificados).
fn fill(template: &str, item: &Value) -> String {
    let mut vars: BTreeMap<&str, String> = BTreeMap::new();
    if let Some(obj) = item.as_object() {
        for (k, v) in obj {
            let text = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => continue,
            };
            vars.insert(k.as_str(), text);
        }
    }
    template
        .split('/')
        .map(|seg| match seg.strip_prefix(':') {
            Some(name) => encode(vars.get(name).map(String::as_str).unwrap_or("")),
            None => seg.to_string(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn encode(value: &str) -> String {
    value.bytes().map(|b| if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// `2025-03-10` o `2025-03-10T08:00:00Z` → `Mon, 10 Mar 2025 00:00:00 GMT`.
fn rfc822(date: &str) -> Option<String> {
    let day = date.get(..10)?;
    let mut p = day.split('-');
    let (y, m, d): (i64, i64, i64) = (p.next()?.parse().ok()?, p.next()?.parse().ok()?, p.next()?.parse().ok()?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || y < 1970 {
        return None;
    }
    let time = date.get(11..19).filter(|t| t.len() == 8 && t.as_bytes()[2] == b':').unwrap_or("00:00:00");
    // día de la semana (Sakamoto)
    const T: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let wd = (yy + yy / 4 - yy / 100 + yy / 400 + T[(m - 1) as usize] + d) % 7;
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    Some(format!("{}, {d:02} {} {y} {time} GMT", DAYS[wd as usize], MONTHS[(m - 1) as usize]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn feed() -> FeedSection {
        FeedSection { title: "Mi <blog>".into(), description: None, source: "content:/posts".into(), link: "/blog/:slug".into(), output: None, limit: None }
    }

    #[test]
    fn renders_valid_rss_with_absolute_escaped_links() {
        let items = vec![
            json!({ "title": "Hola & adiós", "slug": "hola", "date": "2025-03-10", "summary": "<b>x</b>" }),
            json!({ "slug": "sin-titulo" }),
            json!({ "title": "Con espacio", "slug": "a b" }),
        ];
        let xml = render(&feed(), &items, Some("https://ejemplo.com/"));
        assert!(xml.contains("<title>Mi &lt;blog&gt;</title>"));
        assert!(xml.contains("<link>https://ejemplo.com/blog/hola</link>"));
        assert!(xml.contains("<title>Hola &amp; adiós</title>"));
        assert!(xml.contains("<pubDate>Mon, 10 Mar 2025 00:00:00 GMT</pubDate>"));
        assert!(xml.contains("<description>&lt;b&gt;x&lt;/b&gt;</description>"), "el texto nunca es HTML");
        assert!(xml.contains("https://ejemplo.com/blog/a%20b"));
        assert!(!xml.contains("sin-titulo"), "una entrada sin título se omite");
    }

    #[test]
    fn weekdays_are_right() {
        assert_eq!(rfc822("2025-03-10").unwrap(), "Mon, 10 Mar 2025 00:00:00 GMT");
        assert_eq!(rfc822("2024-02-29T08:30:15Z").unwrap(), "Thu, 29 Feb 2024 08:30:15 GMT");
        assert_eq!(rfc822("2000-01-01").unwrap(), "Sat, 01 Jan 2000 00:00:00 GMT");
        assert!(rfc822("hoy").is_none() && rfc822("2025-13-01").is_none());
    }

    #[test]
    fn the_head_link_points_at_the_feed() {
        assert_eq!(head_link(&feed()), "<link rel=\"alternate\" type=\"application/rss+xml\" title=\"Mi &lt;blog&gt;\" href=\"/feed.xml\">");
    }
}
