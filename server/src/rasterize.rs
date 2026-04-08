use image::codecs::jpeg::JpegEncoder;
use image::ImageEncoder;
use resvg::usvg;

/// Supported output formats.
pub enum Format {
    Svg,
    Png,
    Jpeg,
}

impl Format {
    /// Detect format from file extension.
    pub fn from_extension(path: &str) -> (String, Self) {
        if let Some(name) = path.strip_suffix(".png") {
            (name.to_string(), Format::Png)
        } else if let Some(name) = path.strip_suffix(".jpg") {
            (name.to_string(), Format::Jpeg)
        } else if let Some(name) = path.strip_suffix(".jpeg") {
            (name.to_string(), Format::Jpeg)
        } else if let Some(name) = path.strip_suffix(".svg") {
            (name.to_string(), Format::Svg)
        } else {
            (path.to_string(), Format::Svg)
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Format::Svg => "image/svg+xml",
            Format::Png => "image/png",
            Format::Jpeg => "image/jpeg",
        }
    }
}

/// Parse SVG string into a usvg tree with the embedded font available.
fn parse_svg(svg: &str) -> Result<usvg::Tree, String> {
    let mut opts = usvg::Options::default();
    opts.font_family = "DejaVu Sans".to_string();
    opts.fontdb_mut()
        .load_font_data(include_bytes!("../fonts/DejaVuSans-Bold.ttf").to_vec());
    usvg::Tree::from_str(svg, &opts).map_err(|e| format!("SVG parse error: {}", e))
}

/// Render an SVG string to PNG bytes.
pub fn svg_to_png(svg: &str) -> Result<Vec<u8>, String> {
    let tree = parse_svg(svg)?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).ok_or("failed to create pixmap")?;

    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("PNG encode error: {}", e))
}

/// Render an SVG string to JPEG bytes.
pub fn svg_to_jpeg(svg: &str) -> Result<Vec<u8>, String> {
    let tree = parse_svg(svg)?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).ok_or("failed to create pixmap")?;

    // White background for JPEG (no alpha channel)
    pixmap.fill(resvg::tiny_skia::Color::WHITE);

    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Convert RGBA to RGB for JPEG
    let rgba = pixmap.data();
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for pixel in rgba.chunks(4) {
        rgb.push(pixel[0]);
        rgb.push(pixel[1]);
        rgb.push(pixel[2]);
    }

    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, 90);
    encoder
        .write_image(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| format!("JPEG encode error: {}", e))?;

    Ok(buf)
}
