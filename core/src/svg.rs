use crate::ToxicityMetrics;

const FONT_STACK: &str = "DejaVu Sans,-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif";

const COLOR_GREEN: &str = "#22c55e";
const COLOR_YELLOW: &str = "#eab308";
const COLOR_RED: &str = "#ef4444";

fn score_color(score: f64) -> &'static str {
    match score {
        s if s <= 35.0 => COLOR_GREEN,
        s if s <= 65.0 => COLOR_YELLOW,
        _ => COLOR_RED,
    }
}

fn score_label(score: f64) -> &'static str {
    match score {
        s if s <= 20.0 => "Very Low",
        s if s <= 35.0 => "Low",
        s if s <= 50.0 => "Moderate",
        s if s <= 65.0 => "High",
        _ => "Very High",
    }
}

/// Render a toxicity badge as an SVG string.
pub fn render_badge(metrics: &ToxicityMetrics, width: u32) -> String {
    let height = (width as f64 / 3.5).round() as u32;
    let accent = score_color(metrics.score);
    let label = score_label(metrics.score);
    let score = metrics.score.round() as u32;
    let sub = html_escape(&metrics.subreddit);

    let s = width as f64 / 420.0;
    let bar_w = (8.0 * s).round();
    let text_x = 20.0 * s;
    let fix_bar = (bar_w - 3.0).max(0.0);

    // Circle
    let cx = width as f64 * 0.82;
    let cy = height as f64 * 0.5;
    let cr = height as f64 * 0.28;
    let score_size = cr * 1.2;
    let score_y = cy + score_size * 0.35;

    // Text
    let sub_size = 24.0 * s;
    let label_size = 13.0 * s;
    let cat_size = 16.0 * s;
    let sub_y = height as f64 * 0.15 + sub_size * 0.85;
    let label_y = sub_y + sub_size * 0.9;
    let cat_y = label_y + label_size * 1.8;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" rx="6" fill="#1e1e24"/>
  <rect width="{bar_w}" height="{height}" rx="6" fill="{accent}"/>
  <rect x="3" width="{fix_bar}" height="{height}" fill="{accent}"/>
  <circle cx="{cx}" cy="{cy}" r="{cr}" fill="{accent}"/>
  <text x="{cx}" y="{score_y}" text-anchor="middle" fill="#fff" font-family="{FONT_STACK}" font-weight="700" font-size="{score_size}">{score}</text>
  <text x="{text_x}" y="{sub_y}" fill="#fff" font-family="{FONT_STACK}" font-weight="700" font-size="{sub_size}">r/{sub}</text>
  <text x="{text_x}" y="{label_y}" fill="#b4b4be" font-family="{FONT_STACK}" font-size="{label_size}">Toxicity Index</text>
  <text x="{text_x}" y="{cat_y}" fill="{accent}" font-family="{FONT_STACK}" font-weight="600" font-size="{cat_size}">{label}</text>
</svg>"##
    )
}

/// Render a gray error badge as an SVG string.
pub fn render_error_badge(message: &str, width: u32) -> String {
    let height = (width as f64 / 3.5).round() as u32;
    let s = width as f64 / 420.0;
    let bar_w = (8.0 * s).round();
    let fix_bar = (bar_w - 3.0).max(0.0);
    let text_x = 20.0 * s;

    let title_size = 22.0 * s;
    let msg_size = 14.0 * s;
    let title_y = height as f64 * 0.25 + title_size * 0.85;
    let msg_y = title_y + title_size * 1.3;
    let msg = html_escape(message);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" rx="6" fill="#323238"/>
  <rect width="{bar_w}" height="{height}" rx="6" fill="#78787e"/>
  <rect x="3" width="{fix_bar}" height="{height}" fill="#78787e"/>
  <text x="{text_x}" y="{title_y}" fill="#c8c8d2" font-family="{FONT_STACK}" font-weight="700" font-size="{title_size}">Toxicity Index</text>
  <text x="{text_x}" y="{msg_y}" fill="#78787e" font-family="{FONT_STACK}" font-size="{msg_size}">{msg}</text>
</svg>"##
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
