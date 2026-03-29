/**
 * Appearance Settings Component
 *
 * React implementation of AppearanceSettings component.
 * Allows users to configure animation effects ON/OFF.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../settings/appearance_settings.rs
 */

import React, { useState, useEffect, useRef } from 'react';
import { useSettings } from '../../../stores/settingsStore';
import { useAnimation } from '../../../stores/animationStore';
import { ToggleSwitch } from '../../ui/form';

export const AppearanceSettings: React.FC = () => {
  const { settings, isLoading, error: storeError, updateSettings } = useSettings();
  const animationEnabled = useAnimation((s) => s.enabled);
  const setAnimationEnabled = useAnimation((s) => s.setEnabled);
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

  // Toggle animations
  const toggleAnimations = () => {
    if (!settings) return;

    const newValue = !settings.animationsEnabled;

    // Update global animation context immediately
    setAnimationEnabled(newValue);

    // Update settings
    updateSettings({
      ...settings,
      animationsEnabled: newValue,
    }).catch((e) => {
      setError(`設定の保存に失敗しました: ${e}`);
    });
  };

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (debounceHandleRef.current !== null) {
        clearTimeout(debounceHandleRef.current);
      }
    };
  }, []);

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
      {settings && !loading && (
        <div className="space-y-3">
          {/* Animation toggle */}
          <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <span className="text-white block font-gaming font-bold" id="animations-label">
                  アニメーション効果
                </span>
                <span className="text-sm text-dt-text-sub mt-1 block">
                  XP獲得、レベルアップ、バッジ獲得時の
                  <br />
                  アニメーション効果を有効にする
                </span>
              </div>
              <ToggleSwitch
                enabled={settings.animationsEnabled}
                onToggle={toggleAnimations}
                labelId="animations-label"
              />
            </div>
          </div>

          {/* Hint text */}
          <div className="text-xs text-dt-text-sub p-3 bg-gm-bg-card/30 rounded-lg">
            ※ OFFにするとパフォーマンスが向上する場合があります
          </div>

          {/* Animation preview (when enabled) */}
          {settings.animationsEnabled && (
            <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
              <h4 className="text-sm font-gaming font-bold text-white mb-3">プレビュー</h4>
              <div className="flex items-center justify-center gap-4">
                <div className="text-3xl animate-bounce">✨</div>
                <div className="text-3xl animate-pulse">🔥</div>
                <div className="text-3xl animate-bounce-slow">🏆</div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
