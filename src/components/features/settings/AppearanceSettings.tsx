/**
 * Appearance Settings Component
 *
 * Solid.js implementation of AppearanceSettings component.
 * Allows users to configure animation effects ON/OFF.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/appearance_settings.rs
 */

import { Component, Show, createSignal, createEffect, onCleanup } from 'solid-js';
import { useSettings } from '../../../stores/settingsStore';
import { useAnimation } from '../../../stores/animationStore';
import { ToggleSwitch } from '../../ui/form';

export const AppearanceSettings: Component = () => {
  const settingsStore = useSettings();
  const animation = useAnimation();
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [initialLoadComplete, setInitialLoadComplete] = createSignal(false);
  const [debounceHandle, setDebounceHandle] = createSignal<number | null>(null);

  // Load settings on mount
  createEffect(() => {
    if (initialLoadComplete()) return;

    if (!settingsStore.store.isLoading && settingsStore.store.settings) {
      setLoading(false);
      setInitialLoadComplete(true);
    } else if (!settingsStore.store.isLoading && settingsStore.store.error) {
      setError(`設定の読み込みに失敗しました: ${settingsStore.store.error}`);
      setLoading(false);
      setInitialLoadComplete(true);
    }
  });

  // Toggle animations
  const toggleAnimations = () => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    const newValue = !currentSettings.animationsEnabled;

    // Update global animation context immediately
    animation.setEnabled(newValue);

    // Update settings
    settingsStore
      .updateSettings({
        ...currentSettings,
        animationsEnabled: newValue,
      })
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Auto-save when settings change with debouncing
  createEffect(() => {
    const currentSettings = settingsStore.store.settings;
    const isInitialLoadComplete = initialLoadComplete();

    // Skip if settings are not loaded or initial load is not complete
    if (!currentSettings || loading() || !isInitialLoadComplete) {
      return;
    }

    // Clear previous timeout if exists
    if (debounceHandle() !== null) {
      clearTimeout(debounceHandle()!);
      setDebounceHandle(null);
    }

    // Debounce: save after 500ms of no changes
    const handle = setTimeout(() => {
      // Settings are already saved by updateSettings in toggleAnimations
      setDebounceHandle(null);
    }, 500);

    setDebounceHandle(handle);
  });

  // Cleanup timeout on component unmount
  onCleanup(() => {
    if (debounceHandle() !== null) {
      clearTimeout(debounceHandle()!);
    }
  });

  const settings = () => settingsStore.store.settings;

  return (
    <div class="space-y-6">
      {/* Loading state */}
      <Show when={loading()}>
        <div class="text-center py-8 text-dt-text-sub">設定を読み込み中...</div>
      </Show>

      {/* Error message */}
      <Show when={error()}>
        <div class="p-3 bg-red-900/30 border border-red-500/50 rounded-lg text-red-200 text-sm">
          {error()}
        </div>
      </Show>

      {/* Settings form */}
      <Show when={settings() && !loading()}>
        {(s) => (
          <div class="space-y-3">
            {/* Animation toggle */}
            <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
              <div class="flex items-center justify-between">
                <div class="flex-1">
                  <span class="text-white block font-gaming font-bold" id="animations-label">
                    アニメーション効果
                  </span>
                  <span class="text-sm text-dt-text-sub mt-1 block">
                    XP獲得、レベルアップ、バッジ獲得時の
                    <br />
                    アニメーション効果を有効にする
                  </span>
                </div>
                <ToggleSwitch
                  enabled={s().animationsEnabled}
                  onToggle={toggleAnimations}
                  labelId="animations-label"
                />
              </div>
            </div>

            {/* Hint text */}
            <div class="text-xs text-dt-text-sub p-3 bg-gm-bg-card/30 rounded-lg">
              ※ OFFにするとパフォーマンスが向上する場合があります
            </div>

            {/* Animation preview (when enabled) */}
            <Show when={s().animationsEnabled}>
              <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                <h4 class="text-sm font-gaming font-bold text-white mb-3">プレビュー</h4>
                <div class="flex items-center justify-center gap-4">
                  <div class="text-3xl animate-bounce">✨</div>
                  <div class="text-3xl animate-pulse">🔥</div>
                  <div class="text-3xl animate-bounce-slow">🏆</div>
                </div>
              </div>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
};

