// Challenge-related types

/// チャレンジタイプ
export type ChallengeType = 'daily' | 'weekly';

/// ターゲットメトリクス
export type TargetMetric = 'commits' | 'prs' | 'reviews' | 'issues';

/// チャレンジステータス
export type ChallengeStatus = 'active' | 'completed' | 'failed';

/// チャレンジ情報
export interface ChallengeInfo {
  id: number;
  userId: number;
  challengeType: ChallengeType;
  targetMetric: TargetMetric;
  targetValue: number;
  currentValue: number;
  rewardXp: number;
  startDate: string;
  endDate: string;
  status: ChallengeStatus;
  completedAt: string | null;
  // Computed fields
  progressPercent: number;
  remainingTimeHours: number;
  isCompleted: boolean;
  isExpired: boolean;
}

/// チャレンジ作成リクエスト
export interface CreateChallengeRequest {
  challengeType: ChallengeType;
  targetMetric: TargetMetric;
  targetValue: number;
  rewardXp: number | null;
}

/// チャレンジ統計
export interface ChallengeStats {
  totalCompleted: number;
  consecutiveWeeklyCompletions: number;
  activeCount: number;
}

/// チャレンジタイプの選択肢
export const CHALLENGE_TYPES: [string, string][] = [
  ['daily', 'デイリー'],
  ['weekly', 'ウィークリー'],
];

/// ターゲットメトリクスの選択肢
export const TARGET_METRICS: [string, string, string][] = [
  ['commits', 'コミット', '📝'],
  ['prs', 'PR', '🔀'],
  ['reviews', 'レビュー', '👀'],
  ['issues', 'Issue', '🐛'],
];

/// Get display name for challenge type
export function challengeTypeLabel(challengeType: ChallengeType): string {
  switch (challengeType) {
    case 'daily':
      return 'デイリー';
    case 'weekly':
      return 'ウィークリー';
  }
}

/// Get display name for target metric
export function targetMetricLabel(targetMetric: TargetMetric): string {
  switch (targetMetric) {
    case 'commits':
      return 'コミット';
    case 'prs':
      return 'PR';
    case 'reviews':
      return 'レビュー';
    case 'issues':
      return 'Issue';
  }
}

/// Get icon for target metric
export function targetMetricIcon(targetMetric: TargetMetric): string {
  switch (targetMetric) {
    case 'commits':
      return '📝';
    case 'prs':
      return '🔀';
    case 'reviews':
      return '👀';
    case 'issues':
      return '🐛';
  }
}

/// Get status label
export function statusLabel(status: ChallengeStatus): string {
  switch (status) {
    case 'active':
      return '進行中';
    case 'completed':
      return '達成';
    case 'failed':
      return '失敗';
  }
}

/// Format remaining time as human-readable string
export function remainingTimeLabel(remainingTimeHours: number): string {
  if (remainingTimeHours <= 0) {
    return '終了';
  }

  if (remainingTimeHours >= 24) {
    const days = Math.ceil(remainingTimeHours / 24);
    return `残り ${days}日`;
  } else {
    return `残り ${Math.floor(remainingTimeHours)}時間`;
  }
}

