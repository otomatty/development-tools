/**
 * Stats Display Component
 *
 * Solid.js implementation of StatsDisplay component.
 * Shows detailed statistics in a grid layout.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./stats_display.rs
 */

import { Component, Show, createMemo } from 'solid-js';
import { AnimatedEmoji, AnimatedEmojiWithIntensity } from '../../ui/animated-emoji';
import type { GitHubStats, UserStats, StatsDiffResult } from '../../../types';

interface StatsDisplayProps {
  githubStats?: GitHubStats | null;
  userStats?: UserStats | null;
  statsDiff?: StatsDiffResult | null;
}

// Streak milestone thresholds
const STREAK_MILESTONES: Array<[number, string]> = [
  [7, 'On Fire 🔥'],
  [14, 'Two Weeks 💪'],
  [30, 'Month Strong 🌟'],
  [100, 'Century 💯'],
  [365, 'Legendary 👑'],
];

// Get next milestone for a streak
function getNextMilestone(currentStreak: number): [number, string] | null {
  const milestone = STREAK_MILESTONES.find(([days]) => days > currentStreak);
  return milestone ? milestone : null;
}

// Streak section component with prominent display
const StreakSection: Component<{ userStats?: UserStats | null }> = (props) => {
  const currentStreak = () => props.userStats?.currentStreak ?? 0;
  const longestStreak = () => props.userStats?.longestStreak ?? 0;

  const nextMilestoneInfo = createMemo(() => {
    const streak = currentStreak();
    const milestone = getNextMilestone(streak);
    if (!milestone) return null;
    const [days, name] = milestone;
    const daysLeft = days - streak;
    return { daysLeft, name };
  });

  return (
    <div class="p-6 bg-gradient-to-br from-orange-900/30 via-gm-bg-card/80 to-red-900/20 backdrop-blur-sm rounded-2xl border border-orange-500/30 shadow-lg shadow-orange-500/10">
      <div class="flex items-center justify-between">
        {/* Left side: Current streak with animated flame */}
        <div class="flex items-center gap-4">
          <AnimatedEmojiWithIntensity
            emoji="Fire"
            size="text-5xl"
            hoverOnly={true}
            value={currentStreak}
            thresholds={[1, 7, 30]}
          />
          <div>
            <div class="text-4xl font-gaming-mono font-bold text-orange-400">
              {currentStreak()}
              <span class="text-lg text-orange-300/70"> days</span>
            </div>
            <div class="text-sm text-dt-text-sub">Current Streak</div>
          </div>
        </div>

        {/* Right side: Best streak and next milestone */}
        <div class="text-right space-y-2">
          <div class="flex items-center gap-2 justify-end">
            <AnimatedEmoji emoji="Trophy" size="text-lg" class="text-badge-gold" hoverOnly={true} />
            <span class="text-lg font-gaming-mono text-badge-gold">{longestStreak()}</span>
            <span class="text-xs text-dt-text-sub">Best</span>
          </div>

          {/* Next milestone */}
          <Show when={nextMilestoneInfo()}>
            {(info) => (
              <div class="text-xs text-dt-text-sub">
                <span class="text-gm-accent-cyan">{info().daysLeft}</span> days to{' '}
                <span class="text-orange-300">{info().name}</span>
              </div>
            )}
          </Show>
        </div>
      </div>

      {/* Progress to next milestone */}
      <Show when={nextMilestoneInfo()}>
        {(info) => {
          const streak = currentStreak();
          const target = streak + info().daysLeft;
          const progress = target > 0 ? (streak / target) * 100 : 0;

          return (
            <div class="mt-4">
              <div class="h-2 bg-slate-700/50 rounded-full overflow-hidden">
                <div
                  class="h-full bg-gradient-to-r from-orange-500 to-red-500 rounded-full transition-all duration-500"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          );
        }}
      </Show>
    </div>
  );
};

// Individual stat card with optional day-over-day diff
interface StatCardProps {
  icon: string;
  label: string;
  value: string;
  color: 'cyan' | 'purple' | 'pink' | 'green' | 'orange' | 'gold';
  diff?: number | null;
}

const StatCard: Component<StatCardProps> = (props) => {
  const colorClass = () => {
    switch (props.color) {
      case 'cyan':
        return 'text-gm-accent-cyan';
      case 'purple':
        return 'text-gm-accent-purple';
      case 'pink':
        return 'text-gm-accent-pink';
      case 'green':
        return 'text-gm-success';
      case 'orange':
        return 'text-gm-warning';
      case 'gold':
        return 'text-badge-gold';
      default:
        return 'text-dt-text';
    }
  };

  return (
    <div class="p-4 bg-gm-bg-secondary/50 rounded-xl border border-slate-700/30 hover:border-gm-accent-cyan/30 transition-all duration-200">
      <div class="flex items-center gap-3">
        <span class="text-2xl">{props.icon}</span>
        <div class="flex-1">
          <div class="flex items-center gap-2">
            <div class={`text-xl font-gaming-mono font-bold ${colorClass()}`}>{props.value}</div>
            <Show when={props.diff !== null && props.diff !== undefined}>
              {(diff) => {
                const d = diff();
                if (d > 0) {
                  return (
                    <span class="text-xs font-bold text-gm-success flex items-center gap-0.5">
                      ↑{d}
                    </span>
                  );
                } else if (d < 0) {
                  return (
                    <span class="text-xs font-bold text-gm-error flex items-center gap-0.5">
                      ↓{Math.abs(d)}
                    </span>
                  );
                } else {
                  return (
                    <span class="text-xs text-slate-500 flex items-center gap-0.5">→0</span>
                  );
                }
              }}
            </Show>
          </div>
          <div class="text-xs text-dt-text-sub">{props.label}</div>
        </div>
      </div>
    </div>
  );
};

export const StatsDisplay: Component<StatsDisplayProps> = (props) => {
  const getDiff = (field: keyof StatsDiffResult): number | null => {
    return props.statsDiff?.[field] ?? null;
  };

  return (
    <div class="space-y-6">
      {/* Streak Section - Prominent display */}
      <StreakSection userStats={props.userStats} />

      {/* Stats Grid */}
      <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-purple/20">
        <h3 class="text-xl font-gaming font-bold text-gm-accent-purple mb-4">📊 Statistics</h3>

        <div class="grid grid-cols-2 gap-4">
          {/* Commits */}
          <StatCard
            icon="📝"
            label="Total Commits"
            value={props.githubStats?.totalCommits.toString() ?? '-'}
            color="cyan"
            diff={getDiff('commitsDiff')}
          />

          {/* Pull Requests */}
          <StatCard
            icon="🔀"
            label="Pull Requests"
            value={props.githubStats?.totalPrs.toString() ?? '-'}
            color="purple"
            diff={getDiff('prsDiff')}
          />

          {/* Reviews */}
          <StatCard
            icon="👁️"
            label="Code Reviews"
            value={props.githubStats?.totalReviews.toString() ?? '-'}
            color="pink"
            diff={getDiff('reviewsDiff')}
          />

          {/* Issues */}
          <StatCard
            icon="🎯"
            label="Issues"
            value={props.githubStats?.totalIssues.toString() ?? '-'}
            color="green"
            diff={getDiff('issuesDiff')}
          />

          {/* Stars */}
          <StatCard
            icon="⭐"
            label="Stars Received"
            value={props.githubStats?.totalStarsReceived.toString() ?? '-'}
            color="gold"
            diff={getDiff('starsDiff')}
          />

          {/* Languages (no diff for this) */}
          <StatCard
            icon="🌍"
            label="Languages"
            value={props.githubStats?.languagesCount.toString() ?? '-'}
            color="cyan"
          />
        </div>
      </div>
    </div>
  );
};

