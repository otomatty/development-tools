/**
 * Dashboard Content Component
 *
 * Solid.js implementation of DashboardContent component.
 * The main dashboard content for logged-in users on the home page.
 * Displays profile card, stats, challenges, badges, and contribution graph.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./dashboard_content.rs
 */

import { Component, Suspense } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { ProfileCard } from './ProfileCard';
import { StatsDisplay } from './StatsDisplay';
import { ChallengeCard } from './ChallengeCard';
import { BadgeGrid } from './BadgeGrid';
import { ContributionGraph } from './ContributionGraph';
import type { LevelInfo, UserStats, GitHubStats, StatsDiffResult } from '../../../types';

interface DashboardContentProps {
  levelInfo?: LevelInfo | null;
  userStats?: UserStats | null;
  githubStats?: GitHubStats | null;
  statsDiff?: StatsDiffResult | null;
}

// Skeleton components
const ProfileCardSkeleton: Component = () => (
  <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 animate-pulse">
    <div class="flex items-center gap-6">
      <div class="w-20 h-20 bg-slate-700 rounded-xl"></div>
      <div class="space-y-2 flex-1">
        <div class="h-6 w-32 bg-slate-700 rounded"></div>
        <div class="h-4 w-48 bg-slate-700 rounded"></div>
      </div>
    </div>
  </div>
);

const StatsDisplaySkeleton: Component = () => (
  <div class="space-y-6 animate-pulse">
    <div class="p-6 bg-gm-bg-card/80 rounded-2xl">
      <div class="h-6 w-32 bg-slate-700 rounded mb-4"></div>
      <div class="grid grid-cols-2 gap-4">
        {Array.from({ length: 6 }).map(() => (
          <div class="h-20 bg-slate-700 rounded-xl"></div>
        ))}
      </div>
    </div>
  </div>
);

const ChallengeCardSkeleton: Component = () => (
  <div class="p-6 bg-gm-bg-card/80 rounded-2xl animate-pulse">
    <div class="h-6 w-32 bg-slate-700 rounded mb-4"></div>
    <div class="space-y-3">
      {Array.from({ length: 2 }).map(() => (
        <div class="h-24 bg-slate-700 rounded-xl"></div>
      ))}
    </div>
  </div>
);

const BadgeGridSkeleton: Component = () => (
  <div class="p-6 bg-gm-bg-card/80 rounded-2xl animate-pulse">
    <div class="h-6 w-32 bg-slate-700 rounded mb-4"></div>
    <div class="grid grid-cols-4 gap-3">
      {Array.from({ length: 8 }).map(() => (
        <div class="h-16 bg-slate-700 rounded-xl"></div>
      ))}
    </div>
  </div>
);

const ContributionGraphSkeleton: Component = () => (
  <div class="p-6 bg-gm-bg-card/80 rounded-2xl animate-pulse">
    <div class="h-6 w-32 bg-slate-700 rounded mb-4"></div>
    <div class="flex gap-1">
      {Array.from({ length: 53 }).map(() => (
        <div class="flex flex-col gap-1">
          {Array.from({ length: 7 }).map(() => (
            <div class="w-3 h-3 bg-slate-700 rounded-sm"></div>
          ))}
        </div>
      ))}
    </div>
  </div>
);

export const DashboardContent: Component<DashboardContentProps> = (props) => {
  const navigate = useNavigate();

  return (
    <div class="space-y-6">
      {/* Profile Card */}
      <Suspense fallback={<ProfileCardSkeleton />}>
        <ProfileCard levelInfo={props.levelInfo} userStats={props.userStats} />
      </Suspense>

      {/* Quick Actions */}
      <div class="flex justify-end">
        <button
          class="flex items-center gap-2 px-4 py-2 text-sm text-dt-text-sub hover:text-gm-accent-cyan transition-colors"
          onClick={() => navigate('/xp-history')}
        >
          <span>📜</span>
          <span>XP獲得履歴を見る</span>
          <span class="text-xs">→</span>
        </button>
      </div>

      {/* Stats and Challenges Grid */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Stats Display */}
        <Suspense fallback={<StatsDisplaySkeleton />}>
          <StatsDisplay
            githubStats={props.githubStats}
            userStats={props.userStats}
            statsDiff={props.statsDiff}
          />
        </Suspense>

        {/* Challenges */}
        <Suspense fallback={<ChallengeCardSkeleton />}>
          <ChallengeCard />
        </Suspense>
      </div>

      {/* Badges */}
      <Suspense fallback={<BadgeGridSkeleton />}>
        <BadgeGrid />
      </Suspense>

      {/* Contribution Graph */}
      <Suspense fallback={<ContributionGraphSkeleton />}>
        <ContributionGraph githubStats={props.githubStats} />
      </Suspense>
    </div>
  );
};

