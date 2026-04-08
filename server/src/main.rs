mod about;
mod fetcher;
mod rasterize;
mod routes;
mod state;

use axum::Router;
use fetcher::RedditClient;
use state::AppState;
use std::net::SocketAddr;
use tower::limit::ConcurrencyLimitLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Load .env file if present (silently ignored if missing)
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let cache_ttl_hours: u64 = std::env::var("CACHE_TTL_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24);

    let client_id = std::env::var("REDDIT_CLIENT_ID").ok();
    let client_secret = std::env::var("REDDIT_CLIENT_SECRET").ok();

    let reddit = RedditClient::new(client_id, client_secret);
    let state = AppState::new(reddit, cache_ttl_hours);

    let app = Router::new()
        .route("/", axum::routing::get(routes::about))
        .route("/about", axum::routing::get(routes::about))
        .route("/health", axum::routing::get(routes::health))
        .route(
            "/toxicity/r/{subreddit}",
            axum::routing::get(routes::badge_with_prefix),
        )
        .route(
            "/toxicity/{subreddit}",
            axum::routing::get(routes::badge_without_prefix),
        )
        .layer(ConcurrencyLimitLayer::new(20))
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse().expect("invalid HOST:PORT");
    tracing::info!("listening on {} (cache TTL: {}h)", addr, cache_ttl_hours);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
