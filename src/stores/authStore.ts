/**
 * Authentication Store
 *
 * Manages authentication state using Solid.js stores.
 * Handles login, logout, and authentication state synchronization with Tauri backend.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Types: src/types/auth.ts
 *   - Tauri API: src/lib/tauri/commands.ts
 */

import { createStore } from 'solid-js/store';
import { onMount } from 'solid-js';
import type { AuthState } from '@/types';
import { auth as authApi } from '@/lib/tauri/commands';

interface AuthStore {
  state: AuthState;
  isLoading: boolean;
  error: string | null;
}

const [authStore, setAuthStore] = createStore<AuthStore>({
  state: {
    isLoggedIn: false,
    user: null,
  },
  isLoading: true,
  error: null,
});

/**
 * Authentication hook
 *
 * Provides authentication state and methods for login/logout.
 * Automatically fetches authentication state on mount.
 */
export const useAuth = () => {
  const fetchAuthState = async () => {
    try {
      setAuthStore('isLoading', true);
      setAuthStore('error', null);
      const state = await authApi.getState();
      setAuthStore('state', state);
      setAuthStore('isLoading', false);
    } catch (e) {
      setAuthStore('error', String(e));
      setAuthStore('isLoading', false);
    }
  };

  const logout = async () => {
    try {
      setAuthStore('error', null);
      await authApi.logout();
      setAuthStore('state', { isLoggedIn: false, user: null });
    } catch (e) {
      setAuthStore('error', String(e));
      throw e;
    }
  };

  // Fetch authentication state on mount
  onMount(() => {
    fetchAuthState();
  });

  return {
    store: authStore,
    fetchAuthState,
    logout,
  };
};

