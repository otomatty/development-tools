/**
 * Dashboard Content Component
 *
 * React implementation of DashboardContent component.
 * The main dashboard content for logged-in users on the home page.
 * Displays profile card, stats, challenges, badges, and contribution graph.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./dashboard_content.rs
 */

import React, { Suspense } from 'react';
import { useNavigate } from 'react-router-dom';
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
const ProfileCardSkeleton: React.FC = () => (
  <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 animate-pulse">
    <div className="flex items-center gap-6">
      <div className="w-20 h-20 bg-slate-700 rounded-xl"></div>
      <div className="space-y-2 flex-1">
        <div className="h-6 w-32 bg-slate-700 rounded"></div>
        <div className="h-4 w-48 bg-slate-700 rounded"></div>
      </div>
    </div>
  </div>
);

const StatsDisplaySkeleton: React.FC = () => (
  <div className="space-y-6 animate-pulse">
    <div className="p-6 bg-gm-bg-card/80 rounded-2xl">
      <div className="h-6 w-32 bg-slate-700 rounded mb-4"></div>
      <div className="grid grid-cols-2 gap-4">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="h-20 bg-slate-700 rounded-xl"></div>
        ))}
      </div>
    </div>
  </div>
);

const ChallengeCardSkeleton: React.FC = () => (
  <div className="p-6 bg-gm-bg-card/80 rounded-2xl animate-pulse">
    <div className="h-6 w-32 bg-slate-700 rounded mb-4"></div>
    <div className="space-y-3">
      {Array.from({ length: 2 }).map((_, i) => (
        <div key={i} className="h-24 bg-slate-700 rounded-xl"></div>
      ))}
    </div>
  </div>
);

const BadgeGridSkeleton: React.FC = () => (
  <div className="p-6 bg-gm-bg-card/80 rounded-2xl animate-pulse">
    <div className="h-6 w-32 bg-slate-700 rounded mb-4"></div>
    <div className="grid grid-cols-4 gap-3">
      {Array.from({ length: 8 }).map((_, i) => (
        <div key={i} className="h-16 bg-slate-700 rounded-xl"></div>
      ))}
    </div>
  </div>
);

const ContributionGraphSkeleton: React.FC = () => (
  <div className="p-6 bg-gm-bg-card/80 rounded-2xl animate-pulse">
    <div className="h-6 w-32 bg-slate-700 rounded mb-4"></div>
    <div className="flex gap-1">
      {Array.from({ length: 53 }).map((_, wi) => (
        <div key={wi} className="flex flex-col gap-1">
          {Array.from({ length: 7 }).map((_, di) => (
            <div key={di} className="w-3 h-3 bg-slate-700 rounded-sm"></div>
          ))}
        </div>
      ))}
    </div>
  </div>
);

export const DashboardContent: React.FC<DashboardContentProps> = ({ levelInfo, userStats, githubStats, statsDiff }) => {
  const navigate = useNavigate();

  return (
    <div className="space-y-6">
      {/* Profile Card */}
      <Suspense fallback={<ProfileCardSkeleton />}>
        <ProfileCard levelInfo={levelInfo} userStats={userStats} />
      </Suspense>

      {/* Quick Actions */}
      <div className="flex justify-end">
        <button
          className="flex items-center gap-2 px-4 py-2 text-sm text-dt-text-sub hover:text-gm-accent-cyan transition-colors"
          onClick={() => navigate('/xp-history')}
        >
          <span>📜</span>
          <span>XP獲得履歴を見る</span>
          <span className="text-xs">→</span>
        </button>
      </div>

      {/* Stats and Challenges Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Stats Display */}
        <Suspense fallback={<StatsDisplaySkeleton />}>
          <StatsDisplay
            githubStats={githubStats}
            userStats={userStats}
            statsDiff={statsDiff}
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
        <ContributionGraph githubStats={githubStats} />
      </Suspense>
    </div>
  );
};
