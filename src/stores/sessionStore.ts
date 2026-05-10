/**
 * Session (Pomodoro) Store
 *
 * Holds the active focus / break timer plus local history of completed
 * sessions. Persistence is intentionally local-only (`localStorage`) — the
 * Pomodoro feature is fully self-contained per the issue, and writing through
 * to the SQLite-backed `xp_history` only happens via the existing
 * `gamification.addXp` Tauri command on focus-session completion.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 *   - Types: src/types/session.ts
 *   - Tauri API: src/lib/tauri/commands.ts (gamification.addXp)
 */

import { create } from 'zustand';
import {
  DEFAULT_POMODORO_CONFIG,
  POMODORO_SESSION_COMPLETED_EVENT,
  type PomodoroConfig,
  type PomodoroSessionCompletedDetail,
  type SessionPhase,
  type SessionRecord,
  type SessionStatus,
  sessionPhaseMinutes,
} from '@/types/session';
import { gamification } from '@/lib/tauri/commands';

const STORAGE_KEY = 'development-tools.pomodoro.v1';
const HISTORY_LIMIT = 200;

interface PersistedState {
  config: PomodoroConfig;
  history: SessionRecord[];
  cycleCount: number;
}

const loadPersisted = (): PersistedState => {
  if (typeof window === 'undefined') {
    return { config: DEFAULT_POMODORO_CONFIG, history: [], cycleCount: 0 };
  }
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return { config: DEFAULT_POMODORO_CONFIG, history: [], cycleCount: 0 };
    const parsed = JSON.parse(raw) as Partial<PersistedState>;
    return {
      config: { ...DEFAULT_POMODORO_CONFIG, ...(parsed.config ?? {}) },
      history: Array.isArray(parsed.history) ? parsed.history.slice(0, HISTORY_LIMIT) : [],
      cycleCount: typeof parsed.cycleCount === 'number' ? parsed.cycleCount : 0,
    };
  } catch (err) {
    console.error('[sessionStore] Failed to parse persisted state:', err);
    return { config: DEFAULT_POMODORO_CONFIG, history: [], cycleCount: 0 };
  }
};

const persist = (state: PersistedState) => {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (err) {
    console.error('[sessionStore] Failed to persist state:', err);
  }
};

const generateId = (): string =>
  `pomo_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;

const dispatchSessionCompleted = (detail: PomodoroSessionCompletedDetail) => {
  if (typeof window === 'undefined') return;
  window.dispatchEvent(
    new CustomEvent<PomodoroSessionCompletedDetail>(POMODORO_SESSION_COMPLETED_EVENT, {
      detail,
    }),
  );
};

const computeTotalFocusCompleted = (history: SessionRecord[]): number =>
  history.filter((r) => r.phase === 'focus' && r.completed).length;

interface SessionStoreState {
  config: PomodoroConfig;
  status: SessionStatus;
  phase: SessionPhase;
  /// 現在のフェーズの開始時刻（ISO8601）
  phaseStartedAt: string | null;
  /// `running` のとき、フェーズが終わる Date.now() の値
  endsAt: number | null;
  /// `paused` のときに保持する残り秒数
  remainingSeconds: number;
  /// このフェーズで実際にカウントされた秒数（一時停止分を除く）
  elapsedSeconds: number;
  /// long_break 周期内で完了した focus フェーズ数（long_break 後にリセット）
  cycleCount: number;
  /// 通算で完了した focus フェーズ数（履歴の下限）
  totalFocusCompleted: number;
  history: SessionRecord[];

  start: (phase?: SessionPhase) => void;
  pause: () => void;
  resume: () => void;
  /// セッションを完全に停止（中断扱いで履歴に記録、idle に戻る）
  stop: () => void;
  /// 現在のフェーズをスキップして次のフェーズへ（中断扱い）
  skipPhase: () => void;
  setConfig: (patch: Partial<PomodoroConfig>) => void;
  resetConfig: () => void;
  clearHistory: () => void;
  /// タイマーティック（内部用）— 1秒ごとに残り時間を更新し、0 で完了
  tick: () => void;
}

const persisted = loadPersisted();
const initialPhase: SessionPhase = 'focus';
const initialRemaining = persisted.config.focusMinutes * 60;

interface FinalizeResult {
  history: SessionRecord[];
  cycleCount: number;
  totalFocusCompleted: number;
  record: SessionRecord;
}

/// 純粋関数: 現在のフェーズを履歴に書き込み、付与すべき XP を計算する。
/// 外部副作用（XP 加算 API 呼び出し / イベント発火 / 永続化）は呼び出し側で行う。
const finalizePhasePure = (state: SessionStoreState, completed: boolean): FinalizeResult | null => {
  if (state.phaseStartedAt === null) return null;

  const planned = Math.max(
    1,
    Math.floor(sessionPhaseMinutes(state.phase, state.config) * 60),
  );
  const remaining = state.endsAt
    ? Math.max(0, Math.ceil((state.endsAt - Date.now()) / 1000))
    : state.remainingSeconds;
  const actual = completed ? planned : Math.max(0, planned - remaining);
  const xpAwarded =
    completed && state.phase === 'focus' ? state.config.focusCompletionXp : 0;

  const record: SessionRecord = {
    id: generateId(),
    phase: state.phase,
    startedAt: state.phaseStartedAt,
    endedAt: new Date().toISOString(),
    plannedDurationSeconds: planned,
    actualDurationSeconds: actual,
    completed,
    xpAwarded,
  };

  const nextHistory = [record, ...state.history].slice(0, HISTORY_LIMIT);
  const nextCycleCount =
    completed && state.phase === 'focus' ? state.cycleCount + 1 : state.cycleCount;
  const nextTotalFocus =
    completed && state.phase === 'focus'
      ? state.totalFocusCompleted + 1
      : state.totalFocusCompleted;

  return {
    history: nextHistory,
    cycleCount: nextCycleCount,
    totalFocusCompleted: nextTotalFocus,
    record,
  };
};

/// 副作用付きの後処理: XP 加算 + イベント発火 + localStorage 永続化。
const applyFinalizeSideEffects = (
  state: SessionStoreState,
  result: FinalizeResult,
): void => {
  persist({
    config: state.config,
    history: result.history,
    cycleCount: result.cycleCount,
  });

  if (result.record.xpAwarded > 0) {
    void gamification
      .addXp(
        result.record.xpAwarded,
        'pomodoro_focus',
        `Pomodoro 集中セッション完了 (${state.config.focusMinutes}分)`,
      )
      .catch((err) => {
        // XP grant failures shouldn't block the timer flow — log and let the
        // next focus completion try again. Most likely cause is "Not logged
        // in" when the user hasn't authenticated with GitHub yet, which is a
        // legitimate state for someone using the timer in isolation.
        console.warn('[sessionStore] addXp failed:', err);
      });
  }

  dispatchSessionCompleted({
    record: result.record,
    totalFocusCompleted: result.totalFocusCompleted,
  });
};

/// 次のフェーズを決定する純粋関数（cycleCount は finalize 後の値を渡すこと）。
const computeNextPhase = (
  finishedPhase: SessionPhase,
  cycleCountAfterFinalize: number,
  config: PomodoroConfig,
): { phase: SessionPhase; cycleCount: number } => {
  if (finishedPhase === 'focus') {
    const ready =
      cycleCountAfterFinalize > 0 && cycleCountAfterFinalize % config.longBreakInterval === 0;
    return { phase: ready ? 'long_break' : 'short_break', cycleCount: cycleCountAfterFinalize };
  }
  if (finishedPhase === 'long_break') {
    return { phase: 'focus', cycleCount: 0 };
  }
  return { phase: 'focus', cycleCount: cycleCountAfterFinalize };
};

export const useSession = create<SessionStoreState>((set, get) => ({
  config: persisted.config,
  status: 'idle',
  phase: initialPhase,
  phaseStartedAt: null,
  endsAt: null,
  remainingSeconds: initialRemaining,
  elapsedSeconds: 0,
  cycleCount: persisted.cycleCount,
  totalFocusCompleted: computeTotalFocusCompleted(persisted.history),
  history: persisted.history,

  start: (phase) => {
    const state = get();
    const targetPhase = phase ?? state.phase;
    const minutes = sessionPhaseMinutes(targetPhase, state.config);
    const seconds = Math.max(1, Math.floor(minutes * 60));
    const now = Date.now();
    set({
      status: 'running',
      phase: targetPhase,
      phaseStartedAt: new Date(now).toISOString(),
      endsAt: now + seconds * 1000,
      remainingSeconds: seconds,
      elapsedSeconds: 0,
    });
  },

  pause: () => {
    const state = get();
    if (state.status !== 'running' || state.endsAt === null) return;
    const remaining = Math.max(0, Math.ceil((state.endsAt - Date.now()) / 1000));
    set({
      status: 'paused',
      endsAt: null,
      remainingSeconds: remaining,
    });
  },

  resume: () => {
    const state = get();
    if (state.status !== 'paused') return;
    const seconds = Math.max(1, state.remainingSeconds);
    set({
      status: 'running',
      endsAt: Date.now() + seconds * 1000,
      remainingSeconds: seconds,
    });
  },

  stop: () => {
    const state = get();
    if (state.status === 'idle') return;
    const result = finalizePhasePure(state, false);
    if (result) {
      set({
        history: result.history,
        cycleCount: result.cycleCount,
        totalFocusCompleted: result.totalFocusCompleted,
      });
      applyFinalizeSideEffects(state, result);
    }
    const focusSeconds = state.config.focusMinutes * 60;
    set({
      status: 'idle',
      phase: 'focus',
      phaseStartedAt: null,
      endsAt: null,
      remainingSeconds: focusSeconds,
      elapsedSeconds: 0,
    });
  },

  skipPhase: () => {
    const state = get();
    if (state.status === 'idle') return;
    const result = finalizePhasePure(state, false);
    let cycleCountAfter = state.cycleCount;
    if (result) {
      cycleCountAfter = result.cycleCount;
      set({
        history: result.history,
        cycleCount: result.cycleCount,
        totalFocusCompleted: result.totalFocusCompleted,
      });
      applyFinalizeSideEffects(state, result);
    }
    const next = computeNextPhase(state.phase, cycleCountAfter, state.config);
    const nextSeconds = Math.max(
      1,
      Math.floor(sessionPhaseMinutes(next.phase, state.config) * 60),
    );
    set({
      status: 'idle',
      phase: next.phase,
      cycleCount: next.cycleCount,
      phaseStartedAt: null,
      endsAt: null,
      remainingSeconds: nextSeconds,
      elapsedSeconds: 0,
    });
    persist({
      config: state.config,
      history: get().history,
      cycleCount: next.cycleCount,
    });
  },

  setConfig: (patch) => {
    const next: PomodoroConfig = { ...get().config, ...patch };
    // Clamp to sane ranges so a fat-fingered 0 doesn't lock the timer.
    next.focusMinutes = Math.max(1, Math.min(180, Math.floor(next.focusMinutes)));
    next.shortBreakMinutes = Math.max(1, Math.min(60, Math.floor(next.shortBreakMinutes)));
    next.longBreakMinutes = Math.max(1, Math.min(120, Math.floor(next.longBreakMinutes)));
    next.longBreakInterval = Math.max(1, Math.min(12, Math.floor(next.longBreakInterval)));
    next.focusCompletionXp = Math.max(0, Math.min(1000, Math.floor(next.focusCompletionXp)));
    set({ config: next });
    persist({ config: next, history: get().history, cycleCount: get().cycleCount });

    // If we're idle and the phase duration changed, refresh the displayed
    // remaining time so the user sees the new value before they hit start.
    if (get().status === 'idle') {
      set({ remainingSeconds: sessionPhaseMinutes(get().phase, next) * 60 });
    }
  },

  resetConfig: () => {
    set({ config: { ...DEFAULT_POMODORO_CONFIG } });
    persist({
      config: { ...DEFAULT_POMODORO_CONFIG },
      history: get().history,
      cycleCount: get().cycleCount,
    });
    if (get().status === 'idle') {
      set({
        remainingSeconds: sessionPhaseMinutes(get().phase, DEFAULT_POMODORO_CONFIG) * 60,
      });
    }
  },

  clearHistory: () => {
    set({ history: [], totalFocusCompleted: 0, cycleCount: 0 });
    persist({ config: get().config, history: [], cycleCount: 0 });
  },

  tick: () => {
    const state = get();
    if (state.status !== 'running' || state.endsAt === null) return;
    const remaining = Math.max(0, Math.ceil((state.endsAt - Date.now()) / 1000));
    if (remaining <= 0) {
      const result = finalizePhasePure(state, true);
      let cycleCountAfter = state.cycleCount;
      if (result) {
        cycleCountAfter = result.cycleCount;
        set({
          history: result.history,
          cycleCount: result.cycleCount,
          totalFocusCompleted: result.totalFocusCompleted,
        });
        applyFinalizeSideEffects(state, result);
      }
      const next = computeNextPhase(state.phase, cycleCountAfter, state.config);
      const nextSeconds = Math.max(
        1,
        Math.floor(sessionPhaseMinutes(next.phase, state.config) * 60),
      );
      const autoStart = state.config.autoStartNext;
      const now = Date.now();
      set({
        phase: next.phase,
        cycleCount: next.cycleCount,
        status: autoStart ? 'running' : 'idle',
        phaseStartedAt: autoStart ? new Date(now).toISOString() : null,
        endsAt: autoStart ? now + nextSeconds * 1000 : null,
        remainingSeconds: nextSeconds,
        elapsedSeconds: 0,
      });
      persist({
        config: state.config,
        history: get().history,
        cycleCount: next.cycleCount,
      });
      return;
    }
    const planned = Math.max(
      1,
      Math.floor(sessionPhaseMinutes(state.phase, state.config) * 60),
    );
    const elapsed = Math.max(0, planned - remaining);
    set({ remainingSeconds: remaining, elapsedSeconds: elapsed });
  },
}));

// Single global ticker. We use Date.now()-based math so a backgrounded tab /
// system sleep doesn't desync the displayed remaining time — when the tab
// wakes up the very next tick recomputes from `endsAt` and (if the phase
// already elapsed) finalises immediately.
if (typeof window !== 'undefined') {
  setInterval(() => {
    useSession.getState().tick();
  }, 1000);
}
