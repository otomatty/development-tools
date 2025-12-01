//! Challenge-related types

use serde::{Deserialize, Serialize};

/// チャレンジ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeInfo {
    pub id: i64,
    pub user_id: i64,
    pub challenge_type: String,
    pub target_metric: String,
    pub target_value: i32,
    pub current_value: i32,
    pub reward_xp: i32,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub completed_at: Option<String>,
    // Computed fields
    pub progress_percent: f32,
    pub remaining_time_hours: i64,
    pub is_completed: bool,
    pub is_expired: bool,
}

impl ChallengeInfo {
    /// Get display name for challenge type
    pub fn challenge_type_label(&self) -> &'static str {
        match self.challenge_type.as_str() {
            "daily" => "デイリー",
            "weekly" => "ウィークリー",
            _ => "その他",
        }
    }

    /// Get display name for target metric
    pub fn target_metric_label(&self) -> &'static str {
        match self.target_metric.as_str() {
            "commits" => "コミット",
            "prs" => "PR",
            "reviews" => "レビュー",
            "issues" => "Issue",
            _ => "その他",
        }
    }

    /// Get icon for target metric
    pub fn target_metric_icon(&self) -> &'static str {
        match self.target_metric.as_str() {
            "commits" => "📝",
            "prs" => "🔀",
            "reviews" => "👀",
            "issues" => "🐛",
            _ => "🎯",
        }
    }

    /// Get status label
    pub fn status_label(&self) -> &'static str {
        match self.status.as_str() {
            "active" => "進行中",
            "completed" => "達成",
            "failed" => "失敗",
            _ => "不明",
        }
    }

    /// Format remaining time as human-readable string
    pub fn remaining_time_label(&self) -> String {
        if self.remaining_time_hours <= 0 {
            return "終了".to_string();
        }

        let hours = self.remaining_time_hours;
        if hours >= 24 {
            let days = hours / 24;
            format!("残り {}日", days)
        } else {
            format!("残り {}時間", hours)
        }
    }
}

/// チャレンジ作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChallengeRequest {
    pub challenge_type: String,
    pub target_metric: String,
    pub target_value: i32,
    pub reward_xp: Option<i32>,
}

/// チャレンジ統計
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeStats {
    pub total_completed: i32,
    pub consecutive_weekly_completions: i32,
    pub active_count: i32,
}

/// チャレンジタイプの選択肢
pub const CHALLENGE_TYPES: &[(&str, &str)] = &[("daily", "デイリー"), ("weekly", "ウィークリー")];

/// ターゲットメトリクスの選択肢
pub const TARGET_METRICS: &[(&str, &str, &str)] = &[
    ("commits", "コミット", "📝"),
    ("prs", "PR", "🔀"),
    ("reviews", "レビュー", "👀"),
    ("issues", "Issue", "🐛"),
];
