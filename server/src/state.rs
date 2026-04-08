use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;

use crate::error::AppError;
use crate::fetcher::RedditClient;
use reddit_toxicity_core::ToxicityMetrics;

/// Shared application state, cheaply cloneable across request handlers.
#[derive(Clone)]
pub struct AppState {
    reddit: Arc<RedditClient>,
    cache: Cache<String, Arc<ToxicityMetrics>>,
}

impl AppState {
    pub fn new(reddit: RedditClient, cache_ttl_hours: u64) -> Self {
        Self {
            reddit: Arc::new(reddit),
            cache: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(cache_ttl_hours * 3600))
                .build(),
        }
    }

    /// Retrieve metrics for a subreddit, serving from cache when possible.
    pub async fn get_metrics(&self, subreddit: &str) -> Result<Arc<ToxicityMetrics>, AppError> {
        let key = subreddit.to_lowercase();

        if let Some(cached) = self.cache.get(&key).await {
            tracing::info!(subreddit = %key, "cache hit");
            return Ok(cached);
        }

        tracing::info!(subreddit = %key, "cache miss — fetching from Reddit");
        let metrics = Arc::new(self.reddit.fetch_toxicity(&key).await?);
        self.cache.insert(key, Arc::clone(&metrics)).await;
        Ok(metrics)
    }
}
