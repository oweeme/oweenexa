//! `[api] headers`: el token de servidor a servidor viaja en la petición a la API.
//! Archivo aparte (otro proceso de prueba): las cabeceras por defecto son globales.

use std::collections::BTreeMap;
use std::sync::mpsc;
use std::thread;

use nexa_ast::Loader;
use tiny_http::{ListenAddr, Response, Server};

#[test]
fn the_configured_headers_are_sent_to_the_api() {
    let server = Server::http("127.0.0.1:0").unwrap();
    #[allow(unreachable_patterns)]
    let port = match server.server_addr() {
        ListenAddr::IP(addr) => addr.port(),
        _ => panic!("expected an IP listen address"),
    };
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Ok(request) = server.recv() {
            let auth = request.headers().iter().find(|h| h.field.equiv("Authorization")).map(|h| h.value.to_string());
            tx.send(auth).unwrap();
            let _ = request.respond(Response::from_string("{}"));
        }
    });

    nexa_loader::set_default_headers(vec![("Authorization".into(), "Bearer s3cr3t".into())]);
    let loader = Loader { url_template: "/x".to_string() };
    nexa_loader::load(&loader, &BTreeMap::new(), &format!("http://127.0.0.1:{port}")).unwrap();
    assert_eq!(rx.recv().unwrap().as_deref(), Some("Bearer s3cr3t"));
}
