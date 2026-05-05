/**
 * GitHub Notifications Types
 *
 * Mirrors `commands::notifications::NotificationItem` /
 * `NotificationsPayload` on the backend.
 *
 * Related Issue: https://github.com/otomatty/development-tools/issues/186
 */

/// One row in the notifications dropdown / list.
export interface NotificationItem {
  id: string;
  unread: boolean;
  /// `mention` / `review_requested` / `assign` / `comment` / etc.
  reason: string;
  title: string;
  /// `Issue` / `PullRequest` / `Commit` / `Discussion`.
  subjectType: string;
  repoFullName: string;
  repoUrl: string;
  /// Browser URL (already translated from the API URL by the backend).
  htmlUrl: string;
  /// ISO8601.
  updatedAt: string;
  /// ISO8601 or null when never read.
  lastReadAt: string | null;
}

/// Aggregated payload returned by `get_notifications`.
export interface NotificationsPayload {
  items: NotificationItem[];
  unreadCount: number;
  /// True when GitHub returned 304 — the items list is empty and the caller
  /// should keep showing whatever it already had.
  fromCache: boolean;
  /// `x-poll-interval` (seconds) hint from GitHub. The scheduler honours
  /// this; the UI surfaces it for diagnostics.
  pollIntervalSeconds: number | null;
}

/// Event payload emitted when the backend observes new notification activity.
export interface NotificationsUpdatedEvent {
  unreadCount: number;
  newCount: number;
}

/// Display label for the notification's `reason` field.
export function notificationReasonLabel(reason: string): string {
  switch (reason) {
    case 'mention':
    case 'team_mention':
      return 'メンション';
    case 'review_requested':
      return 'レビュー依頼';
    case 'assign':
      return 'アサイン';
    case 'author':
      return '作成者';
    case 'comment':
      return 'コメント';
    case 'subscribed':
      return 'ウォッチ';
    case 'state_change':
      return '状態変更';
    case 'ci_activity':
      return 'CI';
    default:
      return reason;
  }
}
