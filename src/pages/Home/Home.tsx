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

import { Component, createSignal, createResource, Show, onMount } from 'solid-js';
import { LoginCard } from '../../components/features/auth';
import { DashboardContent, XpNotification } from '../../components/features/gamification';
import { useAuth } from '../../stores/authStore';
import { useSettings } from '../../stores/settingsStore';
import { gamification } from '../../lib/tauri/commands';
import type { GitHubStats, LevelInfo, UserStats, XpGainedEvent } from '../../types';

// Home skeleton loader
const HomeSkeleton: Component = () => (
  <div class="p-6 space-y-6 animate-pulse">
    <div class="h-32 bg-slate-700 rounded-2xl"></div>
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div class="h-64 bg-slate-700 rounded-2xl"></div>
      <div class="h-64 bg-slate-700 rounded-2xl"></div>
    </div>
    <div class="h-48 bg-slate-700 rounded-2xl"></div>
    <div class="h-64 bg-slate-700 rounded-2xl"></div>
  </div>
);

export const Home: Component = () => {
  const auth = useAuth();
  const [xpEvent, setXpEvent] = createSignal<XpGainedEvent | null>(null);
  const [loading, setLoading] = createSignal(true);

  // Load user data
  const [githubStats] = createResource(
    () => auth.store.state.isLoggedIn,
    async (isLoggedIn) => {
      if (!isLoggedIn) return null;
      try {
        return await gamification.getGitHubStats();
      } catch (e) {
        console.error('Failed to load GitHub stats:', e);
        return null;
      }
    }
  );

  const [levelInfo] = createResource(
    () => auth.store.state.isLoggedIn,
    async (isLoggedIn) => {
      if (!isLoggedIn) return null;
      try {
        return await gamification.getLevelInfo();
      } catch (e) {
        console.error('Failed to load level info:', e);
        return null;
      }
    }
  );

  const [userStats] = createResource(
    () => auth.store.state.isLoggedIn,
    async (isLoggedIn) => {
      if (!isLoggedIn) return null;
      try {
        return await gamification.getUserStats();
      } catch (e) {
        console.error('Failed to load user stats:', e);
        return null;
      }
    }
  );

  // TODO: [FEATURE] Implement stats diff resource when sync result tracking is available
  // Stats diff is typically available after sync, but currently not implemented
  // const [statsDiff] = createResource(...)

  // Initial data load
  onMount(async () => {
    setLoading(false);
  });

  const isLoggedIn = () => auth.store.state.isLoggedIn;
  const isLoading = () => loading() || (isLoggedIn() && (githubStats.loading || levelInfo.loading || userStats.loading));

  return (
    <div class="flex-1 overflow-y-auto p-6">
      <Show
        when={!isLoading()}
        fallback={<HomeSkeleton />}
      >
        <Show
          when={isLoggedIn()}
          fallback={<LoginCard />}
        >
          <DashboardContent
            levelInfo={levelInfo()}
            userStats={userStats()}
            githubStats={githubStats()}
            statsDiff={null}
          />
        </Show>
      </Show>

      {/* XP Notification */}
      <XpNotification event={xpEvent()} onClose={() => setXpEvent(null)} />
    </div>
  );
};

export default Home;
