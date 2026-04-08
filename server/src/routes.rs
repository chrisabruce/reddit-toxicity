use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use reddit_toxicity_core::svg;
use serde::Deserialize;

use crate::rasterize::{self, Format};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct BadgeParams {
    pub size: Option<u32>,
}

fn clamp_size(size: Option<u32>) -> u32 {
    size.unwrap_or(420).clamp(200, 650)
}

pub async fn badge_with_prefix(
    State(state): State<AppState>,
    Path(subreddit): Path<String>,
    Query(params): Query<BadgeParams>,
) -> Response {
    serve_badge(state, &subreddit, params).await
}

pub async fn badge_without_prefix(
    State(state): State<AppState>,
    Path(subreddit): Path<String>,
    Query(params): Query<BadgeParams>,
) -> Response {
    serve_badge(state, &subreddit, params).await
}

async fn serve_badge(state: AppState, raw_subreddit: &str, params: BadgeParams) -> Response {
    let (subreddit, format) = Format::from_extension(raw_subreddit);
    let width = clamp_size(params.size);

    match state.get_metrics(&subreddit).await {
        Ok(metrics) => {
            let svg_str = svg::render_badge(&metrics, width);
            render_format(&svg_str, &format)
        }
        Err(e) => {
            tracing::warn!(%subreddit, error = %e, "failed to generate badge");
            let svg_str = svg::render_error_badge(&e, width);
            render_format(&svg_str, &format)
        }
    }
}

fn render_format(svg_str: &str, format: &Format) -> Response {
    let cache_header = (header::CACHE_CONTROL, "public, max-age=3600");

    match format {
        Format::Svg => (
            [(header::CONTENT_TYPE, format.content_type()), cache_header],
            svg_str.to_string(),
        )
            .into_response(),

        Format::Png => match rasterize::svg_to_png(svg_str) {
            Ok(bytes) => (
                [(header::CONTENT_TYPE, format.content_type()), cache_header],
                bytes,
            )
                .into_response(),
            Err(e) => {
                tracing::error!(error = %e, "PNG render failed");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
            }
        },

        Format::Jpeg => match rasterize::svg_to_jpeg(svg_str) {
            Ok(bytes) => (
                [(header::CONTENT_TYPE, format.content_type()), cache_header],
                bytes,
            )
                .into_response(),
            Err(e) => {
                tracing::error!(error = %e, "JPEG render failed");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
            }
        },
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
