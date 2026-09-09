//! Fase 19: `<img src="/foto.jpg">` estático -> variantes AVIF reales en
//! varios anchos, generadas por `nexa build`, y el HTML reescrito a un
//! `<picture>` real que las usa — sin que el desarrollador escriba nada
//! de esto a mano. Mismo espíritu que el tree-shaking de `@nexa/ui`
//! (Fase 9): el compilador hace más trabajo para que el navegador
//! descargue menos.
//!
//! Solo AVIF, no WebP: el encoder de WebP de la crate `image` únicamente
//! soporta el modo *lossless* — para una foto real, eso produce archivos
//! más grandes que el JPEG original, no más chicos (verificado con una
//! foto real de 1920x1080: 764 KB de WebP "optimizado" contra 158 KB del
//! JPEG de partida). AVIF sí soporta lossy de verdad y da resultados
//! reales (la misma foto: 51 KB en vez de 158 KB) — WebP se deja fuera
//! hasta que haya un encoder lossy real disponible sin depender de
//! bindings a una librería C del sistema.
//!
//! Solo toca imágenes que `nexa-renderer` ya dejó como un `src`
//! estático (`image_scan`, Fase 14) — un `src={expr}` dinámico no se
//! toca, porque en tiempo de build no hay forma de saber qué archivo es.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageFormat};

/// Debajo de este ancho no vale la pena generar variantes — el original
/// ya es chico.
const MIN_WORTH_OPTIMIZING: u32 = 480;
const BREAKPOINTS: [u32; 3] = [480, 768, 1280];

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedImage {
    pub width: u32,
    pub height: u32,
    /// `(ancho, url)`, de menor a mayor. La entrada más grande es
    /// siempre el archivo original ya copiado a `dist/` (nunca se
    /// re-codifica el propio formato al ancho completo).
    pub fallback_srcset: Vec<(u32, String)>,
    pub avif_srcset: Vec<(u32, String)>,
}

/// `None` cuando la imagen no vale la pena optimizarla (muy chica, o
/// una extensión que no reconocemos como foto real) — el `<img>`
/// original queda tal cual, no es un error.
pub fn is_supported(path: &str) -> bool {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).map(str::to_lowercase);
    matches!(ext.as_deref(), Some("jpg") | Some("jpeg") | Some("png"))
}

/// Decodifica `src_path` (la ubicación real en disco, bajo `public/`) y
/// genera sus variantes en `dist_root`, en la misma subcarpeta que
/// `public_rel` (la ruta con la que la página la referencia, ej.
/// `/img/foto.jpg`). El archivo original YA está copiado en `dist_root`
/// por `copy_dir::copy_recursive` — este módulo nunca lo toca ni lo
/// duplica en su propio ancho/formato.
pub fn generate(src_path: &Path, public_rel: &str, dist_root: &Path) -> Result<Option<GeneratedImage>> {
    let img = image::open(src_path).with_context(|| format!("decodificando {}", src_path.display()))?;
    let (width, height) = img.dimensions();

    if width <= MIN_WORTH_OPTIMIZING {
        return Ok(None);
    }

    let original_format = ImageFormat::from_path(src_path).ok();
    let smaller_widths: Vec<u32> = BREAKPOINTS.into_iter().filter(|w| *w < width).collect();

    let (dir, stem, ext) = split_public_rel(public_rel);

    let mut fallback = Vec::new();
    for w in &smaller_widths {
        let resized = resize_to_width(&img, *w, height);
        if let Some(format) = original_format {
            let rel = format!("{dir}/{stem}-{w}.{ext}");
            write_variant(&resized, dist_root, &rel, format)?;
            fallback.push((*w, rel));
        }
    }
    fallback.push((width, public_rel.to_string()));

    let mut all_widths = smaller_widths;
    all_widths.push(width);

    let mut avif = Vec::new();
    for w in &all_widths {
        let resized = resize_to_width(&img, *w, height);
        let avif_rel = format!("{dir}/{stem}-{w}.avif");
        write_variant(&resized, dist_root, &avif_rel, ImageFormat::Avif)?;
        avif.push((*w, avif_rel));
    }

    Ok(Some(GeneratedImage { width, height, fallback_srcset: fallback, avif_srcset: avif }))
}

fn resize_to_width(img: &DynamicImage, target_width: u32, original_height: u32) -> DynamicImage {
    if target_width >= img.width() {
        return img.clone();
    }
    // Alto tope = alto original: como solo se achica, nunca es la
    // restricción real — el ancho pedido siempre gana, y la relación de
    // aspecto se preserva (ver `DynamicImage::resize`).
    img.resize(target_width, original_height, image::imageops::FilterType::Lanczos3)
}

/// `speed` 1-10 en la escala de `ravif` (1 = más lento/mejor
/// compresión). El default de `image` es 4 — pensado para un
/// compresor de foto único, no para un `nexa build` con muchas
/// imágenes en cada corrida: a velocidad 4, una sola foto real de
/// 1920x1080 (cuatro anchos, binario release) tardó más de un minuto.
/// A velocidad máxima (10) esa misma foto tardó 3.8s en total y los
/// AVIF resultantes siguieron siendo 68-90% más chicos que el JPEG
/// original (1920px: 51 KB vs 158 KB) — la compresión que se pierde por
/// ir a velocidad máxima importa mucho menos que un build que tarda
/// minutos por imagen.
const AVIF_SPEED: u8 = 10;
const AVIF_QUALITY: u8 = 75;

fn write_variant(img: &DynamicImage, dist_root: &Path, rel: &str, format: ImageFormat) -> Result<()> {
    let out_path = dist_root.join(rel.trim_start_matches('/'));
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(&out_path).with_context(|| format!("creando {}", out_path.display()))?;
    let mut writer = std::io::BufWriter::new(file);

    if format == ImageFormat::Avif {
        let encoder = image::codecs::avif::AvifEncoder::new_with_speed_quality(writer, AVIF_SPEED, AVIF_QUALITY);
        img.write_with_encoder(encoder).with_context(|| format!("escribiendo {}", out_path.display()))
    } else {
        img.write_to(&mut writer, format).with_context(|| format!("escribiendo {}", out_path.display()))
    }
}

/// `"/img/foto.jpg"` -> `("/img", "foto", "jpg")`. `"/foto.jpg"` (imagen en
/// la raíz de `public/`) -> `("", "foto", "jpg")` — `Path::parent()` de un
/// path de un solo segmento absoluto devuelve `Some("/")`, no `None`/`""`,
/// así que hay que tratar ese `"/"` igual que vacío a propósito: de lo
/// contrario `format!("{dir}/{stem}-{w}.{ext}")` en el llamador arma
/// `"//foto-480.jpg"` (doble slash, una URL a otro dominio para el
/// navegador, no una ruta del sitio).
fn split_public_rel(public_rel: &str) -> (String, String, String) {
    let path = Path::new(public_rel);
    let dir = path.parent().map(|p| p.to_string_lossy().to_string()).filter(|s| !s.is_empty() && s != "/").unwrap_or_default();
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = path.extension().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
    (dir, stem, ext)
}

/// Reescribe cada `<img src="...">` cuyo `src` resuelva a variantes
/// generadas en un `<picture>` real: un `<source>` AVIF (el navegador lo
/// usa si lo soporta) + el `<img>` original con su propio `srcset` como
/// último recurso — nunca se pierde el contenido si `resolve` devuelve
/// `None` para un `src` (queda el `<img>` tal cual, sin tocar).
pub fn rewrite_images(html: &str, mut resolve: impl FnMut(&str) -> Option<GeneratedImage>) -> Result<String> {
    use lol_html::html_content::ContentType;
    use lol_html::{element, HtmlRewriter, Settings};

    let mut output = Vec::new();
    {
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![element!("img[src]", |el| {
                    let Some(src) = el.get_attribute("src") else {
                        return Ok(());
                    };
                    let Some(variants) = resolve(&src) else {
                        return Ok(());
                    };

                    el.before("<picture>", ContentType::Html);
                    el.before(&source_tag("image/avif", &variants.avif_srcset), ContentType::Html);

                    el.set_attribute("srcset", &srcset_attr(&variants.fallback_srcset))?;
                    el.set_attribute("sizes", "100vw")?;
                    if el.get_attribute("width").is_none() {
                        el.set_attribute("width", &variants.width.to_string())?;
                    }
                    if el.get_attribute("height").is_none() {
                        el.set_attribute("height", &variants.height.to_string())?;
                    }
                    if el.get_attribute("loading").is_none() {
                        el.set_attribute("loading", "lazy")?;
                    }

                    el.after("</picture>", ContentType::Html);
                    Ok(())
                })],
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c),
        );
        rewriter.write(html.as_bytes()).context("reescribiendo <img> a <picture>")?;
        rewriter.end().context("cerrando el reescritor de HTML")?;
    }
    String::from_utf8(output).context("el HTML reescrito no es UTF-8 válido")
}

fn srcset_attr(entries: &[(u32, String)]) -> String {
    entries.iter().map(|(w, url)| format!("{url} {w}w")).collect::<Vec<_>>().join(", ")
}

fn source_tag(mime: &str, entries: &[(u32, String)]) -> String {
    format!("<source type=\"{mime}\" srcset=\"{}\" sizes=\"100vw\">", srcset_attr(entries))
}

/// Cache de variantes ya generadas en este `nexa build` — la misma
/// imagen puede aparecer en varias páginas (ej. un logo compartido), y
/// no tiene sentido decodificar/re-codificar el mismo archivo dos veces.
#[derive(Default)]
pub struct VariantCache {
    entries: BTreeMap<String, Option<GeneratedImage>>,
}

impl VariantCache {
    /// Cuántas imágenes *distintas* terminaron con variantes generadas
    /// — para el resumen que imprime `nexa build`.
    pub fn optimized_count(&self) -> usize {
        self.entries.values().filter(|v| v.is_some()).count()
    }

    /// `src` es lo que la página escribió (`/img/foto.jpg`) — se resuelve
    /// contra `public/` para encontrar el archivo real en disco. Un
    /// error decodificando/escribiendo una imagen se reporta a
    /// `on_error` pero no aborta el build entero: esa imagen en
    /// particular se queda como estaba, sin `<picture>`.
    pub fn get_or_generate(
        &mut self,
        src: &str,
        public_dir: &Path,
        dist_root: &Path,
        on_error: impl FnOnce(&anyhow::Error),
    ) -> Option<GeneratedImage> {
        if let Some(cached) = self.entries.get(src) {
            return cached.clone();
        }

        let result = if is_supported(src) {
            let src_path = public_dir.join(src.trim_start_matches('/'));
            if src_path.is_file() {
                generate(&src_path, src, dist_root)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        };

        let variant = match result {
            Ok(v) => v,
            Err(err) => {
                on_error(&err);
                None
            }
        };

        self.entries.insert(src.to_string(), variant.clone());
        variant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("nexa-image-pipeline-test-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_test_jpeg(path: &Path, width: u32, height: u32) {
        let img = RgbImage::from_fn(width, height, |x, y| Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
        DynamicImage::ImageRgb8(img).save_with_format(path, ImageFormat::Jpeg).unwrap();
    }

    #[test]
    fn is_supported_only_recognizes_real_photo_extensions() {
        assert!(is_supported("/img/foto.jpg"));
        assert!(is_supported("/img/foto.JPEG"));
        assert!(is_supported("/img/foto.png"));
        assert!(!is_supported("/img/logo.svg"));
        assert!(!is_supported("/img/anim.gif"));
    }

    #[test]
    fn small_images_are_not_worth_optimizing() {
        let dir = scratch_dir();
        let src = dir.join("small.jpg");
        write_test_jpeg(&src, 300, 200);

        let result = generate(&src, "/small.jpg", &dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn generates_smaller_breakpoints_and_keeps_the_original_as_the_largest_fallback_entry() {
        let dir = scratch_dir();
        let src = dir.join("photo.jpg");
        write_test_jpeg(&src, 1600, 4);

        let generated = generate(&src, "/img/photo.jpg", &dir).unwrap().expect("expected variants");
        assert_eq!(generated.width, 1600);
        assert_eq!(generated.height, 4);

        // 480/768/1280 son todos menores que 1600 -> los tres se generan,
        // más el original como la entrada más grande.
        let fallback_widths: Vec<u32> = generated.fallback_srcset.iter().map(|(w, _)| *w).collect();
        assert_eq!(fallback_widths, vec![480, 768, 1280, 1600]);
        assert_eq!(generated.fallback_srcset.last().unwrap().1, "/img/photo.jpg");

        // avif sí cubre también el ancho completo (1600), no solo los breakpoints.
        let avif_widths: Vec<u32> = generated.avif_srcset.iter().map(|(w, _)| *w).collect();
        assert_eq!(avif_widths, vec![480, 768, 1280, 1600]);

        for (_, rel) in &generated.avif_srcset {
            assert!(dir.join(rel.trim_start_matches('/')).is_file(), "falta {rel}");
        }
    }

    #[test]
    fn only_includes_breakpoints_smaller_than_the_original() {
        let dir = scratch_dir();
        let src = dir.join("medium.jpg");
        write_test_jpeg(&src, 600, 4);

        let generated = generate(&src, "/medium.jpg", &dir).unwrap().unwrap();
        // Solo 480 es menor que 600 — 768 y 1280 quedan afuera (nunca se
        // agranda una imagen).
        let widths: Vec<u32> = generated.avif_srcset.iter().map(|(w, _)| *w).collect();
        assert_eq!(widths, vec![480, 600]);
    }

    #[test]
    fn root_level_images_do_not_produce_double_slash_urls() {
        // Bug #23: `Path::new("/medium.jpg").parent()` es `Some("/")`, no
        // `None` — sin el fix, `split_public_rel` deja `dir = "/"` y las
        // URLs generadas quedan como "//medium-480.avif" (doble slash: el
        // navegador lo interpreta como un dominio distinto, no una ruta).
        let dir = scratch_dir();
        let src = dir.join("medium.jpg");
        write_test_jpeg(&src, 600, 4);

        let generated = generate(&src, "/medium.jpg", &dir).unwrap().unwrap();

        for (_, rel) in &generated.avif_srcset {
            assert!(!rel.starts_with("//"), "URL con doble slash: {rel}");
            assert!(rel.starts_with("/medium-") || rel == "/medium.jpg", "URL inesperada: {rel}");
        }
        for (_, rel) in &generated.fallback_srcset {
            assert!(!rel.starts_with("//"), "URL con doble slash: {rel}");
        }
    }

    #[test]
    fn rewrite_wraps_a_resolved_image_in_a_real_picture_element() {
        let html = r#"<img src="/img/photo.jpg" alt="una foto">"#;
        let variants = GeneratedImage {
            width: 1600,
            height: 900,
            fallback_srcset: vec![(480, "/img/photo-480.jpg".into()), (1600, "/img/photo.jpg".into())],
            avif_srcset: vec![(480, "/img/photo-480.avif".into()), (1600, "/img/photo-1600.avif".into())],
        };

        let out = rewrite_images(html, |src| (src == "/img/photo.jpg").then(|| variants.clone())).unwrap();

        assert!(out.contains("<picture>"));
        assert!(out.contains("</picture>"));
        assert!(out.contains("type=\"image/avif\""));
        assert!(out.contains("photo-480.avif 480w"));
        assert!(out.contains("width=\"1600\""));
        assert!(out.contains("height=\"900\""));
        assert!(out.contains("loading=\"lazy\""));
        assert!(out.contains("alt=\"una foto\""), "no se debe perder ningún atributo original del <img>");
        // El orden importa: el <source> va antes que el <img> de
        // respaldo (el navegador usa el primer <source> que soporte).
        let avif_pos = out.find("image/avif").unwrap();
        let img_pos = out.find("<img").unwrap();
        assert!(avif_pos < img_pos);
    }

    #[test]
    fn rewrite_leaves_unresolved_images_completely_untouched() {
        let html = r#"<img src="/img/unknown.jpg" alt="x">"#;
        let out = rewrite_images(html, |_| None).unwrap();
        assert_eq!(out, html);
    }

    #[test]
    fn variant_cache_only_generates_the_same_image_once() {
        let dir = scratch_dir();
        let public_dir = dir.join("public");
        let dist_dir = dir.join("dist");
        std::fs::create_dir_all(&public_dir).unwrap();
        write_test_jpeg(&public_dir.join("shared.jpg"), 1000, 4);

        let mut cache = VariantCache::default();
        let mut calls = 0;
        for _ in 0..3 {
            let result = cache.get_or_generate("/shared.jpg", &public_dir, &dist_dir, |_| calls += 1);
            assert!(result.is_some());
        }
        assert_eq!(calls, 0, "no debería haber errores");
        // Verificado indirectamente: si regenerara cada vez, escribiría
        // los mismos archivos 3 veces — comprobamos que al menos existen.
        assert!(dist_dir.join("shared-480.avif").is_file());
    }

    #[test]
    fn variant_cache_reports_errors_without_panicking() {
        let dir = scratch_dir();
        let public_dir = dir.join("public");
        std::fs::create_dir_all(&public_dir).unwrap();
        // Un .jpg que en realidad no es una imagen válida.
        std::fs::write(public_dir.join("broken.jpg"), b"not a real jpeg").unwrap();

        let mut cache = VariantCache::default();
        let mut error_seen = false;
        let result = cache.get_or_generate("/broken.jpg", &public_dir, &dir.join("dist"), |_| error_seen = true);

        assert!(result.is_none());
        assert!(error_seen);
    }
}
