/**
 * Sync Settings Component
 *
 * Solid.js implementation of SyncSettings component.
 * Allows users to configure sync intervals, background sync, and startup sync options.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/sync_settings.rs
 */

import { Component, Show, createSignal, createEffect, onCleanup } from 'solid-js';
import { createResource } from 'solid-js';
import { useSettings } from '../../../stores/settingsStore';
import { settings as settingsApi, github as githubApi } from '../../../lib/tauri/commands';
import { ToggleSwitch } from '../../ui/form';
import { InlineToast } from '../../ui/feedback';
import { Button } from '../../ui/button';
import type { SyncIntervalOption, SyncResult } from '../../../types';

export const SyncSettings: Component = () => {
  const settingsStore = useSettings();
  const [loading, setLoading] = createSignal(true);
  const [syncing, setSyncing] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [successMessage, setSuccessMessage] = createSignal<string | null>(null);
  const [lastSyncTime, setLastSyncTime] = createSignal<string | null>(null);
  const [initialLoadComplete, setInitialLoadComplete] = createSignal(false);
  const [debounceHandle, setDebounceHandle] = createSignal<number | null>(null);
  const [successMsgHandle, setSuccessMsgHandle] = createSignal<number | null>(null);

  // Load sync intervals
  const [syncIntervals] = createResource<SyncIntervalOption[]>(async () => {
    try {
      return await settingsApi.getSyncIntervals();
    } catch (e) {
      console.error('Failed to load sync intervals:', e);
      // Use fallback intervals
      return [
        { value: 5, label: '5分' },
        { value: 15, label: '15分' },
        { value: 30, label: '30分' },
        { value: 60, label: '1時間' },
        { value: 180, label: '3時間' },
        { value: 0, label: '手動のみ' },
      ];
    }
  });

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

  // Update sync interval
  const updateSyncInterval = (interval: number) => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    settingsStore
      .updateSettings({
        ...currentSettings,
        syncIntervalMinutes: interval,
      })
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Toggle background sync
  const toggleBackgroundSync = () => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    settingsStore
      .updateSettings({
        ...currentSettings,
        backgroundSync: !currentSettings.backgroundSync,
      })
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Toggle sync on startup
  const toggleSyncOnStartup = () => {
    const currentSettings = settingsStore.store.settings;
    if (!currentSettings) return;

    settingsStore
      .updateSettings({
        ...currentSettings,
        syncOnStartup: !currentSettings.syncOnStartup,
      })
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Manual sync
  const onManualSync = async () => {
    setSyncing(true);
    setError(null);
    setSuccessMessage(null);

    // Clear any existing success message timeout
    if (successMsgHandle() !== null) {
      clearTimeout(successMsgHandle()!);
      setSuccessMsgHandle(null);
    }

    try {
      const syncResult: SyncResult = await githubApi.syncStats();

      // Update last sync time
      const now = new Date();
      const timeStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')} ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}`;
      setLastSyncTime(timeStr);

      const xpMsg = syncResult.xpGained > 0 ? ` (+${syncResult.xpGained} XP)` : '';
      setSuccessMessage(`同期が完了しました${xpMsg}`);

      // Auto-hide success message after 3 seconds
      const handle = setTimeout(() => {
        setSuccessMessage(null);
        setSuccessMsgHandle(null);
      }, 3000);
      setSuccessMsgHandle(handle);
    } catch (e) {
      setError(`同期に失敗しました: ${e}`);
    } finally {
      setSyncing(false);
    }
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
      // Settings are already saved by updateSettings
      setDebounceHandle(null);
    }, 500);

    setDebounceHandle(handle);
  });

  // Cleanup timeouts on component unmount
  onCleanup(() => {
    if (debounceHandle() !== null) {
      clearTimeout(debounceHandle()!);
    }
    if (successMsgHandle() !== null) {
      clearTimeout(successMsgHandle()!);
    }
  });

  const settings = () => settingsStore.store.settings;

  return (
    <div class="space-y-6">
      {/* Loading state */}
      <Show when={loading()}>
        <div class="text-center py-8 text-dt-text-sub">設定を読み込み中...</div>
      </Show>

      {/* Error message with InlineToast */}
      <InlineToast
        visible={() => error() !== null}
        message={error() || ''}
        type="error"
      />

      {/* Success message with InlineToast */}
      <InlineToast
        visible={() => successMessage() !== null}
        message={successMessage() || ''}
        type="success"
      />

      {/* Settings form */}
      <Show when={settings() && !loading()}>
        {(s) => {
          const intervals = syncIntervals() || [];

          return (
            <>
              {/* Sync interval selection */}
              <div class="space-y-3">
                <h3 class="text-lg font-gaming font-bold text-white" id="sync-interval-label">
                  自動同期間隔
                </h3>
                <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                  <select
                    class="w-full px-4 py-3 bg-gm-bg-primary border border-gm-accent-cyan/30 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan cursor-pointer appearance-none"
                    style={{
                      'background-image': `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%2306b6d4' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E")`,
                      'background-position': 'right 0.75rem center',
                      'background-repeat': 'no-repeat',
                      'background-size': '1.5em 1.5em',
                      'padding-right': '2.5rem',
                    }}
                    aria-labelledby="sync-interval-label"
                    value={s().syncIntervalMinutes}
                    onChange={(e) => {
                      const value = parseInt(e.currentTarget.value, 10);
                      if (!isNaN(value)) {
                        updateSyncInterval(value);
                      }
                    }}
                  >
                    {intervals.map((interval) => (
                      <option value={interval.value} selected={s().syncIntervalMinutes === interval.value}>
                        {interval.label}
                      </option>
                    ))}
                  </select>
                  <p class="mt-2 text-sm text-dt-text-sub">
                    {s().syncIntervalMinutes === 0
                      ? '自動同期は無効です。手動で同期を実行してください。'
                      : 'GitHubの統計情報を自動的に取得する間隔を設定します。'}
                  </p>
                </div>
              </div>

              {/* Divider */}
              <div class="border-t border-gm-accent-cyan/20"></div>

              {/* Toggle settings */}
              <div class="space-y-3">
                <h3 class="text-lg font-gaming font-bold text-white">同期オプション</h3>
                <div class="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                  {/* Background sync toggle */}
                  <div class="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                    <div class="flex-1">
                      <span class="text-white block font-gaming font-bold" id="background-sync-label">
                        バックグラウンド同期
                      </span>
                      <span class="text-sm text-dt-text-sub mt-1 block">
                        アプリがバックグラウンドにある時も同期を続ける
                      </span>
                    </div>
                    <ToggleSwitch
                      enabled={s().backgroundSync}
                      onToggle={toggleBackgroundSync}
                      labelId="background-sync-label"
                    />
                  </div>

                  {/* Sync on startup toggle */}
                  <div class="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                    <div class="flex-1">
                      <span class="text-white block font-gaming font-bold" id="sync-on-startup-label">
                        起動時に同期
                      </span>
                      <span class="text-sm text-dt-text-sub mt-1 block">
                        アプリ起動時に自動的に同期を実行する
                      </span>
                    </div>
                    <ToggleSwitch
                      enabled={s().syncOnStartup}
                      onToggle={toggleSyncOnStartup}
                      labelId="sync-on-startup-label"
                    />
                  </div>
                </div>
              </div>

              {/* Divider */}
              <div class="border-t border-gm-accent-cyan/20"></div>

              {/* Manual sync section */}
              <div class="space-y-3">
                <h3 class="text-lg font-gaming font-bold text-white">手動同期</h3>
                <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                  {/* Last sync time */}
                  <Show when={lastSyncTime()}>
                    <div class="mb-4 text-sm text-dt-text-sub">
                      <span class="font-medium">最終同期: </span>
                      <span>{lastSyncTime()}</span>
                    </div>
                  </Show>

                  {/* Sync button */}
                  <Button
                    variant="primary"
                    onClick={onManualSync}
                    disabled={syncing()}
                    fullWidth
                    isLoading={syncing()}
                  >
                    {syncing() ? '同期中...' : '今すぐ同期'}
                  </Button>
                  <p class="mt-2 text-sm text-dt-text-sub text-center">
                    GitHubの統計情報を今すぐ取得します
                  </p>
                </div>
              </div>
            </>
          );
        }}
      </Show>
    </div>
  );
};

