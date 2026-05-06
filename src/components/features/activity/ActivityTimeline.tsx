/**
 * Activity Timeline Component
 *
 * Renders the authenticated user's recent GitHub activity in a scrollable
 * timeline. Each row links out to the relevant artefact (PR, issue,
 * release, repo) via the system browser.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/187
 */

import React, { useMemo } from 'react';
import { Icon } from '@/components/icons';
import { settings as settingsApi } from '@/lib/tauri/commands';
import type { ActivityFeedItem } from '@/types';

interface ActivityTimelineProps {
  items: ActivityFeedItem[] | null;
  isLoading?: boolean;
  isRevalidating?: boolean;
  error?: Error | null;
  onRetry?: () => void;
}

/**
 * Map an event type / action pair to (icon name, accent class).
 * Uses the same lucide names already exposed by the shared `Icon` component.
 */
function getEventVisual(
  eventType: string,
  action: string | null,
): { icon: string; tone: string } {
  switch (eventType) {
    case 'PushEvent':
      return { icon: 'git-branch', tone: 'text-gm-accent-cyan' };
    case 'PullRequestEvent':
      if (action === 'closed') return { icon: 'git-merge', tone: 'text-gm-accent-purple' };
      return { icon: 'git-pull-request', tone: 'text-gm-accent-purple' };
    case 'PullRequestReviewEvent':
    case 'PullRequestReviewCommentEvent':
      return { icon: 'check-square', tone: 'text-gm-accent-pink' };
    case 'IssuesEvent':
    case 'IssueCommentEvent':
      return { icon: 'alert-circle', tone: 'text-gm-success' };
    case 'ReleaseEvent':
      return { icon: 'star', tone: 'text-badge-gold' };
    case 'ForkEvent':
      return { icon: 'git-branch', tone: 'text-gm-accent-cyan' };
    case 'WatchEvent':
      return { icon: 'star', tone: 'text-badge-gold' };
    case 'CreateEvent':
      return { icon: 'plus', tone: 'text-gm-success' };
    case 'DeleteEvent':
      return { icon: 'trash', tone: 'text-gm-error' };
    default:
      return { icon: 'github', tone: 'text-dt-text-sub' };
  }
}

/**
 * Strip the `refs/heads/` / `refs/tags/` prefix that the events API returns.
 * Falls back to the original string when the prefix is unknown so we never
 * silently drop information.
 */
function shortRef(ref: string): string {
  if (ref.startsWith('refs/heads/')) return ref.slice('refs/heads/'.length);
  if (ref.startsWith('refs/tags/')) return ref.slice('refs/tags/'.length);
  return ref;
}

/**
 * Build the human-readable description for a single activity row.
 *
 * Returns plain text — the JSX wraps it with the title element, so this
 * stays easy to unit-test.
 */
export function describeEvent(item: ActivityFeedItem): string {
  switch (item.eventType) {
    case 'PushEvent': {
      const branch = item.refName ? shortRef(item.refName) : 'unknown';
      const count = item.commitsCount ?? 0;
      if (count === 1) return `${branch} に 1 件のコミットをプッシュ`;
      return `${branch} に ${count} 件のコミットをプッシュ`;
    }
    case 'PullRequestEvent': {
      const verb = (() => {
        switch (item.action) {
          case 'opened':
            return 'オープン';
          case 'closed':
            return 'クローズ';
          case 'reopened':
            return '再オープン';
          case 'edited':
            return '編集';
          default:
            return item.action ?? '更新';
        }
      })();
      return `Pull Request を${verb}`;
    }
    case 'PullRequestReviewEvent':
      return 'Pull Request をレビュー';
    case 'PullRequestReviewCommentEvent':
      return 'レビューコメントを投稿';
    case 'IssuesEvent': {
      const verb = (() => {
        switch (item.action) {
          case 'opened':
            return 'オープン';
          case 'closed':
            return 'クローズ';
          case 'reopened':
            return '再オープン';
          default:
            return item.action ?? '更新';
        }
      })();
      return `Issue を${verb}`;
    }
    case 'IssueCommentEvent':
      return 'Issue にコメント';
    case 'ReleaseEvent':
      return 'リリースを公開';
    case 'CreateEvent': {
      const what = item.refType ?? 'something';
      return `${what} を作成${item.refName ? `: ${shortRef(item.refName)}` : ''}`;
    }
    case 'DeleteEvent': {
      const what = item.refType ?? 'something';
      return `${what} を削除${item.refName ? `: ${shortRef(item.refName)}` : ''}`;
    }
    case 'ForkEvent':
      return 'リポジトリをフォーク';
    case 'WatchEvent':
      return 'リポジトリにスターを付けた';
    case 'PublicEvent':
      return 'リポジトリを公開';
    case 'MemberEvent':
      return 'コラボレーターの更新';
    default:
      return item.eventType;
  }
}

/**
 * "5分前" / "2時間前" / fallback to localized date.
 *
 * Identical contract to the formatter in NotificationsDropdown — kept local
 * to avoid creating a tiny shared util before there's a third caller.
 */
function formatRelativeTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;

  // Clamp negative deltas to 0 so client clock skew (event timestamp from
  // GitHub vs. local `Date.now()`) never produces strings like "-1秒前".
  const diffMs = Math.max(0, Date.now() - date.getTime());
  const diffSec = Math.floor(diffMs / 1000);
  if (diffSec < 60) return `${diffSec}秒前`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}分前`;
  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return `${diffHour}時間前`;
  const diffDay = Math.floor(diffHour / 24);
  if (diffDay < 7) return `${diffDay}日前`;
  return date.toLocaleDateString('ja-JP');
}

/**
 * Whitelist GitHub-only `https` URLs before handing them to the system
 * browser. The activity feed payload is server-controlled, but defense in
 * depth: if a future event type leaks a non-GitHub URL we'd rather no-op
 * than open it.
 */
function isSafeGitHubUrl(raw: string): boolean {
  try {
    const u = new URL(raw);
    if (u.protocol !== 'https:') return false;
    const host = u.hostname.toLowerCase();
    return host === 'github.com' || host.endsWith('.github.com');
  } catch {
    return false;
  }
}

const handleOpen = async (url: string | null) => {
  if (!url) return;
  if (!isSafeGitHubUrl(url)) {
    console.warn('Refusing to open non-GitHub URL from activity feed:', url);
    return;
  }
  try {
    await settingsApi.openExternalUrl(url);
  } catch (e) {
    console.error('Failed to open URL', e);
  }
};

const TimelineSkeleton: React.FC = () => (
  <ul className="divide-y divide-slate-800 animate-pulse">
    {Array.from({ length: 5 }).map((_, i) => (
      <li key={i} className="flex items-start gap-3 px-4 py-3">
        <div className="w-5 h-5 bg-slate-700 rounded-full mt-1" />
        <div className="flex-1 space-y-2">
          <div className="h-4 w-2/3 bg-slate-700 rounded" />
          <div className="h-3 w-1/2 bg-slate-700 rounded" />
        </div>
      </li>
    ))}
  </ul>
);

interface ActivityRowProps {
  item: ActivityFeedItem;
}

const ActivityRow: React.FC<ActivityRowProps> = ({ item }) => {
  const visual = getEventVisual(item.eventType, item.action);
  const description = describeEvent(item);
  const target = item.targetUrl ?? item.repoUrl;

  return (
    <li className="border-b border-slate-800 last:border-b-0">
      <div className="flex items-start gap-3 px-4 py-3 hover:bg-slate-800/40 transition-colors">
        <Icon
          name={visual.icon}
          className={`w-5 h-5 mt-0.5 flex-shrink-0 ${visual.tone}`}
        />
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-0.5 text-xs">
            <button
              type="button"
              onClick={() => void handleOpen(item.repoUrl)}
              className="text-slate-400 hover:text-gm-accent-cyan font-medium truncate transition-colors"
            >
              {item.repoName}
            </button>
            <span className="text-slate-600">·</span>
            <span className="text-slate-500" title={item.createdAt}>
              {formatRelativeTime(item.createdAt)}
            </span>
          </div>
          <div className="text-sm text-dt-text-sub">{description}</div>
          {item.title && (
            <button
              type="button"
              onClick={() => void handleOpen(target)}
              className="block w-full text-left mt-0.5 text-sm text-dt-text hover:text-gm-accent-cyan truncate transition-colors"
            >
              {item.number !== null ? (
                <span className="text-slate-500 mr-1">#{item.number}</span>
              ) : null}
              {item.title}
            </button>
          )}
        </div>
      </div>
    </li>
  );
};

export const ActivityTimeline: React.FC<ActivityTimelineProps> = ({
  items,
  isLoading = false,
  isRevalidating = false,
  error,
  onRetry,
}) => {
  // Defensive de-dupe: GitHub occasionally repeats event ids across pages
  // when activity straddles a refresh window. Render each id only once.
  const dedupedItems = useMemo(() => {
    if (!items) return null;
    const seen = new Set<string>();
    return items.filter(item => {
      if (seen.has(item.id)) return false;
      seen.add(item.id);
      return true;
    });
  }, [items]);

  return (
    <div className="bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-purple/20 overflow-hidden">
      <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800">
        <h3 className="text-xl font-gaming font-bold text-gm-accent-purple flex items-center gap-2">
          <Icon name="clock" className="w-5 h-5" />
          最近のアクティビティ
        </h3>
        {isRevalidating && (
          <span className="text-xs text-slate-500 flex items-center gap-1">
            <Icon name="refresh-cw" className="w-3 h-3 animate-spin" />
            更新中
          </span>
        )}
      </div>

      <div className="max-h-[480px] overflow-y-auto">
        {error && (
          <div className="px-6 py-4 text-sm text-red-400 bg-red-500/10 border-b border-red-500/20 flex items-center justify-between">
            <span>アクティビティを取得できませんでした: {error.message}</span>
            {onRetry && (
              <button
                type="button"
                onClick={onRetry}
                className="text-xs px-2 py-1 rounded bg-red-500/20 hover:bg-red-500/30 transition-colors"
              >
                再試行
              </button>
            )}
          </div>
        )}

        {isLoading && !dedupedItems && <TimelineSkeleton />}

        {dedupedItems && dedupedItems.length === 0 && !error && (
          <div className="px-6 py-12 text-center text-sm text-slate-400">
            最近のアクティビティはありません
          </div>
        )}

        {dedupedItems && dedupedItems.length > 0 && (
          <ul role="list" className="divide-y divide-slate-800">
            {dedupedItems.map(item => (
              <ActivityRow key={item.id} item={item} />
            ))}
          </ul>
        )}
      </div>
    </div>
  );
};

export default ActivityTimeline;
