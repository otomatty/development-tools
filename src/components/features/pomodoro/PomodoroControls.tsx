/**
 * Pomodoro Controls
 *
 * Start / Pause / Resume / Skip / Stop button row. Hides actions that don't
 * apply to the current `status` so the user can't pause an idle timer or
 * resume a running one.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 */

import { Icon } from '@/components/icons';
import { useSession } from '@/stores/sessionStore';

interface ControlButtonProps {
  label: string;
  icon: string;
  onClick: () => void;
  variant?: 'primary' | 'secondary' | 'danger';
}

const ControlButton = ({ label, icon, onClick, variant = 'secondary' }: ControlButtonProps) => {
  const variantClass =
    variant === 'primary'
      ? 'bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white shadow-neon-cyan hover:opacity-90'
      : variant === 'danger'
        ? 'bg-slate-800 text-red-400 border border-red-400/30 hover:bg-red-500/10'
        : 'bg-slate-800 text-dt-text border border-slate-700 hover:bg-slate-700';

  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex items-center gap-2 px-5 py-2.5 rounded-lg font-gaming text-sm font-semibold transition-all duration-200 ${variantClass}`}
      aria-label={label}
    >
      <Icon name={icon} className="w-4 h-4" />
      <span>{label}</span>
    </button>
  );
};

export const PomodoroControls = () => {
  const status = useSession((s) => s.status);
  const start = useSession((s) => s.start);
  const pause = useSession((s) => s.pause);
  const resume = useSession((s) => s.resume);
  const stop = useSession((s) => s.stop);
  const skipPhase = useSession((s) => s.skipPhase);

  return (
    <div className="flex flex-wrap items-center justify-center gap-3">
      {status === 'idle' && (
        <ControlButton label="開始" icon="play" onClick={() => start()} variant="primary" />
      )}
      {status === 'running' && (
        <ControlButton label="一時停止" icon="pause" onClick={pause} variant="primary" />
      )}
      {status === 'paused' && (
        <ControlButton label="再開" icon="play" onClick={resume} variant="primary" />
      )}
      {status !== 'idle' && (
        <ControlButton label="スキップ" icon="skip-forward" onClick={skipPhase} />
      )}
      {status !== 'idle' && (
        <ControlButton label="停止" icon="stop" onClick={stop} variant="danger" />
      )}
    </div>
  );
};
