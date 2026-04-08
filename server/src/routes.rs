use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::rasterize::{self, Format};
use crate::state::AppState;
use reddit_toxicity_core::svg;

#[derive(Deserialize)]
pub struct BadgeParams {
    size: Option<u32>,
}

/// Single handler for both `/toxicity/r/{sub}` and `/toxicity/{sub}`.
pub async fn badge(
    State(state): State<AppState>,
    Path(raw): Path<String>,
    Query(params): Query<BadgeParams>,
) -> Response {
    let (subreddit, format) = Format::strip(&raw);
    let width = params.size.unwrap_or(420).clamp(200, 650);

    match state.get_metrics(subreddit).await {
        Ok(metrics) => {
            let svg_str = svg::render_badge(&metrics, width);
            rasterize::into_response(&svg_str, &format)
        }
        Err(e) => e.into_badge_response(width, &format),
    }
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
