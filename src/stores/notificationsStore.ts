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

  fetch: () => Promise<void>;
  markRead: (threadId: string) => Promise<void>;
}

export const useNotifications = create<NotificationsStore>((set, get) => ({
  items: [],
  unreadCount: 0,
  isLoading: false,
  error: null,
  lastFetchedAt: null,

  fetch: async () => {
    // Skip when logged out — the backend would return "Not logged in" and
    // clutter the error state. Stays silent so the bell badge can render
    // 0 without flashing an error.
    if (!useAuth.getState().state.isLoggedIn) return;

    set({ isLoading: true, error: null });
    try {
      const payload = await notificationsApi.list();
      // 304: backend returns an empty list with `fromCache=true`. Don't
      // overwrite the cached items — keep showing what the user already
      // sees, but refresh `lastFetchedAt` so the UI knows the poll
      // succeeded.
      if (payload.fromCache) {
        set({
          isLoading: false,
          lastFetchedAt: new Date().toISOString(),
        });
        return;
      }
      set({
        items: payload.items,
        unreadCount: payload.unreadCount,
        isLoading: false,
        lastFetchedAt: new Date().toISOString(),
      });
    } catch (e) {
      set({
        isLoading: false,
        error: typeof e === 'string' ? e : (e as Error).message ?? 'Failed to fetch notifications',
      });
    }
  },

  markRead: async (threadId: string) => {
    // Optimistic update: mark the row read locally before the API call so
    // the bell badge reacts instantly. Roll back if the API rejects.
    const prev = get().items;
    const updated = prev.map((n) => (n.id === threadId ? { ...n, unread: false } : n));
    const newUnread = updated.filter((n) => n.unread).length;
    set({ items: updated, unreadCount: newUnread });

    try {
      await notificationsApi.markRead(threadId);
    } catch (e) {
      set({
        items: prev,
        unreadCount: prev.filter((n) => n.unread).length,
        error: typeof e === 'string' ? e : (e as Error).message ?? 'Failed to mark as read',
      });
    }
  },
}));
