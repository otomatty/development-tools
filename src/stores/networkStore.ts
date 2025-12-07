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

import { createSignal } from 'solid-js';
import type { NetworkState } from '@/types';
import { newNetworkState, setOnline, setOffline } from '@/types/network';

/**
 * Singleton network state and listeners
 * Network status is managed in a single global instance to avoid duplicate event listeners.
 */
const [networkState, setNetworkState] = createSignal<NetworkState>(
  newNetworkState(navigator.onLine)
);

const handleOnline = () => {
  setNetworkState((prev) => setOnline(prev));
};

const handleOffline = () => {
  setNetworkState((prev) => setOffline(prev));
};

// Only add listeners once at module load (singleton pattern)
window.addEventListener('online', handleOnline);
window.addEventListener('offline', handleOffline);

/**
 * Network status hook (singleton)
 *
 * Returns the shared network connectivity state.
 * Event listeners are set up once at module load to avoid duplicates.
 */
export const useNetworkStatus = () => {
  return {
    networkState,
    isOnline: () => networkState().isOnline,
  };
};

