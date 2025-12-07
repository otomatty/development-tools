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
 * Fetch authentication state from Tauri backend
 * This is called once at module load to initialize the store.
 */
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

/**
 * Logout current user
 */
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

// Fetch authentication state immediately at module load (singleton pattern)
fetchAuthState();

/**
 * Authentication hook
 *
 * Provides authentication state and methods for login/logout.
 * Authentication state is automatically fetched once at module load.
 */
export const useAuth = () => {
  return {
    store: authStore,
    fetchAuthState,
    logout,
  };
};

