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
  enabled: true, // Default: animations enabled
});

/**
 * Animation hook
 *
 * Provides animation state and automatically syncs with settings store.
 * When settings change, animation state is updated accordingly.
 */
export const useAnimation = () => {
  const settings = useSettings();

  // Sync animation state from settings
  createEffect(() => {
    if (settings.store.settings) {
      setAnimationStore('enabled', settings.store.settings.animationsEnabled);
    }
  });

  return {
    store: animationStore,
  };
};

