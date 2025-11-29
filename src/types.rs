use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ツール情報（一覧表示用）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub tool_dir: String,
}

/// ツール設定（詳細用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub binary: String,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub options: Vec<ToolOption>,
    pub result_parser: Option<ResultParser>,
}

/// コマンドラインオプションの定義
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOption {
    pub name: String,
    pub flag: String,
    pub short_flag: Option<String>,
    #[serde(rename = "type")]
    pub option_type: OptionType,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    pub options: Option<Vec<String>>,
}

/// オプションの型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OptionType {
    String,
    Path,
    Boolean,
    Select,
    Number,
}

/// 結果パーサーの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultParser {
    #[serde(rename = "type")]
    pub parser_type: ParserType,
    pub output_flag: Option<String>,
    pub schema: Option<ResultSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParserType {
    Json,
    Text,
}

/// 結果のスキーマ定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSchema {
    pub summary: Option<Vec<SummaryItem>>,
    pub details: Option<DetailsConfig>,
}

/// サマリー項目の定義
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryItem {
    pub key: String,
    pub label: String,
    pub path: String,
    pub count_type: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// 詳細表示の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailsConfig {
    pub items: String,
    pub columns: Vec<ColumnConfig>,
}

/// カラム設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfig {
    pub key: String,
    pub label: String,
    pub width: Option<String>,
    pub flex: Option<i32>,
}

/// ツール実行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub parsed_result: Option<serde_json::Value>,
}

/// リアルタイムログイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub tool_name: String,
    pub line: String,
    pub stream: LogStream,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// ツール実行状態イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatusEvent {
    pub tool_name: String,
    pub status: ToolStatus,
    pub result: Option<ToolResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// オプション値のマップ
pub type OptionValues = HashMap<String, serde_json::Value>;

/// ログエントリ
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub line: String,
    pub stream: LogStream,
    pub timestamp: String,
}

// ============================================
// 認証・ゲーミフィケーション関連の型
// ============================================

/// 認証状態
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
    pub is_logged_in: bool,
    pub user: Option<UserInfo>,
}

/// ユーザー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub github_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<String>,
}

// ============================================
// Device Flow 認証関連の型
// ============================================

/// Device Flow開始時のレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: i64,
    pub interval: i64,
}

/// Device Flowトークンポーリングのステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceTokenStatus {
    /// 認証待ち - ユーザーがまだ認証を完了していない
    Pending,
    /// 認証成功 - ログイン完了
    Success { auth_state: AuthState },
    /// エラー発生
    Error { message: String },
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

/// GitHubユーザー
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub public_repos: i32,
    pub followers: i32,
    pub following: i32,
    pub created_at: String,
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

/// アプリのページ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppPage {
    #[default]
    Home,
    Tools,
    Settings,
}

// ============================================
// 設定関連の型
// ============================================

/// ユーザー設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub id: i64,
    pub user_id: i64,
    pub notification_method: String,
    pub notify_xp_gain: bool,
    pub notify_level_up: bool,
    pub notify_badge_earned: bool,
    pub notify_streak_update: bool,
    pub notify_streak_milestone: bool,
    pub sync_interval_minutes: i32,
    pub background_sync: bool,
    pub sync_on_startup: bool,
    pub animations_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 設定更新リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub notification_method: String,
    pub notify_xp_gain: bool,
    pub notify_level_up: bool,
    pub notify_badge_earned: bool,
    pub notify_streak_update: bool,
    pub notify_streak_milestone: bool,
    pub sync_interval_minutes: i32,
    pub background_sync: bool,
    pub sync_on_startup: bool,
    pub animations_enabled: bool,
}

impl From<&UserSettings> for UpdateSettingsRequest {
    fn from(settings: &UserSettings) -> Self {
        Self {
            notification_method: settings.notification_method.clone(),
            notify_xp_gain: settings.notify_xp_gain,
            notify_level_up: settings.notify_level_up,
            notify_badge_earned: settings.notify_badge_earned,
            notify_streak_update: settings.notify_streak_update,
            notify_streak_milestone: settings.notify_streak_milestone,
            sync_interval_minutes: settings.sync_interval_minutes,
            background_sync: settings.background_sync,
            sync_on_startup: settings.sync_on_startup,
            animations_enabled: settings.animations_enabled,
        }
    }
}

/// データベース情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    pub cache_size_bytes: u64,
}

/// キャッシュクリア結果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClearCacheResult {
    pub cleared_entries: i32,
    pub freed_bytes: u64,
}

/// アプリケーション情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub build_date: String,
    pub tauri_version: String,
    pub leptos_version: String,
    pub rust_version: String,
}

/// 通知方法の選択肢
/// 
/// **IMPORTANT**: This enum must be kept in sync with the backend definition at:
/// `src-tauri/src/database/models.rs::NotificationMethod`
/// 
/// Both implementations use the same string values (app_only, os_only, both, none)
/// for serialization to ensure compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationMethod {
    AppOnly,
    OsOnly,
    #[default]
    Both,
    None,
}

impl NotificationMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationMethod::AppOnly => "app_only",
            NotificationMethod::OsOnly => "os_only",
            NotificationMethod::Both => "both",
            NotificationMethod::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "app_only" => NotificationMethod::AppOnly,
            "os_only" => NotificationMethod::OsOnly,
            "both" => NotificationMethod::Both,
            "none" => NotificationMethod::None,
            _ => NotificationMethod::Both,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            NotificationMethod::AppOnly => "アプリ内のみ",
            NotificationMethod::OsOnly => "OSネイティブのみ",
            NotificationMethod::Both => "両方",
            NotificationMethod::None => "通知なし",
        }
    }
}

/// 同期間隔の選択肢
/// 
/// **IMPORTANT**: This constant must be kept in sync with the backend definition at:
/// `src-tauri/src/database/models.rs::settings_defaults::SYNC_INTERVALS`
/// 
/// Alternatively, use `tauri_api::get_sync_intervals()` to fetch the authoritative
/// list from the backend, which is the recommended approach for dynamic UI.
pub const SYNC_INTERVALS: &[(i32, &str)] = &[
    (5, "5分"),
    (15, "15分"),
    (30, "30分"),
    (60, "1時間"),
    (180, "3時間"),
    (0, "手動のみ"),
];

/// 同期間隔のラベルを取得
pub fn get_sync_interval_label(minutes: i32) -> &'static str {
    SYNC_INTERVALS
        .iter()
        .find(|(m, _)| *m == minutes)
        .map(|(_, label)| *label)
        .unwrap_or("不明")
}

/// 同期間隔オプション（バックエンドから取得）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncIntervalOption {
    pub value: i32,
    pub label: String,
}

