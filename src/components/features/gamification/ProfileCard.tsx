/**
 * Profile Card Component
 *
 * React implementation of ProfileCard component.
 * Displays user profile, level, and XP progress.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ./profile_card.rs
 */

import React from 'react';
import { useNavigate } from 'react-router-dom';
import { DropdownMenu, DropdownMenuItem } from '../../ui/dropdown';
import { Icon } from '../../icons';
import { useAuth } from '../../../stores/authStore';
import type { LevelInfo, UserStats } from '../../../types';

interface ProfileCardProps {
  levelInfo?: LevelInfo | null;
  userStats?: UserStats | null;
}

export const ProfileCard: React.FC<ProfileCardProps> = ({ levelInfo, userStats }) => {
  const logout = useAuth((s) => s.logout);
  const user = useAuth((s) => s.state.user);
  const navigate = useNavigate();

  const handleLogout = async () => {
    try {
      await logout();
      navigate('/');
    } catch (e) {
      console.error('Failed to logout:', e);
    }
  };

  const handleSettings = () => {
    navigate('/settings');
  };

  return (
    <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-lg">
      <div className="flex items-start justify-between">
        {/* User info section */}
        <div className="flex items-center gap-6">
          {/* Avatar */}
          <div className="relative">
            {user ? (
              <img
                src={user.avatarUrl || ''}
                alt="Avatar"
                className="w-20 h-20 rounded-xl border-2 border-gm-accent-cyan shadow-neon-cyan"
              />
            ) : (
              <div className="w-20 h-20 rounded-xl bg-gm-bg-secondary border-2 border-gm-accent-cyan flex items-center justify-center">
                <span className="text-3xl">👤</span>
              </div>
            )}

            {/* Level badge */}
            {levelInfo && (
              <div className="absolute -bottom-2 -right-2 px-2 py-1 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-lg text-white font-gaming text-sm font-bold shadow-neon-cyan">
                Lv. {levelInfo.currentLevel}
              </div>
            )}
          </div>

          {/* Username and XP */}
          <div className="space-y-2">
            <h2 className="text-2xl font-gaming font-bold text-white">
              {user?.username || 'User'}
            </h2>

            {/* XP Progress Bar */}
            {levelInfo && (
              <div className="space-y-1">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gm-accent-cyan font-gaming-mono">{levelInfo.totalXp} XP</span>
                  <span className="text-dt-text-sub">{levelInfo.xpToNextLevel} to next level</span>
                </div>
                <div className="w-64 h-3 bg-gm-bg-secondary rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-full transition-all duration-500"
                    style={{ width: `${levelInfo.progressPercent}%` }}
                  />
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Stats quick view */}
        <div className="flex items-center gap-6">
          {/* Streak */}
          {userStats && (
            <div className="text-center">
              <div className="flex items-center gap-2 text-gm-warning">
                <span className="text-2xl">🔥</span>
                <span className="text-3xl font-gaming-mono font-bold">{userStats.currentStreak}</span>
              </div>
              <div className="text-xs text-dt-text-sub">Day Streak</div>
            </div>
          )}

          {/* Total Commits */}
          {userStats && (
            <div className="text-center">
              <div className="flex items-center gap-2 text-gm-success">
                <span className="text-2xl">⭐</span>
                <span className="text-3xl font-gaming-mono font-bold">{userStats.totalCommits}</span>
              </div>
              <div className="text-xs text-dt-text-sub">Commits</div>
            </div>
          )}

          {/* Actions dropdown menu (Settings, Logout) */}
          <DropdownMenu
            trigger={() => <Icon name="more-vertical" className="w-5 h-5" />}
            align="right"
          >
            <DropdownMenuItem onClick={handleSettings}>
              <Icon name="settings" className="w-4 h-4" />
              <span>Settings</span>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={handleLogout} danger>
              <Icon name="logout" className="w-4 h-4" />
              <span>Logout</span>
            </DropdownMenuItem>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
};
