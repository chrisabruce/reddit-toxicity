use crate::fetcher::RedditClient;
use moka::future::Cache;
use reddit_toxicity_core::ToxicityMetrics;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub reddit: Arc<RedditClient>,
    pub cache: Cache<String, Arc<ToxicityMetrics>>,
}

impl AppState {
    pub fn new(reddit: RedditClient, cache_ttl_hours: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(500)
            .time_to_live(Duration::from_secs(cache_ttl_hours * 3600))
            .build();
        Self {
            reddit: Arc::new(reddit),
            cache,
        }
    }

    pub async fn get_metrics(&self, subreddit: &str) -> Result<Arc<ToxicityMetrics>, String> {
        let key = subreddit.to_lowercase();

        if let Some(cached) = self.cache.get(&key).await {
            tracing::info!(subreddit = %key, "cache hit");
            return Ok(cached);
        }

        tracing::info!(subreddit = %key, "cache miss — fetching from Reddit");
        let metrics = self.reddit.fetch_toxicity(&key).await?;
        let metrics = Arc::new(metrics);
        self.cache.insert(key, metrics.clone()).await;
        Ok(metrics)
    }
}
