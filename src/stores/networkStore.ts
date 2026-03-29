/**
 * Network Status Store
 *
 * Manages network connectivity status (online/offline) using zustand.
 * Monitors browser online/offline events and updates state accordingly.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Types: src/types/network.ts (NetworkState, newNetworkState, setOnline, setOffline)
 */

import { create } from 'zustand';
import type { NetworkState } from '@/types';
import { newNetworkState, setOnline, setOffline } from '@/types/network';

interface NetworkStore {
  networkState: NetworkState;
  isOnline: boolean;
}

export const useNetworkStatus = create<NetworkStore>((set) => {
  const initial = newNetworkState(navigator.onLine);

  const handleOnline = () => {
    set((state) => {
      const next = setOnline(state.networkState);
      return { networkState: next, isOnline: next.isOnline };
    });
  };

  const handleOffline = () => {
    set((state) => {
      const next = setOffline(state.networkState);
      return { networkState: next, isOnline: next.isOnline };
    });
  };

  window.addEventListener('online', handleOnline);
  window.addEventListener('offline', handleOffline);

  return {
    networkState: initial,
    isOnline: initial.isOnline,
  };
});
