use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::rasterize::{self, Format};
use crate::state::AppState;
use reddit_toxicity_core::svg;
use reddit_toxicity_core::ToxicityMetrics;

#[derive(Deserialize)]
pub struct BadgeParams {
    size: Option<u32>,
}

/// Single handler for both `/toxicity/r/{sub}` and `/toxicity/{sub}`.
pub async fn badge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(raw): Path<String>,
    Query(params): Query<BadgeParams>,
) -> Response {
    let (subreddit, format) = Format::strip(&raw);
    let width = params.size.unwrap_or(420).clamp(200, 650);

    match state.get_metrics(subreddit).await {
        Ok(metrics) => match format {
            Format::Html => {
                let host = headers
                    .get(header::HOST)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("localhost:3000");
                social_card_response(host, subreddit, &metrics)
            }
            _ => {
                let svg_str = svg::render_badge(&metrics, width);
                rasterize::into_response(&svg_str, &format)
            }
        },
        Err(e) => e.into_badge_response(width, &format),
    }
}

/// HTML page with Open Graph and Twitter Card meta tags for social media unfurling.
///
/// When this URL is shared on Twitter, Slack, Discord, LinkedIn, etc., the platform
/// crawls it, reads the `og:image` tag, and displays the badge as a rich preview card.
fn social_card_response(host: &str, subreddit: &str, metrics: &ToxicityMetrics) -> Response {
    let score = metrics.score.round() as u32;
    let label = match metrics.score {
        s if s <= 20.0 => "Very Low",
        s if s <= 35.0 => "Low",
        s if s <= 50.0 => "Moderate",
        s if s <= 65.0 => "High",
        _ => "Very High",
    };

    // Point og:image at the PNG version — social platforms need raster images
    let image_url = format!("https://{host}/toxicity/r/{subreddit}.png?size=600");
    let page_url = format!("https://{host}/toxicity/r/{subreddit}.html");
    let reddit_url = format!("https://www.reddit.com/r/{subreddit}");

    let title = format!("r/{subreddit} — Toxicity Index: {score}/100");
    let description = format!(
        "Toxicity level: {label} ({score}/100). Scored from upvote ratios and comment negativity on new posts. No AI — purely vote-based.",
    );

    let html = format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>

  <!-- Open Graph (Facebook, LinkedIn, Slack, Discord) -->
  <meta property="og:type" content="website">
  <meta property="og:url" content="{page_url}">
  <meta property="og:title" content="{title}">
  <meta property="og:description" content="{description}">
  <meta property="og:image" content="{image_url}">
  <meta property="og:image:width" content="600">
  <meta property="og:image:height" content="171">

  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary_large_image">
  <meta name="twitter:title" content="{title}">
  <meta name="twitter:description" content="{description}">
  <meta name="twitter:image" content="{image_url}">

  <style>
    body {{
      background: #111116;
      color: #e4e4e8;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      margin: 0;
      padding: 1rem;
    }}
    img {{
      max-width: 100%;
      height: auto;
      margin-block-end: 1.5rem;
    }}
    a {{
      color: #22c55e;
      text-decoration: none;
      font-size: 1.1rem;
    }}
    a:hover {{
      text-decoration: underline;
    }}
    .meta {{
      color: #8b8b96;
      font-size: 0.85rem;
      margin-block-start: 1rem;
    }}
  </style>
</head>
<body>
  <img src="/toxicity/r/{subreddit}.svg?size=500" alt="{title}" width="500" height="143">
  <a href="{reddit_url}">Visit r/{subreddit} on Reddit</a>
  <p class="meta">{description}</p>
</body>
</html>"##
    );

    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        html,
    )
        .into_response()
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn about() -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        crate::about::page(),
    )
        .into_response()
}
