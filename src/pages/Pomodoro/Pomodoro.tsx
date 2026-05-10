/**
 * Pomodoro Page
 *
 * Hosts the focus-session timer, its controls, settings, and history list.
 * The completion toast lives in `MainLayout` so it shows up regardless of
 * which page the user is on when a session ends.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 *   - Store: src/stores/sessionStore.ts
 *   - Components: src/components/features/pomodoro/*
 */

import {
  PomodoroControls,
  PomodoroSettings,
  PomodoroTimer,
  SessionHistory,
} from '@/components/features/pomodoro';

export const Pomodoro = () => {
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
    </div>
  );
};

export default Pomodoro;
