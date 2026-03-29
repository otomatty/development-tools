/**
 * Settings Store
 *
 * Manages application settings using zustand.
 * Handles fetching and updating settings from Tauri backend.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Types: src/types/settings.ts (UserSettings, UpdateSettingsRequest)
 *   - Tauri API: src/lib/tauri/commands.ts (settings.get(), settings.update())
 */

import { create } from 'zustand';
import type { UserSettings, UpdateSettingsRequest } from '@/types';
import { settings as settingsApi } from '@/lib/tauri/commands';

interface SettingsStore {
  settings: UserSettings | null;
  isLoading: boolean;
  error: string | null;
  fetchSettings: () => Promise<void>;
  updateSettings: (request: UpdateSettingsRequest) => Promise<UserSettings>;
}

export const useSettings = create<SettingsStore>((set) => ({
  settings: null,
  isLoading: true,
  error: null,
  fetchSettings: async () => {
    try {
      set({ isLoading: true, error: null });
      const settings = await settingsApi.get();
      set({ settings, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },
  updateSettings: async (request: UpdateSettingsRequest) => {
    try {
      set({ error: null });
      const updated = await settingsApi.update(request);
      set({ settings: updated });
      return updated;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },
}));

useSettings.getState().fetchSettings();
