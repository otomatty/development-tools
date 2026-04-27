/**
 * Authentication Store
 *
 * Manages authentication state using zustand.
 * Handles login, logout, and authentication state synchronization with Tauri backend.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Issue (auth-expired flow): https://github.com/otomatty/development-tools/issues/181
 *   - Types: src/types/auth.ts
 *   - Tauri API: src/lib/tauri/commands.ts
 */

import { create } from 'zustand';
import type { AuthState, AuthExpiredEvent } from '@/types';
import { auth as authApi } from '@/lib/tauri/commands';
import { events } from '@/lib/tauri/events';

interface AuthStore {
  state: AuthState;
  isLoading: boolean;
  error: string | null;
  /**
   * Set when the backend reports the stored GitHub token is no longer valid
   * (revoked / expired). The UI uses this to show a re-login prompt and
   * suppress API calls until the user re-authenticates.
   *
   * Cleared automatically when `fetchAuthState()` confirms the user is logged
   * in again, or manually via `dismissAuthExpired()`.
   */
  authExpired: AuthExpiredEvent | null;
  fetchAuthState: () => Promise<void>;
  logout: () => Promise<void>;
  dismissAuthExpired: () => void;
}

let authRequestSeq = 0;

export const useAuth = create<AuthStore>((set, get) => ({
  state: { isLoggedIn: false, user: null },
  isLoading: true,
  error: null,
  authExpired: null,
  fetchAuthState: async () => {
    const seq = ++authRequestSeq;
    try {
      set({ isLoading: true, error: null });
      const state = await authApi.getState();
      if (seq !== authRequestSeq) return;
      // A successful re-login clears any stale "session expired" banner.
      const cleared = state.isLoggedIn ? { authExpired: null } : {};
      set({ state, isLoading: false, ...cleared });
    } catch (e) {
      if (seq !== authRequestSeq) return;
      set({ error: String(e), isLoading: false });
    }
  },
  logout: async () => {
    const seq = ++authRequestSeq;
    try {
      set({ isLoading: true, error: null });
      await authApi.logout();
      if (seq !== authRequestSeq) return;
      set({
        state: { isLoggedIn: false, user: null },
        isLoading: false,
        // Manual logout — no need to keep nagging the user about the expired
        // session they just resolved.
        authExpired: null,
      });
    } catch (e) {
      if (seq !== authRequestSeq) return;
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },
  dismissAuthExpired: () => {
    if (get().authExpired !== null) {
      set({ authExpired: null });
    }
  },
}));

// Subscribe to backend auth-expired events so the UI surfaces a re-login
// prompt without polling. The backend has already cleared the credential by
// the time this fires; we only need to flip local state so the dashboard
// stops issuing API requests and the LoginCard reappears.
//
// Listener registration is fire-and-forget: if it fails (e.g. Tauri not
// initialised yet during tests), the dashboard still works — the user just
// won't see the auto-prompt until they next reload.
events
  .onAuthExpired((event) => {
    // Cancel any in-flight `fetchAuthState` / `logout` so a stale success
    // response can't sneak in after the event and overwrite the logged-out
    // state we're about to set. Concrete failure mode without this bump:
    // at startup, `fetchAuthState()` runs concurrently with the backend's
    // startup token probe; the probe hits 401 and emits `auth-expired`,
    // but the `getState` call (initiated before the credential was cleared)
    // resolves with the still-cached `user` row and flips `isLoggedIn` back
    // to true — leaving the UI in a phantom logged-in state.
    ++authRequestSeq;
    useAuth.setState({
      state: { isLoggedIn: false, user: null },
      authExpired: event,
      isLoading: false,
    });
  })
  .catch((e) => {
    // eslint-disable-next-line no-console
    console.error('Failed to subscribe to auth-expired events:', e);
  });

// Fetch auth state on module load
useAuth.getState().fetchAuthState();
