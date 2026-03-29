/**
 * Account Settings Component
 *
 * React implementation of AccountSettings component.
 * Displays account information and provides logout functionality.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/account_settings.rs
 */

import React, { useState } from 'react';
import { useAuth } from '../../../stores/authStore';
import { useAppNavigation } from '../../../lib/navigation';
import { AppPage } from '../../../types';
import { auth as authApi } from '../../../lib/tauri/commands';
import { ConfirmDialog } from '../../ui/dialog';
import { Button } from '../../ui/button';

export const AccountSettings: React.FC = () => {
  const logout = useAuth((s) => s.logout);
  const user = useAuth((s) => s.state.user);
  const navigation = useAppNavigation();
  const [showLogoutDialog, setShowLogoutDialog] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Format date helper - extract date part from RFC3339 string, with validation
  const formatDate = (dateStr: string | null | undefined): string => {
    if (!dateStr) return '不明';
    // Try to parse the date string
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return '不明';
    // Format as YYYY-MM-DD
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
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
      await logout();
      navigation.goTo(AppPage.Home);
    } catch (e) {
      setError(`ログアウトに失敗しました: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* Account info section */}
      {user ? (
        <div className="flex items-center gap-4 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
          {/* Avatar */}
          <img
            src={user.avatarUrl || ''}
            alt={`${user.username}のアバター`}
            className="w-16 h-16 rounded-xl border-2 border-gm-accent-cyan"
          />
          <div className="flex-1">
            <div className="text-white font-gaming font-bold text-lg">@{user.username}</div>
            <div className="text-dt-text-sub text-sm mt-1">GitHub ID: {user.githubId}</div>
            <div className="text-dt-text-sub text-sm">
              接続日: {formatDate(user.createdAt)}
            </div>
          </div>
        </div>
      ) : (
        <div className="text-dt-text-sub text-center py-8">
          アカウント情報を取得できませんでした
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="p-3 bg-red-900/30 border border-red-500/50 rounded-lg text-red-200 text-sm">
          {error}
        </div>
      )}

      {/* Success message */}
      {successMessage && (
        <div className="p-3 bg-green-900/30 border border-green-500/50 rounded-lg text-green-200 text-sm">
          {successMessage}
        </div>
      )}

      {/* Action buttons */}
      <div className="flex gap-3">
        <Button
          variant="primary"
          onClick={handleValidateToken}
          disabled={loading}
          className="flex-1"
        >
          {loading ? '確認中...' : '🔄 トークンを確認'}
        </Button>
        <Button
          variant="danger"
          onClick={() => setShowLogoutDialog(true)}
          disabled={loading}
          className="flex-1"
        >
          🚪 ログアウト
        </Button>
      </div>

      {/* Note */}
      <div className="text-xs text-dt-text-sub p-3 bg-gm-bg-card/30 rounded-lg">
        ※ログアウトしてもXP・バッジ・統計データは保持されます
      </div>

      {/* Logout confirmation dialog */}
      <ConfirmDialog
        title="ログアウトの確認"
        message="ログアウトしますか？トークンは削除されますが、XP・バッジ・統計データは保持されます。"
        confirmLabel="ログアウト"
        cancelLabel="キャンセル"
        visible={showLogoutDialog}
        onConfirm={handleLogout}
        onCancel={() => setShowLogoutDialog(false)}
      />
    </div>
  );
};
