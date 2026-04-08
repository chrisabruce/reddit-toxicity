use serde::Deserialize;

pub const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";
pub const API_BASE: &str = "https://oauth.reddit.com";
pub const USER_AGENT: &str = "rust:reddit-toxicity-badge:v0.1 (by /u/toxicity-badge-bot)";

/// Reddit OAuth token response.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Build the API URL for a subreddit listing.
pub fn listing_url(subreddit: &str, sort: &str) -> String {
    format!(
        "{}/r/{}/{}.json?limit=100&raw_json=1",
        API_BASE, subreddit, sort
    )
}

/// Build the API URL for a post's comments.
pub fn comments_url(permalink: &str) -> String {
    format!("{}{}.json?limit=50&raw_json=1", API_BASE, permalink)
}
