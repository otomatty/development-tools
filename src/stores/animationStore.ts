/**
 * Animation Store
 *
 * Manages animation settings using zustand.
 * Synchronizes with settings store to reflect user preferences.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Settings Store: src/stores/settingsStore.ts
 */

import { create } from 'zustand';
import { useSettings } from './settingsStore';

interface AnimationStore {
  enabled: boolean;
  setEnabled: (enabled: boolean) => void;
}

export const useAnimation = create<AnimationStore>((set) => ({
  enabled: true,
  setEnabled: (enabled: boolean) => set({ enabled }),
}));

// Sync with settings store after settings load
useSettings.subscribe((state) => {
  if (state.settings) {
    useAnimation.getState().setEnabled(state.settings.animationsEnabled);
  }
});
