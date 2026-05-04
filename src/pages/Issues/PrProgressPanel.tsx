/**
 * PrProgressPanel Component
 *
 * Dashboard panel listing the user's open PRs with mergeable / checks /
 * reviewDecision state. Lives on the Issues page above the Today / Inbox
 * list so the user can triage their own work and review queue in one view.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/185
 *   - Backend: src-tauri/src/commands/issues.rs (get_my_pr_progress_with_cache)
 */

import { useCallback } from 'react';
import { Icon } from '@/components/icons';
import { useCachedFetch } from '@/hooks/useCachedFetch';
import { auth, issues } from '@/lib/tauri/commands';
import type {
  PrChecksState,
  PrMergeable,
  PrProgress,
  PrProgressItem,
  PrReviewDecision,
} from '@/types';

const STALE_TIME_MS = 5 * 60 * 1000; // 5 minutes — matches backend TTL

const EMPTY_PAYLOAD: PrProgress = {
  items: [],
  totalCount: 0,
  truncated: false,
};

interface PrProgressPanelProps {
  /** Disable fetching while auth is unresolved. */
  enabled: boolean;
}

interface BadgeSpec {
  label: string;
  className: string;
  icon: string;
}

function checksBadge(state: PrChecksState | string | null): BadgeSpec | null {
  if (state === null) return null;
  switch (state) {
    case 'SUCCESS':
      return {
        label: 'CI 成功',
        className: 'text-green-300 bg-green-500/10 border-green-500/30',
        icon: 'check',
      };
    case 'FAILURE':
    case 'ERROR':
      return {
        label: 'CI 失敗',
        className: 'text-red-300 bg-red-500/10 border-red-500/40',
        icon: 'x',
      };
    case 'PENDING':
    case 'EXPECTED':
      return {
        label: 'CI 実行中',
        className: 'text-yellow-300 bg-yellow-500/10 border-yellow-500/30',
        icon: 'clock',
      };
    default:
      return {
        label: `CI ${state}`,
        className: 'text-dt-text-sub bg-slate-700/40 border-slate-600/40',
        icon: 'alert-circle',
      };
  }
}

function reviewBadge(
  decision: PrReviewDecision | string | null,
): BadgeSpec | null {
  if (decision === null) return null;
  switch (decision) {
    case 'APPROVED':
      return {
        label: 'レビュー OK',
        className: 'text-green-300 bg-green-500/10 border-green-500/30',
        icon: 'check-square',
      };
    case 'CHANGES_REQUESTED':
      return {
        label: '修正依頼',
        className: 'text-orange-300 bg-orange-500/10 border-orange-500/40',
        icon: 'alert-triangle',
      };
    case 'REVIEW_REQUIRED':
      return {
        label: 'レビュー待ち',
        className: 'text-blue-300 bg-blue-500/10 border-blue-500/30',
        icon: 'user',
      };
    default:
      return {
        label: decision,
        className: 'text-dt-text-sub bg-slate-700/40 border-slate-600/40',
        icon: 'info',
      };
  }
}

function mergeableBadge(state: PrMergeable | string): BadgeSpec | null {
  switch (state) {
    case 'CONFLICTING':
      return {
        label: 'コンフリクト',
        className: 'text-red-300 bg-red-500/10 border-red-500/40',
        icon: 'alert-circle',
      };
    case 'MERGEABLE':
      // Only emphasize the success path when there's something to communicate
      // beyond "the default". Showing this for every PR adds noise; we leave
      // it implicit and let CI / review badges carry the signal.
      return null;
    case 'UNKNOWN':
    default:
      // GitHub computes mergeable lazily — `UNKNOWN` is normal for fresh PRs
      // and would be misleading to flag as a problem.
      return null;
  }
}

function formatRelative(iso: string): string {
  const ms = new Date(iso).getTime();
  if (Number.isNaN(ms)) return '';
  const diffMs = Date.now() - ms;
  const minutes = Math.floor(diffMs / 60_000);
  if (minutes < 1) return 'たった今';
  if (minutes < 60) return `${minutes}分前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}時間前`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}日前`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}ヶ月前`;
  const years = Math.floor(days / 365);
  return `${years}年前`;
}

const Badge = ({ spec }: { spec: BadgeSpec }) => (
  <span
    className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded border text-[11px] ${spec.className}`}
  >
    <Icon name={spec.icon} className="w-3 h-3" />
    {spec.label}
  </span>
);

const PrRow = ({ item }: { item: PrProgressItem }) => {
  const onOpen = () => {
    auth.openUrl(item.url).catch((err) => {
      console.error('Failed to open URL:', err);
    });
  };

  const checks = checksBadge(item.checksState);
  const review = reviewBadge(item.reviewDecision);
  const mergeable = mergeableBadge(item.mergeable);

  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        className="w-full text-left flex items-start gap-3 px-4 py-3 hover:bg-slate-800/60 transition-colors border-b border-slate-700/50 last:border-b-0"
      >
        <span
          className="flex-shrink-0 mt-0.5 text-gm-accent-cyan"
          aria-label={item.isDraft ? 'Draft Pull Request' : 'Pull Request'}
        >
          <Icon
            name={item.isDraft ? 'git-pull-request-closed' : 'git-pull-request'}
            className="w-4 h-4"
          />
        </span>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 text-xs text-dt-text-sub mb-1">
            <span className="font-mono truncate">{item.repoFullName}</span>
            <span>·</span>
            <span>#{item.number}</span>
            {item.isDraft && (
              <>
                <span>·</span>
                <span className="text-dt-text-sub">Draft</span>
              </>
            )}
          </div>

          <div className="text-sm text-dt-text font-medium truncate">
            {item.title}
          </div>

          <div className="flex flex-wrap items-center gap-2 mt-2 text-xs">
            {mergeable && <Badge spec={mergeable} />}
            {checks && <Badge spec={checks} />}
            {review && <Badge spec={review} />}
            <span className="text-dt-text-sub">
              更新: {formatRelative(item.updatedAt)}
            </span>
          </div>
        </div>

        <span className="flex-shrink-0 text-dt-text-sub mt-1">
          <Icon name="external-link" className="w-3.5 h-3.5" />
        </span>
      </button>
    </li>
  );
};

const PanelHeader = ({
  count,
  totalCount,
  truncated,
  isRevalidating,
  onRefresh,
}: {
  count: number;
  totalCount: number;
  truncated: boolean;
  isRevalidating: boolean;
  onRefresh: () => void;
}) => (
  <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700/50 bg-slate-800/40">
    <div className="flex items-center gap-2">
      <Icon name="git-pull-request" className="w-4 h-4 text-gm-accent-cyan" />
      <h2 className="text-sm font-semibold text-dt-text">自分の Open PR</h2>
      <span className="text-xs text-dt-text-sub">
        {truncated
          ? `${count} / ${totalCount}`
          : `${count}`}
      </span>
    </div>
    <button
      type="button"
      onClick={onRefresh}
      disabled={isRevalidating}
      className="inline-flex items-center gap-1 px-2 py-1 text-xs rounded border border-slate-600 text-dt-text-sub hover:bg-slate-700/40 disabled:opacity-50"
      aria-label="再取得"
    >
      <Icon
        name={isRevalidating ? 'loader' : 'refresh'}
        className={`w-3 h-3 ${isRevalidating ? 'animate-spin' : ''}`}
      />
      再取得
    </button>
  </div>
);

export const PrProgressPanel = ({ enabled }: PrProgressPanelProps) => {
  const query = useCachedFetch(issues.getMyPrProgressWithCache, {
    enabled,
    staleTime: STALE_TIME_MS,
  });

  const handleRetry = useCallback(() => {
    void query.revalidate();
  }, [query]);

  if (!enabled) return null;

  const data = query.data ?? EMPTY_PAYLOAD;
  const isInitialLoading = query.data === null && query.error === null;

  return (
    <section className="rounded-2xl border border-slate-700 bg-gm-bg-secondary overflow-hidden mb-6">
      <PanelHeader
        count={data.items.length}
        totalCount={data.totalCount}
        truncated={data.truncated}
        isRevalidating={query.isRevalidating}
        onRefresh={handleRetry}
      />

      {isInitialLoading ? (
        <div className="px-4 py-8 text-center text-sm text-dt-text-sub">
          PR 状況を取得中...
        </div>
      ) : query.error !== null && query.data === null ? (
        <div className="px-4 py-6 text-center">
          <p className="text-sm text-red-300 mb-3">
            PR 状況の取得に失敗しました。GitHub に接続できないか、レート制限に達した可能性があります。
          </p>
          <button
            type="button"
            onClick={handleRetry}
            disabled={query.isRevalidating}
            className="inline-flex items-center px-3 py-1.5 text-sm rounded-lg border border-red-500/40 text-red-200 hover:bg-red-500/10 disabled:opacity-50"
          >
            {query.isRevalidating ? '再試行中...' : '再試行'}
          </button>
        </div>
      ) : data.items.length === 0 ? (
        <div className="px-4 py-8 text-center text-sm text-dt-text-sub">
          現在 Open な Pull Request はありません。
        </div>
      ) : (
        <ul className="divide-y-0">
          {data.items.map((item) => (
            <PrRow key={item.id} item={item} />
          ))}
        </ul>
      )}

      {data.truncated && data.items.length > 0 && (
        <div className="px-4 py-2 text-[11px] text-dt-text-sub border-t border-slate-700/50 bg-slate-800/40">
          上位 {data.items.length} 件のみ表示しています（全 {data.totalCount} 件）。
        </div>
      )}
    </section>
  );
};
