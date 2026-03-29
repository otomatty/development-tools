/**
 * Badge Grid Component
 *
 * React implementation of BadgeGrid component.
 * Displays earned badges and near-completion badges with progress.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./badge_grid.rs
 */

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { Modal, ModalHeader, ModalBody } from '../../ui/dialog';
import { github } from '../../../lib/tauri/commands';
import type { BadgeWithProgress } from '../../../types';

// Threshold for near-completion badges (percentage)
const NEAR_COMPLETION_THRESHOLD = 50;

// Individual earned badge item (compact)
const BadgeItem: React.FC<{ badge: BadgeWithProgress; onClick: () => void }> = ({ badge, onClick }) => {
  const rarityClass = (() => {
    switch (badge.rarity) {
      case 'bronze':
        return 'border-badge-bronze text-badge-bronze';
      case 'silver':
        return 'border-badge-silver text-badge-silver';
      case 'gold':
        return 'border-badge-gold text-badge-gold shadow-neon-cyan';
      case 'platinum':
        return 'border-badge-platinum text-badge-platinum shadow-neon-purple';
      default:
        return 'border-slate-600 text-slate-400';
    }
  })();

  const title = `${badge.name}: ${badge.description}`;

  return (
    <div
      className={`relative p-3 rounded-xl border-2 ${rarityClass} transition-all duration-200 hover:scale-105 cursor-pointer group`}
      title={title}
      onClick={onClick}
    >
      {/* Badge icon */}
      <div className="text-center">
        <span className="text-2xl">{badge.icon}</span>
      </div>

      {/* Badge name (on hover) */}
      <div className="absolute inset-0 flex items-center justify-center bg-gm-bg-card/90 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity p-2">
        <div className="text-center">
          <div className="text-xs font-bold text-white truncate">{badge.name}</div>
        </div>
      </div>
    </div>
  );
};

// Near completion badge item with progress bar
const NearCompletionBadgeItem: React.FC<{ badge: BadgeWithProgress; onClick: () => void }> = ({ badge, onClick }) => {
  const rarityClass = (() => {
    switch (badge.rarity) {
      case 'bronze':
        return 'border-badge-bronze/50';
      case 'silver':
        return 'border-badge-silver/50';
      case 'gold':
        return 'border-badge-gold/50';
      case 'platinum':
        return 'border-badge-platinum/50';
      default:
        return 'border-slate-600/50';
    }
  })();

  const progressBarClass = (() => {
    switch (badge.rarity) {
      case 'bronze':
        return 'bg-badge-bronze';
      case 'silver':
        return 'bg-badge-silver';
      case 'gold':
        return 'bg-badge-gold';
      case 'platinum':
        return 'bg-badge-platinum';
      default:
        return 'bg-slate-600';
    }
  })();

  const progress = badge.progress;
  const progressPercent = progress?.progressPercent ?? 0;
  const currentValue = progress?.currentValue ?? 0;
  const targetValue = progress?.targetValue ?? 0;

  return (
    <div
      className={`flex items-center gap-3 p-3 rounded-xl border ${rarityClass} bg-slate-800/30 hover:bg-slate-800/50 transition-colors cursor-pointer`}
      onClick={onClick}
    >
      {/* Badge icon */}
      <div className="flex-shrink-0 opacity-60">
        <span className="text-2xl">{badge.icon}</span>
      </div>

      {/* Badge info and progress */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-1">
          <span className="text-sm font-medium text-white truncate">{badge.name}</span>
          <span className="text-xs text-dt-text-sub ml-2">
            {currentValue}/{targetValue}
          </span>
        </div>

        {/* Progress bar */}
        <div className="h-1.5 bg-slate-700 rounded-full overflow-hidden">
          <div
            className={`h-full ${progressBarClass} transition-all duration-300`}
            style={{ width: `${Math.min(progressPercent, 100)}%` }}
          />
        </div>
      </div>

      {/* Progress percentage */}
      <div className="flex-shrink-0 text-sm font-bold text-gm-accent-cyan">
        {Math.round(progressPercent)}%
      </div>
    </div>
  );
};

// Badge detail modal
const BadgeDetailModal: React.FC<{
  badge: BadgeWithProgress | null;
  onClose: () => void;
}> = ({ badge, onClose }) => {
  const visible = badge !== null;

  const textClass = (() => {
    if (!badge) return 'text-slate-400';
    switch (badge.rarity) {
      case 'bronze':
        return 'text-badge-bronze';
      case 'silver':
        return 'text-badge-silver';
      case 'gold':
        return 'text-badge-gold';
      case 'platinum':
        return 'text-badge-platinum';
      default:
        return 'text-slate-400';
    }
  })();

  const progressBarClass = (() => {
    if (!badge) return 'bg-slate-600';
    switch (badge.rarity) {
      case 'bronze':
        return 'bg-badge-bronze';
      case 'silver':
        return 'bg-badge-silver';
      case 'gold':
        return 'bg-badge-gold';
      case 'platinum':
        return 'bg-badge-platinum';
      default:
        return 'bg-slate-600';
    }
  })();

  const borderClass = (() => {
    if (!badge) return 'border border-slate-600';
    switch (badge.rarity) {
      case 'bronze':
        return 'border-2 border-badge-bronze';
      case 'silver':
        return 'border-2 border-badge-silver';
      case 'gold':
        return 'border-2 border-badge-gold';
      case 'platinum':
        return 'border-2 border-badge-platinum';
      default:
        return 'border border-slate-600';
    }
  })();

  const categoryLabel = (() => {
    if (!badge) return '📌 Other';
    switch (badge.badgeType) {
      case 'milestone':
        return '🏁 Milestone';
      case 'streak':
        return '🔥 Streak';
      case 'consistency':
        return '📅 Consistency';
      case 'collaboration':
        return '🤝 Collaboration';
      case 'quality':
        return '✨ Quality';
      case 'challenge':
        return '🎯 Challenge';
      case 'level':
        return '⭐ Level';
      case 'stars':
        return '🌟 Stars';
      case 'language':
        return '🌍 Language';
      default:
        return '📌 Other';
    }
  })();

  return (
    <Modal visible={visible} onClose={onClose} size="sm" borderClass={borderClass}>
      {badge && (
        <>
          <ModalHeader onClose={onClose}>
            <h3 className={`text-xl font-gaming font-bold ${textClass}`}>{badge.name}</h3>
          </ModalHeader>
          <ModalBody className="text-center">
            <div className="space-y-4">
              {/* Badge icon */}
              <div className={badge.earned ? '' : 'opacity-50 grayscale'}>
                <span className="text-7xl">{badge.icon}</span>
              </div>

              {/* Description */}
              <p className="text-dt-text-sub">{badge.description}</p>

              {/* Category and rarity */}
              <div className="flex justify-center gap-4 text-sm">
                <span className="px-3 py-1 rounded-full bg-slate-800/50 text-dt-text-sub">
                  {categoryLabel}
                </span>
                <span className={`px-3 py-1 rounded-full bg-slate-800/50 uppercase font-bold ${textClass}`}>
                  {badge.rarity}
                </span>
              </div>

              {/* Status and progress */}
              <div className="pt-2">
                {badge.earned ? (
                  <div className="flex items-center justify-center gap-2 text-gm-success">
                    <span>✓</span>
                    <span className="font-bold">Unlocked!</span>
                  </div>
                ) : badge.progress ? (
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-dt-text-sub">Progress</span>
                      <span className="text-white font-medium">
                        {badge.progress.currentValue}/{badge.progress.targetValue}
                      </span>
                    </div>
                    <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                      <div
                        className={`h-full ${progressBarClass} transition-all duration-500`}
                        style={{ width: `${Math.min(badge.progress.progressPercent, 100)}%` }}
                      />
                    </div>
                    <div className="text-sm text-gm-accent-cyan font-bold">
                      {Math.round(badge.progress.progressPercent)}% complete
                    </div>
                  </div>
                ) : (
                  <div className="text-dt-text-sub italic">Not yet unlocked</div>
                )}
              </div>
            </div>
          </ModalBody>
        </>
      )}
    </Modal>
  );
};

export const BadgeGrid: React.FC = () => {
  const [selectedBadge, setSelectedBadge] = useState<BadgeWithProgress | null>(null);

  // Fetch badges with progress
  const [badges, setBadges] = useState<BadgeWithProgress[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchBadges = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await github.getBadgesWithProgress();
      setBadges(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBadges();
  }, [fetchBadges]);

  const earned = useMemo(() => badges?.filter((b) => b.earned) ?? [], [badges]);
  const nearCompletion = useMemo(
    () => badges?.filter((b) => !b.earned && (b.progress?.progressPercent ?? 0) >= NEAR_COMPLETION_THRESHOLD) ?? [],
    [badges]
  );
  const totalCount = badges?.length ?? 0;
  const earnedCount = earned.length;

  return (
    <>
      <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-badge-gold/20">
        {loading ? (
          <div className="animate-pulse space-y-4">
            <div className="h-6 w-32 bg-slate-700 rounded"></div>
            <div className="grid grid-cols-4 gap-3">
              {Array.from({ length: 8 }).map((_, i) => (
                <div key={i} className="h-16 bg-slate-700 rounded-xl"></div>
              ))}
            </div>
          </div>
        ) : error ? (
          <div className="text-center py-4 text-gm-error">
            Failed to load badges: {error}
          </div>
        ) : badges ? (
          <>
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-xl font-gaming font-bold text-badge-gold">🏅 Badges</h3>
              <span className="text-sm text-dt-text-sub">
                {earnedCount} / {totalCount} unlocked
              </span>
            </div>

            {/* Earned badges section */}
            {earned.length > 0 && (
              <div className="mb-6">
                <h4 className="text-sm font-bold text-gm-success mb-3 flex items-center gap-2">
                  <span>✓</span>
                  <span>Unlocked</span>
                  <span className="text-dt-text-sub font-normal">({earned.length})</span>
                </h4>
                <div className="grid grid-cols-5 gap-3">
                  {earned.map((badge) => (
                    <BadgeItem key={badge.name} badge={badge} onClick={() => setSelectedBadge(badge)} />
                  ))}
                </div>
              </div>
            )}

            {/* Near completion badges section */}
            {nearCompletion.length > 0 && (
              <div>
                <h4 className="text-sm font-bold text-gm-accent-cyan mb-3 flex items-center gap-2">
                  <span>🎯</span>
                  <span>Almost There</span>
                  <span className="text-dt-text-sub font-normal">({nearCompletion.length})</span>
                </h4>
                <div className="space-y-2">
                  {nearCompletion.map((badge) => (
                    <NearCompletionBadgeItem key={badge.name} badge={badge} onClick={() => setSelectedBadge(badge)} />
                  ))}
                </div>
              </div>
            )}

            {/* Empty state */}
            {earned.length === 0 && nearCompletion.length === 0 && (
              <div className="text-center py-8 text-dt-text-sub">
                <span className="text-4xl mb-2 block">🏅</span>
                <p>No badges yet. Keep coding to earn your first badge!</p>
              </div>
            )}
          </>
        ) : null}
      </div>

      {/* Badge detail modal */}
      <BadgeDetailModal badge={selectedBadge} onClose={() => setSelectedBadge(null)} />
    </>
  );
};
