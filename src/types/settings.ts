// Settings-related types

/// 通知方法の選択肢
///
/// **IMPORTANT**: This enum must be kept in sync with the backend definition at:
/// `src-tauri/src/database/models.rs::NotificationMethod`
///
/// Both implementations use the same string values (app_only, os_only, both, none)
/// for serialization to ensure compatibility.
export type NotificationMethod = 'app_only' | 'os_only' | 'both' | 'none';

export function notificationMethodFromStr(s: string): NotificationMethod {
  switch (s) {
    case 'app_only':
      return 'app_only';
    case 'os_only':
      return 'os_only';
    case 'both':
      return 'both';
    case 'none':
      return 'none';
    default:
      return 'both';
  }
}

export function notificationMethodLabel(method: NotificationMethod): string {
  switch (method) {
    case 'app_only':
      return 'アプリ内のみ';
    case 'os_only':
      return 'OSネイティブのみ';
    case 'both':
      return '両方';
    case 'none':
      return '通知なし';
  }
}

/// ユーザー設定
export interface UserSettings {
  id: number;
  userId: number;
  notificationMethod: NotificationMethod;
  notifyXpGain: boolean;
  notifyLevelUp: boolean;
  notifyBadgeEarned: boolean;
  notifyStreakUpdate: boolean;
  notifyStreakMilestone: boolean;
  syncIntervalMinutes: number;
  backgroundSync: boolean;
  syncOnStartup: boolean;
  animationsEnabled: boolean;
  createdAt: string;
  updatedAt: string;
}

/// 設定更新リクエスト
export interface UpdateSettingsRequest {
  notificationMethod: NotificationMethod;
  notifyXpGain: boolean;
  notifyLevelUp: boolean;
  notifyBadgeEarned: boolean;
  notifyStreakUpdate: boolean;
  notifyStreakMilestone: boolean;
  syncIntervalMinutes: number;
  backgroundSync: boolean;
  syncOnStartup: boolean;
  animationsEnabled: boolean;
}

/// データベース情報
export interface DatabaseInfo {
  path: string;
  sizeBytes: number;
  cacheSizeBytes: number;
}

/// キャッシュクリア結果
export interface ClearCacheResult {
  clearedEntries: number;
  freedBytes: number;
}

/// アプリケーション情報
export interface AppInfo {
  version: string;
  buildDate: string;
  tauriVersion: string;
  rustVersion: string;
}

/// 同期間隔の選択肢
///
/// **IMPORTANT**: This constant must be kept in sync with the backend definition at:
/// `src-tauri/src/database/models.rs::settings_defaults::SYNC_INTERVALS`
///
/// Alternatively, use `tauri_api::get_sync_intervals()` to fetch the authoritative
/// list from the backend, which is the recommended approach for dynamic UI.
export const SYNC_INTERVALS: [number, string][] = [
  [5, '5分'],
  [15, '15分'],
  [30, '30分'],
  [60, '1時間'],
  [180, '3時間'],
  [0, '手動のみ'],
];

/// 同期間隔のラベルを取得
export function getSyncIntervalLabel(minutes: number): string {
  const found = SYNC_INTERVALS.find(([m]) => m === minutes);
  return found ? found[1] : '不明';
}

/// 同期間隔オプション（バックエンドから取得）
export interface SyncIntervalOption {
  value: number;
  label: string;
}

