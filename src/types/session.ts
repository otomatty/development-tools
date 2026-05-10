/**
 * Pomodoro / Focus Session types
 *
 * In-app timer for Pomodoro-style work sessions. Completed focus sessions are
 * counted towards local history and award XP through the existing gamification
 * pipeline (`add_xp` Tauri command).
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 *   - Store: src/stores/sessionStore.ts
 */

/// セッションのフェーズ
export type SessionPhase = 'focus' | 'short_break' | 'long_break';

/// タイマーの状態
export type SessionStatus = 'idle' | 'running' | 'paused';

/// Pomodoro 設定（25 / 5 / 15 分など）
export interface PomodoroConfig {
  /// 集中フェーズの長さ（分）
  focusMinutes: number;
  /// 短い休憩の長さ（分）
  shortBreakMinutes: number;
  /// 長い休憩の長さ（分）
  longBreakMinutes: number;
  /// 何回の集中フェーズの後に長い休憩を入れるか
  longBreakInterval: number;
  /// フェーズ完了時に次のフェーズを自動開始するか
  autoStartNext: boolean;
  /// 集中フェーズ完了時に獲得する XP
  focusCompletionXp: number;
}

/// 完了 / 中断したセッションの記録
export interface SessionRecord {
  /// 一意な ID（タイムスタンプ + ランダム）
  id: string;
  /// セッションの種類
  phase: SessionPhase;
  /// 開始時刻（ISO8601）
  startedAt: string;
  /// 終了時刻（ISO8601）
  endedAt: string;
  /// 計画していた長さ（秒）
  plannedDurationSeconds: number;
  /// 実際にカウントした時間（秒、一時停止分を除く）
  actualDurationSeconds: number;
  /// 完了したか（true: 規定時間まで到達 / false: 途中停止）
  completed: boolean;
  /// 付与された XP（休憩や中断時は 0）
  xpAwarded: number;
}

/// デフォルト設定
export const DEFAULT_POMODORO_CONFIG: PomodoroConfig = {
  focusMinutes: 25,
  shortBreakMinutes: 5,
  longBreakMinutes: 15,
  longBreakInterval: 4,
  autoStartNext: false,
  focusCompletionXp: 25,
};

/// `phase` の表示用ラベル
export function sessionPhaseLabel(phase: SessionPhase): string {
  switch (phase) {
    case 'focus':
      return '集中';
    case 'short_break':
      return '休憩';
    case 'long_break':
      return '長い休憩';
    default: {
      const exhaustive: never = phase;
      throw new Error(`Unhandled session phase: ${exhaustive}`);
    }
  }
}

/// `phase` の絵文字（バナー / 通知などで使用）
export function sessionPhaseEmoji(phase: SessionPhase): string {
  switch (phase) {
    case 'focus':
      return '🎯';
    case 'short_break':
      return '☕';
    case 'long_break':
      return '🌿';
    default: {
      const exhaustive: never = phase;
      throw new Error(`Unhandled session phase: ${exhaustive}`);
    }
  }
}

/// `phase` の長さを `config` から取り出す
export function sessionPhaseMinutes(phase: SessionPhase, config: PomodoroConfig): number {
  switch (phase) {
    case 'focus':
      return config.focusMinutes;
    case 'short_break':
      return config.shortBreakMinutes;
    case 'long_break':
      return config.longBreakMinutes;
    default: {
      const exhaustive: never = phase;
      throw new Error(`Unhandled session phase: ${exhaustive}`);
    }
  }
}

/// 秒を `MM:SS` 表記に整形
export function formatRemaining(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const minutes = Math.floor(safe / 60);
  const seconds = safe % 60;
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

/// `pomodoro:session-completed` カスタムイベントのペイロード
///
/// バックエンドの Tauri イベントではなく、`window.dispatchEvent` で発火する
/// フロントエンド限定のイベント。XP 付与・バッジ評価のフックとして利用する。
export interface PomodoroSessionCompletedDetail {
  record: SessionRecord;
  totalFocusCompleted: number;
}

export const POMODORO_SESSION_COMPLETED_EVENT = 'pomodoro:session-completed';
