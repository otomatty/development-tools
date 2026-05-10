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

const STORAGE_KEY = 'development-tools.pomodoro.v2';
const HISTORY_LIMIT = 200;

interface PersistedState {
  config: PomodoroConfig;
  history: SessionRecord[];
  cycleCount: number;
  /// 通算完了数。`history` は HISTORY_LIMIT で切られるため、再計算ではなく
  /// 永続化された値を信頼することで restart 後にも正しい総計を保つ。
  totalFocusCompleted: number;
  /// 進行中フェーズの状態（リロード後も継続できるように保存）。
  /// `idle` のときは undefined になる（読み込み時は idle として扱う）。
  status?: SessionStatus;
  phase?: SessionPhase;
  phaseStartedAt?: string | null;
  endsAt?: number | null;
  remainingSeconds?: number;
  elapsedSeconds?: number;
  phasePlannedSeconds?: number;
  phaseXpReward?: number;
}

const isSessionStatus = (v: unknown): v is SessionStatus =>
  v === 'idle' || v === 'running' || v === 'paused';
const isSessionPhase = (v: unknown): v is SessionPhase =>
  v === 'focus' || v === 'short_break' || v === 'long_break';

const loadPersisted = (): PersistedState => {
  if (typeof window === 'undefined') {
    return { config: DEFAULT_POMODORO_CONFIG, history: [], cycleCount: 0, totalFocusCompleted: 0 };
  }
  try {
    // Read the current key first; fall back to the v1 key so users who
    // started with the previous release don't lose their config / history.
    const raw =
      window.localStorage.getItem(STORAGE_KEY) ??
      window.localStorage.getItem('development-tools.pomodoro.v1');
    if (!raw) return { config: DEFAULT_POMODORO_CONFIG, history: [], cycleCount: 0, totalFocusCompleted: 0 };
    const parsed = JSON.parse(raw) as Partial<PersistedState>;
    const history = Array.isArray(parsed.history)
      ? parsed.history.slice(0, HISTORY_LIMIT)
      : [];
    return {
      config: { ...DEFAULT_POMODORO_CONFIG, ...(parsed.config ?? {}) },
      history,
      cycleCount: typeof parsed.cycleCount === 'number' ? parsed.cycleCount : 0,
      // `history` is truncated at HISTORY_LIMIT, so once a user crosses
      // that line a recompute would silently rewind the lifetime tally.
      // Trust the persisted counter when present and only fall back to
      // recompute for migrating users who never had it written.
      totalFocusCompleted:
        typeof parsed.totalFocusCompleted === 'number'
          ? Math.max(0, Math.floor(parsed.totalFocusCompleted))
          : computeTotalFocusCompleted(history),
      status: isSessionStatus(parsed.status) ? parsed.status : undefined,
      phase: isSessionPhase(parsed.phase) ? parsed.phase : undefined,
      phaseStartedAt:
        typeof parsed.phaseStartedAt === 'string' ? parsed.phaseStartedAt : null,
      endsAt: typeof parsed.endsAt === 'number' ? parsed.endsAt : null,
      remainingSeconds:
        typeof parsed.remainingSeconds === 'number' ? parsed.remainingSeconds : undefined,
      elapsedSeconds:
        typeof parsed.elapsedSeconds === 'number' ? parsed.elapsedSeconds : undefined,
      phasePlannedSeconds:
        typeof parsed.phasePlannedSeconds === 'number' ? parsed.phasePlannedSeconds : undefined,
      phaseXpReward:
        typeof parsed.phaseXpReward === 'number' ? parsed.phaseXpReward : undefined,
    };
  } catch (err) {
    console.error('[sessionStore] Failed to parse persisted state:', err);
    return { config: DEFAULT_POMODORO_CONFIG, history: [], cycleCount: 0, totalFocusCompleted: 0 };
  }
};

const persistRaw = (state: PersistedState) => {
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
  /// セッション開始時に確定した総時間（秒）。設定をフェーズ実行中に変更しても
  /// 進行中のフェーズ長は変わらない（次のフェーズから新しい設定が適用される）
  /// — UI と finalize の両方がこのスナップショットを参照する。
  phasePlannedSeconds: number;
  /// セッション開始時に確定した完了時 XP。同様にフェーズ実行中の設定変更は
  /// 進行中のセッションには影響しない。
  phaseXpReward: number;
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

/// Read the persistable subset out of the live store and write it. Called
/// after every state-mutating action so a refresh / restart picks up where
/// the user left off (running timers resume from the same `endsAt`, paused
/// timers restore their `remainingSeconds`, etc.).
const persistAll = () => {
  const s = useSession.getState();
  persistRaw({
    config: s.config,
    history: s.history,
    cycleCount: s.cycleCount,
    totalFocusCompleted: s.totalFocusCompleted,
    status: s.status,
    phase: s.phase,
    phaseStartedAt: s.phaseStartedAt,
    endsAt: s.endsAt,
    remainingSeconds: s.remainingSeconds,
    elapsedSeconds: s.elapsedSeconds,
    phasePlannedSeconds: s.phasePlannedSeconds,
    phaseXpReward: s.phaseXpReward,
  });
};

// Restore the active timer state when present. If a `running` session was
// persisted and its `endsAt` is already in the past (e.g. the user closed
// the app mid-session and reopened it 40 minutes later), we keep the
// `running` status — the very next ticker tick (within 1s) will detect
// `remaining <= 0` and finalize the phase, awarding XP and advancing the
// cycle exactly as if the timer had completed in the foreground.
const initialPhase: SessionPhase = persisted.phase ?? 'focus';
const initialStatus: SessionStatus = persisted.status ?? 'idle';
const initialPhaseStartedAt = persisted.phaseStartedAt ?? null;
const initialEndsAt = initialStatus === 'running' ? (persisted.endsAt ?? null) : null;
const initialRemaining = (() => {
  if (initialStatus === 'running' && initialEndsAt !== null) {
    return Math.max(0, Math.ceil((initialEndsAt - Date.now()) / 1000));
  }
  if (initialStatus === 'paused' && typeof persisted.remainingSeconds === 'number') {
    return Math.max(0, persisted.remainingSeconds);
  }
  return sessionPhaseMinutes(initialPhase, persisted.config) * 60;
})();
// Snapshot the planned duration / XP for the active phase. When idle, mirror
// the current config so the dial can render its "ready to start" total. When
// running or paused, prefer the value persisted with the session — that's
// the contract the user signed up for when they hit Start, even if they
// later edit the focus duration in settings.
const initialPhasePlannedSeconds = (() => {
  if (
    initialStatus !== 'idle' &&
    typeof persisted.phasePlannedSeconds === 'number' &&
    persisted.phasePlannedSeconds > 0
  ) {
    return Math.floor(persisted.phasePlannedSeconds);
  }
  return Math.max(1, Math.floor(sessionPhaseMinutes(initialPhase, persisted.config) * 60));
})();
const initialPhaseXpReward = (() => {
  if (
    initialStatus !== 'idle' &&
    typeof persisted.phaseXpReward === 'number' &&
    persisted.phaseXpReward >= 0
  ) {
    return Math.floor(persisted.phaseXpReward);
  }
  return Math.max(0, Math.floor(persisted.config.focusCompletionXp));
})();
const initialElapsed = (() => {
  if (initialStatus === 'idle') return 0;
  if (typeof persisted.elapsedSeconds === 'number') {
    return Math.max(0, persisted.elapsedSeconds);
  }
  return Math.max(0, initialPhasePlannedSeconds - initialRemaining);
})();

interface FinalizeResult {
  history: SessionRecord[];
  cycleCount: number;
  totalFocusCompleted: number;
  record: SessionRecord;
}

/// 進行中フェーズのカウントダウンが 0 まで届いたかを判定する。
///
/// `tick` 由来の自動完了と、`stop` / `skipPhase` を「すでに終わったフェーズに対して」
/// 押された場合の手動完了を、同じ条件で扱うためのヘルパー。
const isPhaseElapsed = (state: SessionStoreState): boolean => {
  if (state.status === 'running' && state.endsAt !== null) {
    return state.endsAt - Date.now() <= 0;
  }
  if (state.status === 'paused') {
    return state.remainingSeconds <= 0;
  }
  return false;
};

/// 純粋関数: 現在のフェーズを履歴に書き込み、付与すべき XP を計算する。
/// 外部副作用（XP 加算 API 呼び出し / イベント発火 / 永続化）は呼び出し側で行う。
///
/// `plannedDurationSeconds` と `xpAwarded` は **セッション開始時にスナップショットした値**
/// (`state.phasePlannedSeconds` / `state.phaseXpReward`) から計算する。
/// セッション中の設定変更で進行中のセッションの履歴値が遡って書き換わらないようにするため。
const finalizePhasePure = (state: SessionStoreState, completed: boolean): FinalizeResult | null => {
  if (state.phaseStartedAt === null) return null;

  const planned = Math.max(1, Math.floor(state.phasePlannedSeconds));
  const remaining = state.endsAt
    ? Math.max(0, Math.ceil((state.endsAt - Date.now()) / 1000))
    : state.remainingSeconds;
  const actual = completed ? planned : Math.max(0, planned - remaining);
  const xpAwarded =
    completed && state.phase === 'focus' ? Math.max(0, Math.floor(state.phaseXpReward)) : 0;

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

/// 副作用付きの後処理: XP 加算 + イベント発火。永続化は呼び出し側で `persistAll()`
/// を実行することで、`set` 後の最新 state 全体を 1 回で書き出す。
const applyFinalizeSideEffects = (
  _state: SessionStoreState,
  result: FinalizeResult,
): void => {
  if (result.record.xpAwarded > 0) {
    const minutes = Math.max(1, Math.round(result.record.plannedDurationSeconds / 60));
    void gamification
      .addXp(
        result.record.xpAwarded,
        'pomodoro_focus',
        `Pomodoro 集中セッション完了 (${minutes}分)`,
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
  status: initialStatus,
  phase: initialPhase,
  phaseStartedAt: initialPhaseStartedAt,
  endsAt: initialEndsAt,
  remainingSeconds: initialRemaining,
  elapsedSeconds: initialElapsed,
  phasePlannedSeconds: initialPhasePlannedSeconds,
  phaseXpReward: initialPhaseXpReward,
  cycleCount: persisted.cycleCount,
  totalFocusCompleted: persisted.totalFocusCompleted,
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
      // Lock in the planned duration + reward at start time so a mid-phase
      // setting tweak doesn't retroactively rewrite this session's history.
      phasePlannedSeconds: seconds,
      phaseXpReward:
        targetPhase === 'focus'
          ? Math.max(0, Math.floor(state.config.focusCompletionXp))
          : 0,
    });
    persistAll();
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
    persistAll();
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
    persistAll();
  },

  stop: () => {
    const state = get();
    if (state.status === 'idle') return;
    // If the phase has already elapsed (e.g. background-throttled tab or
    // restored from a stale `endsAt`), treat the click as completing the
    // session — the planned time *did* finish, the tick just hasn't run
    // yet. Hard-coding `false` here would silently demote a fully-served
    // focus phase to "interrupted" and skip the XP grant.
    const completed = isPhaseElapsed(state);
    const result = finalizePhasePure(state, completed);
    if (result) {
      set({
        history: result.history,
        cycleCount: result.cycleCount,
        totalFocusCompleted: result.totalFocusCompleted,
      });
      applyFinalizeSideEffects(state, result);
    }
    const focusSeconds = Math.max(1, state.config.focusMinutes * 60);
    set({
      status: 'idle',
      phase: 'focus',
      phaseStartedAt: null,
      endsAt: null,
      remainingSeconds: focusSeconds,
      elapsedSeconds: 0,
      // Reset the snapshot to mirror the live config so the dial / labels
      // for the next-up phase reflect the current settings.
      phasePlannedSeconds: focusSeconds,
      phaseXpReward: Math.max(0, Math.floor(state.config.focusCompletionXp)),
    });
    persistAll();
  },

  skipPhase: () => {
    const state = get();
    if (state.status === 'idle') return;
    // Same elapsed-detection as `stop`: skipping a phase whose timer has
    // already hit zero should be recorded as a completion, not an early
    // exit. This matters when the next tick is delayed (suspended tab,
    // OS sleep) — the user shouldn't lose XP for clicking the button.
    const completed = isPhaseElapsed(state);
    const result = finalizePhasePure(state, completed);
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
      phasePlannedSeconds: nextSeconds,
      phaseXpReward:
        next.phase === 'focus'
          ? Math.max(0, Math.floor(state.config.focusCompletionXp))
          : 0,
    });
    persistAll();
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

    // If we're idle and the phase duration changed, refresh the displayed
    // remaining time + planned snapshot so the user sees the new value
    // before they hit start. Running / paused sessions keep their
    // pre-edit snapshot — the new config takes effect at the next phase.
    if (get().status === 'idle') {
      const seconds = Math.max(1, sessionPhaseMinutes(get().phase, next) * 60);
      set({
        remainingSeconds: seconds,
        phasePlannedSeconds: seconds,
        phaseXpReward:
          get().phase === 'focus' ? Math.max(0, Math.floor(next.focusCompletionXp)) : 0,
      });
    }
    persistAll();
  },

  resetConfig: () => {
    set({ config: { ...DEFAULT_POMODORO_CONFIG } });
    if (get().status === 'idle') {
      const seconds = Math.max(
        1,
        sessionPhaseMinutes(get().phase, DEFAULT_POMODORO_CONFIG) * 60,
      );
      set({
        remainingSeconds: seconds,
        phasePlannedSeconds: seconds,
        phaseXpReward:
          get().phase === 'focus'
            ? Math.max(0, Math.floor(DEFAULT_POMODORO_CONFIG.focusCompletionXp))
            : 0,
      });
    }
    persistAll();
  },

  clearHistory: () => {
    // The button is labelled "履歴を削除" — only wipe the history list and
    // its lifetime counter. Leave `cycleCount` alone so a clear performed
    // mid-session doesn't reroute the next phase (e.g. flipping a pending
    // long break back to a short one).
    set({ history: [], totalFocusCompleted: 0 });
    persistAll();
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
      const nextXp =
        next.phase === 'focus' ? Math.max(0, Math.floor(state.config.focusCompletionXp)) : 0;
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
        // The next phase's snapshot reflects the live config — autoStart
        // commits this immediately, idle just previews it for the user.
        phasePlannedSeconds: nextSeconds,
        phaseXpReward: nextXp,
      });
      persistAll();
      return;
    }
    const planned = Math.max(1, Math.floor(state.phasePlannedSeconds));
    const elapsed = Math.max(0, planned - remaining);
    set({ remainingSeconds: remaining, elapsedSeconds: elapsed });
    // Skip persisting on every tick — `endsAt` is fixed while running, so
    // the only field changing is the derived display. We re-persist on
    // pause / stop / phase boundary, which is enough to recover state on
    // reload.
  },
}));

// Single global ticker. We use Date.now()-based math so a backgrounded tab /
// system sleep doesn't desync the displayed remaining time — when the tab
// wakes up the very next tick recomputes from `endsAt` and (if the phase
// already elapsed) finalises immediately.
//
// Stash the timer handle on `globalThis` so Vite HMR re-running this module
// in dev doesn't stack multiple intervals (which would make the timer tick
// faster and faster on every save). Production runs through this once.
declare global {
  // eslint-disable-next-line no-var
  var __pomodoroTickInterval__: ReturnType<typeof setInterval> | undefined;
}

if (typeof window !== 'undefined') {
  if (globalThis.__pomodoroTickInterval__ !== undefined) {
    clearInterval(globalThis.__pomodoroTickInterval__);
  }
  globalThis.__pomodoroTickInterval__ = setInterval(() => {
    useSession.getState().tick();
  }, 1000);
}
