//! Content-Security-Policy sin `'unsafe-inline'` (Fase 75).
//!
//! Nexa pone scripts en línea (el import map, el arranque de la página). Una CSP estricta los
//! bloquearía; la solución estándar es permitir cada uno por su **hash SHA-256**. Con
//! `[security] csp = "default-src 'self'"` en `nexa.toml`, cada página lleva un
//! `<meta http-equiv="Content-Security-Policy">` con tu política más `script-src 'self'` y
//! el hash de CADA script en línea de ESA página. Los scripts de terceros inyectados (XSS) no
//! tienen hash, así que el navegador no los ejecuta.
//!
//! Una `<meta>` no puede llevar `frame-ancestors`, `report-uri` ni `sandbox`; esas directivas
//! van por cabecera (`[headers."/*"] Content-Security-Policy = "..."`).

const MARK: &str = "data-nexa-csp";

/// Política de `nexa.toml` (`[security] csp`), si hay.
pub fn policy() -> Option<String> {
    crate::manifest::load_lenient(std::path::Path::new("nexa.toml")).security.and_then(|s| s.csp).filter(|p| !p.trim().is_empty())
}

/// Añade (o rehace) la `<meta>` de CSP de `html` con los hashes de sus scripts en línea.
pub fn apply(html: &str, policy: &str) -> String {
    let html = remove_existing(html);
    let hashes: Vec<String> = inline_scripts(&html).iter().map(|body| format!("'sha256-{}'", base64(&sha256(body.as_bytes())))).collect();
    let content = merge(policy, &hashes);
    let meta = format!("<meta http-equiv=\"Content-Security-Policy\" {MARK} content=\"{}\">", content.replace('&', "&amp;").replace('"', "&quot;"));
    match html.find("<head>") {
        Some(at) => format!("{}{}\n{}", &html[..at + 6], meta, &html[at + 6..]),
        None => html,
    }
}

/// `apply` con la política del proyecto (no hace nada si no hay).
pub fn reapply(html: &str) -> String {
    match policy() {
        Some(p) => apply(html, &p),
        None => html.to_string(),
    }
}

fn remove_existing(html: &str) -> String {
    let Some(at) = html.find(&format!("<meta http-equiv=\"Content-Security-Policy\" {MARK}")) else { return html.to_string() };
    let end = html[at..].find('>').map(|e| at + e + 1).unwrap_or(at);
    let mut out = html[..at].to_string();
    out.push_str(html[end..].trim_start_matches('\n'));
    out
}

/// Una directiva `script-src` con `'self'` y los hashes; si la política ya trae una, se amplía.
fn merge(policy: &str, hashes: &[String]) -> String {
    let mut found = false;
    let mut directives: Vec<String> = policy
        .split(';')
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(|d| {
            if d.split_whitespace().next() == Some("script-src") {
                found = true;
                let mut parts = vec![d.to_string()];
                if !d.contains("'self'") {
                    parts.push("'self'".into());
                }
                parts.extend(hashes.iter().cloned());
                parts.join(" ")
            } else {
                d.to_string()
            }
        })
        .collect();
    if !found {
        directives.push(format!("script-src 'self' {}", hashes.join(" ")).trim_end().to_string());
    }
    directives.join("; ")
}

/// Texto de cada `<script>` en línea que el navegador ejecutaría (no los de datos: `application/json`…).
fn inline_scripts(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<script") {
        let after = &rest[start + 7..];
        // `<scripts>` u otra etiqueta que empieza igual
        if !after.starts_with(|c: char| c.is_whitespace() || c == '>') {
            rest = after;
            continue;
        }
        let Some(tag_end) = after.find('>') else { break };
        let attrs = &after[..tag_end];
        let body_start = tag_end + 1;
        let Some(close) = after[body_start..].find("</script") else { break };
        let body = &after[body_start..body_start + close];
        let executes = !attrs.contains("src=") && is_executable(attrs);
        if executes {
            out.push(body.to_string());
        }
        rest = &after[body_start + close..];
    }
    out
}

fn is_executable(attrs: &str) -> bool {
    match attr(attrs, "type") {
        None => true,
        Some(t) => matches!(t.as_str(), "" | "module" | "importmap" | "text/javascript" | "application/javascript"),
    }
}

fn attr(attrs: &str, name: &str) -> Option<String> {
    let key = format!("{name}=");
    let at = attrs.find(&key)?;
    let v = attrs[at + key.len()..].trim_start();
    let (quote, rest) = match v.chars().next()? {
        q @ ('"' | '\'') => (Some(q), &v[1..]),
        _ => (None, v),
    };
    let end = match quote {
        Some(q) => rest.find(q)?,
        None => rest.find(char::is_whitespace).unwrap_or(rest.len()),
    };
    Some(rest[..end].to_string())
}

// ------------------------------------------------------------------ SHA-256 y base64 (sin dependencias)

fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74,
        0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d,
        0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e,
        0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(v);
        }
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_the_known_vectors() {
        let hex = |b: [u8; 32]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
        assert_eq!(hex(sha256(b"")), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(hex(sha256(b"abc")), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        let long = "a".repeat(1000);
        assert_eq!(hex(sha256(long.as_bytes())), "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3");
    }

    #[test]
    fn base64_pads_like_the_standard() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
    }

    #[test]
    fn the_hash_of_an_inline_script_is_the_browsers() {
        // "alert(1)" → sha256-… (valor de referencia calculado con openssl)
        let html = "<html><head><title>x</title></head><body><script>alert(1)</script></body></html>";
        let out = apply(html, "default-src 'self'");
        assert!(out.contains("script-src 'self' 'sha256-bhHHL3z2vDgxUt0W3dWQOrprscmda2Y5pLsLg4GF+pI='"), "{out}");
        assert!(out.contains("default-src 'self';"));
    }

    #[test]
    fn external_and_data_scripts_get_no_hash_and_reapplying_does_not_duplicate() {
        let html = "<html><head></head><body><script src=\"/a.js\"></script><script type=\"application/json\" data-x>{\"a\":1}</script><script type=\"module\">import 'x'</script></body></html>";
        let once = apply(html, "default-src 'self'");
        assert_eq!(once.matches("'sha256-").count(), 1);
        let twice = apply(&once, "default-src 'self'");
        assert_eq!(twice.matches("Content-Security-Policy").count(), 1);
        assert_eq!(once, twice);
    }

    #[test]
    fn an_existing_script_src_is_extended_not_duplicated() {
        let out = apply("<html><head></head><body><script>1</script></body></html>", "script-src https://cdn.x.com; img-src 'self'");
        assert_eq!(out.matches("script-src").count(), 1);
        assert!(out.contains("script-src https://cdn.x.com 'self' 'sha256-"));
        assert!(out.contains("img-src 'self'"));
    }
}
