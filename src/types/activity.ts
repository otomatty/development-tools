/**
 * Activity Timeline Types
 *
 * Mirrors `commands::activity::ActivityFeedItem` /
 * `ActivityFeed` on the backend.
 *
 * Related Issue: https://github.com/otomatty/development-tools/issues/187
 */

/// Known event types we render with bespoke copy. Extra strings are tolerated
/// (the `(string & {})` trick) so a future GitHub event type still gets a
/// generic row instead of a runtime cast error.
export type ActivityEventType =
  | 'PushEvent'
  | 'PullRequestEvent'
  | 'PullRequestReviewEvent'
  | 'PullRequestReviewCommentEvent'
  | 'IssuesEvent'
  | 'IssueCommentEvent'
  | 'ReleaseEvent'
  | 'CreateEvent'
  | 'DeleteEvent'
  | 'ForkEvent'
  | 'WatchEvent'
  | 'PublicEvent'
  | 'MemberEvent';

/// One row in the home activity timeline.
///
/// Mirrors `commands::activity::ActivityFeedItem` on the backend.
export interface ActivityFeedItem {
  id: string;
  /// `PushEvent` / `PullRequestEvent` / etc. Forwarded verbatim.
  eventType: ActivityEventType | (string & {});
  /// ISO8601.
  createdAt: string;
  /// `owner/repo`.
  repoName: string;
  /// `https://github.com/{owner}/{repo}`.
  repoUrl: string;
  /// `opened` / `closed` / `started` / etc., when the event carries an action.
  action: string | null;
  /// PR / issue / release title (or branch name as a fallback).
  title: string | null;
  /// Browser URL of the artefact (PR / issue / release / comment). Falls back
  /// to `repoUrl` when missing.
  targetUrl: string | null;
  /// PR or issue number.
  number: number | null;
  /// Branch / tag name for `PushEvent` / `CreateEvent` / `DeleteEvent`.
  refName: string | null;
  /// `branch` / `tag` / `repository` for `CreateEvent` / `DeleteEvent`.
  refType: string | null;
  /// Number of commits in a `PushEvent`.
  commitsCount: number | null;
}

/// Aggregated payload returned by `get_activity_feed_with_cache`.
export interface ActivityFeed {
  items: ActivityFeedItem[];
}
