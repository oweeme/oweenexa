// Generado por `nexa add tauri` — el lado Rust de tu app de escritorio.
// `dist/` (lo que produce `nexa build`) se embebe en el ejecutable y se
// muestra en una ventana nativa. Tu código de página llama a las
// funciones de abajo con `platform.native.invoke("nombre", { args })`
// (`@nexa/platform`) — es un puente tipado JS <-> Rust, no una API web.
//
// Para agregar una función propia:
//   1. escríbela con `#[tauri::command]` (puede leer archivos, usar una
//      base de datos, llamar a una librería nativa...),
//   2. agrégala a `tauri::generate_handler![...]` más abajo,
//   3. llámala desde la página: `await platform.native.invoke("mi_comando", { ... })`.

use serde::Serialize;

/// Lo que devuelve `app_info` — espejo de `AppInfo` en `@nexa/platform`.
#[derive(Serialize)]
struct AppInfo {
    name: String,
    version: String,
    os: &'static str,
    arch: &'static str,
}

/// Comando de ejemplo: datos de la app y del sistema operativo en el que corre.
#[tauri::command]
fn app_info(app: tauri::AppHandle) -> AppInfo {
    let info = app.package_info();
    AppInfo {
        name: info.name.clone(),
        version: info.version.to_string(),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("error corriendo la aplicación de Tauri");
}
