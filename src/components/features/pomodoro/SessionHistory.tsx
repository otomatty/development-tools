/**
 * Session History
 *
 * Recent Pomodoro sessions, newest first. Shows phase, planned vs actual
 * duration, completion state, and any XP awarded.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/192
 */

import { useMemo } from 'react';
import { Icon } from '@/components/icons';
import { useSession } from '@/stores/sessionStore';
import {
  type SessionRecord,
  sessionPhaseEmoji,
  sessionPhaseLabel,
} from '@/types/session';

const phaseRowAccent: Record<string, string> = {
  focus: 'border-l-gm-accent-cyan',
  short_break: 'border-l-gm-success',
  long_break: 'border-l-gm-accent-purple',
};

const formatDuration = (seconds: number): string => {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  if (mins === 0) return `${secs}秒`;
  if (secs === 0) return `${mins}分`;
  return `${mins}分${secs}秒`;
};

const formatTime = (iso: string): string => {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString('ja-JP', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
};

interface RowProps {
  record: SessionRecord;
}

const HistoryRow = ({ record }: RowProps) => {
  const accent = phaseRowAccent[record.phase] ?? 'border-l-gm-accent-cyan';
  return (
    <li
      className={`flex items-center justify-between gap-3 px-4 py-3 rounded-lg bg-slate-900/60 border-l-4 ${accent}`}
    >
      <div className="flex items-center gap-3 min-w-0">
        <span className="text-2xl" aria-hidden="true">
          {sessionPhaseEmoji(record.phase)}
        </span>
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-gaming text-sm text-dt-text">
              {sessionPhaseLabel(record.phase)}
            </span>
            {record.completed ? (
              <span className="text-xs px-1.5 py-0.5 rounded bg-gm-success/15 text-gm-success">
                完了
              </span>
            ) : (
              <span className="text-xs px-1.5 py-0.5 rounded bg-slate-700 text-dt-text-sub">
                中断
              </span>
            )}
          </div>
          <div className="text-xs text-dt-text-sub mt-0.5 truncate">
            {formatTime(record.startedAt)} ・ {formatDuration(record.actualDurationSeconds)} /{' '}
            {formatDuration(record.plannedDurationSeconds)}
          </div>
        </div>
      </div>
      {record.xpAwarded > 0 && (
        <span className="shrink-0 text-sm font-gaming-mono text-gm-accent-cyan">
          +{record.xpAwarded} XP
        </span>
      )}
    </li>
  );
};

export const SessionHistory = () => {
  const history = useSession((s) => s.history);
  const totalFocusCompleted = useSession((s) => s.totalFocusCompleted);
  const clearHistory = useSession((s) => s.clearHistory);

  const todayCompleted = useMemo(() => {
    const startOfToday = new Date();
    startOfToday.setHours(0, 0, 0, 0);
    const startOfTomorrow = new Date(startOfToday);
    startOfTomorrow.setDate(startOfTomorrow.getDate() + 1);
    const startMs = startOfToday.getTime();
    const endMs = startOfTomorrow.getTime();

    return history.filter((r) => {
      if (!(r.phase === 'focus' && r.completed)) return false;
      const endedMs = new Date(r.endedAt).getTime();
      // Bound on tomorrow midnight too — clock skew or restored backups
      // can sneak in records dated in the future, and we don't want them
      // counted in today's tally.
      return endedMs >= startMs && endedMs < endMs;
    }).length;
  }, [history]);

  return (
    <div className="bg-gm-bg-card/60 backdrop-blur-sm rounded-2xl border border-slate-700/50 p-6">
      <div className="flex items-center justify-between mb-4">
        <div>
          <div className="flex items-center gap-2">
            <Icon name="clock" className="w-5 h-5 text-gm-accent-cyan" />
            <h2 className="font-gaming text-lg text-dt-text">セッション履歴</h2>
          </div>
          <p className="text-xs text-dt-text-sub mt-1">
            今日の集中完了:{' '}
            <span className="text-gm-accent-cyan font-gaming-mono">{todayCompleted}</span>
            {' / 通算: '}
            <span className="text-gm-accent-cyan font-gaming-mono">
              {totalFocusCompleted}
            </span>
          </p>
        </div>
        {history.length > 0 && (
          <button
            type="button"
            onClick={() => {
              if (typeof window !== 'undefined') {
                if (window.confirm('セッション履歴をすべて削除しますか？')) {
                  clearHistory();
                }
              } else {
                clearHistory();
              }
            }}
            className="flex items-center gap-1 text-xs text-dt-text-sub hover:text-red-400 transition-colors"
            title="履歴をすべて削除"
          >
            <Icon name="trash" className="w-3.5 h-3.5" />
            履歴を削除
          </button>
        )}
      </div>

      {history.length === 0 ? (
        <div className="text-center py-8 text-dt-text-sub text-sm">
          まだセッションがありません。タイマーを開始して最初のセッションを記録しましょう。
        </div>
      ) : (
        <ul className="space-y-2 max-h-[480px] overflow-y-auto pr-1">
          {history.map((record) => (
            <HistoryRow key={record.id} record={record} />
          ))}
        </ul>
      )}
    </div>
  );
};
