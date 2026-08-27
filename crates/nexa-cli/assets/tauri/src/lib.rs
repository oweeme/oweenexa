// Generado por `nexa add tauri` — esto es lo mínimo real para que
// `dist/` (lo que produce `nexa build`) se muestre dentro de una ventana
// de escritorio nativa. `@nexa/platform` (en el propio `dist/`, vía tu
// código de página) detecta `window.__TAURI__` para usar las APIs
// nativas cuando estén disponibles.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error corriendo la aplicación de Tauri");
}
