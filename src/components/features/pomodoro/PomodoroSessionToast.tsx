/**
 * Pomodoro Session Toast
 *
 * Listens for the `pomodoro:session-completed` window event and renders a
 * short-lived toast in the top-right. Mounted once at the layout root so
 * the user gets feedback regardless of the page they happen to be on when
 * the timer finishes.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 *   - Store: src/stores/sessionStore.ts (emits the event)
 */

import { useEffect, useState } from 'react';
import {
  POMODORO_SESSION_COMPLETED_EVENT,
  type PomodoroSessionCompletedDetail,
  type SessionPhase,
  sessionPhaseEmoji,
  sessionPhaseLabel,
} from '@/types/session';

interface CompletionToast {
  id: string;
  phase: SessionPhase;
  xp: number;
  completed: boolean;
}

const TOAST_TIMEOUT_MS = 4500;

export const PomodoroSessionToast = () => {
  const [toast, setToast] = useState<CompletionToast | null>(null);

  useEffect(() => {
    const handler = (event: Event) => {
      const detail = (event as CustomEvent<PomodoroSessionCompletedDetail>).detail;
      if (!detail) return;
      setToast({
        id: detail.record.id,
        phase: detail.record.phase,
        xp: detail.record.xpAwarded,
        completed: detail.record.completed,
      });
    };
    window.addEventListener(POMODORO_SESSION_COMPLETED_EVENT, handler);
    return () => window.removeEventListener(POMODORO_SESSION_COMPLETED_EVENT, handler);
  }, []);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), TOAST_TIMEOUT_MS);
    return () => window.clearTimeout(timer);
  }, [toast]);

  if (!toast) return null;

  return (
    // `key={toast.id}` forces React to remount this node for every new
    // session — without it the slide-in animation only plays once and the
    // second back-to-back completion lands silently in place.
    <div
      key={toast.id}
      className="fixed top-4 right-4 z-50 animate-slide-in"
      role="status"
      aria-live="polite"
    >
      <div className="p-4 bg-gm-bg-card/95 backdrop-blur-sm rounded-xl border border-gm-accent-cyan/30 shadow-neon-cyan min-w-72">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-2xl" aria-hidden="true">
            {sessionPhaseEmoji(toast.phase)}
          </span>
          <span className="font-gaming text-gm-accent-cyan">
            {toast.completed
              ? `${sessionPhaseLabel(toast.phase)}フェーズ完了!`
              : `${sessionPhaseLabel(toast.phase)}フェーズ中断`}
          </span>
        </div>
        {toast.xp > 0 && (
          <div className="text-sm text-dt-text-sub">
            <span className="text-gm-success font-gaming-mono">+{toast.xp} XP</span>{' '}
            を獲得しました
          </div>
        )}
        {toast.xp === 0 && toast.completed && toast.phase !== 'focus' && (
          <div className="text-sm text-dt-text-sub">
            休憩おつかれさまでした。次は集中フェーズです。
          </div>
        )}
      </div>
    </div>
  );
};
