/**
 * Profile Card Component
 *
 * Solid.js implementation of ProfileCard component.
 * Displays user profile, level, and XP progress.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ./profile_card.rs
 */

import { Component, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { DropdownMenu, DropdownMenuItem } from '../../ui/dropdown';
import { Icon } from '../../icons';
import { useAuth } from '../../../stores/authStore';
import type { LevelInfo, UserStats } from '../../../types';

interface ProfileCardProps {
  levelInfo?: LevelInfo | null;
  userStats?: UserStats | null;
}

export const ProfileCard: Component<ProfileCardProps> = (props) => {
  const auth = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await auth.logout();
    navigate('/');
  };

  const handleSettings = () => {
    navigate('/settings');
  };

  const user = () => auth.store.state.user;
  const levelInfo = () => props.levelInfo;
  const userStats = () => props.userStats;

  return (
    <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-lg">
      <div class="flex items-start justify-between">
        {/* User info section */}
        <div class="flex items-center gap-6">
          {/* Avatar */}
          <div class="relative">
            <Show
              when={user()}
              fallback={
                <div class="w-20 h-20 rounded-xl bg-gm-bg-secondary border-2 border-gm-accent-cyan flex items-center justify-center">
                  <span class="text-3xl">👤</span>
                </div>
              }
            >
              {(u) => (
                <img
                  src={u().avatarUrl || ''}
                  alt="Avatar"
                  class="w-20 h-20 rounded-xl border-2 border-gm-accent-cyan shadow-neon-cyan"
                />
              )}
            </Show>

            {/* Level badge */}
            <Show when={levelInfo()}>
              {(info) => (
                <div class="absolute -bottom-2 -right-2 px-2 py-1 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-lg text-white font-gaming text-sm font-bold shadow-neon-cyan">
                  Lv. {info().currentLevel}
                </div>
              )}
            </Show>
          </div>

          {/* Username and XP */}
          <div class="space-y-2">
            <h2 class="text-2xl font-gaming font-bold text-white">
              {user()?.username || 'User'}
            </h2>

            {/* XP Progress Bar */}
            <Show when={levelInfo()}>
              {(info) => (
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-sm">
                    <span class="text-gm-accent-cyan font-gaming-mono">{info().totalXp} XP</span>
                    <span class="text-dt-text-sub">{info().xpToNextLevel} to next level</span>
                  </div>
                  <div class="w-64 h-3 bg-gm-bg-secondary rounded-full overflow-hidden">
                    <div
                      class="h-full bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-full transition-all duration-500"
                      style={{ width: `${info().progressPercent}%` }}
                    />
                  </div>
                </div>
              )}
            </Show>
          </div>
        </div>

        {/* Stats quick view */}
        <div class="flex items-center gap-6">
          {/* Streak */}
          <Show when={userStats()}>
            {(stats) => (
              <div class="text-center">
                <div class="flex items-center gap-2 text-gm-warning">
                  <span class="text-2xl">🔥</span>
                  <span class="text-3xl font-gaming-mono font-bold">{stats().currentStreak}</span>
                </div>
                <div class="text-xs text-dt-text-sub">Day Streak</div>
              </div>
            )}
          </Show>

          {/* Total Commits */}
          <Show when={userStats()}>
            {(stats) => (
              <div class="text-center">
                <div class="flex items-center gap-2 text-gm-success">
                  <span class="text-2xl">⭐</span>
                  <span class="text-3xl font-gaming-mono font-bold">{stats().totalCommits}</span>
                </div>
                <div class="text-xs text-dt-text-sub">Commits</div>
              </div>
            )}
          </Show>

          {/* Actions dropdown menu (Settings, Logout) */}
          <DropdownMenu
            trigger={() => <Icon name="more-vertical" class="w-5 h-5" />}
            align="right"
          >
            <DropdownMenuItem onClick={handleSettings}>
              <Icon name="settings" class="w-4 h-4" />
              <span>Settings</span>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={handleLogout} danger>
              <Icon name="logout" class="w-4 h-4" />
              <span>Logout</span>
            </DropdownMenuItem>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
};

