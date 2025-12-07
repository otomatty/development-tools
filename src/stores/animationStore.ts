/**
 * Animation Store
 *
 * Manages animation settings using Solid.js stores.
 * Synchronizes with settings store to reflect user preferences.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134
 *   - Settings Store: src/stores/settingsStore.ts
 */

import { createStore } from 'solid-js/store';
import { createEffect } from 'solid-js';
import { useSettings } from './settingsStore';

interface AnimationStore {
  enabled: boolean;
}

const [animationStore, setAnimationStore] = createStore<AnimationStore>({
  enabled: true, // Default: animations enabled (will be synced from settings when loaded)
});

// Initialize settings and sync effect once at module scope
const settings = useSettings();

// Sync animation state from settings
createEffect(() => {
  if (settings.store.settings) {
    setAnimationStore('enabled', settings.store.settings.animationsEnabled);
  }
});

/**
 * Set animation enabled state
 * This updates the store directly (settings will be synced via the effect above)
 */
const setEnabled = (enabled: boolean) => {
  setAnimationStore('enabled', enabled);
};

/**
 * Animation hook
 *
 * Provides animation state that is automatically synced with settings store.
 * Animation state is synced once at module load to avoid duplicate effects.
 */
export const useAnimation = () => {
  return {
    store: animationStore,
    setEnabled,
  };
};

