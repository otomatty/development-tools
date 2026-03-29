/**
 * Authentication Store
 *
 * Manages authentication state using zustand.
 * Handles login, logout, and authentication state synchronization with Tauri backend.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Types: src/types/auth.ts
 *   - Tauri API: src/lib/tauri/commands.ts
 */

import { create } from 'zustand';
import type { AuthState } from '@/types';
import { auth as authApi } from '@/lib/tauri/commands';

interface AuthStore {
  state: AuthState;
  isLoading: boolean;
  error: string | null;
  fetchAuthState: () => Promise<void>;
  logout: () => Promise<void>;
}

let authRequestSeq = 0;

export const useAuth = create<AuthStore>((set) => ({
  state: { isLoggedIn: false, user: null },
  isLoading: true,
  error: null,
  fetchAuthState: async () => {
    const seq = ++authRequestSeq;
    try {
      set({ isLoading: true, error: null });
      const state = await authApi.getState();
      if (seq !== authRequestSeq) return;
      set({ state, isLoading: false });
    } catch (e) {
      if (seq !== authRequestSeq) return;
      set({ error: String(e), isLoading: false });
    }
  },
  logout: async () => {
    const seq = ++authRequestSeq;
    try {
      set({ error: null });
      await authApi.logout();
      if (seq !== authRequestSeq) return;
      set({ state: { isLoggedIn: false, user: null } });
    } catch (e) {
      if (seq !== authRequestSeq) return;
      set({ error: String(e) });
      throw e;
    }
  },
}));

// Fetch auth state on module load
useAuth.getState().fetchAuthState();
