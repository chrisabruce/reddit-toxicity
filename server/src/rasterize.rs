use std::sync::LazyLock;

use axum::http::header;
use axum::response::{IntoResponse, Response};
use image::codecs::jpeg::JpegEncoder;
use image::ImageEncoder;
use resvg::usvg;

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Font database — built once, reused for every render
// ---------------------------------------------------------------------------

static FONT_DATA: &[u8] = include_bytes!("../fonts/DejaVuSans-Bold.ttf");

fn make_usvg_options() -> usvg::Options<'static> {
    let mut opts = usvg::Options {
        font_family: "DejaVu Sans".to_string(),
        ..Default::default()
    };
    opts.fontdb_mut().load_font_data(FONT_DATA.to_vec());
    opts
}

static USVG_OPTIONS: LazyLock<usvg::Options<'static>> = LazyLock::new(make_usvg_options);

// ---------------------------------------------------------------------------
// Output format
// ---------------------------------------------------------------------------

/// Supported badge output formats, detected from the URL extension.
pub enum Format {
    Svg,
    Png,
    Jpeg,
    /// HTML page with Open Graph meta tags for social media link unfurling.
    Html,
}

impl Format {
    /// Parse the subreddit name and format from a path segment like `"rust.png"`.
    /// Returns `("rust", Format::Png)`. Defaults to SVG if no known extension.
    pub fn strip(path: &str) -> (&str, Self) {
        for (suffix, fmt) in [
            (".png", Format::Png),
            (".jpg", Format::Jpeg),
            (".jpeg", Format::Jpeg),
            (".svg", Format::Svg),
            (".html", Format::Html),
        ] {
            if let Some(name) = path.strip_suffix(suffix) {
                return (name, fmt);
            }
        }
        (path, Format::Svg)
    }

    fn content_type(&self) -> &'static str {
        match self {
            Format::Svg => "image/svg+xml",
            Format::Png => "image/png",
            Format::Jpeg => "image/jpeg",
            Format::Html => "text/html; charset=utf-8",
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Convert an SVG string into an HTTP response in the requested format.
///
/// Note: `Format::Html` is handled separately in the route layer since it
/// needs access to the metrics and request URL for Open Graph tags.
pub fn into_response(svg_str: &str, format: &Format) -> Response {
    let body = match format {
        Format::Svg => return svg_response(svg_str),
        Format::Png => svg_to_png(svg_str),
        Format::Jpeg => svg_to_jpeg(svg_str),
        Format::Html => unreachable!("Html format handled in route layer"),
    };

    match body {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, format.content_type()),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            bytes,
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

fn svg_response(svg_str: &str) -> Response {
    (
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        svg_str.to_owned(),
    )
        .into_response()
}

/// Rasterize SVG to a `tiny_skia::Pixmap`.
fn render_pixmap(
    svg: &str,
    background: Option<resvg::tiny_skia::Color>,
) -> Result<resvg::tiny_skia::Pixmap, AppError> {
    let tree =
        usvg::Tree::from_str(svg, &USVG_OPTIONS).map_err(|e| AppError::Render(e.to_string()))?;

    let size = tree.size();
    let w = size.width().ceil() as u32;
    let h = size.height().ceil() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| AppError::Render("failed to create pixmap".into()))?;

    if let Some(bg) = background {
        pixmap.fill(bg);
    }

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

fn svg_to_png(svg: &str) -> Result<Vec<u8>, AppError> {
    let pixmap = render_pixmap(svg, None)?;
    pixmap
        .encode_png()
        .map_err(|e| AppError::Render(e.to_string()))
}

/// Render the badge centered on a 1200×630 dark canvas — the recommended
/// size for Twitter/OG social card images. Returns PNG bytes.
pub fn social_card_png(svg: &str) -> Result<Vec<u8>, AppError> {
    use resvg::tiny_skia::{Color, Pixmap, Transform};

    const CARD_W: u32 = 1200;
    const CARD_H: u32 = 630;

    let bg = Color::from_rgba8(17, 17, 22, 255); // #111116

    // Render the badge at its natural size
    let badge = render_pixmap(svg, None)?;

    // Create the card canvas
    let mut card = Pixmap::new(CARD_W, CARD_H)
        .ok_or_else(|| AppError::Render("failed to create card pixmap".into()))?;
    card.fill(bg);

    // Center the badge on the canvas
    let x = (CARD_W as i32 - badge.width() as i32) / 2;
    let y = (CARD_H as i32 - badge.height() as i32) / 2;

    card.draw_pixmap(
        x,
        y,
        badge.as_ref(),
        &resvg::tiny_skia::PixmapPaint::default(),
        Transform::default(),
        None,
    );

    card.encode_png()
        .map_err(|e| AppError::Render(e.to_string()))
}

fn svg_to_jpeg(svg: &str) -> Result<Vec<u8>, AppError> {
    let pixmap = render_pixmap(svg, Some(resvg::tiny_skia::Color::WHITE))?;

    let rgba = pixmap.data();
    let rgb: Vec<u8> = rgba
        .chunks_exact(4)
        .flat_map(|px| &px[..3])
        .copied()
        .collect();

    let w = pixmap.width();
    let h = pixmap.height();
    let mut buf = Vec::new();
    JpegEncoder::new_with_quality(&mut buf, 90)
        .write_image(&rgb, w, h, image::ExtendedColorType::Rgb8)
        .map_err(|e| AppError::Render(e.to_string()))?;

    Ok(buf)
}
