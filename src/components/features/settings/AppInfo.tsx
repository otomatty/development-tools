/**
 * App Info Component
 *
 * Solid.js implementation of AppInfoSection component.
 * Displays application version, build info, and provides links to resources.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/app_info.rs
 */

import { Component, Show, createSignal, createResource } from 'solid-js';
import { settings as settingsApi } from '../../../lib/tauri/commands';
import { Button } from '../../ui/button';
import type { AppInfo } from '../../../types';

const GITHUB_REPO_URL = 'https://github.com/otomatty/development-tools';

// Info item component for displaying version/build info
const InfoItem: Component<{ label: string; value: string }> = (props) => {
  return (
    <div class="p-3 bg-gm-bg-darker/50 rounded-lg">
      <dt class="text-xs text-dt-text-sub uppercase tracking-wider">{props.label}</dt>
      <dd class="mt-1 text-white font-mono text-sm">{props.value}</dd>
    </div>
  );
};

export const AppInfo: Component = () => {
  const [openingUrl, setOpeningUrl] = createSignal(false);

  // Load app info
  const [appInfo] = createResource<AppInfo>(async () => {
    return await settingsApi.getAppInfo();
  });

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
    <div class="space-y-6">
      {/* Loading state */}
      <Show when={appInfo.loading}>
        <div class="flex items-center justify-center py-8">
          <div class="animate-spin w-8 h-8 border-4 border-gm-accent-cyan/30 border-t-gm-accent-cyan rounded-full"></div>
        </div>
      </Show>

      {/* Error state */}
      <Show when={appInfo.error}>
        <div class="p-4 bg-red-900/20 border border-red-500/30 rounded-xl">
          <p class="text-red-300 text-sm">
            アプリ情報の取得に失敗しました: {appInfo.error?.message || String(appInfo.error)}
          </p>
        </div>
      </Show>

      {/* App info display */}
      <Show when={appInfo()}>
        {(info) => (
          <div class="space-y-6">
            {/* App name and icon */}
            <div class="flex items-center gap-4">
              <div class="w-16 h-16 rounded-xl bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple flex items-center justify-center">
                <span class="text-3xl">🛠️</span>
              </div>
              <div>
                <h3 class="text-xl font-gaming font-bold text-white">Development Tools</h3>
                <p class="text-dt-text-sub">開発者向けツールコレクション</p>
              </div>
            </div>

            {/* Version info */}
            <div class="grid grid-cols-2 gap-4">
              <InfoItem label="バージョン" value={info().version} />
              <InfoItem label="ビルド日時" value={info().buildDate} />
              <InfoItem label="Tauri" value={info().tauriVersion} />
              <InfoItem label="Rust" value={info().rustVersion} />
            </div>

            {/* Action buttons */}
            <div class="flex flex-wrap gap-3 pt-4 border-t border-gm-accent-cyan/20">
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
              <Button variant="secondary" onClick={openGithub} disabled={openingUrl()}>
                {openingUrl() ? '開いています...' : '🐙 GitHubリポジトリ'}
              </Button>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
};

