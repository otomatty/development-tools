/**
 * Home Page
 *
 * Main dashboard page showing gamification features.
 * Displays LoginCard for unauthenticated users or DashboardContent for authenticated users.
 *
 * GitHub stats and user stats are fetched through `*_with_cache` Tauri commands
 * with SWR-style revalidation so the dashboard stays responsive even when the
 * GitHub API is slow, rate-limited, or unreachable.
 *
 * Related Documentation:
 *   - Issues: https://github.com/otomatty/development-tools/issues/149
 *             https://github.com/otomatty/development-tools/issues/182
 */

import { useCallback, useEffect, useState } from 'react';
import { LoginCard } from '../../components/features/auth';
import { DashboardContent, XpNotification } from '../../components/features/gamification';
import { useAuth } from '../../stores/authStore';
import { useCachedFetch } from '../../hooks/useCachedFetch';
import { gamification, github } from '../../lib/tauri/commands';
import type { LevelInfo, XpGainedEvent } from '../../types';
import { CacheStatusBanner } from './CacheStatusBanner';

const STATS_STALE_TIME_MS = 30 * 60 * 1000; // 30 minutes
const USER_STATS_STALE_TIME_MS = 60 * 60 * 1000; // 60 minutes

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
  const authLoading = useAuth(s => s.isLoading);
  const [xpEvent, setXpEvent] = useState<XpGainedEvent | null>(null);

  const enabled = !authLoading && isLoggedIn;

  const githubStatsQuery = useCachedFetch(github.getStatsWithCache, {
    enabled,
    staleTime: STATS_STALE_TIME_MS,
  });

  const userStatsQuery = useCachedFetch(github.getUserStatsWithCache, {
    enabled,
    staleTime: USER_STATS_STALE_TIME_MS,
  });

  // `level_info` does not yet have a `_with_cache` variant — it is computed
  // from local DB state and is cheap, so a plain fetch is fine here.
  const [levelInfo, setLevelInfo] = useState<LevelInfo | null>(null);
  const [levelLoading, setLevelLoading] = useState(false);

  useEffect(() => {
    if (!enabled) {
      setLevelInfo(null);
      return;
    }
    let cancelled = false;
    setLevelLoading(true);
    gamification
      .getLevelInfo()
      .then(info => {
        if (!cancelled) setLevelInfo(info);
      })
      .catch(e => {
        console.error('Failed to load level info:', e);
        if (!cancelled) setLevelInfo(null);
      })
      .finally(() => {
        if (!cancelled) setLevelLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [enabled]);

  const githubRevalidate = githubStatsQuery.revalidate;
  const userRevalidate = userStatsQuery.revalidate;
  const handleRetry = useCallback(() => {
    void githubRevalidate();
    void userRevalidate();
  }, [githubRevalidate, userRevalidate]);

  // Initial loading: wait for auth restore and the first data fetch when
  // logged in. Cached data short-circuits this — once any cached value is
  // available we render the dashboard and let the banner communicate
  // freshness.
  //
  // We deliberately do NOT depend on the hooks' `isLoading` flags here. When
  // `enabled` flips from false to true, those flags only become true after
  // the initial-load effect fires — which is one render frame too late.
  // Anchoring on `data === null && error === null` shows the skeleton until
  // either succeeds, avoiding a flash of empty dashboard.
  const githubPending =
    githubStatsQuery.data === null && githubStatsQuery.error === null;
  const userPending =
    userStatsQuery.data === null && userStatsQuery.error === null;
  const initialDataLoading =
    isLoggedIn && (githubPending || userPending || levelLoading);

  const isLoading = authLoading || initialDataLoading;

  const fromCache = githubStatsQuery.fromCache || userStatsQuery.fromCache;
  const hasError = githubStatsQuery.error !== null || userStatsQuery.error !== null;
  const hasData =
    githubStatsQuery.data !== null || userStatsQuery.data !== null;
  const isRevalidating =
    githubStatsQuery.isRevalidating || userStatsQuery.isRevalidating;
  // Surface the older of the two cache timestamps so the user sees the
  // worst-case staleness rather than the freshest sub-component's.
  const bannerCachedAt =
    [githubStatsQuery.cachedAt, userStatsQuery.cachedAt]
      .filter((value): value is string => value !== null)
      .sort()[0] ?? null;

  return (
    <div className="flex-1 overflow-y-auto p-6">
      {isLoading ? (
        <HomeSkeleton />
      ) : isLoggedIn ? (
        <>
          <CacheStatusBanner
            fromCache={fromCache}
            hasError={hasError}
            hasData={hasData}
            isRevalidating={isRevalidating}
            cachedAt={bannerCachedAt}
            onRetry={handleRetry}
          />
          <DashboardContent
            levelInfo={levelInfo}
            userStats={userStatsQuery.data}
            githubStats={githubStatsQuery.data}
            statsDiff={null}
          />
        </>
      ) : (
        <LoginCard />
      )}

      {/* XP Notification */}
      <XpNotification event={xpEvent} onClose={() => setXpEvent(null)} />
    </div>
  );
};

export default Home;
