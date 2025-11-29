//! Tool-related types

use serde::{Deserialize, Serialize};

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
    /// パスの種類 ("file", "directory", "any") - Pathタイプの場合のみ使用
    pub path_type: Option<String>,
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

/// ログエントリ
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub line: String,
    pub stream: LogStream,
    pub timestamp: String,
}
