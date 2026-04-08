use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use tokio::sync::RwLock;

use crate::error::AppError;
use reddit_toxicity_core::oauth;
use reddit_toxicity_core::scoring::{
    self, Child, CommentData, Listing, PostComments, PostData, ToxicityMetrics,
};

// ---------------------------------------------------------------------------
// OAuth credentials
// ---------------------------------------------------------------------------

struct OAuthCreds {
    client_id: String,
    client_secret: String,
    token: RwLock<Option<CachedToken>>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

// ---------------------------------------------------------------------------
// Reddit client
// ---------------------------------------------------------------------------

/// HTTP client for the Reddit API with optional OAuth authentication
/// and optional SOCKS5/HTTP proxy support.
pub struct RedditClient {
    http: Client,
    oauth: Option<Arc<OAuthCreds>>,
    base_url: &'static str,
    delay: Duration,
}

impl RedditClient {
    pub fn new(
        client_id: Option<String>,
        client_secret: Option<String>,
        proxy_url: Option<String>,
    ) -> Self {
        let mut builder = Client::builder().user_agent(oauth::BOT_USER_AGENT);

        if let Some(ref url) = proxy_url {
            let proxy = reqwest::Proxy::all(url).expect("invalid PROXY_URL");
            builder = builder.proxy(proxy);
            tracing::info!(%url, "routing Reddit requests through proxy");
        }

        let http = builder.build().expect("failed to build HTTP client");

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
                    "no Reddit OAuth credentials — using public API (shared rate limit)"
                );
                None
            }
        };

        let base_url = if oauth.is_some() {
            oauth::API_BASE
        } else {
            oauth::PUBLIC_BASE
        };
        let delay = if oauth.is_some() {
            Duration::from_millis(100)
        } else {
            Duration::from_millis(200)
        };

        Self {
            http,
            oauth,
            base_url,
            delay,
        }
    }

    // -----------------------------------------------------------------------
    // Token management (double-checked locking)
    // -----------------------------------------------------------------------

    async fn access_token(&self) -> Result<String, AppError> {
        let creds = self
            .oauth
            .as_ref()
            .expect("access_token called without OAuth");

        // Fast path — read lock.
        {
            let guard = creds.token.read().await;
            if let Some(ref t) = *guard {
                if t.expires_at > Instant::now() {
                    return Ok(t.access_token.clone());
                }
            }
        }

        // Slow path — write lock with double-check to avoid redundant refreshes.
        let mut guard = creds.token.write().await;
        if let Some(ref t) = *guard {
            if t.expires_at > Instant::now() {
                return Ok(t.access_token.clone());
            }
        }

        let resp = self
            .http
            .post(oauth::TOKEN_URL)
            .basic_auth(&creds.client_id, Some(&creds.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?;

        let token_resp: oauth::TokenResponse = resp
            .json()
            .await
            .map_err(|e| AppError::OAuthToken(e.to_string()))?;

        tracing::info!("refreshed Reddit OAuth token");

        *guard = Some(CachedToken {
            access_token: token_resp.access_token.clone(),
            expires_at: Instant::now()
                + Duration::from_secs(token_resp.expires_in.saturating_sub(60)),
        });

        Ok(token_resp.access_token)
    }

    // -----------------------------------------------------------------------
    // URL helpers
    // -----------------------------------------------------------------------

    fn listing_url(&self, subreddit: &str, sort: &str) -> String {
        format!(
            "{}/r/{}/{}.json?limit=100&raw_json=1",
            self.base_url, subreddit, sort
        )
    }

    fn comments_url(&self, permalink: &str) -> String {
        format!("{}{}.json?limit=50&raw_json=1", self.base_url, permalink)
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Fetch Reddit data for a subreddit and compute its toxicity score.
    pub async fn fetch_toxicity(&self, subreddit: &str) -> Result<ToxicityMetrics, AppError> {
        let posts = self.fetch_posts(subreddit).await?;
        if posts.is_empty() {
            return Err(AppError::SubredditNotFound(subreddit.to_owned()));
        }

        let avg_ratio = scoring::avg_upvote_ratio(&posts);
        let threads = self.sample_comment_threads(&posts).await;
        let stats = scoring::analyze_comments(&threads);

        tracing::debug!(
            subreddit,
            avg_ratio,
            op_negative_pct = stats.op_negative_pct,
            negative_comment_pct = stats.negative_comment_pct,
            "raw metrics"
        );

        Ok(scoring::compute_score(subreddit, avg_ratio, &stats))
    }

    // -----------------------------------------------------------------------
    // Internal fetching
    // -----------------------------------------------------------------------

    async fn fetch_posts(&self, subreddit: &str) -> Result<Vec<Child<PostData>>, AppError> {
        let url = self.listing_url(subreddit, "new");
        let listing: Listing<PostData> = self.get_json(&url).await?;
        Ok(listing.data.children)
    }

    /// Sample up to 8 comment threads from posts that have comments.
    async fn sample_comment_threads(&self, posts: &[Child<PostData>]) -> Vec<PostComments> {
        let mut threads = Vec::new();

        for post in scoring::posts_with_comments(posts).take(8) {
            let url = self.comments_url(&post.data.permalink);
            let result: Result<Vec<Listing<CommentData>>, _> = self.get_json(&url).await;

            if let Ok(mut listings) = result {
                if listings.len() > 1 {
                    threads.push(PostComments {
                        post_author: post.data.author.clone(),
                        comments: listings.swap_remove(1).data.children,
                    });
                }
            }

            tokio::time::sleep(self.delay).await;
        }

        threads
    }

    /// GET a URL, parse JSON, and handle Reddit-specific status codes.
    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, AppError> {
        let mut req = self.http.get(url);

        if self.oauth.is_some() {
            req = req.bearer_auth(self.access_token().await?);
        }

        let resp = req.send().await?;
        let status = resp.status();

        match status {
            s if s.is_success() => {}
            reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::FORBIDDEN => {
                tracing::debug!(url, %status, "Reddit rejected request");
                return Err(AppError::SubredditNotFound(url.to_owned()));
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(AppError::RateLimited),
            _ => {
                tracing::warn!(url, %status, "unexpected Reddit response");
                return Err(AppError::UnexpectedStatus(status.as_u16()));
            }
        }

        resp.json()
            .await
            .map_err(|e| AppError::JsonParse(e.to_string()))
    }
}
