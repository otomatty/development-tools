/**
 * Account Settings Component
 *
 * Solid.js implementation of AccountSettings component.
 * Displays account information and provides logout functionality.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/account_settings.rs
 */

import { Component, Show, createSignal } from 'solid-js';
import { useAuth } from '../../../stores/authStore';
import { useAppNavigation } from '../../../lib/navigation';
import { AppPage } from '../../../types';
import { auth as authApi } from '../../../lib/tauri/commands';
import { ConfirmDialog } from '../../ui/dialog';
import { Button } from '../../ui/button';

export const AccountSettings: Component = () => {
  const auth = useAuth();
  const navigation = useAppNavigation();
  const [showLogoutDialog, setShowLogoutDialog] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [successMessage, setSuccessMessage] = createSignal<string | null>(null);

  // Format date helper - extract date part from RFC3339 string
  const formatDate = (dateStr: string | null | undefined): string => {
    if (!dateStr) return '不明';
    // RFC3339 format: "2024-11-26T15:30:00Z" -> extract "2024-11-26"
    const datePart = dateStr.split('T')[0];
    return datePart || '不明';
  };

  // Handle token validation
  const handleValidateToken = async () => {
    setLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const isValid = await authApi.validateToken();
      if (isValid) {
        setSuccessMessage('トークンは有効です');
      } else {
        setError('トークンが無効です。再認証が必要です。');
      }
    } catch (e) {
      setError(`トークンの確認に失敗しました: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  // Handle logout
  const handleLogout = async () => {
    setShowLogoutDialog(false);
    setLoading(true);
    setError(null);

    try {
      await auth.logout();
      navigation.goTo(AppPage.Home);
    } catch (e) {
      setError(`ログアウトに失敗しました: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const user = () => auth.store.state.user;

  return (
    <div class="space-y-4">
      {/* Account info section */}
      <Show
        when={user()}
        fallback={
          <div class="text-dt-text-sub text-center py-8">
            アカウント情報を取得できませんでした
          </div>
        }
      >
        {(u) => (
          <div class="flex items-center gap-4 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
            {/* Avatar */}
            <img
              src={u().avatarUrl || ''}
              alt={`${u().username}のアバター`}
              class="w-16 h-16 rounded-xl border-2 border-gm-accent-cyan"
            />
            <div class="flex-1">
              <div class="text-white font-gaming font-bold text-lg">@{u().username}</div>
              <div class="text-dt-text-sub text-sm mt-1">GitHub ID: {u().githubId}</div>
              <div class="text-dt-text-sub text-sm">
                接続日: {formatDate(u().createdAt)}
              </div>
            </div>
          </div>
        )}
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

      {/* Action buttons */}
      <div class="flex gap-3">
        <Button
          variant="primary"
          onClick={handleValidateToken}
          disabled={loading()}
          class="flex-1"
        >
          {loading() ? '確認中...' : '🔄 トークンを確認'}
        </Button>
        <Button
          variant="danger"
          onClick={() => setShowLogoutDialog(true)}
          disabled={loading()}
          class="flex-1"
        >
          🚪 ログアウト
        </Button>
      </div>

      {/* Note */}
      <div class="text-xs text-dt-text-sub p-3 bg-gm-bg-card/30 rounded-lg">
        ※ログアウトしてもXP・バッジ・統計データは保持されます
      </div>

      {/* Logout confirmation dialog */}
      <ConfirmDialog
        title="ログアウトの確認"
        message="ログアウトしますか？トークンは削除されますが、XP・バッジ・統計データは保持されます。"
        confirmLabel="ログアウト"
        cancelLabel="キャンセル"
        visible={showLogoutDialog()}
        onConfirm={handleLogout}
        onCancel={() => setShowLogoutDialog(false)}
      />
    </div>
  );
};

