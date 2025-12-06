/**
 * Network Status Store
 *
 * Manages network connectivity status (online/offline) using Solid.js signals.
 * Monitors browser online/offline events and updates state accordingly.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Types: src/types/network.ts (NetworkState, newNetworkState, setOnline, setOffline)
 */

import { createSignal, onMount, onCleanup } from 'solid-js';
import type { NetworkState } from '@/types';
import { newNetworkState, setOnline, setOffline } from '@/types/network';

/**
 * Network status hook
 *
 * Provides network connectivity state and monitors online/offline events.
 * Automatically sets up event listeners on mount and cleans up on unmount.
 */
export const useNetworkStatus = () => {
  const [networkState, setNetworkState] = createSignal<NetworkState>(
    newNetworkState(navigator.onLine)
  );

  onMount(() => {
    const handleOnline = () => {
      setNetworkState((prev) => setOnline(prev));
    };

    const handleOffline = () => {
      setNetworkState((prev) => setOffline(prev));
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    onCleanup(() => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    });
  });

  return {
    networkState,
    isOnline: () => networkState().isOnline,
  };
};

