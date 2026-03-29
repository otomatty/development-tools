/**
 * Home Page
 *
 * Main dashboard page showing gamification features.
 * Displays LoginCard for unauthenticated users or DashboardContent for authenticated users.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ../components/pages/home/mod.rs
 */

import { useState, useEffect } from 'react';
import { LoginCard } from '../../components/features/auth';
import { DashboardContent, XpNotification } from '../../components/features/gamification';
import { useAuth } from '../../stores/authStore';
import { gamification, github } from '../../lib/tauri/commands';
import type { GitHubStats, LevelInfo, UserStats, XpGainedEvent } from '../../types';

// Home skeleton loader
const HomeSkeleton = () => (
  <div className="p-6 space-y-6 animate-pulse">
    <div className="h-32 bg-slate-700 rounded-2xl"></div>
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div className="h-64 bg-slate-700 rounded-2xl"></div>
      <div className="h-64 bg-slate-700 rounded-2xl"></div>
    </div>
    <div className="h-48 bg-slate-700 rounded-2xl"></div>
    <div className="h-64 bg-slate-700 rounded-2xl"></div>
  </div>
);

export const Home = () => {
  const isLoggedIn = useAuth(s => s.state.isLoggedIn);
  const [xpEvent, setXpEvent] = useState<XpGainedEvent | null>(null);
  const [loading, setLoading] = useState(true);
  const [githubStats, setGithubStats] = useState<GitHubStats | null>(null);
  const [levelInfo, setLevelInfo] = useState<LevelInfo | null>(null);
  const [userStats, setUserStats] = useState<UserStats | null>(null);
  const [dataLoading, setDataLoading] = useState(false);

  useEffect(() => {
    if (!isLoggedIn) {
      setLoading(false);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setDataLoading(true);
    Promise.all([
      github.getStats().catch(e => { console.error('Failed to load GitHub stats:', e); return null; }),
      gamification.getLevelInfo().catch(e => { console.error('Failed to load level info:', e); return null; }),
      github.getUserStats().catch(e => { console.error('Failed to load user stats:', e); return null; }),
    ]).then(([stats, level, uStats]) => {
      if (cancelled) return;
      setGithubStats(stats);
      setLevelInfo(level);
      setUserStats(uStats);
      setDataLoading(false);
      setLoading(false);
    });

    return () => { cancelled = true; };
  }, [isLoggedIn]);

  // TODO: [FEATURE] Implement stats diff resource when sync result tracking is available
  // Stats diff is typically available after sync, but currently not implemented

  const isLoading = loading || (isLoggedIn && dataLoading);

  return (
    <div className="flex-1 overflow-y-auto p-6">
      {isLoading ? (
        <HomeSkeleton />
      ) : isLoggedIn ? (
        <DashboardContent
          levelInfo={levelInfo}
          userStats={userStats}
          githubStats={githubStats}
          statsDiff={null}
        />
      ) : (
        <LoginCard />
      )}

      {/* XP Notification */}
      <XpNotification event={xpEvent} onClose={() => setXpEvent(null)} />
    </div>
  );
};

export default Home;
