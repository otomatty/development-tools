/**
 * Notifications Button
 *
 * Sidebar bell + unread badge that opens the notifications dropdown.
 * Subscribes to the backend's `notifications-updated` event so it stays
 * fresh without manual polling from the UI.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/186
 *   - Backend: src-tauri/src/commands/notifications.rs
 *   - Store: src/stores/notificationsStore.ts
 */

import { useEffect, useRef, useState } from 'react';
import { Icon } from '@/components/icons';
import { useNotifications } from '@/stores/notificationsStore';
import { useAuth } from '@/stores/authStore';
import { events } from '@/lib/tauri/events';
import { NotificationsDropdown } from './NotificationsDropdown';

export const NotificationsButton = () => {
  const isLoggedIn = useAuth((s) => s.state.isLoggedIn);
  const unreadCount = useNotifications((s) => s.unreadCount);
  const fetchNotifications = useNotifications((s) => s.fetch);
  const setFromEvent = useNotifications((s) => s.setFromEvent);
  const resetNotifications = useNotifications((s) => s.reset);
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Initial load + subscribe to backend pushes. Only run when logged in —
  // the store already short-circuits when logged out, but this also avoids
  // wiring up event listeners we won't use.
  useEffect(() => {
    if (!isLoggedIn) {
      // Wipe local state so account switches don't briefly show the
      // previous user's unread badge / dropdown contents before the
      // next fetch lands.
      resetNotifications();
      return;
    }

    void fetchNotifications();

    // The `disposed` flag protects against the cleanup running before
    // the listener Promise resolves: without it, an unmount during the
    // brief async window would leak the Tauri listener and re-mounting
    // (e.g. on re-login) would attach a duplicate, double-firing
    // `fetchNotifications` for every backend push.
    let disposed = false;
    let unlistenFn: (() => void) | null = null;
    void events
      .onNotificationsUpdated((event) => {
        // Drop events whose `userId` doesn't match the currently
        // logged-in user — the scheduler captures user.id *before* the
        // GitHub round-trip, so an account switch mid-flight produces
        // an event with the previous user's data. Applying it would
        // leak repo titles / unread counts across accounts.
        const currentUserId = useAuth.getState().state.user?.id;
        if (currentUserId === undefined || event.userId !== currentUserId) {
          return;
        }
        // Apply the items from the event payload directly. A re-fetch
        // would race the just-persisted ETag and come back as 304,
        // leaving the UI showing the pre-event list.
        setFromEvent(event.items, event.unreadCount);
      })
      .then((unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        unlistenFn = unlisten;
      });

    return () => {
      disposed = true;
      if (unlistenFn) unlistenFn();
    };
  }, [isLoggedIn, fetchNotifications, setFromEvent, resetNotifications]);

  // Close the dropdown when clicking outside.
  useEffect(() => {
    if (!isOpen) return;

    const handleClick = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [isOpen]);

  if (!isLoggedIn) return null;

  // GitHub caps the unread count display at 99+ in the web UI; we mirror
  // that to keep the badge from blowing past two characters.
  const badgeLabel = unreadCount > 99 ? '99+' : unreadCount.toString();

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setIsOpen((open) => !open)}
        aria-label={
          unreadCount > 0
            ? `通知 (${unreadCount}件未読)`
            : '通知'
        }
        aria-expanded={isOpen}
        className={`relative p-2 rounded-lg transition-all duration-200 ${
          isOpen
            ? 'bg-gm-accent-cyan/20 text-gm-accent-cyan'
            : 'text-slate-400 hover:bg-slate-800 hover:text-dt-text'
        }`}
        title="通知"
      >
        <Icon name="bell" className="w-5 h-5" />
        {unreadCount > 0 && (
          <span
            className="absolute -top-1 -right-1 min-w-[18px] h-[18px] px-1 rounded-full bg-red-500 text-white text-[10px] font-bold flex items-center justify-center"
            aria-hidden="true"
          >
            {badgeLabel}
          </span>
        )}
      </button>
      {isOpen && <NotificationsDropdown onClose={() => setIsOpen(false)} />}
    </div>
  );
};
