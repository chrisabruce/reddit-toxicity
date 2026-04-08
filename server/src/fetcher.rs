use reddit_toxicity_core::oauth;
use reddit_toxicity_core::scoring::{self, Listing, ListingChild, ToxicityMetrics};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

struct OAuthCreds {
    client_id: String,
    client_secret: String,
    token: RwLock<Option<CachedToken>>,
}

struct CachedToken {
    access_token: String,
    expires_at: std::time::Instant,
}

pub struct RedditClient {
    http: Client,
    /// None = public API (www.reddit.com), Some = OAuth API (oauth.reddit.com)
    oauth: Option<Arc<OAuthCreds>>,
}

impl RedditClient {
    pub fn new(client_id: Option<String>, client_secret: Option<String>) -> Self {
        let http = Client::builder()
            .user_agent(oauth::USER_AGENT)
            .build()
            .expect("failed to build HTTP client");

        let oauth = match (client_id, client_secret) {
            (Some(id), Some(secret)) if !id.is_empty() && !secret.is_empty() => {
                tracing::info!("using Reddit OAuth API (dedicated rate limit)");
                Some(Arc::new(OAuthCreds {
                    client_id: id,
                    client_secret: secret,
                    token: RwLock::new(None),
                }))
            }
            _ => {
                tracing::warn!(
                    "REDDIT_CLIENT_ID / REDDIT_CLIENT_SECRET not set — using public API (shared rate limit)"
                );
                None
            }
        };

        Self { http, oauth }
    }

    fn uses_oauth(&self) -> bool {
        self.oauth.is_some()
    }

    /// Get a valid OAuth access token, refreshing if expired.
    async fn get_token(&self) -> Result<String, String> {
        let creds = self.oauth.as_ref().ok_or("OAuth not configured")?;

        {
            let guard = creds.token.read().await;
            if let Some(ref cached) = *guard {
                if cached.expires_at > std::time::Instant::now() {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        let resp = self
            .http
            .post(oauth::TOKEN_URL)
            .basic_auth(&creds.client_id, Some(&creds.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e| format!("oauth token error: {}", e))?;

        let token_resp: oauth::TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("oauth token parse error: {}", e))?;

        let cached = CachedToken {
            access_token: token_resp.access_token.clone(),
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(token_resp.expires_in.saturating_sub(60)),
        };

        let mut guard = creds.token.write().await;
        *guard = Some(cached);

        tracing::info!("refreshed Reddit OAuth token");
        Ok(token_resp.access_token)
    }

    /// Build the listing URL based on whether OAuth is configured.
    fn listing_url(&self, subreddit: &str, sort: &str) -> String {
        if self.uses_oauth() {
            oauth::listing_url(subreddit, sort)
        } else {
            format!(
                "https://www.reddit.com/r/{}/{}.json?limit=100&raw_json=1",
                subreddit, sort
            )
        }
    }

    /// Build the comments URL based on whether OAuth is configured.
    fn comments_url(&self, permalink: &str) -> String {
        if self.uses_oauth() {
            oauth::comments_url(permalink)
        } else {
            format!(
                "https://www.reddit.com{}.json?limit=50&raw_json=1",
                permalink
            )
        }
    }

    /// Fetch and score a subreddit's toxicity.
    pub async fn fetch_toxicity(&self, subreddit: &str) -> Result<ToxicityMetrics, String> {
        let new_url = self.listing_url(subreddit, "new");
        let new_posts = self.fetch_listing(&new_url).await?;
        if new_posts.is_empty() {
            return Err(format!("subreddit not found: {}", subreddit));
        }

        let new_avg_ratio = scoring::avg_upvote_ratio(&new_posts);

        let commentable = scoring::posts_with_comments(&new_posts);
        let sample_count = commentable.len().min(8);
        let mut post_comments: Vec<(String, Vec<ListingChild>)> = Vec::new();

        for post in commentable.iter().take(sample_count) {
            let permalink = match post.data.get("permalink").and_then(|v| v.as_str()) {
                Some(p) => p,
                None => continue,
            };
            let post_author = post
                .data
                .get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let url = self.comments_url(permalink);

            let listings: Vec<Listing> = match self.fetch_json(&url).await {
                Ok(l) => l,
                Err(_) => continue,
            };

            if let Some(comment_listing) = listings.into_iter().nth(1) {
                post_comments.push((post_author, comment_listing.data.children));
            }

            let delay = if self.uses_oauth() { 100 } else { 200 };
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }

        let comment_stats = scoring::analyze_comments(&post_comments);

        tracing::debug!(
            subreddit,
            new_avg_ratio,
            op_negative_pct = comment_stats.op_negative_pct,
            negative_comment_pct = comment_stats.negative_comment_pct,
            "raw metrics"
        );

        Ok(scoring::compute_score(
            subreddit,
            new_avg_ratio,
            &comment_stats,
        ))
    }

    async fn fetch_listing(&self, url: &str) -> Result<Vec<ListingChild>, String> {
        let listing: Listing = self.fetch_json(url).await?;
        Ok(listing.data.children)
    }

    async fn fetch_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        let mut req = self.http.get(url);

        if self.uses_oauth() {
            let token = self.get_token().await?;
            req = req.bearer_auth(&token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("reddit API error: {}", e))?;

        let status = resp.status();

        if status == reqwest::StatusCode::NOT_FOUND
            || status == reqwest::StatusCode::FORBIDDEN
        {
            tracing::debug!(url, %status, "Reddit returned error");
            return Err(format!("subreddit not found ({})", status.as_u16()));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err("rate limited by Reddit".to_string());
        }

        if !status.is_success() {
            tracing::warn!(url, %status, "unexpected Reddit response");
            return Err(format!("Reddit returned HTTP {}", status.as_u16()));
        }

        resp.json()
            .await
            .map_err(|e| format!("json parse error: {}", e))
    }
}
