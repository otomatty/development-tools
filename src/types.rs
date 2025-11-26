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

/// アプリのページ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppPage {
    #[default]
    Home,
    Tools,
    Settings,
}

