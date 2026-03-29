/**
 * App Info Component
 *
 * React implementation of AppInfoSection component.
 * Displays application version, build info, and provides links to resources.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/app_info.rs
 */

import React, { useState, useEffect, useCallback } from 'react';
import { settings as settingsApi } from '../../../lib/tauri/commands';
import { Button } from '../../ui/button';
import type { AppInfo as AppInfoType } from '../../../types';

const GITHUB_REPO_URL = 'https://github.com/otomatty/development-tools';

// Info item component for displaying version/build info
const InfoItem: React.FC<{ label: string; value: string }> = ({ label, value }) => {
  return (
    <div className="p-3 bg-gm-bg-darker/50 rounded-lg">
      <dt className="text-xs text-dt-text-sub uppercase tracking-wider">{label}</dt>
      <dd className="mt-1 text-white font-mono text-sm">{value}</dd>
    </div>
  );
};

export const AppInfo: React.FC = () => {
  const [openingUrl, setOpeningUrl] = useState(false);

  // Load app info
  const [appInfo, setAppInfo] = useState<AppInfoType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchAppInfo = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await settingsApi.getAppInfo();
      setAppInfo(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAppInfo();
  }, [fetchAppInfo]);

  // Open GitHub repository
  const openGithub = async () => {
    setOpeningUrl(true);
    try {
      await settingsApi.openExternalUrl(GITHUB_REPO_URL);
    } catch (e) {
      console.error('Failed to open URL:', e);
    } finally {
      setOpeningUrl(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Loading state */}
      {loading && (
        <div className="flex items-center justify-center py-8">
          <div className="animate-spin w-8 h-8 border-4 border-gm-accent-cyan/30 border-t-gm-accent-cyan rounded-full"></div>
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="p-4 bg-red-900/20 border border-red-500/30 rounded-xl">
          <p className="text-red-300 text-sm">
            アプリ情報の取得に失敗しました: {error}
          </p>
        </div>
      )}

      {/* App info display */}
      {appInfo && (
        <div className="space-y-6">
          {/* App name and icon */}
          <div className="flex items-center gap-4">
            <div className="w-16 h-16 rounded-xl bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple flex items-center justify-center">
              <span className="text-3xl">🛠️</span>
            </div>
            <div>
              <h3 className="text-xl font-gaming font-bold text-white">Development Tools</h3>
              <p className="text-dt-text-sub">開発者向けツールコレクション</p>
            </div>
          </div>

          {/* Version info */}
          <div className="grid grid-cols-2 gap-4">
            <InfoItem label="バージョン" value={appInfo.version} />
            <InfoItem label="ビルド日時" value={appInfo.buildDate} />
            <InfoItem label="Tauri" value={appInfo.tauriVersion} />
            <InfoItem label="Rust" value={appInfo.rustVersion} />
          </div>

          {/* Action buttons */}
          <div className="flex flex-wrap gap-3 pt-4 border-t border-gm-accent-cyan/20">
            {/* License info button (placeholder) */}
            <Button
              variant="secondary"
              onClick={() => {
                // TODO: Show license modal
                console.log('License info clicked');
              }}
            >
              📄 ライセンス情報
            </Button>

            {/* GitHub repo button */}
            <Button variant="secondary" onClick={openGithub} disabled={openingUrl}>
              {openingUrl ? '開いています...' : '🐙 GitHubリポジトリ'}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
};
