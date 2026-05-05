/**
 * Notifications Store
 *
 * Holds the GitHub Notifications list and unread count, refreshed via
 * `notifications.list()` and pushed to via the `notifications-updated`
 * Tauri event.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/186
 *   - Backend: src-tauri/src/commands/notifications.rs
 *   - Tauri API: src/lib/tauri/commands.ts (notifications)
 */

import { create } from 'zustand';
import type { NotificationItem } from '@/types';
import { notifications as notificationsApi } from '@/lib/tauri/commands';
import { useAuth } from './authStore';

interface NotificationsStore {
  items: NotificationItem[];
  unreadCount: number;
  isLoading: boolean;
  error: string | null;
  /** ISO8601 timestamp of the last successful refresh. */
  lastFetchedAt: string | null;
  /**
   * Monotonic counter bumped on every state-mutating action (fetch start,
   * setFromEvent, reset, markRead). In-flight fetches capture this on
   * launch and discard their result if it changed while they were
   * awaiting the API — this prevents user A's response from being
   * written back into the store after user A logged out and user B
   * already started a new session.
   */
  sessionGen: number;

  fetch: () => Promise<void>;
  /** Replace the list directly (used by the `notifications-updated` event). */
  setFromEvent: (items: NotificationItem[], unreadCount: number) => void;
  markRead: (threadId: string) => Promise<void>;
  /**
   * Wipe local state. Called when the user logs out so the next account
   * doesn't see the previous user's unread count or repo titles in the
   * brief window before the first authenticated fetch completes.
   */
  reset: () => void;
}

export const useNotifications = create<NotificationsStore>((set, get) => ({
  items: [],
  unreadCount: 0,
  isLoading: false,
  error: null,
  lastFetchedAt: null,
  sessionGen: 0,

  fetch: async () => {
    // Skip when logged out — the backend would return "Not logged in" and
    // clutter the error state. Stays silent so the bell badge can render
    // 0 without flashing an error.
    if (!useAuth.getState().state.isLoggedIn) return;

    // Capture the session generation at launch. If `reset()` (logout) or
    // a `setFromEvent` push runs while we're awaiting the API, the
    // captured value will mismatch on resume and we drop the response —
    // otherwise an in-flight fetch from user A could overwrite user B's
    // empty state after an account switch.
    const launchGen = get().sessionGen;
    set({ isLoading: true, error: null });
    try {
      const payload = await notificationsApi.list();
      if (get().sessionGen !== launchGen) {
        // Stale: newer data is already in the store. Clear `isLoading`
        // so the spinner stops — `setFromEvent` / `markRead` only bump
        // `sessionGen`, they don't reset `isLoading`.
        set({ isLoading: false });
        return;
      }
      set({
        items: payload.items,
        unreadCount: payload.unreadCount,
        isLoading: false,
        lastFetchedAt: new Date().toISOString(),
      });
    } catch (e) {
      if (get().sessionGen !== launchGen) {
        set({ isLoading: false });
        return;
      }
      set({
        isLoading: false,
        error: typeof e === 'string' ? e : (e as Error).message ?? 'Failed to fetch notifications',
      });
    }
  },

  setFromEvent: (items, unreadCount) => {
    set((s) => ({
      items,
      unreadCount,
      error: null,
      lastFetchedAt: new Date().toISOString(),
      sessionGen: s.sessionGen + 1,
    }));
  },

  reset: () => {
    // Bumping `sessionGen` here is what closes the cross-account leak —
    // any fetch in flight at logout time will compare against the new
    // value on resume and drop its response.
    set((s) => ({
      items: [],
      unreadCount: 0,
      isLoading: false,
      error: null,
      lastFetchedAt: null,
      sessionGen: s.sessionGen + 1,
    }));
  },

  markRead: async (threadId: string) => {
    // Optimistic update: mark the row read locally before the API call so
    // the bell badge reacts instantly.
    const prev = get().items;
    const updated = prev.map((n) => (n.id === threadId ? { ...n, unread: false } : n));
    set((s) => ({
      items: updated,
      unreadCount: updated.filter((n) => n.unread).length,
      sessionGen: s.sessionGen + 1,
    }));

    try {
      await notificationsApi.markRead(threadId);
    } catch (e) {
      // Don't blanket-restore `prev` — a concurrent successful fetch may
      // have replaced the list with newer data, and rolling back to the
      // pre-click snapshot would clobber it. Surface the error and let a
      // re-fetch reconcile the truth from the backend (which is also
      // serving from its own cache, so this is cheap).
      set({
        error: typeof e === 'string' ? e : (e as Error).message ?? 'Failed to mark as read',
      });
      void get().fetch();
    }
  },
}));
