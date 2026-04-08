use serde::Deserialize;

pub const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";
pub const API_BASE: &str = "https://oauth.reddit.com";
pub const PUBLIC_BASE: &str = "https://www.reddit.com";
pub const BOT_USER_AGENT: &str = "rust:reddit-toxicity-badge:v0.1 (by /u/toxicity-badge-bot)";

/// Reddit OAuth token response.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Build a subreddit listing URL for the OAuth API.
pub fn listing_url(subreddit: &str, sort: &str) -> String {
    format!("{API_BASE}/r/{subreddit}/{sort}.json?limit=100&raw_json=1")
}

/// Build a comment thread URL for the OAuth API.
pub fn comments_url(permalink: &str) -> String {
    format!("{API_BASE}{permalink}.json?limit=50&raw_json=1")
}
