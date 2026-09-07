/// Hash corto (8 hex) de contenido, para cache-busting real: dos chunks
/// con el mismo código fuente producen el mismo nombre de archivo, y uno
/// con código distinto produce un nombre distinto. No es una firma
/// criptográfica — mismo propósito y algoritmo (FNV-1a) que ya usa
/// `nexa-cli` para el `content_hash` de `nexa.lock`; se duplica acá en
/// vez de crear una dependencia cruzada nueva entre crates por un
/// algoritmo de una decena de líneas.
pub(crate) fn short_hash(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (hash ^ (hash >> 32)) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_content_produces_the_same_hash() {
        assert_eq!(short_hash(b"hola"), short_hash(b"hola"));
    }

    #[test]
    fn different_content_produces_a_different_hash() {
        assert_ne!(short_hash(b"hola"), short_hash(b"chau"));
    }

    #[test]
    fn hash_is_eight_lowercase_hex_chars() {
        let hash = short_hash(b"cualquier cosa");
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
