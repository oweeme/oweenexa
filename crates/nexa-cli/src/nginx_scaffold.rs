//! `nexa add nginx` (Fase 20): genera un `deploy/nginx.conf` real para
//! servir `dist/` — mismo patrón que `tauri_scaffold`/`capacitor_scaffold`
//! (Fase 13): un archivo real, generado a partir de una plantilla, no
//! texto suelto pegado a mano.
//!
//! **Sobre el cacheo de `/assets/`:** los archivos que `nexa build` mismo
//! genera (`nexa-runtime.<hash>.js`, chunks de activación como
//! `Home-15.<hash>.js`, `nexa-ui.<hash>.css`) llevan un hash de su
//! contenido en el nombre desde Fase 23 — si el contenido cambia en un
//! redeploy, el nombre de archivo cambia con él, y el HTML que lo
//! referencia siempre apunta al nombre correcto. Para esos, `Cache-Control:
//! immutable` con `max-age` de un año es seguro de verdad: una URL vieja
//! nunca vuelve a existir con contenido distinto. Por eso el `location`
//! de abajo usa una regex que solo matchea ese patrón (`.<8 hex>.js`/
//! `.<8 hex>.css`) — cualquier otro archivo bajo `/assets/` (por ejemplo
//! algo copiado a mano desde `public/assets/`) NO lleva ese hash, así que
//! cae al `location /assets/` genérico con un `max-age` corto.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

/// `server_name`/rutas son genéricas a propósito — el usuario las ajusta
/// a su dominio real. `try_files` sirve cualquier ruta estática que
/// `nexa build` haya generado (incluidas las rutas dinámicas
/// pre-renderizadas por `paths`, Fase 17); una ruta dinámica que NO
/// declaró `paths` no existe como archivo y cae al `404` — para esas
/// hace falta `nexa preview` corriendo detrás (ver el bloque comentado
/// de `proxy_pass` al final).
const TEMPLATE: &str = r#"# Generado por `nexa add nginx` — revisá y ajustá `server_name` y `root`
# antes de usarlo en producción. Pensado para el `dist/` que produce
# `nexa build`: HTML/CSS/JS estáticos + lo que `[pwa]` (Fase 18) haya
# generado (manifest.webmanifest, sw.js) + imágenes AVIF (Fase 19).
server {
    listen 80;
    listen [::]:80;
    server_name _;                      # <- poné tu dominio real acá
    root /var/www/{{NAME}}/dist;
    index index.html;

    # `text/html` no va en la lista: nginx ya lo comprime siempre que
    # `gzip on` está activo — declararlo de nuevo en `gzip_types` es un
    # error real de sintaxis, no cosmético (`nginx -t` lo marca como
    # "duplicate MIME type" — encontrado validando este archivo con un
    # nginx real).
    gzip on;
    gzip_min_length 512;
    gzip_types text/css application/javascript text/javascript application/json
               image/svg+xml application/manifest+json application/xml;

    # Cabeceras de seguridad — las mismas que ya aplica `nexa preview`/
    # `nexa dev` en desarrollo, para que el comportamiento sea consistente.
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Los archivos con hash de contenido en el nombre (nexa-runtime.<hash>.js,
    # chunks de activación, nexa-ui.<hash>.css — ver el comentario del módulo
    # que genera este archivo) pueden cachearse "immutable" por un año: si el
    # contenido cambia, el nombre cambia con él.
    location ~* ^/assets/.+\.[0-9a-f]{8}\.(js|css)$ {
        add_header Cache-Control "public, max-age=31536000, immutable";
    }

    # Cualquier otro archivo bajo /assets/ (ej. algo copiado a mano desde
    # public/assets/) no lleva ese hash — cacheo corto, no "immutable".
    location /assets/ {
        add_header Cache-Control "public, max-age=3600";
    }

    location / {
        try_files $uri $uri/ $uri/index.html =404;
    }

    # Si tu proyecto tiene rutas dinámicas SIN `paths` declarado (Fase
    # 17), `try_files` de arriba nunca las va a encontrar como archivo
    # — necesitás un `nexa preview` corriendo detrás y reenviarle esas
    # rutas. Ejemplo (ajustá el puerto y las rutas):
    #
    # location /products/ {
    #     try_files $uri $uri/index.html @nexa_preview;
    # }
    # location @nexa_preview {
    #     proxy_pass http://127.0.0.1:4321;
    #     proxy_set_header Host $host;
    # }
}
"#;

pub fn render(project_name: &str) -> String {
    TEMPLATE.replace("{{NAME}}", &sanitize_name(project_name))
}

/// Genera `<project_root>/deploy/nginx.conf`. Falla si ya existe — igual
/// que `tauri_scaffold`/`capacitor_scaffold`, no se sobrescribe trabajo
/// del usuario en silencio.
pub fn scaffold(project_root: &Path, project_name: &str) -> Result<()> {
    let deploy_dir = project_root.join("deploy");
    let out_path = deploy_dir.join("nginx.conf");
    if out_path.exists() {
        bail!("{} ya existe — bórralo primero si quieres regenerarlo", out_path.display());
    }

    fs::create_dir_all(&deploy_dir)?;
    fs::write(&out_path, render(project_name)).with_context(|| format!("escribiendo {}", out_path.display()))
}

fn sanitize_name(name: &str) -> String {
    let cleaned: String =
        name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    if cleaned.is_empty() {
        "app".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-nginx-scaffold-test-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn render_substitutes_the_project_name_into_the_root_path() {
        let conf = render("mi-tienda");
        assert!(conf.contains("root /var/www/mi-tienda/dist;"));
        assert!(!conf.contains("{{NAME}}"), "no debe quedar ningún placeholder sin sustituir");
    }

    #[test]
    fn render_sanitizes_a_name_with_spaces_and_uppercase() {
        let conf = render("Mi Tienda Real");
        assert!(conf.contains("root /var/www/Mi-Tienda-Real/dist;"));
    }

    #[test]
    fn render_includes_the_same_security_headers_that_serve_rs_applies() {
        let conf = render("app");
        assert!(conf.contains(r#"add_header X-Content-Type-Options "nosniff" always;"#));
        assert!(conf.contains(r#"add_header X-Frame-Options "SAMEORIGIN" always;"#));
        assert!(conf.contains(r#"add_header Referrer-Policy "strict-origin-when-cross-origin" always;"#));
    }

    #[test]
    fn render_never_lists_text_html_in_gzip_types() {
        // Bug real, encontrado validando el archivo con un nginx real
        // (`nginx -t`): nginx ya comprime text/html siempre que `gzip
        // on` está activo — declararlo de nuevo en `gzip_types` dispara
        // "duplicate MIME type" como warning de sintaxis real.
        let conf = render("app");
        let gzip_line = conf.lines().find(|line| line.trim_start().starts_with("gzip_types")).unwrap();
        let continuation = conf.lines().skip_while(|l| !l.contains("gzip_types")).nth(1).unwrap_or("");
        assert!(!gzip_line.contains("text/html"));
        assert!(!continuation.contains("text/html"));
    }

    #[test]
    fn render_recommends_a_year_long_immutable_cache_only_for_hashed_assets() {
        // Fase 23: los assets que genera `nexa build` sí llevan un hash
        // de contenido en el nombre — "immutable" es seguro de verdad
        // para ESOS, pero no para cualquier cosa bajo /assets/ (ej. un
        // archivo copiado a mano en public/assets/, sin hash).
        let conf = render("app");

        let hashed_location = conf
            .lines()
            .find(|line| line.trim_start().starts_with("location ~"))
            .expect("se espera un location con regex para los assets con hash");
        assert!(hashed_location.contains(r"\.[0-9a-f]{8}\."));

        let hashed_cache_control = conf
            .lines()
            .skip_while(|l| !l.trim_start().starts_with("location ~"))
            .find(|line| line.contains("Cache-Control"))
            .expect("se espera un Cache-Control dentro del location con hash");
        assert!(hashed_cache_control.contains("immutable"));
        assert!(hashed_cache_control.contains("max-age=31536000"));

        let generic_cache_control = conf
            .lines()
            .skip_while(|l| l.trim() != "location /assets/ {")
            .find(|line| line.contains("Cache-Control"))
            .expect("se espera un Cache-Control dentro del location genérico");
        assert!(!generic_cache_control.contains("immutable"));
    }

    #[test]
    fn scaffold_writes_a_real_file_under_deploy() {
        let dir = scratch_dir();
        scaffold(&dir, "app").unwrap();
        assert!(dir.join("deploy/nginx.conf").is_file());
    }

    #[test]
    fn scaffold_refuses_to_overwrite_an_existing_config() {
        let dir = scratch_dir();
        scaffold(&dir, "app").unwrap();
        assert!(scaffold(&dir, "app").is_err());
    }
}
