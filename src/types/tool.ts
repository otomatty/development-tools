// Tool-related types

/// オプションの型
export type OptionType = 'string' | 'path' | 'boolean' | 'select' | 'number';

/// ツール情報（一覧表示用）
export interface ToolInfo {
  name: string;
  displayName: string;
  description: string;
  version: string;
  icon: string | null;
  category: string | null;
  toolDir: string;
}

/// ツール設定（詳細用）
export interface ToolConfig {
  name: string;
  displayName: string;
  description: string;
  version: string;
  binary: string;
  icon: string | null;
  category: string | null;
  options: ToolOption[];
  resultParser: ResultParser | null;
}

/// コマンドラインオプションの定義
export interface ToolOption {
  name: string;
  flag: string;
  shortFlag: string | null;
  type: OptionType;
  description: string;
  required: boolean;
  default: unknown | null;
  placeholder: string | null;
  options: string[] | null;
  /// パスの種類 ("file", "directory", "any") - Pathタイプの場合のみ使用
  pathType: string | null;
}

/// パーサーの型
export type ParserType = 'json' | 'text';

/// 結果パーサーの設定
export interface ResultParser {
  type: ParserType;
  outputFlag: string | null;
  schema: ResultSchema | null;
}

/// 結果のスキーマ定義
export interface ResultSchema {
  summary: SummaryItem[] | null;
  details: DetailsConfig | null;
}

/// サマリー項目の定義
export interface SummaryItem {
  key: string;
  label: string;
  path: string;
  countType: string | null;
  color: string | null;
  icon: string | null;
}

/// 詳細表示の設定
export interface DetailsConfig {
  items: string;
  columns: ColumnConfig[];
}

/// カラム設定
export interface ColumnConfig {
  key: string;
  label: string;
  width: string | null;
  flex: number | null;
}

/// ツール実行結果
export interface ToolResult {
  success: boolean;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  parsedResult: unknown | null;
}

/// ログストリーム
export type LogStream = 'stdout' | 'stderr';

/// リアルタイムログイベント
export interface LogEvent {
  toolName: string;
  line: string;
  stream: LogStream;
  timestamp: string;
}

/// ツール実行状態
export type ToolStatus = 'running' | 'completed' | 'failed' | 'cancelled';

/// ツール実行状態イベント
export interface ToolStatusEvent {
  toolName: string;
  status: ToolStatus;
  result: ToolResult | null;
}

/// ログエントリ
export interface LogEntry {
  line: string;
  stream: LogStream;
  timestamp: string;
}

