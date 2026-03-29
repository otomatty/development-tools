/**
 * Data Management Component
 *
 * React implementation of DataManagement component.
 * Allows users to manage cache, export data, and reset all data.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/data_management.rs
 */

import React, { useState, useEffect, useCallback } from 'react';
import { useNetworkStatus } from '../../../stores/networkStore';
import { settings as settingsApi, cache as cacheApi } from '../../../lib/tauri/commands';
import { Modal, ModalHeader, ModalBody, ModalFooter } from '../../ui/dialog';
import { Button } from '../../ui/button';
import { Input } from '../../ui/form';
import type { DatabaseInfo, CacheStats, ClearCacheResult } from '../../../types';

// Format bytes to human-readable string (KB, MB, GB)
const formatBytes = (bytes: number): string => {
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;

  if (bytes >= GB) {
    return `${(bytes / GB).toFixed(2)} GB`;
  } else if (bytes >= MB) {
    return `${(bytes / MB).toFixed(2)} MB`;
  } else if (bytes >= KB) {
    return `${(bytes / KB).toFixed(2)} KB`;
  } else {
    return `${bytes} bytes`;
  }
};

// Reset confirmation dialog component with "RESET" input confirmation
const CONFIRMATION_TEXT = 'RESET';

const ResetConfirmDialog: React.FC<{
  visible: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}> = ({ visible, onConfirm, onCancel }) => {
  const [inputValue, setInputValue] = useState('');
  const isConfirmEnabled = inputValue === CONFIRMATION_TEXT;

  // Reset input when dialog closes
  useEffect(() => {
    if (!visible) {
      setInputValue('');
    }
  }, [visible]);

  return (
    <Modal
      visible={visible}
      onClose={onCancel}
      size="md"
      closeOnOverlay={false}
      closeOnEscape={false}
      borderClass="border-2 border-red-500/50"
    >
      <ModalHeader onClose={onCancel}>
        <div className="flex items-center gap-3">
          <div className="w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center border border-red-500/30">
            <span className="text-2xl">⚠️</span>
          </div>
          <h3 id="reset-dialog-title" className="text-xl font-gaming font-bold text-red-400">
            データリセットの確認
          </h3>
        </div>
      </ModalHeader>
      <ModalBody>
        <div className="space-y-4">
          <p className="text-dt-text-sub">
            この操作により以下のデータが <span className="text-red-400 font-bold">完全に削除</span>{' '}
            されます：
          </p>

          <ul className="list-none space-y-2 pl-2">
            <li className="flex items-center gap-2 text-dt-text-sub">
              <span className="text-red-400">✗</span>経験値（XP）
            </li>
            <li className="flex items-center gap-2 text-dt-text-sub">
              <span className="text-red-400">✗</span>レベル
            </li>
            <li className="flex items-center gap-2 text-dt-text-sub">
              <span className="text-red-400">✗</span>バッジ
            </li>
            <li className="flex items-center gap-2 text-dt-text-sub">
              <span className="text-red-400">✗</span>ストリーク記録
            </li>
            <li className="flex items-center gap-2 text-dt-text-sub">
              <span className="text-red-400">✗</span>チャレンジ履歴
            </li>
            <li className="flex items-center gap-2 text-dt-text-sub">
              <span className="text-red-400">✗</span>キャッシュデータ
            </li>
          </ul>

          <div className="p-4 bg-red-900/30 border border-red-500/40 rounded-xl">
            <p className="text-red-200 text-sm font-bold flex items-center gap-2">
              <span>🚫</span>
              この操作は取り消せません
            </p>
          </div>

          <div className="space-y-2">
            <label htmlFor="reset-confirm-input" className="text-white text-sm font-gaming">
              続行するには「<span className="text-red-400 font-bold">{CONFIRMATION_TEXT}</span>」と入力してください：
            </label>
            <Input
              id="reset-confirm-input"
              value={inputValue}
              onInput={(value) => setInputValue(value)}
              placeholder={CONFIRMATION_TEXT}
              className="w-full px-4 py-3 bg-gm-bg-primary border-2 border-red-500/30 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-red-500/50 focus:border-red-500 placeholder-red-300/30 font-mono tracking-wider"
            />
          </div>
        </div>
      </ModalBody>
      <ModalFooter>
        <Button variant="secondary" onClick={onCancel}>
          キャンセル
        </Button>
        <Button
          variant="danger"
          onClick={onConfirm}
          disabled={!isConfirmEnabled}
          className={
            isConfirmEnabled
              ? 'shadow-[0_0_15px_rgba(239,68,68,0.4)] hover:shadow-[0_0_20px_rgba(239,68,68,0.6)]'
              : 'opacity-50 cursor-not-allowed border border-red-500/20'
          }
        >
          リセットを実行
        </Button>
      </ModalFooter>
    </Modal>
  );
};

export const DataManagement: React.FC = () => {
  const isOnline = useNetworkStatus((s) => s.isOnline);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [clearingCache, setClearingCache] = useState(false);
  const [cleaningExpired, setCleaningExpired] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [resetting, setResetting] = useState(false);
  const [showResetDialog, setShowResetDialog] = useState(false);
  const [successMsgHandle, setSuccessMsgHandle] = useState<number | null>(null);

  // Load database info
  const [dbInfo, setDbInfo] = useState<DatabaseInfo | null>(null);
  const [dbInfoLoading, setDbInfoLoading] = useState(true);
  const [dbInfoError, setDbInfoError] = useState<string | null>(null);

  const fetchDbInfo = useCallback(async () => {
    setDbInfoLoading(true);
    setDbInfoError(null);
    try {
      const data = await settingsApi.getDatabaseInfo();
      setDbInfo(data);
    } catch (e) {
      console.error('Failed to load database info:', e);
      setDbInfoError(`データベース情報の取得に失敗しました: ${e}`);
    } finally {
      setDbInfoLoading(false);
    }
  }, []);

  // Load cache stats
  const [cacheStats, setCacheStats] = useState<CacheStats | null>(null);

  const fetchCacheStats = useCallback(async () => {
    try {
      const data = await cacheApi.getStats();
      setCacheStats(data);
    } catch (e) {
      // Don't throw error for cache stats - it's okay if not logged in
      console.warn('Failed to load cache stats:', e);
      setCacheStats(null);
    }
  }, []);

  useEffect(() => {
    fetchDbInfo();
    fetchCacheStats();
  }, [fetchDbInfo, fetchCacheStats]);

  // Helper to clear success message timeout
  const clearSuccessTimeout = useCallback(() => {
    if (successMsgHandle !== null) {
      clearTimeout(successMsgHandle);
      setSuccessMsgHandle(null);
    }
  }, [successMsgHandle]);

  // Helper to show success message with auto-hide
  const showSuccess = useCallback((message: string) => {
    clearSuccessTimeout();
    setSuccessMessage(message);

    const handle = window.setTimeout(() => {
      setSuccessMessage(null);
      setSuccessMsgHandle(null);
    }, 3000);
    setSuccessMsgHandle(handle);
  }, [clearSuccessTimeout]);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (successMsgHandle !== null) {
        clearTimeout(successMsgHandle);
      }
    };
  }, [successMsgHandle]);

  // Clear cache handler
  const onClearCache = async () => {
    setClearingCache(true);
    setError(null);

    try {
      const result: ClearCacheResult = await settingsApi.clearCache();
      showSuccess(
        `キャッシュをクリアしました（${result.clearedEntries}エントリ、${formatBytes(result.freedBytes)}解放）`
      );

      // Refresh database info and cache stats
      await fetchDbInfo();
      fetchCacheStats();
    } catch (e) {
      setError(`キャッシュのクリアに失敗しました: ${e}`);
    } finally {
      setClearingCache(false);
    }
  };

  // Cleanup expired cache handler
  const onCleanupExpired = async () => {
    setCleaningExpired(true);
    setError(null);

    try {
      const deletedCount = await cacheApi.cleanupExpired();
      if (deletedCount > 0) {
        showSuccess(`期限切れキャッシュをクリーンアップしました（${deletedCount}エントリ削除）`);
      } else {
        showSuccess('期限切れのキャッシュはありませんでした');
      }

      // Refresh database info and cache stats
      await fetchDbInfo();
      fetchCacheStats();
    } catch (e) {
      setError(`クリーンアップに失敗しました: ${e}`);
    } finally {
      setCleaningExpired(false);
    }
  };

  // Export data handler
  const onExportData = async () => {
    setExporting(true);
    setError(null);

    try {
      const jsonData = await settingsApi.exportData();

      // Create a downloadable file using data URL
      const encodedData = encodeURIComponent(jsonData);
      const dataUrl = `data:application/json;charset=utf-8,${encodedData}`;

      const a = document.createElement('a');
      a.href = dataUrl;

      // Generate filename with timestamp
      const now = new Date();
      const timestamp = now.toISOString().replace(/[:.]/g, '-').slice(0, -5); // Format: YYYY-MM-DDTHH-MM-SS
      const filename = `development-tools-export-${timestamp}.json`;
      a.download = filename;

      // Trigger download
      a.click();
      showSuccess('データをエクスポートしました');
    } catch (e) {
      setError(`データのエクスポートに失敗しました: ${e}`);
    } finally {
      setExporting(false);
    }
  };

  // Reset all data handler
  const onResetConfirmed = async () => {
    setShowResetDialog(false);
    setResetting(true);
    setError(null);

    try {
      await settingsApi.resetAllData();
      showSuccess('全てのデータをリセットしました');

      // Refresh database info
      await fetchDbInfo();
    } catch (e) {
      setError(`データのリセットに失敗しました: ${e}`);
    } finally {
      setResetting(false);
    }
  };

  const expiredCount = cacheStats?.expiredCount ?? 0;
  const entryCount = cacheStats?.entryCount ?? 0;

  return (
    <div className="space-y-6">
      {/* Reset confirmation dialog */}
      <ResetConfirmDialog
        visible={showResetDialog}
        onConfirm={onResetConfirmed}
        onCancel={() => setShowResetDialog(false)}
      />

      {/* Loading state */}
      {dbInfoLoading && (
        <div className="text-center py-8 text-dt-text-sub">データ情報を読み込み中...</div>
      )}

      {/* Error message */}
      {dbInfoError && (
        <div className="p-3 bg-red-900/30 border border-red-500/50 rounded-lg text-red-200 text-sm">
          {dbInfoError}
        </div>
      )}

      {/* Success message */}
      {successMessage && (
        <div className="p-3 bg-green-900/30 border border-green-500/50 rounded-lg text-green-200 text-sm">
          {successMessage}
        </div>
      )}

      {/* Error from actions */}
      {error && (
        <div className="p-3 bg-red-900/30 border border-red-500/50 rounded-lg text-red-200 text-sm">
          {error}
        </div>
      )}

      {!dbInfoLoading && !dbInfoError && (
        <>
          {/* Cache section */}
          <div className="space-y-3">
            <h3 className="text-lg font-gaming font-bold text-white flex items-center gap-2">
              📦 キャッシュ管理
              {/* Offline indicator */}
              {!isOnline && (
                <span className="text-xs px-2 py-0.5 bg-gm-warning/20 text-gm-warning rounded-full border border-gm-warning/30">
                  オフライン
                </span>
              )}
            </h3>

            {/* Cache statistics card */}
            <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
              {/* Statistics grid */}
              <div className="grid grid-cols-2 gap-4 mb-4">
                {/* Total size */}
                <div className="p-3 bg-gm-bg-primary/50 rounded-lg border border-gm-accent-cyan/10">
                  <div className="text-dt-text-sub text-xs mb-1">総サイズ</div>
                  <div className="text-gm-accent-cyan font-gaming text-lg">
                    {cacheStats ? formatBytes(cacheStats.totalSizeBytes) : dbInfo ? formatBytes(dbInfo.cacheSizeBytes) : '--'}
                  </div>
                </div>

                {/* Entry count */}
                <div className="p-3 bg-gm-bg-primary/50 rounded-lg border border-gm-accent-cyan/10">
                  <div className="text-dt-text-sub text-xs mb-1">エントリ数</div>
                  <div className="text-gm-accent-purple font-gaming text-lg">
                    {cacheStats ? String(cacheStats.entryCount) : '--'}
                  </div>
                </div>

                {/* Expired count */}
                <div className="p-3 bg-gm-bg-primary/50 rounded-lg border border-gm-accent-cyan/10">
                  <div className="text-dt-text-sub text-xs mb-1">期限切れ</div>
                  <div
                    className={
                      expiredCount > 0
                        ? 'text-gm-warning font-gaming text-lg'
                        : 'text-gm-success font-gaming text-lg'
                    }
                  >
                    {cacheStats ? String(expiredCount) : '--'}
                  </div>
                </div>

                {/* Status indicator */}
                <div className="p-3 bg-gm-bg-primary/50 rounded-lg border border-gm-accent-cyan/10">
                  <div className="text-dt-text-sub text-xs mb-1">ステータス</div>
                  <div
                    className={
                      !isOnline
                        ? 'text-gm-warning font-gaming text-sm'
                        : 'text-gm-success font-gaming text-sm'
                    }
                  >
                    {!isOnline ? '⚠️ オフライン' : '✅ オンライン'}
                  </div>
                </div>
              </div>

              {/* Action buttons */}
              <div className="space-y-2">
                {/* Cleanup expired cache button */}
                <Button
                  variant={expiredCount > 0 ? 'primary' : 'secondary'}
                  onClick={onCleanupExpired}
                  disabled={cleaningExpired || expiredCount === 0}
                  fullWidth
                  isLoading={cleaningExpired}
                >
                  {cleaningExpired
                    ? 'クリーンアップ中...'
                    : expiredCount > 0
                      ? `期限切れをクリーンアップ (${expiredCount}件)`
                      : '期限切れなし'}
                </Button>

                {/* Clear all cache button */}
                <Button
                  variant="outline"
                  onClick={onClearCache}
                  disabled={clearingCache || entryCount === 0}
                  fullWidth
                  isLoading={clearingCache}
                  className="bg-amber-600/80 hover:bg-amber-500 border-amber-500/30 hover:shadow-[0_0_15px_rgba(251,191,36,0.3)]"
                >
                  {clearingCache ? 'クリア中...' : 'すべてのキャッシュをクリア'}
                </Button>
              </div>

              <p className="mt-3 text-xs text-dt-text-sub">
                キャッシュはオフライン時にデータを表示するために使用されます。
                <br />
                期限切れのキャッシュはアプリ起動時に自動でクリーンアップされます。
              </p>
            </div>
          </div>

          {/* Divider */}
          <div className="border-t border-gm-accent-cyan/20"></div>

          {/* Data export section */}
          <div className="space-y-3">
            <h3 className="text-lg font-gaming font-bold text-white">データエクスポート</h3>
            <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
              <p className="text-dt-text-sub mb-4">
                統計データをJSON形式でエクスポートします。
                <br />
                XP、バッジ、統計情報などが含まれます。
              </p>
              <Button
                variant="primary"
                onClick={onExportData}
                disabled={exporting}
                fullWidth
                isLoading={exporting}
              >
                {exporting ? 'エクスポート中...' : 'データをエクスポート'}
              </Button>
            </div>
          </div>

          {/* Divider */}
          <div className="border-t border-gm-accent-cyan/20"></div>

          {/* Data reset section */}
          <div className="space-y-3">
            <h3 className="text-lg font-gaming font-bold text-red-400 flex items-center gap-2">
              <span>⚠️</span>
              危険な操作
            </h3>
            <div className="p-4 bg-red-900/20 rounded-xl border-2 border-red-500/30">
              <div className="flex items-start gap-3 mb-4">
                <div className="w-10 h-10 rounded-lg bg-red-500/20 flex items-center justify-center border border-red-500/30 flex-shrink-0">
                  <span className="text-xl">🚫</span>
                </div>
                <div>
                  <span className="text-red-200 font-gaming font-bold block text-lg">全データをリセット</span>
                  <span className="text-red-200/70 text-sm">
                    全てのXP、バッジ、統計データを <span className="font-bold">完全に削除</span> します
                  </span>
                </div>
              </div>
              <div className="p-3 bg-red-900/30 rounded-lg mb-4 border border-red-500/20">
                <p className="text-red-200/80 text-xs flex items-center gap-2">
                  <span>💡</span>
                  この操作を実行すると、リセットを確認するダイアログが表示されます
                </p>
              </div>
              <Button
                variant="danger"
                onClick={() => setShowResetDialog(true)}
                disabled={resetting}
                fullWidth
                isLoading={resetting}
                className="shadow-[0_0_15px_rgba(239,68,68,0.2)] hover:shadow-[0_0_25px_rgba(239,68,68,0.4)]"
              >
                {resetting ? 'リセット中...' : '全データをリセット'}
              </Button>
            </div>
          </div>

          {/* Divider */}
          <div className="border-t border-gm-accent-cyan/20"></div>

          {/* Database info section */}
          <div className="space-y-3">
            <h3 className="text-lg font-gaming font-bold text-white">データベース情報</h3>
            <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-dt-text-sub">パス</span>
                  <span
                    className="text-white text-sm font-mono truncate max-w-[200px]"
                    title={dbInfo?.path || ''}
                  >
                    {dbInfo
                      ? dbInfo.path.split(/[/\\]/).pop() || dbInfo.path
                      : '不明'}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-dt-text-sub">データベースサイズ</span>
                  <span className="text-gm-accent-cyan font-gaming">
                    {dbInfo ? formatBytes(dbInfo.sizeBytes) : '不明'}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-dt-text-sub">キャッシュサイズ</span>
                  <span className="text-gm-accent-cyan font-gaming">
                    {dbInfo ? formatBytes(dbInfo.cacheSizeBytes) : '不明'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
};
