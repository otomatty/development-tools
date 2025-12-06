/**
 * Settings Store
 *
 * Manages application settings using Solid.js stores.
 * Handles fetching and updating settings from Tauri backend.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Types: src/types/settings.ts (UserSettings, UpdateSettingsRequest)
 *   - Tauri API: src/lib/tauri/commands.ts (settings.get(), settings.update())
 */

import { createStore } from 'solid-js/store';
import { onMount } from 'solid-js';
import type { UserSettings, UpdateSettingsRequest } from '@/types';
import { settings as settingsApi } from '@/lib/tauri/commands';

interface SettingsStore {
  settings: UserSettings | null;
  isLoading: boolean;
  error: string | null;
}

const [settingsStore, setSettingsStore] = createStore<SettingsStore>({
  settings: null,
  isLoading: true,
  error: null,
});

/**
 * Settings hook
 *
 * Provides settings state and methods for fetching and updating settings.
 * Automatically fetches settings on mount.
 */
export const useSettings = () => {
  const fetchSettings = async () => {
    try {
      setSettingsStore('isLoading', true);
      setSettingsStore('error', null);
      const settings = await settingsApi.get();
      setSettingsStore('settings', settings);
      setSettingsStore('isLoading', false);
    } catch (e) {
      setSettingsStore('error', String(e));
      setSettingsStore('isLoading', false);
      // Don't throw - settings may not be available if user is not logged in
    }
  };

  const updateSettings = async (request: UpdateSettingsRequest) => {
    try {
      setSettingsStore('error', null);
      const updated = await settingsApi.update(request);
      setSettingsStore('settings', updated);
      return updated;
    } catch (e) {
      setSettingsStore('error', String(e));
      throw e;
    }
  };

  // Fetch settings on mount
  onMount(() => {
    fetchSettings();
  });

  return {
    store: settingsStore,
    fetchSettings,
    updateSettings,
  };
};

