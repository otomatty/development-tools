/**
 * InboxItemRow Component
 *
 * Single row in the cross-repository inbox list. Click opens the item on
 * GitHub in the system browser via the Tauri opener plugin.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/183
 */

import { Icon } from '@/components/icons';
import { auth } from '@/lib/tauri/commands';
import {
  issuePriorityColorClass,
  issuePriorityDisplayName,
  issuePriorityEmoji,
  type MyOpenWorkItem,
} from '@/types';

interface InboxItemRowProps {
  item: MyOpenWorkItem;
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

export const InboxItemRow = ({ item }: InboxItemRowProps) => {
  const isPr = item.kind === 'pull_request';
  const priorityClass = item.priority
    ? issuePriorityColorClass(item.priority)
    : 'text-dt-text-sub';
  const priorityLabel = item.priority
    ? `${issuePriorityEmoji(item.priority)} ${issuePriorityDisplayName(item.priority)}`
    : null;

  const onOpen = () => {
    // Best-effort: failure here usually means the user doesn't have a
    // browser configured. We swallow the error rather than throwing a
    // toast for every miss-click.
    auth.openUrl(item.htmlUrl).catch((err) => {
      console.error('Failed to open URL:', err);
    });
  };

  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        className="w-full text-left flex items-start gap-3 px-4 py-3 hover:bg-slate-800/60 transition-colors border-b border-slate-700/50 last:border-b-0"
      >
        <span
          className="flex-shrink-0 mt-0.5 text-gm-accent-cyan"
          aria-label={isPr ? 'Pull Request' : 'Issue'}
        >
          <Icon name={isPr ? 'git-branch' : 'circle'} className="w-4 h-4" />
        </span>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 text-xs text-dt-text-sub mb-1">
            <span className="font-mono truncate">{item.repoFullName}</span>
            <span>·</span>
            <span>#{item.number}</span>
            {priorityLabel && (
              <>
                <span>·</span>
                <span className={priorityClass}>{priorityLabel}</span>
              </>
            )}
          </div>

          <div className="text-sm text-dt-text font-medium truncate">
            {item.title}
          </div>

          <div className="flex flex-wrap items-center gap-2 mt-1 text-xs text-dt-text-sub">
            <span>更新: {formatRelative(item.updatedAt)}</span>
            {item.labels.length > 0 && (
              <span className="flex flex-wrap gap-1">
                {item.labels.slice(0, 4).map((label) => (
                  <span
                    key={label}
                    className="px-1.5 py-0.5 rounded bg-slate-700/60 text-[10px]"
                  >
                    {label}
                  </span>
                ))}
                {item.labels.length > 4 && (
                  <span className="text-[10px]">+{item.labels.length - 4}</span>
                )}
              </span>
            )}
          </div>
        </div>

        <span className="flex-shrink-0 text-dt-text-sub mt-1">
          <Icon name="external-link" className="w-3.5 h-3.5" />
        </span>
      </button>
    </li>
  );
};
