/**
 * Active Session Sidebar Widget
 *
 * Compact remaining-time pill rendered in the sidebar while a Pomodoro
 * session is active. Hidden in the `idle` state — Pomodoro users who are
 * not currently in a session shouldn't see chrome they don't need.
 *
 * Clicking the widget navigates to the dedicated Pomodoro page so the user
 * can pause / stop / skip without losing context.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 */

import { Link } from 'react-router-dom';
import { Icon } from '@/components/icons';
import { useSession } from '@/stores/sessionStore';
import { formatRemaining, sessionPhaseEmoji, sessionPhaseLabel } from '@/types/session';

const phaseAccent: Record<string, string> = {
  focus: 'border-gm-accent-cyan/40 text-gm-accent-cyan',
  short_break: 'border-gm-success/40 text-gm-success',
  long_break: 'border-gm-accent-purple/40 text-gm-accent-purple',
};

export const SessionWidget = () => {
  const status = useSession((s) => s.status);
  const phase = useSession((s) => s.phase);
  const remaining = useSession((s) => s.remainingSeconds);

  if (status === 'idle') return null;

  const accent = phaseAccent[phase] ?? 'border-gm-accent-cyan/40 text-gm-accent-cyan';

  return (
    <Link
      to="/pomodoro"
      className={`block px-3 py-2 mx-2 mb-2 rounded-lg bg-slate-800/80 border ${accent} hover:bg-slate-800 transition-colors`}
      title={`${sessionPhaseLabel(phase)}フェーズ（${status === 'paused' ? '一時停止中' : '進行中'}）`}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <span aria-hidden="true">{sessionPhaseEmoji(phase)}</span>
          <span className="text-xs font-gaming uppercase tracking-wide">
            {sessionPhaseLabel(phase)}
          </span>
        </div>
        <span className="font-gaming-mono text-sm tabular-nums">
          {formatRemaining(remaining)}
        </span>
      </div>
      <div className="mt-1 flex items-center gap-1 text-[10px] text-dt-text-sub">
        <Icon name={status === 'paused' ? 'pause' : 'play'} className="w-3 h-3" />
        <span>{status === 'paused' ? '一時停止中' : '進行中'}</span>
      </div>
    </Link>
  );
};
