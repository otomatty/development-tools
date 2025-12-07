/**
 * Challenge Card Component
 *
 * Solid.js implementation of ChallengeCard component.
 * Displays active challenges with progress bars and completion status.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./challenge_card.rs
 */

import { Component, createSignal, onMount, Show } from 'solid-js';
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
const ChallengeItem: Component<{ challenge: ChallengeInfo }> = (props) => {
  const progress = () => Math.min(props.challenge.progressPercent, 100);
  const isCompleted = () => props.challenge.isCompleted;
  const isExpired = () => props.challenge.isExpired;

  // Determine colors based on status
  const getColors = () => {
    if (isCompleted()) {
      return {
        bg: 'bg-gm-success/10',
        border: 'border-gm-success/30',
        progress: 'bg-gradient-to-r from-gm-success to-gm-accent-cyan',
      };
    } else if (isExpired()) {
      return {
        bg: 'bg-gm-error/10',
        border: 'border-gm-error/30',
        progress: 'bg-gm-error/50',
      };
    } else if (progress() >= 75) {
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

  const colors = () => getColors();

  // Challenge type badge color
  const typeBadgeClass = () =>
    props.challenge.challengeType === 'daily'
      ? 'bg-gm-accent-cyan/20 text-gm-accent-cyan border-gm-accent-cyan/30'
      : 'bg-gm-accent-purple/20 text-gm-accent-purple border-gm-accent-purple/30';

  return (
    <div
      class={`p-4 rounded-xl border ${colors().bg} ${colors().border} transition-all duration-300 hover:scale-[1.02]`}
    >
      {/* Header row */}
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          {/* Challenge type badge */}
          <span class={`px-2 py-0.5 text-xs font-medium rounded-full border ${typeBadgeClass()}`}>
            {challengeTypeLabel(props.challenge.challengeType)}
          </span>
          {/* Metric icon and name */}
          <span class="text-gm-text-secondary text-sm">
            {targetMetricIcon(props.challenge.targetMetric)} {targetMetricLabel(props.challenge.targetMetric)}
          </span>
        </div>
        {/* Status/Time remaining */}
        <span class="text-xs text-gm-text-muted">
          {isCompleted()
            ? '✅ 達成!'
            : isExpired()
              ? '⏰ 期限切れ'
              : remainingTimeLabel(props.challenge.remainingTimeHours)}
        </span>
      </div>

      {/* Progress section */}
      <div class="space-y-2">
        {/* Progress text */}
        <div class="flex items-baseline justify-between">
          <span class="text-2xl font-bold text-gm-text-primary">
            {props.challenge.currentValue}
            <span class="text-sm text-gm-text-secondary font-normal">
              {' '}/ {props.challenge.targetValue}
            </span>
          </span>
          <span class="text-sm font-medium text-gm-accent-gold">+{props.challenge.rewardXp} XP</span>
        </div>

        {/* Progress bar */}
        <div class="relative h-3 bg-gm-bg-tertiary rounded-full overflow-hidden">
          <div
            class={`absolute inset-y-0 left-0 ${colors().progress} rounded-full transition-all duration-500`}
            style={{ width: `${progress()}%` }}
          >
            {/* Animated shine effect for active challenges */}
            <Show when={!isCompleted() && !isExpired()}>
              <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />
            </Show>
          </div>
        </div>

        {/* Progress percentage */}
        <div class="text-right">
          <span class="text-xs text-gm-text-muted">{Math.round(progress())}%</span>
        </div>
      </div>
    </div>
  );
};

// Skeleton loader for challenges
const ChallengeSkeleton: Component = () => {
  return (
    <div class="p-4 rounded-xl border border-gm-accent-purple/10 bg-gm-bg-secondary/30 animate-pulse">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          <div class="h-5 w-16 bg-gm-bg-tertiary rounded-full" />
          <div class="h-4 w-20 bg-gm-bg-tertiary rounded" />
        </div>
        <div class="h-4 w-16 bg-gm-bg-tertiary rounded" />
      </div>
      <div class="space-y-2">
        <div class="flex justify-between">
          <div class="h-8 w-24 bg-gm-bg-tertiary rounded" />
          <div class="h-5 w-16 bg-gm-bg-tertiary rounded" />
        </div>
        <div class="h-3 bg-gm-bg-tertiary rounded-full" />
      </div>
    </div>
  );
};

export const ChallengeCard: Component = () => {
  const network = useNetworkStatus();
  const [challenges, setChallenges] = createSignal<ChallengeInfo[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  // Load challenges on mount
  onMount(async () => {
    setLoading(true);
    try {
      const c = await challengeApi.getActive();
      setChallenges(c);
      setError(null);
    } catch (e) {
      console.error('Failed to load challenges:', e);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  });

  // Reload challenges function - only works when online
  const reloadChallenges = async () => {
    if (!network.isOnline()) {
      return;
    }
    try {
      const c = await challengeApi.getActive();
      setChallenges(c);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-gold/20">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-xl font-gaming font-bold text-gm-accent-gold">🎯 Challenges</h3>
        <div class="relative group">
          <button
            class={`p-2 rounded-lg transition-all duration-200 ${
              network.isOnline()
                ? 'bg-gm-bg-secondary/50 hover:bg-gm-bg-secondary text-gm-text-secondary hover:text-gm-text-primary'
                : 'bg-gm-bg-secondary/30 text-gm-text-muted cursor-not-allowed'
            }`}
            onClick={reloadChallenges}
            disabled={!network.isOnline()}
            title={network.isOnline() ? 'チャレンジを更新' : 'オフラインのため更新できません'}
          >
            <Icon name="refresh-cw" class="w-4 h-4" />
          </button>

          {/* Offline tooltip */}
          <Show when={!network.isOnline()}>
            <div class="absolute -bottom-10 right-0 px-3 py-1.5 bg-gm-bg-dark/95 text-gm-warning text-xs rounded-lg border border-gm-warning/30 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10">
              ⚠️ オフライン
            </div>
          </Show>
        </div>
      </div>

      {/* Loading state */}
      <Show when={loading()}>
        <div class="space-y-3">
          <ChallengeSkeleton />
          <ChallengeSkeleton />
        </div>
      </Show>

      {/* Error state */}
      <Show when={error()}>
        <div class="p-3 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error text-sm">
          {error()}
        </div>
      </Show>

      {/* Challenges list */}
      <Show when={!loading() && !error()}>
        <Show
          when={challenges().length > 0}
          fallback={
            <div class="text-center py-8 text-gm-text-secondary">
              <div class="text-4xl mb-2">🎮</div>
              <p class="text-sm">アクティブなチャレンジはありません</p>
              <p class="text-xs mt-1 text-gm-text-muted">GitHub同期時に自動生成されます</p>
            </div>
          }
        >
          <div class="space-y-3">
            {challenges().map((challenge) => (
              <ChallengeItem challenge={challenge} />
            ))}
          </div>
        </Show>
      </Show>
    </div>
  );
};

