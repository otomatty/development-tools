/**
 * Sync Settings Component
 *
 * React implementation of SyncSettings component.
 * Allows users to configure sync intervals, background sync, and startup sync options.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/sync_settings.rs
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useSettings } from '../../../stores/settingsStore';
import {
  settings as settingsApi,
  github as githubApi,
  scheduler as schedulerApi,
} from '../../../lib/tauri/commands';
import { ToggleSwitch } from '../../ui/form';
import { InlineToast } from '../../ui/feedback';
import { Button } from '../../ui/button';
import type {
  SyncIntervalOption,
  SyncResult,
  SchedulerStatus,
} from '../../../types';
import { schedulerSkipReasonLabel } from '../../../types/settings';

export const SyncSettings: React.FC = () => {
  const { settings, isLoading, error: storeError, updateSettings } = useSettings();
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [lastSyncTime, setLastSyncTime] = useState<string | null>(null);
  const initialLoadCompleteRef = useRef(false);
  const successMsgHandleRef = useRef<number | null>(null);

  // Load sync intervals
  const [syncIntervals, setSyncIntervals] = useState<SyncIntervalOption[]>([]);

  // Background sync scheduler status
  const [schedulerStatus, setSchedulerStatus] = useState<SchedulerStatus | null>(null);

  const fetchSyncIntervals = useCallback(async () => {
    try {
      const data = await settingsApi.getSyncIntervals();
      setSyncIntervals(data);
    } catch (e) {
      console.error('Failed to load sync intervals:', e);
      // Use fallback intervals
      setSyncIntervals([
        { value: 5, label: '5分' },
        { value: 15, label: '15分' },
        { value: 30, label: '30分' },
        { value: 60, label: '1時間' },
        { value: 180, label: '3時間' },
        { value: 0, label: '手動のみ' },
      ]);
    }
  }, []);

  const fetchSchedulerStatus = useCallback(async () => {
    try {
      const status = await schedulerApi.getStatus();
      setSchedulerStatus(status);
    } catch (e) {
      // Scheduler status is best-effort; don't surface as a hard error.
      console.warn('Failed to load scheduler status:', e);
    }
  }, []);

  useEffect(() => {
    fetchSyncIntervals();
  }, [fetchSyncIntervals]);

  // Poll the scheduler status so the UI reflects the next sync time as it
  // ticks down. Polling keeps things simple without needing event subscription.
  useEffect(() => {
    fetchSchedulerStatus();
    const handle = window.setInterval(fetchSchedulerStatus, 15_000);
    return () => window.clearInterval(handle);
  }, [fetchSchedulerStatus]);

  // Load settings on mount
  useEffect(() => {
    if (initialLoadCompleteRef.current) return;

    if (!isLoading && settings) {
      setLoading(false);
      initialLoadCompleteRef.current = true;
    } else if (!isLoading && storeError) {
      setError(`設定の読み込みに失敗しました: ${storeError}`);
      setLoading(false);
      initialLoadCompleteRef.current = true;
    }
  }, [isLoading, settings, storeError]);

  // Update sync interval
  const updateSyncInterval = (interval: number) => {
    if (!settings) return;

    updateSettings({
      ...settings,
      syncIntervalMinutes: interval,
    })
      .then(() => fetchSchedulerStatus())
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Toggle background sync
  const toggleBackgroundSync = () => {
    if (!settings) return;

    updateSettings({
      ...settings,
      backgroundSync: !settings.backgroundSync,
    })
      .then(() => fetchSchedulerStatus())
      .catch((e) => {
        setError(`設定の保存に失敗しました: ${e}`);
      });
  };

  // Toggle sync on startup
  const toggleSyncOnStartup = () => {
    if (!settings) return;

    updateSettings({
      ...settings,
      syncOnStartup: !settings.syncOnStartup,
    })
      .then(() => fetchSchedulerStatus())
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
    if (successMsgHandleRef.current !== null) {
      clearTimeout(successMsgHandleRef.current);
      successMsgHandleRef.current = null;
    }

    try {
      const syncResult: SyncResult = await githubApi.syncStats();

      // Update last sync time
      const now = new Date();
      const timeStr = now.toLocaleString('ja-JP', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      });
      setLastSyncTime(timeStr);

      const xpMsg = syncResult.xpGained > 0 ? ` (+${syncResult.xpGained} XP)` : '';
      setSuccessMessage(`同期が完了しました${xpMsg}`);

      // Refresh scheduler status to reflect the new last/next sync times.
      void fetchSchedulerStatus();

      // Auto-hide success message after 3 seconds
      const handle = window.setTimeout(() => {
        setSuccessMessage(null);
        successMsgHandleRef.current = null;
      }, 3000);
      successMsgHandleRef.current = handle;
    } catch (e) {
      setError(`同期に失敗しました: ${e}`);
    } finally {
      setSyncing(false);
    }
  };

  // Format an ISO timestamp for display, or return a fallback string.
  const formatTime = (iso: string | null | undefined, fallback = '—'): string => {
    if (!iso) return fallback;
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return fallback;
    return d.toLocaleString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // Cleanup timeouts on component unmount
  useEffect(() => {
    return () => {
      if (successMsgHandleRef.current !== null) {
        clearTimeout(successMsgHandleRef.current);
      }
    };
  }, []);

  return (
    <div className="space-y-6">
      {/* Loading state */}
      {loading && (
        <div className="text-center py-8 text-dt-text-sub">設定を読み込み中...</div>
      )}

      {/* Error message with InlineToast */}
      <InlineToast
        visible={error !== null}
        message={error || ''}
        type="error"
      />

      {/* Success message with InlineToast */}
      <InlineToast
        visible={successMessage !== null}
        message={successMessage || ''}
        type="success"
      />

      {/* Settings form */}
      {settings && !loading && (() => {
        const intervals = syncIntervals;

        return (
          <>
            {/* Sync interval selection */}
            <div className="space-y-3">
              <h3 className="text-lg font-gaming font-bold text-white" id="sync-interval-label">
                自動同期間隔
              </h3>
              <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                <select
                  className="w-full px-4 py-3 bg-gm-bg-primary border border-gm-accent-cyan/30 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan cursor-pointer appearance-none"
                  style={{
                    backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%2306b6d4' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E")`,
                    backgroundPosition: 'right 0.75rem center',
                    backgroundRepeat: 'no-repeat',
                    backgroundSize: '1.5em 1.5em',
                    paddingRight: '2.5rem',
                  }}
                  aria-labelledby="sync-interval-label"
                  value={settings.syncIntervalMinutes}
                  onChange={(e) => {
                    const value = parseInt(e.currentTarget.value, 10);
                    if (!isNaN(value)) {
                      updateSyncInterval(value);
                    }
                  }}
                >
                  {intervals.map((interval) => (
                    <option key={interval.value} value={interval.value}>
                      {interval.label}
                    </option>
                  ))}
                </select>
                <p className="mt-2 text-sm text-dt-text-sub">
                  {settings.syncIntervalMinutes === 0
                    ? '自動同期は無効です。手動で同期を実行してください。'
                    : 'GitHubの統計情報を自動的に取得する間隔を設定します。'}
                </p>
              </div>
            </div>

            {/* Divider */}
            <div className="border-t border-gm-accent-cyan/20"></div>

            {/* Toggle settings */}
            <div className="space-y-3">
              <h3 className="text-lg font-gaming font-bold text-white">同期オプション</h3>
              <div className="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                {/* Background sync toggle */}
                <div className="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                  <div className="flex-1">
                    <span className="text-white block font-gaming font-bold" id="background-sync-label">
                      バックグラウンド同期
                    </span>
                    <span className="text-sm text-dt-text-sub mt-1 block">
                      アプリがバックグラウンドにある時も同期を続ける
                    </span>
                  </div>
                  <ToggleSwitch
                    enabled={settings.backgroundSync}
                    onToggle={toggleBackgroundSync}
                    labelId="background-sync-label"
                  />
                </div>

                {/* Sync on startup toggle */}
                <div className="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                  <div className="flex-1">
                    <span className="text-white block font-gaming font-bold" id="sync-on-startup-label">
                      起動時に同期
                    </span>
                    <span className="text-sm text-dt-text-sub mt-1 block">
                      アプリ起動時に自動的に同期を実行する
                    </span>
                  </div>
                  <ToggleSwitch
                    enabled={settings.syncOnStartup}
                    onToggle={toggleSyncOnStartup}
                    labelId="sync-on-startup-label"
                  />
                </div>
              </div>
            </div>

            {/* Divider */}
            <div className="border-t border-gm-accent-cyan/20"></div>

            {/* Scheduler status section */}
            <div className="space-y-3">
              <h3 className="text-lg font-gaming font-bold text-white">スケジューラ状態</h3>
              <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20 space-y-2">
                {schedulerStatus ? (
                  <>
                    <div className="flex items-baseline justify-between text-sm">
                      <span className="text-dt-text-sub">最終自動同期</span>
                      <span className="text-white font-mono">
                        {formatTime(schedulerStatus.lastSyncAt, '未実行')}
                      </span>
                    </div>
                    <div className="flex items-baseline justify-between text-sm">
                      <span className="text-dt-text-sub">次回自動同期</span>
                      <span className="text-white font-mono">
                        {schedulerStatus.backgroundSyncEnabled &&
                        schedulerStatus.intervalMinutes > 0
                          ? formatTime(schedulerStatus.nextSyncAt, '計算中')
                          : '— 自動同期はオフ'}
                      </span>
                    </div>
                    {schedulerStatus.lastSkippedReason ? (
                      <div className="mt-2 p-2 rounded bg-yellow-500/10 border border-yellow-500/30 text-sm text-yellow-200">
                        <div className="font-bold">同期がスキップされました</div>
                        <div className="text-xs mt-1">
                          {schedulerSkipReasonLabel(schedulerStatus.lastSkippedReason)}
                        </div>
                        {schedulerStatus.lastSkippedAt && (
                          <div className="text-xs text-yellow-300/80 mt-1">
                            最終スキップ: {formatTime(schedulerStatus.lastSkippedAt)}
                          </div>
                        )}
                      </div>
                    ) : null}
                  </>
                ) : (
                  <div className="text-sm text-dt-text-sub">スケジューラ情報を読み込み中...</div>
                )}
              </div>
            </div>

            {/* Divider */}
            <div className="border-t border-gm-accent-cyan/20"></div>

            {/* Manual sync section */}
            <div className="space-y-3">
              <h3 className="text-lg font-gaming font-bold text-white">手動同期</h3>
              <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                {/* Last sync time */}
                {lastSyncTime && (
                  <div className="mb-4 text-sm text-dt-text-sub">
                    <span className="font-medium">最終同期: </span>
                    <span>{lastSyncTime}</span>
                  </div>
                )}

                {/* Sync button */}
                <Button
                  variant="primary"
                  onClick={onManualSync}
                  disabled={syncing}
                  fullWidth
                  isLoading={syncing}
                >
                  {syncing ? '同期中...' : '今すぐ同期'}
                </Button>
                <p className="mt-2 text-sm text-dt-text-sub text-center">
                  GitHubの統計情報を今すぐ取得します
                </p>
              </div>
            </div>
          </>
        );
      })()}
    </div>
  );
};
