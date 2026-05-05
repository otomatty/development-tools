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
  /// True when the backend served the list from its local cache rather
  /// than a fresh GitHub fetch (e.g. GitHub responded 304).
  fromCache: boolean;
  /// `x-poll-interval` (seconds) hint from GitHub. The scheduler honours
  /// this; the UI surfaces it for diagnostics.
  pollIntervalSeconds: number | null;
}

/// Event payload emitted when the backend observes new notification activity.
///
/// Includes the freshly-fetched items so the UI can replace its local
/// list directly. A re-fetch in response to this event would race the
/// just-persisted ETag and come back as 304, leaving the UI stale.
///
/// `userId` is the DB id of the user the scheduler captured before
/// awaiting GitHub. The UI compares it against the currently logged-in
/// user and drops mismatches — if an account switch happens mid-flight
/// the event still fires for the *previous* user, and applying its
/// items to the new account would leak unread counts and repo titles
/// across users.
export interface NotificationsUpdatedEvent {
  userId: number;
  unreadCount: number;
  newCount: number;
  items: NotificationItem[];
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
