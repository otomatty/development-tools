/**
 * Issues Page — Cross-repository "Today / Inbox"
 *
 * First view shown on the Issues tab. Lists the two things the user has to
 * deal with right now:
 *
 *   1. Open Issues assigned to them (across every repo the token can see)
 *   2. PRs where they are requested as a reviewer
 *
 * Data is fetched via `get_my_open_work_with_cache`, which wraps GitHub's
 * Search API with a 5-minute SQLite cache. The hook layer (`useCachedFetch`)
 * surfaces the cache-state to this page so we can render the cached payload
 * immediately and revalidate on focus / reconnect.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/183
 *   - Backend: src-tauri/src/commands/issues.rs (get_my_open_work_with_cache)
 */

import { useCallback, useMemo, useState } from 'react';
import { useAuth } from '../../stores/authStore';
import { useCachedFetch } from '../../hooks/useCachedFetch';
import { issues } from '../../lib/tauri/commands';
import {
  priorityWeight,
  type IssuePriority,
  type MyOpenWork,
  type MyOpenWorkItem,
  type MyOpenWorkSource,
} from '../../types';
import { CacheStatusBanner } from '../Home/CacheStatusBanner';
import { InboxFilters, type SortKey } from './InboxFilters';
import { InboxItemRow } from './InboxItemRow';

const STALE_TIME_MS = 5 * 60 * 1000; // 5 minutes — matches backend TTL

const EMPTY_PAYLOAD: MyOpenWork = { assigned: [], reviewRequested: [] };

const TabButton = ({
  active,
  count,
  label,
  onClick,
}: {
  active: boolean;
  count: number;
  label: string;
  onClick: () => void;
}) => (
  <button
    type="button"
    onClick={onClick}
    className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
      active
        ? 'bg-gm-accent-cyan/20 text-gm-accent-cyan border border-gm-accent-cyan/40'
        : 'text-dt-text-sub border border-transparent hover:bg-slate-800'
    }`}
    aria-pressed={active}
  >
    {label}
    <span className="ml-2 text-xs opacity-80">{count}</span>
  </button>
);

const SkeletonRow = () => (
  <div className="flex items-center gap-3 px-4 py-3 border-b border-slate-700/50">
    <div className="w-4 h-4 rounded-full bg-slate-700 animate-pulse" />
    <div className="flex-1 space-y-2">
      <div className="h-3 bg-slate-700 rounded w-1/3 animate-pulse" />
      <div className="h-4 bg-slate-700 rounded w-3/4 animate-pulse" />
    </div>
  </div>
);

const SkeletonList = () => (
  <div className="rounded-2xl border border-slate-700 bg-gm-bg-secondary overflow-hidden">
    <SkeletonRow />
    <SkeletonRow />
    <SkeletonRow />
    <SkeletonRow />
  </div>
);

/** Apply free-text + repo + priority filters and the chosen sort. */
function filterAndSort(
  items: MyOpenWorkItem[],
  searchText: string,
  selectedRepo: string,
  selectedPriority: IssuePriority | 'all',
  sortKey: SortKey,
): MyOpenWorkItem[] {
  const needle = searchText.trim().toLowerCase();

  const filtered = items.filter((item) => {
    if (selectedRepo !== 'all' && item.repoFullName !== selectedRepo) {
      return false;
    }
    if (selectedPriority !== 'all' && item.priority !== selectedPriority) {
      return false;
    }
    if (needle.length > 0) {
      // Check fields individually so we short-circuit on the first match
      // and avoid allocating a joined haystack string per item.
      const inTitle = item.title.toLowerCase().includes(needle);
      const inRepo = item.repoFullName.toLowerCase().includes(needle);
      if (!inTitle && !inRepo) {
        const inLabels = item.labels.some((l) =>
          l.toLowerCase().includes(needle),
        );
        if (!inLabels) return false;
      }
    }
    return true;
  });

  const sorted = [...filtered];
  switch (sortKey) {
    case 'priority':
      sorted.sort((a, b) => {
        const diff = priorityWeight(b.priority) - priorityWeight(a.priority);
        if (diff !== 0) return diff;
        // Tie-break: most recently updated first, so an unprioritized item
        // touched today doesn't end up below an unprioritized item from
        // last year.
        return b.updatedAt.localeCompare(a.updatedAt);
      });
      break;
    case 'updated':
      sorted.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
      break;
    case 'created':
      sorted.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
      break;
  }
  return sorted;
}

const Issues = () => {
  const isLoggedIn = useAuth((s) => s.state.isLoggedIn);
  const authLoading = useAuth((s) => s.isLoading);
  const enabled = !authLoading && isLoggedIn;

  const query = useCachedFetch(issues.getMyOpenWorkWithCache, {
    enabled,
    staleTime: STALE_TIME_MS,
  });

  const [activeTab, setActiveTab] = useState<MyOpenWorkSource>('assigned');
  const [searchText, setSearchText] = useState('');
  const [selectedRepo, setSelectedRepo] = useState<string>('all');
  const [selectedPriority, setSelectedPriority] = useState<IssuePriority | 'all'>(
    'all',
  );
  const [sortKey, setSortKey] = useState<SortKey>('priority');

  const data = query.data ?? EMPTY_PAYLOAD;
  const activeItems =
    activeTab === 'assigned' ? data.assigned : data.reviewRequested;

  const repoOptions = useMemo(() => {
    const set = new Set<string>();
    for (const item of [...data.assigned, ...data.reviewRequested]) {
      if (item.repoFullName) set.add(item.repoFullName);
    }
    return Array.from(set).sort();
  }, [data.assigned, data.reviewRequested]);

  const visibleItems = useMemo(
    () =>
      filterAndSort(
        activeItems,
        searchText,
        selectedRepo,
        selectedPriority,
        sortKey,
      ),
    [activeItems, searchText, selectedRepo, selectedPriority, sortKey],
  );

  const handleRetry = useCallback(() => {
    void query.revalidate();
  }, [query]);

  const initialLoading = enabled && query.data === null && query.error === null;

  if (!enabled && !authLoading) {
    return (
      <div className="p-8">
        <h1 className="text-3xl font-bold text-gm-accent-cyan mb-2">Issues</h1>
        <p className="text-dt-text-sub">
          GitHub にログインすると、アサインされた Issue とレビュー依頼を一覧表示できます。
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <header className="mb-6">
        <h1 className="text-3xl font-bold text-gm-accent-cyan">
          Today / Inbox
        </h1>
        <p className="mt-1 text-sm text-dt-text-sub">
          自分にアサインされた Open Issue と、レビュー依頼が来ている Pull Request を横断表示します。
        </p>
      </header>

      <CacheStatusBanner
        fromCache={query.fromCache}
        hasError={query.error !== null}
        hasData={query.data !== null}
        isRevalidating={query.isRevalidating}
        cachedAt={query.cachedAt}
        onRetry={handleRetry}
      />

      <div className="flex items-center gap-2 mb-4">
        <TabButton
          active={activeTab === 'assigned'}
          count={data.assigned.length}
          label="アサインされた Issue"
          onClick={() => setActiveTab('assigned')}
        />
        <TabButton
          active={activeTab === 'review_requested'}
          count={data.reviewRequested.length}
          label="レビュー依頼"
          onClick={() => setActiveTab('review_requested')}
        />
      </div>

      <InboxFilters
        searchText={searchText}
        onSearchChange={setSearchText}
        repoOptions={repoOptions}
        selectedRepo={selectedRepo}
        onRepoChange={setSelectedRepo}
        selectedPriority={selectedPriority}
        onPriorityChange={setSelectedPriority}
        sortKey={sortKey}
        onSortChange={setSortKey}
      />

      {initialLoading ? (
        <SkeletonList />
      ) : query.error !== null && query.data === null ? (
        // Fetch failed and we have no cached payload to fall back on. We
        // must NOT render the "no issues" copy here — the user's actual
        // workload is unknown, and a fake-empty inbox could cause them to
        // miss assigned items. The CacheStatusBanner above carries the
        // human-readable error; this block adds an explicit retry CTA.
        <div className="rounded-2xl border border-red-500/30 bg-red-500/5 p-8 text-center">
          <p className="text-red-300 text-sm mb-3">
            Inbox の取得に失敗しました。GitHub に接続できないか、Search API のレート制限に達した可能性があります。
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
      ) : visibleItems.length === 0 ? (
        <div className="rounded-2xl border border-slate-700 bg-gm-bg-secondary p-8 text-center text-dt-text-sub">
          {activeItems.length === 0
            ? activeTab === 'assigned'
              ? 'アサインされた Open Issue はありません。'
              : 'レビュー依頼はありません。'
            : '条件に一致する項目はありません。'}
        </div>
      ) : (
        <ul className="rounded-2xl border border-slate-700 bg-gm-bg-secondary overflow-hidden divide-y-0">
          {visibleItems.map((item) => (
            <InboxItemRow key={`${item.source}-${item.id}`} item={item} />
          ))}
        </ul>
      )}
    </div>
  );
};

export default Issues;
