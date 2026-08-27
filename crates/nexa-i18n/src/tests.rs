use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use super::*;

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn locales_dir(files: &[(&str, &str)]) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("nexa-i18n-test-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    for (name, contents) in files {
        std::fs::write(dir.join(name), contents).unwrap();
    }

    dir
}

#[test]
fn loads_a_locale_dictionary() {
    let dir = locales_dir(&[("es.json", r#"{"home": {"title": "Bienvenido"}}"#)]);

    let value = load_locale(&dir, "es").unwrap().expect("expected a dictionary");
    assert_eq!(value["home"]["title"], "Bienvenido");
}

#[test]
fn missing_locale_file_is_not_an_error() {
    let dir = locales_dir(&[]);
    assert_eq!(load_locale(&dir, "es").unwrap(), None);
}

#[test]
fn available_locales_lists_every_json_file_sorted() {
    let dir = locales_dir(&[("en.json", "{}"), ("es.json", "{}"), ("readme.md", "no cuenta")]);

    assert_eq!(available_locales(&dir), vec!["en".to_string(), "es".to_string()]);
}

#[test]
fn missing_locales_dir_yields_no_locales() {
    let dir = std::env::temp_dir().join("nexa-i18n-test-does-not-exist");
    assert!(available_locales(&dir).is_empty());
}

#[test]
fn alternate_links_substitute_locale_and_keep_other_params() {
    let mut params = BTreeMap::new();
    params.insert("locale".to_string(), "es".to_string());
    params.insert("slug".to_string(), "iphone-17".to_string());

    let links = alternate_links(
        "/:locale/products/:slug",
        &params,
        &["es".to_string(), "en".to_string()],
    );

    assert_eq!(
        links,
        vec![
            AlternateLink { hreflang: "es".into(), href: "/es/products/iphone-17".into() },
            AlternateLink { hreflang: "en".into(), href: "/en/products/iphone-17".into() },
        ]
    );
}

#[test]
fn alternate_links_skips_a_locale_if_the_pattern_cannot_be_resolved() {
    // Un patrón que necesita un parámetro que no está disponible (aquí,
    // ninguno de los dos) no debe hacer fallar todo el cálculo.
    let links = alternate_links("/:locale/:missing", &BTreeMap::new(), &["es".to_string()]);
    assert!(links.is_empty());
}
