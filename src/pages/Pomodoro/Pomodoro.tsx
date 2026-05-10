/**
 * Pomodoro Page
 *
 * Hosts the focus-session timer, its controls, settings, and history list.
 * Completed focus sessions emit a `pomodoro:session-completed` window event
 * which we listen to here so a small celebratory toast can render on the
 * page itself. XP is granted from the store via the existing
 * `gamification.addXp` Tauri command.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 *   - Store: src/stores/sessionStore.ts
 *   - Components: src/components/features/pomodoro/*
 */

import { useEffect, useState } from 'react';
import {
  PomodoroControls,
  PomodoroSettings,
  PomodoroTimer,
  SessionHistory,
} from '@/components/features/pomodoro';
import {
  POMODORO_SESSION_COMPLETED_EVENT,
  type PomodoroSessionCompletedDetail,
  sessionPhaseEmoji,
  sessionPhaseLabel,
} from '@/types/session';

interface CompletionToast {
  id: string;
  phase: PomodoroSessionCompletedDetail['record']['phase'];
  xp: number;
  completed: boolean;
}

export const Pomodoro = () => {
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
    const timer = window.setTimeout(() => setToast(null), 4500);
    return () => window.clearTimeout(timer);
  }, [toast]);

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="max-w-4xl mx-auto space-y-6">
        <header>
          <h1 className="text-3xl font-gaming text-gm-accent-cyan">Pomodoro</h1>
          <p className="text-sm text-dt-text-sub mt-1">
            集中と休憩のサイクルを回して、コーディングのリズムを作りましょう。集中フェーズを完了すると XP を獲得できます。
          </p>
        </header>

        <section className="bg-gm-bg-card/60 backdrop-blur-sm rounded-2xl border border-slate-700/50 p-6 sm:p-8">
          <div className="flex flex-col items-center gap-6">
            <PomodoroTimer />
            <PomodoroControls />
          </div>
        </section>

        <PomodoroSettings />
        <SessionHistory />
      </div>

      {toast && (
        <div
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
              <div className="text-sm text-dt-text-sub">休憩おつかれさまでした。次は集中フェーズです。</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default Pomodoro;
