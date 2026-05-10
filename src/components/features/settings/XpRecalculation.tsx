/**
 * XP Recalculation Component (Issue #194)
 *
 * Surfaces the past-year XP recalculation feature in the Settings page.
 *
 * UX flow:
 *   1. Button → opens a confirmation modal explaining the risks /
 *      limitations (1-year cap, uncovered categories, audit-only writes).
 *   2. Confirm → invokes `recalculateXpHistory` on the backend.
 *   3. Backend response → renders a "before / after" comparison modal so
 *      the DoD ("計算前後の値を比較表示できる") is satisfied without
 *      navigating away from Settings.
 *
 * The recalculation never mutates `user_stats.total_xp`; it only writes a
 * `source = 'recalculated'` row to `xp_history` for audit purposes.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/194
 *   - Backend: src-tauri/src/commands/gamification.rs `recalculate_xp_history`
 */

import React, { useState } from 'react';
import { gamification } from '../../../lib/tauri/commands';
import { Modal, ModalHeader, ModalBody, ModalFooter } from '../../ui/dialog';
import { Button } from '../../ui/button';
import type { RecalculationResult } from '../../../types';

const RATE_LIMIT_HINT = 'XP 再計算は 1 時間に 1 回までです';

const UNCOVERED_LABELS: Record<string, string> = {
  prs_merged: 'PR マージ数',
  issues_closed: 'クローズした Issue 数',
  stars: '獲得スター数',
};

const formatJstDateTime = (iso: string): string => {
  try {
    return new Date(iso).toLocaleString('ja-JP', { timeZone: 'Asia/Tokyo' });
  } catch {
    return iso;
  }
};

const diffSign = (n: number): string => (n > 0 ? '+' : n < 0 ? '' : '±');

interface ConfirmationModalProps {
  visible: boolean;
  running: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

const ConfirmationModal: React.FC<ConfirmationModalProps> = ({
  visible,
  running,
  onConfirm,
  onCancel,
}) => (
  <Modal
    visible={visible}
    onClose={onCancel}
    size="md"
    closeOnOverlay={!running}
    closeOnEscape={!running}
    borderClass="border-2 border-gm-accent-cyan/50"
  >
    <ModalHeader onClose={running ? undefined : onCancel}>
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-full bg-gm-accent-cyan/20 flex items-center justify-center border border-gm-accent-cyan/30">
          <span className="text-2xl">🔄</span>
        </div>
        <h3 className="text-xl font-gaming font-bold text-white">
          XP 再計算の実行
        </h3>
      </div>
    </ModalHeader>
    <ModalBody>
      <div className="space-y-4 text-sm">
        <p className="text-dt-text-sub">
          GitHub の <span className="font-mono">contributionCalendar</span>{' '}
          から過去 1 年分のアクティビティを取得し、現在の XP ルールで再計算します。
        </p>

        <div className="p-3 bg-amber-900/20 border border-amber-500/30 rounded-lg space-y-2">
          <p className="text-amber-200 font-bold flex items-center gap-2">
            <span>⚠️</span>
            実行前にご確認ください
          </p>
          <ul className="list-disc list-inside text-amber-100/80 space-y-1">
            <li>
              再計算結果は <span className="font-bold">監査用エントリ</span>{' '}
              として `xp_history` に追加されます。
            </li>
            <li>
              既存の合計 XP・レベルは <span className="font-bold">変更されません</span>
              （表示用の比較値として使えます）。
            </li>
            <li>
              `contributionCalendar` の制約上、再計算できるのは{' '}
              <span className="font-bold">過去 1 年分</span> のみです。
            </li>
            <li>
              PR マージ数 / クローズ Issue 数 / 獲得スター数は{' '}
              <span className="font-mono">contributionsCollection</span>{' '}
              から取得できないため <span className="font-bold">0 として計算</span>{' '}
              されます。
            </li>
            <li>{RATE_LIMIT_HINT}。</li>
          </ul>
        </div>
      </div>
    </ModalBody>
    <ModalFooter>
      <Button variant="secondary" onClick={onCancel} disabled={running}>
        キャンセル
      </Button>
      <Button variant="primary" onClick={onConfirm} disabled={running} isLoading={running}>
        {running ? '再計算中...' : '再計算を実行'}
      </Button>
    </ModalFooter>
  </Modal>
);

interface ResultModalProps {
  result: RecalculationResult | null;
  onClose: () => void;
}

const ResultModal: React.FC<ResultModalProps> = ({ result, onClose }) => {
  if (!result) return null;

  const live = result.previousLiveTotalXpInWindow;
  const recalc = result.recalculatedTotalXp;
  const diff = result.xpDiff;
  const breakdown = result.recalculatedBreakdown;

  return (
    <Modal
      visible
      onClose={onClose}
      size="lg"
      closeOnOverlay
      closeOnEscape
      borderClass="border-2 border-gm-accent-cyan/50"
    >
      <ModalHeader onClose={onClose}>
        <div className="flex items-center gap-3">
          <div className="w-12 h-12 rounded-full bg-gm-accent-cyan/20 flex items-center justify-center border border-gm-accent-cyan/30">
            <span className="text-2xl">📊</span>
          </div>
          <h3 className="text-xl font-gaming font-bold text-white">
            XP 再計算結果
          </h3>
        </div>
      </ModalHeader>
      <ModalBody>
        <div className="space-y-4 text-sm">
          <div className="text-dt-text-sub">
            集計期間: {formatJstDateTime(result.since)} 〜{' '}
            {formatJstDateTime(result.until)}（{result.windowDays} 日間）
          </div>

          {/* Before / After 比較 */}
          <div className="grid grid-cols-3 gap-3">
            <div className="p-3 bg-gm-bg-primary/50 rounded-lg border border-slate-700">
              <div className="text-dt-text-sub text-xs mb-1">既存の Live XP</div>
              <div className="text-white font-gaming text-lg">{live} XP</div>
            </div>
            <div className="p-3 bg-gm-bg-primary/50 rounded-lg border border-gm-accent-cyan/40">
              <div className="text-dt-text-sub text-xs mb-1">再計算結果</div>
              <div className="text-gm-accent-cyan font-gaming text-lg">{recalc} XP</div>
            </div>
            <div
              className={`p-3 bg-gm-bg-primary/50 rounded-lg border ${
                diff > 0
                  ? 'border-green-500/40'
                  : diff < 0
                    ? 'border-red-500/40'
                    : 'border-slate-700'
              }`}
            >
              <div className="text-dt-text-sub text-xs mb-1">差分</div>
              <div
                className={`font-gaming text-lg ${
                  diff > 0 ? 'text-green-400' : diff < 0 ? 'text-red-400' : 'text-white'
                }`}
              >
                {diffSign(diff)}
                {diff} XP
              </div>
            </div>
          </div>

          {/* 内訳 */}
          <div className="p-3 bg-gm-bg-card/50 rounded-lg border border-slate-700">
            <h4 className="text-white font-gaming mb-2">XP 内訳（再計算）</h4>
            <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
              <div className="flex justify-between">
                <span className="text-dt-text-sub">Commits</span>
                <span className="text-white font-mono">
                  {breakdown.commitsXp} XP ({result.contributions.commits} 件)
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-dt-text-sub">PRs 作成</span>
                <span className="text-white font-mono">
                  {breakdown.prsCreatedXp} XP ({result.contributions.pullRequests} 件)
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-dt-text-sub">Issues 作成</span>
                <span className="text-white font-mono">
                  {breakdown.issuesCreatedXp} XP ({result.contributions.issues} 件)
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-dt-text-sub">Reviews</span>
                <span className="text-white font-mono">
                  {breakdown.reviewsXp} XP ({result.contributions.reviews} 件)
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-dt-text-sub">Streak Bonus</span>
                <span className="text-white font-mono">
                  {breakdown.streakBonusXp} XP (streak={result.contributions.currentStreak})
                </span>
              </div>
            </div>
          </div>

          {/* 未集計カテゴリの注意 */}
          {result.uncoveredCategories.length > 0 && (
            <div className="p-3 bg-amber-900/20 border border-amber-500/30 rounded-lg">
              <p className="text-amber-200 text-xs">
                <span className="font-bold">未集計のカテゴリ:</span>{' '}
                {result.uncoveredCategories
                  .map((c) => UNCOVERED_LABELS[c] ?? c)
                  .join('、')}{' '}
                は `contributionsCollection` から取得できないため 0 として計算しています。
              </p>
            </div>
          )}

          <p className="text-xs text-dt-text-sub">
            監査エントリ ID: #{result.recalculationHistoryId}{' '}
            （`xp_history` に `source = recalculated` で記録されています）
          </p>
        </div>
      </ModalBody>
      <ModalFooter>
        <Button variant="primary" onClick={onClose}>
          閉じる
        </Button>
      </ModalFooter>
    </Modal>
  );
};

export const XpRecalculation: React.FC = () => {
  const [showConfirm, setShowConfirm] = useState(false);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<RecalculationResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const onConfirm = async () => {
    setRunning(true);
    setError(null);
    try {
      const res = await gamification.recalculateXpHistory();
      setResult(res);
      setShowConfirm(false);
    } catch (e) {
      setError(`${e}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="space-y-3">
      <ConfirmationModal
        visible={showConfirm}
        running={running}
        onConfirm={onConfirm}
        onCancel={() => {
          if (!running) {
            setShowConfirm(false);
            setError(null);
          }
        }}
      />
      <ResultModal result={result} onClose={() => setResult(null)} />

      <h3 className="text-lg font-gaming font-bold text-white flex items-center gap-2">
        🔄 XP 再計算
      </h3>
      <div className="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
        <p className="text-dt-text-sub mb-4 text-sm">
          GitHub の `contributionCalendar` から過去 1 年分のアクティビティを再取得し、
          現在の XP ルールで再計算します。結果は監査用エントリとして履歴に追加され、
          現在の合計 XP・レベルは変更されません。
        </p>
        <p className="text-xs text-dt-text-sub mb-4">
          {RATE_LIMIT_HINT}。連続実行を試みるとエラーが返ります。
        </p>
        <Button
          variant="primary"
          onClick={() => {
            setError(null);
            setShowConfirm(true);
          }}
          disabled={running}
          fullWidth
          isLoading={running}
        >
          {running ? '再計算中...' : '過去 1 年分の XP を再計算'}
        </Button>
        {error && (
          <div className="mt-3 p-3 bg-red-900/30 border border-red-500/50 rounded-lg text-red-200 text-sm">
            {error}
          </div>
        )}
      </div>
    </div>
  );
};
