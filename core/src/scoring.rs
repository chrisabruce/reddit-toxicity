use serde::Deserialize;

/// Calculated toxicity metrics for a subreddit.
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct ToxicityMetrics {
    pub subreddit: String,
    pub new_avg_upvote_ratio: f64,
    pub negative_comment_pct: f64,
    pub op_negative_pct: f64,
    pub score: f64,
}

/// Raw Reddit listing response structures.
#[derive(Debug, Deserialize)]
pub struct Listing {
    pub data: ListingData,
}

#[derive(Debug, Deserialize)]
pub struct ListingData {
    pub children: Vec<ListingChild>,
}

#[derive(Debug, Deserialize)]
pub struct ListingChild {
    pub data: serde_json::Value,
}

/// Comment sampling results.
pub struct CommentStats {
    pub negative_comment_pct: f64,
    pub op_negative_pct: f64,
}

/// Average upvote_ratio across posts that have at least 1 comment.
pub fn avg_upvote_ratio(posts: &[ListingChild]) -> f64 {
    let mut total = 0.0_f64;
    let mut count = 0_usize;

    for child in posts {
        let has_comments = child
            .data
            .get("num_comments")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0;
        if !has_comments {
            continue;
        }
        if let Some(ratio) = child.data.get("upvote_ratio").and_then(|v| v.as_f64()) {
            total += ratio;
            count += 1;
        }
    }

    if count > 0 {
        total / count as f64
    } else {
        0.9
    }
}

/// Analyze comment listings and compute negative comment stats.
///
/// `post_comments` is a list of (post_author, comment_listing_children) pairs.
/// Each platform fetches comments differently but passes the parsed results here.
pub fn analyze_comments(post_comments: &[(String, Vec<ListingChild>)]) -> CommentStats {
    let mut total_comments = 0_usize;
    let mut negative_comments = 0_usize;
    let mut op_total = 0_usize;
    let mut op_negative = 0_usize;

    for (post_author, comments) in post_comments {
        for comment in comments {
            let score = match comment.data.get("score").and_then(|v| v.as_i64()) {
                Some(s) => s,
                None => continue,
            };
            let author = comment
                .data
                .get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            total_comments += 1;
            if score < 2 {
                negative_comments += 1;
            }

            if !post_author.is_empty() && author == post_author {
                op_total += 1;
                if score < 2 {
                    op_negative += 1;
                }
            }
        }
    }

    CommentStats {
        negative_comment_pct: if total_comments > 0 {
            negative_comments as f64 / total_comments as f64
        } else {
            0.0
        },
        op_negative_pct: if op_total > 0 {
            op_negative as f64 / op_total as f64
        } else {
            0.0
        },
    }
}

/// Compute the final toxicity score from raw metrics.
pub fn compute_score(
    subreddit: &str,
    new_avg_ratio: f64,
    comment_stats: &CommentStats,
) -> ToxicityMetrics {
    // Remap upvote ratio from its natural range (0.60–0.95) to 0.0–1.0.
    let ratio_normalized = ((0.95 - new_avg_ratio) / 0.35).clamp(0.0, 1.0);

    // Weights: upvote ratio 55%, OP comment negativity 30%, general comments 15%
    let raw_score = (ratio_normalized * 55.0)
        + (comment_stats.op_negative_pct * 30.0)
        + (comment_stats.negative_comment_pct * 15.0);

    let score = raw_score.clamp(0.0, 100.0);

    ToxicityMetrics {
        subreddit: subreddit.to_string(),
        new_avg_upvote_ratio: new_avg_ratio,
        negative_comment_pct: comment_stats.negative_comment_pct,
        op_negative_pct: comment_stats.op_negative_pct,
        score,
    }
}

/// Filter posts that have at least 1 comment (useful for comment sampling).
pub fn posts_with_comments(posts: &[ListingChild]) -> Vec<&ListingChild> {
    posts
        .iter()
        .filter(|c| {
            c.data
                .get("num_comments")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        })
        .collect()
}
