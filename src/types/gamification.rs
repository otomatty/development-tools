//! Gamification-related types (XP, badges, stats, etc.)

use serde::{Deserialize, Serialize};

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
    pub current_level: u32,
    pub total_xp: u32,
    pub xp_for_current_level: u32,
    pub xp_for_next_level: u32,
    pub xp_to_next_level: u32,
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
    pub total_xp: u32,
    pub old_level: u32,
    pub new_level: u32,
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
    pub old_level: u32,
    pub new_level: u32,
    pub level_up: bool,
    pub xp_breakdown: XpBreakdown,
    pub streak_bonus: StreakBonusInfo,
    pub new_badges: Vec<NewBadgeInfo>,
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
