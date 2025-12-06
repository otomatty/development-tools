// Gamification-related types (XP, badges, stats, etc.)

// Note: XP、レベル、カウント系の値は意味的にはu32（符号なし整数）が適切ですが、
// SQLiteのINTEGER型が符号あり整数であり、sqlxがi32としてマッピングするため、
// バックエンドとの整合性を保つためにnumberで統一しています。
// 実用上、これらの値が負になることはなく、21億を超えることもないため問題ありません。

/// Generic cached response wrapper
export interface CachedResponse<T> {
  /// The actual data
  data: T;
  /// Whether the data was retrieved from cache
  fromCache: boolean;
  /// When the data was cached (ISO8601 format)
  cachedAt: string | null;
  /// When the cache expires (ISO8601 format)
  expiresAt: string | null;
}

/// Cache statistics for display in settings
export interface CacheStats {
  /// Total cache size in bytes
  totalSizeBytes: number;
  /// Number of cache entries
  entryCount: number;
  /// Number of expired entries
  expiredCount: number;
  /// Last cleanup timestamp (ISO8601)
  lastCleanupAt: string | null;
}

/// ユーザー統計
export interface UserStats {
  id: number;
  userId: number;
  totalXp: number;
  currentLevel: number;
  currentStreak: number;
  longestStreak: number;
  lastActivityDate: string | null;
  totalCommits: number;
  totalPrs: number;
  totalReviews: number;
  totalIssues: number;
  updatedAt: string;
}

/// レベル情報
export interface LevelInfo {
  currentLevel: number;
  totalXp: number;
  xpForCurrentLevel: number;
  xpForNextLevel: number;
  xpToNextLevel: number;
  progressPercent: number;
}

/// GitHub統計
export interface GitHubStats {
  totalCommits: number;
  totalPrs: number;
  totalPrsMerged: number;
  totalIssues: number;
  totalIssuesClosed: number;
  totalReviews: number;
  totalStarsReceived: number;
  totalContributions: number;
  contributionCalendar: ContributionCalendar | null;
  currentStreak: number;
  longestStreak: number;
  weeklyStreak: number;
  monthlyStreak: number;
  languagesCount: number;
}

/// コントリビューションカレンダー
export interface ContributionCalendar {
  totalContributions: number;
  weeks: ContributionWeek[];
}

export interface ContributionWeek {
  contributionDays: ContributionDay[];
}

export interface ContributionDay {
  contributionCount: number;
  date: string;
  weekday: number;
}

/// バッジ
export interface Badge {
  id: number;
  userId: number;
  badgeType: string;
  badgeId: string;
  earnedAt: string;
}

/// バッジ定義
export interface BadgeDefinition {
  id: string;
  name: string;
  description: string;
  badgeType: string;
  rarity: string;
  icon: string;
}

/// バッジ進捗情報
export interface BadgeProgress {
  badgeId: string;
  currentValue: number;
  targetValue: number;
  progressPercent: number;
}

/// 進捗情報付きバッジ
export interface BadgeWithProgress {
  id: string;
  name: string;
  description: string;
  badgeType: string;
  rarity: string;
  icon: string;
  earned: boolean;
  earnedAt: string | null;
  progress: BadgeProgress | null;
}

/// XP履歴エントリ
export interface XpHistoryEntry {
  id: number;
  userId: number;
  actionType: string;
  xpAmount: number;
  description: string | null;
  githubEventId: string | null;
  breakdown: XpBreakdown | null;
  createdAt: string;
}

/// XP獲得時のブレークダウン
export interface XpBreakdown {
  commitsXp: number;
  prsCreatedXp: number;
  prsMergedXp: number;
  issuesCreatedXp: number;
  issuesClosedXp: number;
  reviewsXp: number;
  starsXp: number;
  streakBonusXp: number;
  totalXp: number;
}

/// ストリークボーナス情報
export interface StreakBonusInfo {
  dailyBonus: number;
  milestoneBonus: number;
  totalBonus: number;
  milestoneReached: number | null;
  currentStreak: number;
  nextMilestoneDays: number | null;
  daysToNextMilestone: number | null;
}

/// XP獲得イベント
export interface XpGainedEvent {
  xpGained: number;
  totalXp: number;
  oldLevel: number;
  newLevel: number;
  levelUp: boolean;
  xpBreakdown: XpBreakdown;
  streakBonus: StreakBonusInfo;
}

/// ストリークマイルストーン到達イベント
export interface StreakMilestoneEvent {
  milestoneDays: number;
  bonusXp: number;
  currentStreak: number;
}

/// GitHub統計同期結果
export interface SyncResult {
  userStats: UserStats;
  xpGained: number;
  oldLevel: number;
  newLevel: number;
  levelUp: boolean;
  xpBreakdown: XpBreakdown;
  streakBonus: StreakBonusInfo;
  newBadges: NewBadgeInfo[];
  /// 前日比の統計差分（初回同期時はNone）
  statsDiff: StatsDiffResult | null;
}

/// 前日比の統計差分
export interface StatsDiffResult {
  /// コミット数の差分
  commitsDiff: number;
  /// PR数の差分
  prsDiff: number;
  /// レビュー数の差分
  reviewsDiff: number;
  /// Issue数の差分
  issuesDiff: number;
  /// スター獲得数の差分
  starsDiff: number;
  /// コントリビューション数の差分
  contributionsDiff: number;
  /// 比較対象の日付（YYYY-MM-DD形式）
  comparisonDate: string | null;
}

/// 新しく獲得したバッジ情報
export interface NewBadgeInfo {
  badgeId: string;
  badgeType: string;
  name: string;
  description: string;
  rarity: string;
  icon: string;
}

/// バッジ獲得イベント
export interface BadgeEarnedEvent {
  badgeId: string;
  badgeType: string;
  name: string;
  description: string;
  rarity: string;
  icon: string;
}

// ============================================
// コード統計関連の型
// ============================================

/// 日別コード統計
export interface DailyCodeStats {
  id: number;
  userId: number;
  /// 日付 (YYYY-MM-DD形式)
  date: string;
  /// 追加行数
  additions: number;
  /// 削除行数
  deletions: number;
  /// コミット数
  commitsCount: number;
  /// リポジトリ一覧 (JSON配列)
  repositoriesJson: string | null;
  createdAt: string;
  updatedAt: string;
}

/// コード統計サマリー
export interface CodeStatsSummary {
  additions: number;
  deletions: number;
  netChange: number;
  commitsCount: number;
  activeDays: number;
}

/// 統計期間
export type StatsPeriod = 'week' | 'month' | 'quarter' | 'year';

/// 期間の日数を取得
export function statsPeriodDays(period: StatsPeriod): number {
  switch (period) {
    case 'week':
      return 7;
    case 'month':
      return 30;
    case 'quarter':
      return 90;
    case 'year':
      return 365;
  }
}

/// 表示用ラベル
export function statsPeriodLabel(period: StatsPeriod): string {
  switch (period) {
    case 'week':
      return '週間';
    case 'month':
      return '月間';
    case 'quarter':
      return '四半期';
    case 'year':
      return '年間';
  }
}

/// コード統計レスポンス
export interface CodeStatsResponse {
  /// 日別統計
  daily: DailyCodeStats[];
  /// 週間サマリー
  weeklyTotal: CodeStatsSummary;
  /// 月間サマリー
  monthlyTotal: CodeStatsSummary;
  /// リクエストした期間
  period: StatsPeriod;
}

/// レート制限情報
export interface RateLimitInfo {
  /// REST API残量
  restRemaining: number;
  restLimit: number;
  restResetAt: string | null;
  /// GraphQL API残量
  graphqlRemaining: number;
  graphqlLimit: number;
  graphqlResetAt: string | null;
  /// Search API残量
  searchRemaining: number;
  searchLimit: number;
  searchResetAt: string | null;
  /// 制限が危機的か（20%以下）
  isCritical: boolean;
}

/// REST APIの使用率（%）
export function restUsagePercent(rateLimit: RateLimitInfo): number {
  if (rateLimit.restLimit === 0) {
    return 0.0;
  }
  return ((rateLimit.restLimit - rateLimit.restRemaining) / rateLimit.restLimit) * 100.0;
}

/// GraphQL APIの使用率（%）
export function graphqlUsagePercent(rateLimit: RateLimitInfo): number {
  if (rateLimit.graphqlLimit === 0) {
    return 0.0;
  }
  return ((rateLimit.graphqlLimit - rateLimit.graphqlRemaining) / rateLimit.graphqlLimit) * 100.0;
}

/// コード統計同期結果
export interface CodeStatsSyncResult {
  /// 同期した日数
  daysSynced: number;
  /// 同期期間の追加行数合計
  totalAdditions: number;
  /// 同期期間の削除行数合計
  totalDeletions: number;
  /// キャッシュからの取得かどうか
  fromCache: boolean;
  /// 同期後のレート制限情報
  rateLimit: RateLimitInfo | null;
}

/// 純増減行数を取得
export function netChange(stats: DailyCodeStats): number {
  return stats.additions - stats.deletions;
}

/// リポジトリ一覧をパース
export function repositories(stats: DailyCodeStats): string[] {
  if (!stats.repositoriesJson) {
    return [];
  }
  try {
    return JSON.parse(stats.repositoriesJson);
  } catch {
    return [];
  }
}

