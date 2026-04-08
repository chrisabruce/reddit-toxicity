mod about;
mod error;
mod fetcher;
mod rasterize;
mod routes;
mod state;

use axum::Router;
use std::net::SocketAddr;
use tower::limit::ConcurrencyLimitLayer;
use tracing_subscriber::EnvFilter;

/// Server configuration, loaded from environment variables / `.env`.
struct Config {
    host: String,
    port: u16,
    cache_ttl_hours: u64,
    client_id: Option<String>,
    client_secret: Option<String>,
    proxy_url: Option<String>,
}

impl Config {
    fn from_env() -> Self {
        Self {
            host: env_or("HOST", "0.0.0.0"),
            port: env_or("PORT", "3000").parse().expect("PORT must be a number"),
            cache_ttl_hours: env_or("CACHE_TTL_HOURS", "24")
                .parse()
                .expect("CACHE_TTL_HOURS must be a number"),
            client_id: std::env::var("REDDIT_CLIENT_ID").ok().filter(|s| !s.is_empty()),
            client_secret: std::env::var("REDDIT_CLIENT_SECRET").ok().filter(|s| !s.is_empty()),
            proxy_url: std::env::var("PROXY_URL").ok().filter(|s| !s.is_empty()),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env();
    let reddit = fetcher::RedditClient::new(config.client_id, config.client_secret, config.proxy_url);
    let state = state::AppState::new(reddit, config.cache_ttl_hours);

    let app = Router::new()
        .route("/", axum::routing::get(routes::about))
        .route("/about", axum::routing::get(routes::about))
        .route("/health", axum::routing::get(routes::health))
        .route("/toxicity/r/{subreddit}", axum::routing::get(routes::badge))
        .route("/toxicity/{subreddit}", axum::routing::get(routes::badge))
        .layer(ConcurrencyLimitLayer::new(20))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("invalid HOST:PORT");

    tracing::info!("listening on {addr} (cache TTL: {}h)", config.cache_ttl_hours);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
