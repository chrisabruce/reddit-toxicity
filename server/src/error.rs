use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::rasterize;
use reddit_toxicity_core::svg;

/// Application-wide error type.
///
/// Every fallible operation in the server flows through this type,
/// giving us structured error handling with `?` throughout.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("subreddit not found: {0}")]
    SubredditNotFound(String),

    #[error("rate limited by Reddit")]
    RateLimited,

    #[error("Reddit returned HTTP {0}")]
    UnexpectedStatus(u16),

    #[error("{0}")]
    Reddit(#[from] reqwest::Error),

    #[error("OAuth token error: {0}")]
    OAuthToken(String),

    #[error("JSON parse error: {0}")]
    JsonParse(String),

    #[error("SVG render error: {0}")]
    Render(String),
}

impl AppError {
    /// Render this error as a badge image in the requested format.
    pub fn into_badge_response(self, width: u32, format: &rasterize::Format) -> Response {
        tracing::warn!(error = %self, "badge generation failed");
        let msg = self.to_string();

        // HTML format gets a plain text error — no badge to embed
        if matches!(format, rasterize::Format::Html) {
            return (axum::http::StatusCode::BAD_GATEWAY, msg).into_response();
        }

        let svg_str = svg::render_error_badge(&msg, width);
        rasterize::into_response(&svg_str, format)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::SubredditNotFound(_) => StatusCode::NOT_FOUND,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
