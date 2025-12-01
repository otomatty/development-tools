//! Gamification-related types (XP, badges, stats, etc.)
//!
//! Note: XP、レベル、カウント系の値は意味的にはu32（符号なし整数）が適切ですが、
//! SQLiteのINTEGER型が符号あり整数であり、sqlxがi32としてマッピングするため、
//! バックエンドとの整合性を保つためにi32で統一しています。
//! 実用上、これらの値が負になることはなく、21億を超えることもないため問題ありません。

use serde::{Deserialize, Serialize};

/// Generic cached response wrapper
///
/// Wraps any data type with cache metadata to indicate whether the data
/// came from a fresh API call or from local cache (e.g., when offline).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedResponse<T> {
    /// The actual data
    pub data: T,
    /// Whether the data was retrieved from cache
    pub from_cache: bool,
    /// When the data was cached (ISO8601 format)
    pub cached_at: Option<String>,
    /// When the cache expires (ISO8601 format)
    pub expires_at: Option<String>,
}

/// Cache statistics for display in settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    /// Total cache size in bytes
    pub total_size_bytes: u64,
    /// Number of cache entries
    pub entry_count: u64,
    /// Number of expired entries
    pub expired_count: u64,
    /// Last cleanup timestamp (ISO8601)
    pub last_cleanup_at: Option<String>,
}

/// ユーザー統計
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub id: i64,
    pub user_id: i64,
    pub total_xp: i32,
    pub current_level: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_activity_date: Option<String>,
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_reviews: i32,
    pub total_issues: i32,
    pub updated_at: String,
}

/// レベル情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfo {
    pub current_level: i32,
    pub total_xp: i32,
    pub xp_for_current_level: i32,
    pub xp_for_next_level: i32,
    pub xp_to_next_level: i32,
    pub progress_percent: f32,
}

/// GitHub統計
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GitHubStats {
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_prs_merged: i32,
    pub total_issues: i32,
    pub total_issues_closed: i32,
    pub total_reviews: i32,
    pub total_stars_received: i32,
    pub total_contributions: i32,
    pub contribution_calendar: Option<ContributionCalendar>,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub weekly_streak: i32,
    pub monthly_streak: i32,
    pub languages_count: i32,
}

/// コントリビューションカレンダー
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionCalendar {
    pub total_contributions: i32,
    pub weeks: Vec<ContributionWeek>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionWeek {
    pub contribution_days: Vec<ContributionDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionDay {
    pub contribution_count: i32,
    pub date: String,
    pub weekday: i32,
}

/// バッジ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: i64,
    pub user_id: i64,
    pub badge_type: String,
    pub badge_id: String,
    pub earned_at: String,
}

/// バッジ定義
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub badge_type: String,
    pub rarity: String,
    pub icon: String,
}

/// バッジ進捗情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeProgress {
    pub badge_id: String,
    pub current_value: i32,
    pub target_value: i32,
    pub progress_percent: f32,
}

/// 進捗情報付きバッジ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeWithProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub badge_type: String,
    pub rarity: String,
    pub icon: String,
    pub earned: bool,
    pub earned_at: Option<String>,
    pub progress: Option<BadgeProgress>,
}

/// XP履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XpHistoryEntry {
    pub id: i64,
    pub user_id: i64,
    pub action_type: String,
    pub xp_amount: i32,
    pub description: Option<String>,
    pub github_event_id: Option<String>,
    pub created_at: String,
}

/// XP獲得時のブレークダウン
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XpBreakdown {
    pub commits_xp: i32,
    pub prs_created_xp: i32,
    pub prs_merged_xp: i32,
    pub issues_created_xp: i32,
    pub issues_closed_xp: i32,
    pub reviews_xp: i32,
    pub stars_xp: i32,
    pub streak_bonus_xp: i32,
    pub total_xp: i32,
}

/// ストリークボーナス情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StreakBonusInfo {
    pub daily_bonus: i32,
    pub milestone_bonus: i32,
    pub total_bonus: i32,
    pub milestone_reached: Option<i32>,
    pub current_streak: i32,
    pub next_milestone_days: Option<i32>,
    pub days_to_next_milestone: Option<i32>,
}

/// XP獲得イベント
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XpGainedEvent {
    pub xp_gained: i32,
    pub total_xp: i32,
    pub old_level: i32,
    pub new_level: i32,
    pub level_up: bool,
    pub xp_breakdown: XpBreakdown,
    pub streak_bonus: StreakBonusInfo,
}

/// ストリークマイルストーン到達イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreakMilestoneEvent {
    pub milestone_days: i32,
    pub bonus_xp: i32,
    pub current_streak: i32,
}

/// GitHub統計同期結果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub user_stats: UserStats,
    pub xp_gained: i32,
    pub old_level: i32,
    pub new_level: i32,
    pub level_up: bool,
    pub xp_breakdown: XpBreakdown,
    pub streak_bonus: StreakBonusInfo,
    pub new_badges: Vec<NewBadgeInfo>,
    /// 前日比の統計差分（初回同期時はNone）
    pub stats_diff: Option<StatsDiffResult>,
}

/// 前日比の統計差分
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatsDiffResult {
    /// コミット数の差分
    pub commits_diff: i32,
    /// PR数の差分
    pub prs_diff: i32,
    /// レビュー数の差分
    pub reviews_diff: i32,
    /// Issue数の差分
    pub issues_diff: i32,
    /// スター獲得数の差分
    pub stars_diff: i32,
    /// コントリビューション数の差分
    pub contributions_diff: i32,
    /// 比較対象の日付（YYYY-MM-DD形式）
    pub comparison_date: Option<String>,
}

/// 新しく獲得したバッジ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBadgeInfo {
    pub badge_id: String,
    pub badge_type: String,
    pub name: String,
    pub description: String,
    pub rarity: String,
    pub icon: String,
}

/// バッジ獲得イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeEarnedEvent {
    pub badge_id: String,
    pub badge_type: String,
    pub name: String,
    pub description: String,
    pub rarity: String,
    pub icon: String,
}

// ============================================
// コード統計関連の型
// ============================================

/// 日別コード統計
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyCodeStats {
    pub id: i64,
    pub user_id: i64,
    /// 日付 (YYYY-MM-DD形式)
    pub date: String,
    /// 追加行数
    pub additions: i32,
    /// 削除行数
    pub deletions: i32,
    /// コミット数
    pub commits_count: i32,
    /// リポジトリ一覧 (JSON配列)
    pub repositories_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl DailyCodeStats {
    /// 純増減行数を取得
    pub fn net_change(&self) -> i32 {
        self.additions - self.deletions
    }

    /// リポジトリ一覧をパース
    pub fn repositories(&self) -> Vec<String> {
        self.repositories_json
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default()
    }
}

/// コード統計サマリー
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodeStatsSummary {
    pub additions: i32,
    pub deletions: i32,
    pub net_change: i32,
    pub commits_count: i32,
    pub active_days: i32,
}

/// 統計期間
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum StatsPeriod {
    #[default]
    Week,
    Month,
    Quarter,
    Year,
}

impl StatsPeriod {
    /// 期間の日数を取得
    pub fn days(&self) -> i64 {
        match self {
            StatsPeriod::Week => 7,
            StatsPeriod::Month => 30,
            StatsPeriod::Quarter => 90,
            StatsPeriod::Year => 365,
        }
    }

    /// 表示用ラベル
    pub fn label(&self) -> &'static str {
        match self {
            StatsPeriod::Week => "週間",
            StatsPeriod::Month => "月間",
            StatsPeriod::Quarter => "四半期",
            StatsPeriod::Year => "年間",
        }
    }
}

/// コード統計レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeStatsResponse {
    /// 日別統計
    pub daily: Vec<DailyCodeStats>,
    /// 週間サマリー
    pub weekly_total: CodeStatsSummary,
    /// 月間サマリー
    pub monthly_total: CodeStatsSummary,
    /// リクエストした期間
    pub period: StatsPeriod,
}

/// レート制限情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    /// REST API残量
    pub rest_remaining: i32,
    pub rest_limit: i32,
    pub rest_reset_at: Option<String>,
    /// GraphQL API残量
    pub graphql_remaining: i32,
    pub graphql_limit: i32,
    pub graphql_reset_at: Option<String>,
    /// Search API残量
    pub search_remaining: i32,
    pub search_limit: i32,
    pub search_reset_at: Option<String>,
    /// 制限が危機的か（20%以下）
    pub is_critical: bool,
}

impl RateLimitInfo {
    /// REST APIの使用率（%）
    pub fn rest_usage_percent(&self) -> f32 {
        if self.rest_limit == 0 {
            return 0.0;
        }
        ((self.rest_limit - self.rest_remaining) as f32 / self.rest_limit as f32) * 100.0
    }

    /// GraphQL APIの使用率（%）
    pub fn graphql_usage_percent(&self) -> f32 {
        if self.graphql_limit == 0 {
            return 0.0;
        }
        ((self.graphql_limit - self.graphql_remaining) as f32 / self.graphql_limit as f32) * 100.0
    }
}

/// コード統計同期結果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeStatsSyncResult {
    /// 同期した日数
    pub days_synced: i32,
    /// 同期期間の追加行数合計
    pub total_additions: i32,
    /// 同期期間の削除行数合計
    pub total_deletions: i32,
    /// キャッシュからの取得かどうか
    pub from_cache: bool,
    /// 同期後のレート制限情報
    pub rate_limit: Option<RateLimitInfo>,
}
