/**
 * Challenge Card Component
 *
 * React implementation of ChallengeCard component.
 * Displays active challenges with progress bars and completion status.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./challenge_card.rs
 */

import React, { useState, useEffect, useCallback } from 'react';
import { useNetworkStatus } from '../../../stores/networkStore';
import { Icon } from '../../icons';
import { challenges as challengeApi } from '../../../lib/tauri/commands';
import type { ChallengeInfo } from '../../../types';
import {
  challengeTypeLabel,
  targetMetricLabel,
  targetMetricIcon,
  remainingTimeLabel,
} from '../../../types/challenge';

// Single challenge item component
const ChallengeItem: React.FC<{ challenge: ChallengeInfo }> = ({ challenge }) => {
  const progress = Math.min(challenge.progressPercent, 100);
  const isCompleted = challenge.isCompleted;
  const isExpired = challenge.isExpired;

  // Determine colors based on status
  const getColors = () => {
    if (isCompleted) {
      return {
        bg: 'bg-gm-success/10',
        border: 'border-gm-success/30',
        progress: 'bg-gradient-to-r from-gm-success to-gm-accent-cyan',
      };
    } else if (isExpired) {
      return {
        bg: 'bg-gm-error/10',
        border: 'border-gm-error/30',
        progress: 'bg-gm-error/50',
      };
    } else if (progress >= 75) {
      return {
        bg: 'bg-gm-accent-gold/10',
        border: 'border-gm-accent-gold/30',
        progress: 'bg-gradient-to-r from-gm-accent-gold to-gm-accent-pink',
      };
    } else {
      return {
        bg: 'bg-gm-bg-secondary/50',
        border: 'border-gm-accent-purple/20',
        progress: 'bg-gradient-to-r from-gm-accent-purple to-gm-accent-cyan',
      };
    }
  };

  const colors = getColors();

  // Challenge type badge color
  const typeBadgeClass =
    challenge.challengeType === 'daily'
      ? 'bg-gm-accent-cyan/20 text-gm-accent-cyan border-gm-accent-cyan/30'
      : 'bg-gm-accent-purple/20 text-gm-accent-purple border-gm-accent-purple/30';

  return (
    <div
      className={`p-4 rounded-xl border ${colors.bg} ${colors.border} transition-all duration-300 hover:scale-[1.02]`}
    >
      {/* Header row */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          {/* Challenge type badge */}
          <span className={`px-2 py-0.5 text-xs font-medium rounded-full border ${typeBadgeClass}`}>
            {challengeTypeLabel(challenge.challengeType)}
          </span>
          {/* Metric icon and name */}
          <span className="text-gm-text-secondary text-sm">
            {targetMetricIcon(challenge.targetMetric)} {targetMetricLabel(challenge.targetMetric)}
          </span>
        </div>
        {/* Status/Time remaining */}
        <span className="text-xs text-gm-text-muted">
          {isCompleted
            ? '✅ 達成!'
            : isExpired
              ? '⏰ 期限切れ'
              : remainingTimeLabel(challenge.remainingTimeHours)}
        </span>
      </div>

      {/* Progress section */}
      <div className="space-y-2">
        {/* Progress text */}
        <div className="flex items-baseline justify-between">
          <span className="text-2xl font-bold text-gm-text-primary">
            {challenge.currentValue}
            <span className="text-sm text-gm-text-secondary font-normal">
              {' '}/ {challenge.targetValue}
            </span>
          </span>
          <span className="text-sm font-medium text-gm-accent-gold">+{challenge.rewardXp} XP</span>
        </div>

        {/* Progress bar */}
        <div className="relative h-3 bg-gm-bg-tertiary rounded-full overflow-hidden">
          <div
            className={`absolute inset-y-0 left-0 ${colors.progress} rounded-full transition-all duration-500`}
            style={{ width: `${progress}%` }}
          >
            {/* Animated shine effect for active challenges */}
            {!isCompleted && !isExpired && (
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />
            )}
          </div>
        </div>

        {/* Progress percentage */}
        <div className="text-right">
          <span className="text-xs text-gm-text-muted">{Math.round(progress)}%</span>
        </div>
      </div>
    </div>
  );
};

// Skeleton loader for challenges
const ChallengeSkeleton: React.FC = () => {
  return (
    <div className="p-4 rounded-xl border border-gm-accent-purple/10 bg-gm-bg-secondary/30 animate-pulse">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div className="h-5 w-16 bg-gm-bg-tertiary rounded-full" />
          <div className="h-4 w-20 bg-gm-bg-tertiary rounded" />
        </div>
        <div className="h-4 w-16 bg-gm-bg-tertiary rounded" />
      </div>
      <div className="space-y-2">
        <div className="flex justify-between">
          <div className="h-8 w-24 bg-gm-bg-tertiary rounded" />
          <div className="h-5 w-16 bg-gm-bg-tertiary rounded" />
        </div>
        <div className="h-3 bg-gm-bg-tertiary rounded-full" />
      </div>
    </div>
  );
};

export const ChallengeCard: React.FC = () => {
  const isOnline = useNetworkStatus((s) => s.isOnline);
  const [challenges, setChallenges] = useState<ChallengeInfo[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchChallenges = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await challengeApi.getActive();
      setChallenges(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchChallenges();
  }, [fetchChallenges]);

  // Reload challenges function - only works when online
  const reloadChallenges = async () => {
    if (!isOnline) return;
    fetchChallenges();
  };

  return (
    <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-gold/20">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-xl font-gaming font-bold text-gm-accent-gold">🎯 Challenges</h3>
        <div className="relative group">
          <button
            className={`p-2 rounded-lg transition-all duration-200 ${
              isOnline
                ? 'bg-gm-bg-secondary/50 hover:bg-gm-bg-secondary text-gm-text-secondary hover:text-gm-text-primary'
                : 'bg-gm-bg-secondary/30 text-gm-text-muted cursor-not-allowed'
            }`}
            onClick={reloadChallenges}
            disabled={!isOnline}
            title={isOnline ? 'チャレンジを更新' : 'オフラインのため更新できません'}
          >
            <Icon name="refresh-cw" className="w-4 h-4" />
          </button>

          {/* Offline tooltip */}
          {!isOnline && (
            <div className="absolute -bottom-10 right-0 px-3 py-1.5 bg-gm-bg-dark/95 text-gm-warning text-xs rounded-lg border border-gm-warning/30 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10">
              ⚠️ オフライン
            </div>
          )}
        </div>
      </div>

      {/* Loading state */}
      {loading && (
        <div className="space-y-3">
          <ChallengeSkeleton />
          <ChallengeSkeleton />
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="p-3 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error text-sm">
          {error}
        </div>
      )}

      {/* Challenges list */}
      {!loading && !error && (
        <>
          {challenges && challenges.length > 0 ? (
            <div className="space-y-3">
              {challenges.map((challenge, i) => (
                <ChallengeItem key={i} challenge={challenge} />
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-gm-text-secondary">
              <div className="text-4xl mb-2">🎮</div>
              <p className="text-sm">アクティブなチャレンジはありません</p>
              <p className="text-xs mt-1 text-gm-text-muted">GitHub同期時に自動生成されます</p>
            </div>
          )}
        </>
      )}
    </div>
  );
};
