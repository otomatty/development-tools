/**
 * Settings Reset Component
 *
 * React implementation of SettingsReset component.
 * Allows users to reset all settings to defaults.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/settings_reset.rs
 */

import React, { useState } from 'react';
import { settings as settingsApi } from '../../../lib/tauri/commands';
import { ConfirmDialog } from '../../ui/dialog';
import { Button } from '../../ui/button';

// Settings reset confirmation dialog
const SettingsResetDialog: React.FC<{
  visible: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}> = ({ visible, onConfirm, onCancel }) => {
  return (
    <ConfirmDialog
      title="設定をリセットしますか？"
      message="全ての設定がデフォルト値に戻ります。XP・バッジ・統計データは削除されません。"
      confirmLabel="リセット"
      cancelLabel="キャンセル"
      visible={visible}
      onConfirm={onConfirm}
      onCancel={onCancel}
    />
  );
};

export const SettingsReset: React.FC = () => {
  const [showDialog, setShowDialog] = useState(false);
  const [resetting, setResetting] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const onResetClick = () => {
    setShowDialog(true);
  };

  const onConfirm = async () => {
    setShowDialog(false);
    setResetting(true);
    setSuccessMessage(null);
    setErrorMessage(null);

    try {
      await settingsApi.reset();
      setSuccessMessage('設定をリセットしました');
      // Clear success message after 3 seconds
      setTimeout(() => {
        setSuccessMessage(null);
      }, 3000);
    } catch (e) {
      setErrorMessage(`設定のリセットに失敗しました: ${e}`);
    } finally {
      setResetting(false);
    }
  };

  const onCancel = () => {
    setShowDialog(false);
  };

  return (
    <div className="space-y-4">
      {/* Success message */}
      {successMessage && (
        <div className="p-3 bg-green-900/20 border border-green-500/30 rounded-lg">
          <p className="text-green-300 text-sm">✅ {successMessage}</p>
        </div>
      )}

      {/* Error message */}
      {errorMessage && (
        <div className="p-3 bg-red-900/20 border border-red-500/30 rounded-lg">
          <p className="text-red-300 text-sm">{errorMessage}</p>
        </div>
      )}

      {/* Reset button section */}
      <div className="p-4 bg-gm-bg-darker/50 rounded-xl border border-gm-accent-cyan/20">
        <div className="flex items-center justify-between">
          <div>
            <h4 className="text-white font-semibold">全ての設定をリセット</h4>
            <p className="text-dt-text-sub text-sm mt-1">
              設定をデフォルト値に戻します（データは削除されません）
            </p>
          </div>
          <Button variant="outline" onClick={onResetClick} disabled={resetting}>
            {resetting ? 'リセット中...' : 'リセット'}
          </Button>
        </div>
      </div>

      {/* Confirmation dialog */}
      <SettingsResetDialog visible={showDialog} onConfirm={onConfirm} onCancel={onCancel} />
    </div>
  );
};
