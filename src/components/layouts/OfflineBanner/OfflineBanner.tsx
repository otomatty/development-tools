/**
 * OfflineBanner Component
 *
 * Displays a warning banner when the application is offline.
 * Shows the last online timestamp if available.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/137
 *   - Network Store: src/stores/networkStore.ts
 *   - Original (Leptos): src/components/network_status.rs
 */

import { Component, Show } from 'solid-js';
import { useNetworkStatus } from '@/stores/networkStore';
import { Icon } from '@/components/icons';

/**
 * Format ISO 8601 timestamp to human-readable format
 * Converts to local time and returns HH:MM format.
 * Example: "2025-11-30T12:34:56.789Z" -> "21:34" (JST)
 * 
 * @param isoString - ISO 8601 timestamp string
 * @returns Formatted time string (HH:MM) or original string if invalid
 */
function formatTimestamp(isoString: string): string {
  const date = new Date(isoString);
  if (isNaN(date.getTime())) {
    return isoString; // Fallback for invalid date format
  }
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  return `${hours}:${minutes}`;
}

/**
 * OfflineBanner Component
 *
 * Displays a warning banner when offline.
 * Only renders when the network is offline.
 */
export const OfflineBanner: Component = () => {
  const { networkState, isOnline } = useNetworkStatus();

  return (
    <Show when={!isOnline()}>
      <div class="bg-amber-500/90 text-amber-950 px-4 py-2 text-sm flex items-center justify-center gap-2">
        <Icon name="alert-triangle" class="w-4 h-4" />
        <span>⚠️ オフラインモード - キャッシュデータを表示中</span>
        <Show when={networkState()?.lastOnlineAt}>
          <span class="text-amber-800 text-xs">
            最終オンライン: {formatTimestamp(networkState()!.lastOnlineAt!)}
          </span>
        </Show>
      </div>
    </Show>
  );
};

