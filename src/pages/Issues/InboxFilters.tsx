/**
 * InboxFilters Component
 *
 * Filter controls for the cross-repository "Today / Inbox" view: free-text
 * search, repository selector, and priority selector. Stateless — the parent
 * owns the filter values.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/183
 */

import { Icon } from '@/components/icons';
import type { IssuePriority } from '@/types';

export type SortKey = 'priority' | 'updated' | 'created';

interface InboxFiltersProps {
  searchText: string;
  onSearchChange: (value: string) => void;
  repoOptions: string[];
  selectedRepo: string;
  onRepoChange: (value: string) => void;
  selectedPriority: IssuePriority | 'all';
  onPriorityChange: (value: IssuePriority | 'all') => void;
  sortKey: SortKey;
  onSortChange: (value: SortKey) => void;
}

const selectClass =
  'bg-gm-bg-secondary border border-slate-600 text-dt-text text-sm rounded-lg px-3 py-1.5 focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan';

export const InboxFilters = ({
  searchText,
  onSearchChange,
  repoOptions,
  selectedRepo,
  onRepoChange,
  selectedPriority,
  onPriorityChange,
  sortKey,
  onSortChange,
}: InboxFiltersProps) => {
  return (
    <div className="flex flex-wrap items-center gap-3 mb-4">
      <label className="relative flex-1 min-w-[200px]">
        <span className="sr-only">Search</span>
        <span className="absolute left-3 top-1/2 -translate-y-1/2 text-dt-text-sub pointer-events-none">
          <Icon name="search" className="w-4 h-4" />
        </span>
        <input
          type="search"
          value={searchText}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder="タイトル・リポジトリ・ラベルで絞り込み"
          className="w-full bg-gm-bg-secondary border border-slate-600 text-dt-text text-sm rounded-lg pl-9 pr-3 py-1.5 focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan"
        />
      </label>

      <select
        aria-label="Repository"
        className={selectClass}
        value={selectedRepo}
        onChange={(e) => onRepoChange(e.target.value)}
      >
        <option value="all">すべてのリポジトリ</option>
        {repoOptions.map((repo) => (
          <option key={repo} value={repo}>
            {repo}
          </option>
        ))}
      </select>

      <select
        aria-label="Priority"
        className={selectClass}
        value={selectedPriority}
        onChange={(e) => onPriorityChange(e.target.value as IssuePriority | 'all')}
      >
        <option value="all">すべての優先度</option>
        <option value="high">優先度: High</option>
        <option value="medium">優先度: Medium</option>
        <option value="low">優先度: Low</option>
      </select>

      <select
        aria-label="Sort"
        className={selectClass}
        value={sortKey}
        onChange={(e) => onSortChange(e.target.value as SortKey)}
      >
        <option value="priority">優先度順</option>
        <option value="updated">更新が新しい順</option>
        <option value="created">作成が新しい順</option>
      </select>
    </div>
  );
};
