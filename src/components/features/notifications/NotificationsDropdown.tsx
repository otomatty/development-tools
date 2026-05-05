/**
 * Notifications Dropdown
 *
 * List of GitHub notifications shown when the sidebar bell is clicked.
 * Each row links out to the underlying issue / PR and marks the thread
 * as read on click.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/186
 *   - Store: src/stores/notificationsStore.ts
 */

import { Icon } from '@/components/icons';
import { useNotifications } from '@/stores/notificationsStore';
import { settings as settingsApi } from '@/lib/tauri/commands';
import {
  notificationReasonLabel,
  type NotificationItem,
} from '@/types';

interface NotificationsDropdownProps {
  onClose: () => void;
}

export const NotificationsDropdown = ({ onClose }: NotificationsDropdownProps) => {
  const items = useNotifications((s) => s.items);
  const isLoading = useNotifications((s) => s.isLoading);
  const error = useNotifications((s) => s.error);
  const refresh = useNotifications((s) => s.fetch);
  const markRead = useNotifications((s) => s.markRead);

  const handleOpen = async (item: NotificationItem) => {
    // Open the URL first; only mark as read if the user actually got to
    // the issue/PR. If `openExternalUrl` rejects (deny / no browser),
    // leaving the unread state intact lets the user retry without
    // losing track of the item.
    try {
      await settingsApi.openExternalUrl(item.htmlUrl);
    } catch (e) {
      console.error('Failed to open notification URL', e);
      return;
    }
    if (item.unread) {
      void markRead(item.id);
    }
    onClose();
  };

  return (
    <div
      role="dialog"
      aria-label="通知一覧"
      className="absolute bottom-full mb-2 right-0 w-96 max-h-[480px] bg-slate-900 border border-slate-700 rounded-lg shadow-xl overflow-hidden flex flex-col z-50"
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-slate-700">
        <h3 className="text-sm font-semibold text-dt-text">通知</h3>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={isLoading}
          aria-label="再読み込み"
          className="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-dt-text transition-colors disabled:opacity-50"
        >
          <Icon
            name="refresh-cw"
            className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`}
          />
        </button>
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto">
        {error && (
          <div className="px-4 py-3 text-xs text-red-400 bg-red-500/10 border-b border-red-500/20">
            {error}
          </div>
        )}

        {!isLoading && items.length === 0 && !error && (
          <div className="px-4 py-8 text-center text-sm text-slate-400">
            通知はありません
          </div>
        )}

        {items.length > 0 && (
          <ul className="divide-y divide-slate-800">
            {items.map((item) => (
              <li key={item.id}>
                <button
                  type="button"
                  onClick={() => void handleOpen(item)}
                  className={`w-full text-left px-4 py-3 hover:bg-slate-800 transition-colors ${
                    item.unread ? 'bg-slate-800/40' : ''
                  }`}
                >
                  <div className="flex items-start gap-2">
                    {item.unread && (
                      <span
                        aria-hidden="true"
                        className="mt-1.5 w-2 h-2 rounded-full bg-gm-accent-cyan flex-shrink-0"
                      />
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-[10px] uppercase tracking-wide text-gm-accent-purple font-semibold">
                          {notificationReasonLabel(item.reason)}
                        </span>
                        <span className="text-[10px] text-slate-500 truncate">
                          {item.repoFullName}
                        </span>
                      </div>
                      <div
                        className={`text-sm truncate ${
                          item.unread ? 'text-dt-text font-medium' : 'text-slate-400'
                        }`}
                      >
                        {item.title}
                      </div>
                      <div className="text-[11px] text-slate-500 mt-0.5">
                        {formatRelativeTime(item.updatedAt)}
                      </div>
                    </div>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
};

/**
 * Format an ISO8601 timestamp as "5分前" / "2時間前" / etc.
 *
 * Falls back to a localized date when the timestamp is older than a week —
 * the user can hover the date for the precise time anyway.
 */
function formatRelativeTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;

  const diffMs = Date.now() - date.getTime();
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
