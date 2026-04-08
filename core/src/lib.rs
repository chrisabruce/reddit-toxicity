//! Platform-agnostic toxicity scoring and SVG badge rendering.
//!
//! This crate contains no HTTP client, async runtime, or caching logic.
//! Each platform (axum server, Cloudflare Worker) provides its own fetching
//! and passes raw JSON into the scoring functions here.

pub mod oauth;
pub mod scoring;
pub mod svg;

pub use scoring::ToxicityMetrics;
