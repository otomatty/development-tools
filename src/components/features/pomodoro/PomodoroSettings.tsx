/**
 * Pomodoro Settings
 *
 * Inline form for tweaking the focus / break / cycle / XP knobs. Edits are
 * applied immediately through `setConfig`, which clamps each field to a sane
 * range so the timer can't be locked at 0.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 */

import { Icon } from '@/components/icons';
import { useSession } from '@/stores/sessionStore';
import type { PomodoroConfig } from '@/types/session';

interface FieldProps {
  label: string;
  value: number;
  min: number;
  max: number;
  suffix?: string;
  onChange: (value: number) => void;
  description?: string;
}

const NumberField = ({ label, value, min, max, suffix, onChange, description }: FieldProps) => (
  <label className="block">
    <span className="block text-xs font-gaming text-dt-text-sub uppercase tracking-wide mb-1">
      {label}
    </span>
    <div className="flex items-center gap-2">
      <input
        type="number"
        value={value}
        min={min}
        max={max}
        onChange={(e) => {
          const next = Number(e.target.value);
          if (Number.isFinite(next)) onChange(next);
        }}
        className="w-24 px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-dt-text font-gaming-mono focus:border-gm-accent-cyan focus:outline-none"
      />
      {suffix && <span className="text-sm text-dt-text-sub">{suffix}</span>}
    </div>
    {description && (
      <span className="block mt-1 text-xs text-dt-text-sub">{description}</span>
    )}
  </label>
);

export const PomodoroSettings = () => {
  const config = useSession((s) => s.config);
  const setConfig = useSession((s) => s.setConfig);
  const resetConfig = useSession((s) => s.resetConfig);
  const status = useSession((s) => s.status);
  const isRunning = status !== 'idle';

  const update = <K extends keyof PomodoroConfig>(key: K, value: PomodoroConfig[K]) => {
    setConfig({ [key]: value } as Partial<PomodoroConfig>);
  };

  return (
    <div className="bg-gm-bg-card/60 backdrop-blur-sm rounded-2xl border border-slate-700/50 p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Icon name="settings" className="w-5 h-5 text-gm-accent-cyan" />
          <h2 className="font-gaming text-lg text-dt-text">タイマー設定</h2>
        </div>
        <button
          type="button"
          onClick={resetConfig}
          className="text-xs text-dt-text-sub hover:text-dt-text flex items-center gap-1 transition-colors"
          title="既定値に戻す"
        >
          <Icon name="rotate-ccw" className="w-3.5 h-3.5" />
          既定値
        </button>
      </div>

      {isRunning && (
        <p className="text-xs text-dt-text-sub mb-3 italic">
          ※ 進行中の変更は次のフェーズから反映されます
        </p>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <NumberField
          label="集中時間"
          value={config.focusMinutes}
          min={1}
          max={180}
          suffix="分"
          onChange={(v) => update('focusMinutes', v)}
        />
        <NumberField
          label="短い休憩"
          value={config.shortBreakMinutes}
          min={1}
          max={60}
          suffix="分"
          onChange={(v) => update('shortBreakMinutes', v)}
        />
        <NumberField
          label="長い休憩"
          value={config.longBreakMinutes}
          min={1}
          max={120}
          suffix="分"
          onChange={(v) => update('longBreakMinutes', v)}
        />
        <NumberField
          label="長い休憩までの集中回数"
          value={config.longBreakInterval}
          min={1}
          max={12}
          suffix="回"
          onChange={(v) => update('longBreakInterval', v)}
          description="この回数の集中フェーズ完了で長い休憩に切り替わる"
        />
        <NumberField
          label="集中完了 XP"
          value={config.focusCompletionXp}
          min={0}
          max={1000}
          suffix="XP"
          onChange={(v) => update('focusCompletionXp', v)}
        />
        <label className="block">
          <span className="block text-xs font-gaming text-dt-text-sub uppercase tracking-wide mb-1">
            自動継続
          </span>
          <label className="flex items-center gap-2 px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg cursor-pointer">
            <input
              type="checkbox"
              checked={config.autoStartNext}
              onChange={(e) => update('autoStartNext', e.target.checked)}
              className="accent-gm-accent-cyan"
            />
            <span className="text-sm text-dt-text">次のフェーズを自動開始</span>
          </label>
        </label>
      </div>
    </div>
  );
};
