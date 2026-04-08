use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Reddit API data models
// ---------------------------------------------------------------------------

/// Generic Reddit listing envelope — works for both posts and comments.
#[derive(Debug, Deserialize)]
pub struct Listing<T> {
    pub data: ListingData<T>,
}

#[derive(Debug, Deserialize)]
pub struct ListingData<T> {
    pub children: Vec<Child<T>>,
}

#[derive(Debug, Deserialize)]
pub struct Child<T> {
    pub data: T,
}

/// The fields we actually use from a Reddit post.
#[derive(Debug, Deserialize)]
pub struct PostData {
    #[serde(default)]
    pub num_comments: u64,
    #[serde(default = "default_ratio")]
    pub upvote_ratio: f64,
    #[serde(default)]
    pub permalink: String,
    #[serde(default)]
    pub author: String,
}

fn default_ratio() -> f64 {
    1.0
}

/// The fields we actually use from a Reddit comment.
#[derive(Debug, Deserialize)]
pub struct CommentData {
    pub score: Option<i64>,
    #[serde(default)]
    pub author: String,
}

// ---------------------------------------------------------------------------
// Toxicity scoring
// ---------------------------------------------------------------------------

/// Final toxicity result for a subreddit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToxicityMetrics {
    pub subreddit: String,
    pub score: f64,
    pub new_avg_upvote_ratio: f64,
    pub negative_comment_pct: f64,
    pub op_negative_pct: f64,
}

/// A post paired with its comment thread — input to [`analyze_comments`].
pub struct PostComments {
    pub post_author: String,
    pub comments: Vec<Child<CommentData>>,
}

/// Intermediate comment-analysis results.
pub struct CommentStats {
    pub negative_comment_pct: f64,
    pub op_negative_pct: f64,
}

/// Average `upvote_ratio` across posts that have at least one comment.
pub fn avg_upvote_ratio(posts: &[Child<PostData>]) -> f64 {
    let (sum, count) = posts
        .iter()
        .filter(|c| c.data.num_comments > 0)
        .fold((0.0, 0usize), |(sum, n), c| {
            (sum + c.data.upvote_ratio, n + 1)
        });

    if count > 0 { sum / count as f64 } else { 0.9 }
}

/// Iterate over posts that have at least one comment.
pub fn posts_with_comments(posts: &[Child<PostData>]) -> impl Iterator<Item = &Child<PostData>> {
    posts.iter().filter(|c| c.data.num_comments > 0)
}

/// Analyze comment threads and compute negativity statistics.
///
/// A comment is "negative" if its score is below 2. Reddit auto-upvotes
/// your own comment to 1, so `score < 2` means nobody else upvoted it.
pub fn analyze_comments(threads: &[PostComments]) -> CommentStats {
    let mut total = 0usize;
    let mut negative = 0usize;
    let mut op_total = 0usize;
    let mut op_negative = 0usize;

    for thread in threads {
        for comment in &thread.comments {
            let score = match comment.data.score {
                Some(s) => s,
                None => continue,
            };

            total += 1;
            if score < 2 {
                negative += 1;
            }

            if !thread.post_author.is_empty() && comment.data.author == thread.post_author {
                op_total += 1;
                if score < 2 {
                    op_negative += 1;
                }
            }
        }
    }

    CommentStats {
        negative_comment_pct: ratio(negative, total),
        op_negative_pct: ratio(op_negative, op_total),
    }
}

/// Compute the final toxicity score from raw metrics.
///
/// The upvote ratio is remapped from its natural range (0.60–0.95) to 0.0–1.0
/// before weighting. This stretches the narrow band Reddit ratios occupy into
/// the full score range.
pub fn compute_score(
    subreddit: &str,
    new_avg_ratio: f64,
    stats: &CommentStats,
) -> ToxicityMetrics {
    let ratio_normalized = ((0.95 - new_avg_ratio) / 0.35).clamp(0.0, 1.0);

    let raw = (ratio_normalized * 55.0)
        + (stats.op_negative_pct * 30.0)
        + (stats.negative_comment_pct * 15.0);

    ToxicityMetrics {
        subreddit: subreddit.to_owned(),
        score: raw.clamp(0.0, 100.0),
        new_avg_upvote_ratio: new_avg_ratio,
        negative_comment_pct: stats.negative_comment_pct,
        op_negative_pct: stats.op_negative_pct,
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator > 0 {
        numerator as f64 / denominator as f64
    } else {
        0.0
    }
}
