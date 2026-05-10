/**
 * Pomodoro Timer Display
 *
 * Big round dial showing the current phase, the remaining time, and a
 * progress ring. Phase + ring colour are driven by `useSession()` state.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 *   - Store: src/stores/sessionStore.ts
 */

import { useSession } from '@/stores/sessionStore';
import { formatRemaining, sessionPhaseEmoji, sessionPhaseLabel } from '@/types/session';

const RADIUS = 110;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

const phaseRingClass: Record<string, string> = {
  focus: 'stroke-gm-accent-cyan',
  short_break: 'stroke-gm-success',
  long_break: 'stroke-gm-accent-purple',
};

export const PomodoroTimer = () => {
  const phase = useSession((s) => s.phase);
  const status = useSession((s) => s.status);
  const remaining = useSession((s) => s.remainingSeconds);
  const phasePlannedSeconds = useSession((s) => s.phasePlannedSeconds);
  const phaseXpReward = useSession((s) => s.phaseXpReward);
  const longBreakInterval = useSession((s) => s.config.longBreakInterval);
  const cycleCount = useSession((s) => s.cycleCount);

  // Use the snapshot so a mid-session settings tweak doesn't make the ring
  // jump to the wrong fill — the snapshot freezes "what the user signed up
  // for" at start time.
  const total = Math.max(1, phasePlannedSeconds);
  const progress = Math.max(0, Math.min(1, 1 - remaining / total));
  const offset = CIRCUMFERENCE * (1 - progress);

  const ringClass = phaseRingClass[phase] ?? 'stroke-gm-accent-cyan';

  return (
    <div className="flex flex-col items-center">
      <div className="relative w-[260px] h-[260px]">
        <svg className="w-full h-full -rotate-90" viewBox="0 0 260 260">
          <circle
            cx="130"
            cy="130"
            r={RADIUS}
            className="stroke-slate-800 fill-none"
            strokeWidth="14"
          />
          <circle
            cx="130"
            cy="130"
            r={RADIUS}
            className={`fill-none transition-[stroke-dashoffset] duration-1000 ease-linear ${ringClass}`}
            strokeWidth="14"
            strokeLinecap="round"
            strokeDasharray={CIRCUMFERENCE}
            strokeDashoffset={offset}
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center text-center">
          <div className="text-3xl mb-1" aria-hidden="true">
            {sessionPhaseEmoji(phase)}
          </div>
          <div className="font-gaming-mono text-5xl font-bold text-dt-text tabular-nums">
            {formatRemaining(remaining)}
          </div>
          <div className="mt-2 text-sm font-gaming text-dt-text-sub uppercase tracking-wide">
            {sessionPhaseLabel(phase)}
          </div>
          <div className="mt-1 text-xs text-dt-text-sub">
            {status === 'idle' && '待機中'}
            {status === 'running' && '進行中'}
            {status === 'paused' && '一時停止中'}
          </div>
        </div>
      </div>

      <div className="mt-4 flex items-center gap-4 text-xs text-dt-text-sub">
        <span>
          サイクル{' '}
          <span className="text-dt-text font-gaming-mono">
            {cycleCount}/{longBreakInterval}
          </span>
        </span>
        {phase === 'focus' && (
          <span>
            完了で{' '}
            <span className="text-gm-accent-cyan font-gaming-mono">+{phaseXpReward} XP</span>
          </span>
        )}
      </div>
    </div>
  );
};
