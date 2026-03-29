/**
 * Notification Settings Component
 *
 * React implementation of NotificationSettings component.
 * Allows users to configure notification methods and individual notification types.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/notification_settings.rs
 */

import React, { useState, useEffect, useRef } from 'react';
import { useSettings } from '../../../stores/settingsStore';
import { ToggleSwitch } from '../../ui/form';
import type { NotificationMethod } from '../../../types';
import { notificationMethodFromStr, notificationMethodLabel } from '../../../types/settings';

export const NotificationSettings: React.FC = () => {
  const { settings, isLoading, error: storeError, updateSettings } = useSettings();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const initialLoadCompleteRef = useRef(false);
  const debounceHandleRef = useRef<number | null>(null);

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

  // Update notification method
  const updateNotificationMethod = (method: NotificationMethod) => {
    if (!settings) return;

    updateSettings({
      ...settings,
      notificationMethod: method,
    }).catch((e) => {
      setError(`設定の保存に失敗しました: ${e}`);
    });
  };

  // Toggle individual notification setting
  const toggleNotification = (field: 'xp_gain' | 'level_up' | 'badge_earned' | 'streak_update' | 'streak_milestone') => {
    if (!settings) return;

    const fieldMap = {
      xp_gain: 'notifyXpGain',
      level_up: 'notifyLevelUp',
      badge_earned: 'notifyBadgeEarned',
      streak_update: 'notifyStreakUpdate',
      streak_milestone: 'notifyStreakMilestone',
    } as const;

    const updateKey = fieldMap[field];
    const currentValue = settings[updateKey];

    updateSettings({
      ...settings,
      [updateKey]: !currentValue,
    }).catch((e) => {
      setError(`設定の保存に失敗しました: ${e}`);
    });
  };

  // Toggle all notifications on
  const toggleAllOn = () => {
    if (!settings) return;

    updateSettings({
      ...settings,
      notifyXpGain: true,
      notifyLevelUp: true,
      notifyBadgeEarned: true,
      notifyStreakUpdate: true,
      notifyStreakMilestone: true,
    }).catch((e) => {
      setError(`設定の保存に失敗しました: ${e}`);
    });
  };

  // Toggle all notifications off
  const toggleAllOff = () => {
    if (!settings) return;

    updateSettings({
      ...settings,
      notifyXpGain: false,
      notifyLevelUp: false,
      notifyBadgeEarned: false,
      notifyStreakUpdate: false,
      notifyStreakMilestone: false,
    }).catch((e) => {
      setError(`設定の保存に失敗しました: ${e}`);
    });
  };

  // Cleanup timeout on component unmount
  useEffect(() => {
    return () => {
      if (debounceHandleRef.current !== null) {
        clearTimeout(debounceHandleRef.current);
      }
    };
  }, []);

  const notificationOptions: NotificationMethod[] = ['app_only', 'os_only', 'both', 'none'];

  return (
    <div className="space-y-6">
      {/* Loading state */}
      {loading && (
        <div className="text-center py-8 text-dt-text-sub">設定を読み込み中...</div>
      )}

      {/* Error message */}
      {error && (
        <div className="p-3 bg-red-900/30 border border-red-500/50 rounded-lg text-red-200 text-sm">
          {error}
        </div>
      )}

      {/* Settings form */}
      {settings && !loading && (() => {
        const currentMethod = notificationMethodFromStr(settings.notificationMethod);

        return (
          <>
            {/* Notification method selection */}
            <div className="space-y-3">
              <h3 className="text-lg font-gaming font-bold text-white">通知方法</h3>
              <div className="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                {notificationOptions.map((method) => {
                  const isSelected = currentMethod === method;
                  return (
                    <label
                      key={method}
                      className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${
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
                        className="w-4 h-4 text-gm-accent-cyan bg-gm-bg-card border-gm-accent-cyan/50 focus:ring-gm-accent-cyan focus:ring-2"
                      />
                      <span className="text-white">{notificationMethodLabel(method)}</span>
                    </label>
                  );
                })}
              </div>
            </div>

            {/* Divider */}
            <div className="border-t border-gm-accent-cyan/20"></div>

            {/* Individual notification settings */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-gaming font-bold text-white">通知の種類</h3>
                <div className="flex gap-2">
                  <button
                    className="px-3 py-1 text-sm rounded-lg bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 text-gm-accent-cyan transition-colors"
                    onClick={toggleAllOn}
                  >
                    全てON
                  </button>
                  <button
                    className="px-3 py-1 text-sm rounded-lg bg-slate-700/50 hover:bg-slate-700/70 text-white transition-colors"
                    onClick={toggleAllOff}
                  >
                    全てOFF
                  </button>
                </div>
              </div>
              <div className="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
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
                  const value = settings[fieldMap[field]] as boolean;
                  return (
                    <div key={field} className="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                      <span className="text-white font-gaming">{label}</span>
                      <ToggleSwitch enabled={value} onToggle={() => toggleNotification(field)} />
                    </div>
                  );
                })}
              </div>
            </div>
          </>
        );
      })()}
    </div>
  );
};
