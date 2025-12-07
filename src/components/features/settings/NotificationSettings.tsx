/**
 * Notification Settings Component
 *
 * Solid.js implementation of NotificationSettings component.
 * Allows users to configure notification methods and individual notification types.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/notification_settings.rs
 */

import { Component, Show, createSignal, createEffect, onCleanup } from 'solid-js';
import { useSettings } from '../../../stores/settingsStore';
import { ToggleSwitch } from '../../ui/form';
import type { NotificationMethod } from '../../../types';
import { notificationMethodFromStr, notificationMethodLabel } from '../../../types/settings';

export const NotificationSettings: Component = () => {
  const settingsStore = useSettings();
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [successMessage, setSuccessMessage] = createSignal<string | null>(null);
  const [initialLoadComplete, setInitialLoadComplete] = createSignal(false);
  const [timeoutId, setTimeoutId] = createSignal<number | null>(null);

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

  // Update notification method
  const updateNotificationMethod = (method: NotificationMethod) => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    settingsStore
      .updateSettings({
        ...currentSettings,
        notificationMethod: method,
      })
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Toggle individual notification setting
  const toggleNotification = (field: 'xp_gain' | 'level_up' | 'badge_earned' | 'streak_update' | 'streak_milestone') => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    const fieldMap = {
      xp_gain: 'notifyXpGain',
      level_up: 'notifyLevelUp',
      badge_earned: 'notifyBadgeEarned',
      streak_update: 'notifyStreakUpdate',
      streak_milestone: 'notifyStreakMilestone',
    } as const;

    const updateKey = fieldMap[field];
    const currentValue = currentSettings[updateKey];

    settingsStore
      .updateSettings({
        ...currentSettings,
        [updateKey]: !currentValue,
      })
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Toggle all notifications on
  const toggleAllOn = () => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    settingsStore
      .updateSettings({
        ...currentSettings,
        notifyXpGain: true,
        notifyLevelUp: true,
        notifyBadgeEarned: true,
        notifyStreakUpdate: true,
        notifyStreakMilestone: true,
      })
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Toggle all notifications off
  const toggleAllOff = () => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    settingsStore
      .updateSettings({
        ...currentSettings,
        notifyXpGain: false,
        notifyLevelUp: false,
        notifyBadgeEarned: false,
        notifyStreakUpdate: false,
        notifyStreakMilestone: false,
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
    if (timeoutId() !== null) {
      clearTimeout(timeoutId()!);
      setTimeoutId(null);
    }

    // Debounce: save after 500ms of no changes
    const handle = setTimeout(() => {
      // Settings are already saved by updateSettings
      setTimeoutId(null);
    }, 500);

    setTimeoutId(handle);
  });

  // Cleanup timeout on component unmount
  onCleanup(() => {
    if (timeoutId() !== null) {
      clearTimeout(timeoutId()!);
    }
  });

  const settings = () => settingsStore.store.settings;

  const notificationOptions: NotificationMethod[] = ['app_only', 'os_only', 'both', 'none'];

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

      {/* Success message */}
      <Show when={successMessage()}>
        <div class="p-3 bg-green-900/30 border border-green-500/50 rounded-lg text-green-200 text-sm">
          {successMessage()}
        </div>
      </Show>

      {/* Settings form */}
      <Show when={settings() && !loading()}>
        {(s) => {
          const currentMethod = notificationMethodFromStr(s().notificationMethod);

          return (
            <>
              {/* Notification method selection */}
              <div class="space-y-3">
                <h3 class="text-lg font-gaming font-bold text-white">通知方法</h3>
                <div class="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                  {notificationOptions.map((method) => {
                    const isSelected = currentMethod === method;
                    return (
                      <label
                        class={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${
                          isSelected
                            ? 'bg-gm-accent-cyan/20 border border-gm-accent-cyan/50'
                            : 'hover:bg-gm-bg-card/30'
                        }`}
                      >
                        <input
                          type="radio"
                          name="notification_method"
                          checked={isSelected}
                          onChange={() => updateNotificationMethod(method)}
                          class="w-4 h-4 text-gm-accent-cyan bg-gm-bg-card border-gm-accent-cyan/50 focus:ring-gm-accent-cyan focus:ring-2"
                        />
                        <span class="text-white">{notificationMethodLabel(method)}</span>
                      </label>
                    );
                  })}
                </div>
              </div>

              {/* Divider */}
              <div class="border-t border-gm-accent-cyan/20"></div>

              {/* Individual notification settings */}
              <div class="space-y-3">
                <div class="flex items-center justify-between">
                  <h3 class="text-lg font-gaming font-bold text-white">通知の種類</h3>
                  <div class="flex gap-2">
                    <button
                      class="px-3 py-1 text-sm rounded-lg bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 text-gm-accent-cyan transition-colors"
                      onClick={toggleAllOn}
                    >
                      全てON
                    </button>
                    <button
                      class="px-3 py-1 text-sm rounded-lg bg-slate-700/50 hover:bg-slate-700/70 text-white transition-colors"
                      onClick={toggleAllOff}
                    >
                      全てOFF
                    </button>
                  </div>
                </div>
                <div class="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                  {[
                    { field: 'xp_gain' as const, label: 'XP獲得通知' },
                    { field: 'level_up' as const, label: 'レベルアップ通知' },
                    { field: 'badge_earned' as const, label: 'バッジ獲得通知' },
                    { field: 'streak_update' as const, label: 'ストリーク更新通知' },
                    { field: 'streak_milestone' as const, label: 'ストリークマイルストーン' },
                  ].map(({ field, label }) => {
                    const fieldMap = {
                      xp_gain: 'notifyXpGain',
                      level_up: 'notifyLevelUp',
                      badge_earned: 'notifyBadgeEarned',
                      streak_update: 'notifyStreakUpdate',
                      streak_milestone: 'notifyStreakMilestone',
                    } as const;
                    const value = s()[fieldMap[field]] as boolean;
                    return (
                      <div class="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                        <span class="text-white font-gaming">{label}</span>
                        <ToggleSwitch enabled={value} onToggle={() => toggleNotification(field)} />
                      </div>
                    );
                  })}
                </div>
              </div>
            </>
          );
        }}
      </Show>
    </div>
  );
};

