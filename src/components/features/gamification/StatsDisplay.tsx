/**
 * Stats Display Component
 *
 * React implementation of StatsDisplay component.
 * Shows detailed statistics in a grid layout.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./stats_display.rs
 */

import React, { useMemo } from 'react';
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
const StreakSection: React.FC<{ userStats?: UserStats | null }> = ({ userStats }) => {
  const currentStreak = userStats?.currentStreak ?? 0;
  const longestStreak = userStats?.longestStreak ?? 0;

  const nextMilestoneInfo = useMemo(() => {
    const milestone = getNextMilestone(currentStreak);
    if (!milestone) return null;
    const [days, name] = milestone;
    const daysLeft = days - currentStreak;
    return { daysLeft, name };
  }, [currentStreak]);

  return (
    <div className="p-6 bg-gradient-to-br from-orange-900/30 via-gm-bg-card/80 to-red-900/20 backdrop-blur-sm rounded-2xl border border-orange-500/30 shadow-lg shadow-orange-500/10">
      <div className="flex items-center justify-between">
        {/* Left side: Current streak with animated flame */}
        <div className="flex items-center gap-4">
          <AnimatedEmojiWithIntensity
            emoji="Fire"
            size="text-5xl"
            hoverOnly={true}
            value={currentStreak}
            thresholds={[1, 7, 30]}
          />
          <div>
            <div className="text-4xl font-gaming-mono font-bold text-orange-400">
              {currentStreak}
              <span className="text-lg text-orange-300/70"> days</span>
            </div>
            <div className="text-sm text-dt-text-sub">Current Streak</div>
          </div>
        </div>

        {/* Right side: Best streak and next milestone */}
        <div className="text-right space-y-2">
          <div className="flex items-center gap-2 justify-end">
            <AnimatedEmoji emoji="Trophy" size="text-lg" className="text-badge-gold" hoverOnly={true} />
            <span className="text-lg font-gaming-mono text-badge-gold">{longestStreak}</span>
            <span className="text-xs text-dt-text-sub">Best</span>
          </div>

          {/* Next milestone */}
          {nextMilestoneInfo && (
            <div className="text-xs text-dt-text-sub">
              <span className="text-gm-accent-cyan">{nextMilestoneInfo.daysLeft}</span> days to{' '}
              <span className="text-orange-300">{nextMilestoneInfo.name}</span>
            </div>
          )}
        </div>
      </div>

      {/* Progress to next milestone */}
      {nextMilestoneInfo && (() => {
        const target = currentStreak + nextMilestoneInfo.daysLeft;
        const progress = target > 0 ? (currentStreak / target) * 100 : 0;

        return (
          <div className="mt-4">
            <div className="h-2 bg-slate-700/50 rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-orange-500 to-red-500 rounded-full transition-all duration-500"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        );
      })()}
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

const StatCard: React.FC<StatCardProps> = ({ icon, label, value, color, diff }) => {
  const colorClass = (() => {
    switch (color) {
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
  })();

  const renderDiff = () => {
    if (diff === null || diff === undefined) return null;
    if (diff > 0) {
      return (
        <span className="text-xs font-bold text-gm-success flex items-center gap-0.5">
          ↑{diff}
        </span>
      );
    } else if (diff < 0) {
      return (
        <span className="text-xs font-bold text-gm-error flex items-center gap-0.5">
          ↓{Math.abs(diff)}
        </span>
      );
    } else {
      return (
        <span className="text-xs text-slate-500 flex items-center gap-0.5">→0</span>
      );
    }
  };

  return (
    <div className="p-4 bg-gm-bg-secondary/50 rounded-xl border border-slate-700/30 hover:border-gm-accent-cyan/30 transition-all duration-200">
      <div className="flex items-center gap-3">
        <span className="text-2xl">{icon}</span>
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <div className={`text-xl font-gaming-mono font-bold ${colorClass}`}>{value}</div>
            {renderDiff()}
          </div>
          <div className="text-xs text-dt-text-sub">{label}</div>
        </div>
      </div>
    </div>
  );
};

export const StatsDisplay: React.FC<StatsDisplayProps> = ({ githubStats, userStats, statsDiff }) => {
  const getDiff = (field: keyof StatsDiffResult): number | null => {
    const val = statsDiff?.[field];
    return typeof val === 'number' ? val : null;
  };

  return (
    <div className="space-y-6">
      {/* Streak Section - Prominent display */}
      <StreakSection userStats={userStats} />

      {/* Stats Grid */}
      <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-purple/20">
        <h3 className="text-xl font-gaming font-bold text-gm-accent-purple mb-4">📊 Statistics</h3>

        <div className="grid grid-cols-2 gap-4">
          {/* Commits */}
          <StatCard
            icon="📝"
            label="Total Commits"
            value={githubStats?.totalCommits.toString() ?? '-'}
            color="cyan"
            diff={getDiff('commitsDiff')}
          />

          {/* Pull Requests */}
          <StatCard
            icon="🔀"
            label="Pull Requests"
            value={githubStats?.totalPrs.toString() ?? '-'}
            color="purple"
            diff={getDiff('prsDiff')}
          />

          {/* Reviews */}
          <StatCard
            icon="👁️"
            label="Code Reviews"
            value={githubStats?.totalReviews.toString() ?? '-'}
            color="pink"
            diff={getDiff('reviewsDiff')}
          />

          {/* Issues */}
          <StatCard
            icon="🎯"
            label="Issues"
            value={githubStats?.totalIssues.toString() ?? '-'}
            color="green"
            diff={getDiff('issuesDiff')}
          />

          {/* Stars */}
          <StatCard
            icon="⭐"
            label="Stars Received"
            value={githubStats?.totalStarsReceived.toString() ?? '-'}
            color="gold"
            diff={getDiff('starsDiff')}
          />

          {/* Languages (no diff for this) */}
          <StatCard
            icon="🌍"
            label="Languages"
            value={githubStats?.languagesCount.toString() ?? '-'}
            color="cyan"
          />
        </div>
      </div>
    </div>
  );
};
