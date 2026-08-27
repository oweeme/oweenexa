//! Prueba de extremo a extremo con un servidor HTTP real (no un mock):
//! `nexa-loader` debe distinguir 200/404/500 correctamente.

use std::collections::BTreeMap;
use std::thread;

use nexa_ast::Loader;
use nexa_loader::{load, LoaderError};
use tiny_http::{ListenAddr, Response, Server};

fn spawn_test_server(status: u16, body: &'static str) -> String {
    let server = Server::http("127.0.0.1:0").expect("failed to bind test server");
    let port = match server.server_addr() {
        ListenAddr::IP(addr) => addr.port(),
        _ => panic!("expected an IP listen address"),
    };

    thread::spawn(move || {
        if let Ok(request) = server.recv() {
            let response = Response::from_string(body).with_status_code(status);
            let _ = request.respond(response);
        }
    });

    format!("http://127.0.0.1:{port}")
}

fn slug_loader() -> Loader {
    Loader {
        url_template: "/products/:slug".to_string(),
    }
}

fn params_with_slug(slug: &str) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    params.insert("slug".to_string(), slug.to_string());
    params
}

#[test]
fn loads_and_parses_json_on_success() {
    let base = spawn_test_server(200, r#"{"name":"iPhone 17","price":999}"#);

    let value = load(&slug_loader(), &params_with_slug("iphone-17"), &base).expect("should load");

    assert_eq!(value["name"], "iPhone 17");
    assert_eq!(value["price"], 999);
}

#[test]
fn returns_not_found_on_http_404() {
    let base = spawn_test_server(404, "not found");

    let err = load(&slug_loader(), &params_with_slug("no-existe"), &base).unwrap_err();
    assert!(matches!(err, LoaderError::NotFound));
}

#[test]
fn returns_http_error_for_other_status_codes() {
    let base = spawn_test_server(500, "boom");

    let err = load(&slug_loader(), &params_with_slug("x"), &base).unwrap_err();
    assert!(matches!(err, LoaderError::Http(500)));
}

#[test]
fn missing_param_fails_before_touching_the_network() {
    let err = load(&slug_loader(), &BTreeMap::new(), "http://127.0.0.1:1").unwrap_err();
    assert!(matches!(err, LoaderError::MissingParam(name) if name == "slug"));
}
