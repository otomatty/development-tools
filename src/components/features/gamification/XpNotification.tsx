/**
 * XP Notification Component
 *
 * React implementation of XpNotification component.
 * Displays XP gain notifications with animation.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./xp_notification.rs
 */

import React, { useMemo } from 'react';
import { AnimatedEmoji } from '../../ui/animated-emoji';
import { useAnimation } from '../../../stores/animationStore';
import type { XpGainedEvent } from '../../../types';

interface XpNotificationProps {
  event: XpGainedEvent | null;
  onClose: () => void;
}

export const XpNotification: React.FC<XpNotificationProps> = ({ event, onClose }) => {
  const enabled = useAnimation((s) => s.enabled);

  const breakdownItems = useMemo(() => {
    if (!event) return [];
    const bd = event.xpBreakdown;
    const items: Array<{ label: string; value: number }> = [];
    if (bd.commitsXp > 0) items.push({ label: '📝 Commits', value: bd.commitsXp });
    if (bd.prsCreatedXp > 0) items.push({ label: '🔀 PRs Created', value: bd.prsCreatedXp });
    if (bd.prsMergedXp > 0) items.push({ label: '✅ PRs Merged', value: bd.prsMergedXp });
    if (bd.issuesCreatedXp > 0) items.push({ label: '📋 Issues Created', value: bd.issuesCreatedXp });
    if (bd.issuesClosedXp > 0) items.push({ label: '🎯 Issues Closed', value: bd.issuesClosedXp });
    if (bd.reviewsXp > 0) items.push({ label: '👁️ Reviews', value: bd.reviewsXp });
    if (bd.starsXp > 0) items.push({ label: '⭐ Stars', value: bd.starsXp });
    if (bd.streakBonusXp > 0) items.push({ label: '🔥 Streak Bonus', value: bd.streakBonusXp });
    return items;
  }, [event]);

  const showBreakdown = breakdownItems.length > 0;

  if (!event) return null;

  return (
    <div
      className={`fixed top-4 right-4 z-50 ${
        enabled ? 'animate-slide-in' : ''
      }`}
    >
      <div className="p-4 bg-gm-bg-card/95 backdrop-blur-sm rounded-xl border border-gm-accent-cyan/30 shadow-neon-cyan min-w-80">
        {/* Header */}
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <AnimatedEmoji emoji="Sparkles" size="text-2xl" />
            <span className="text-gm-accent-cyan font-gaming font-bold">XP Gained!</span>
          </div>
          <button
            className="text-dt-text-sub hover:text-white transition-colors"
            onClick={onClose}
          >
            ✕
          </button>
        </div>

        {/* XP amount */}
        <div className="text-center mb-3">
          <span
            className={`text-4xl font-gaming-mono font-bold text-gm-success ${
              enabled ? 'animate-pulse' : ''
            }`}
          >
            +{event.xpGained} XP
          </span>
        </div>

        {/* Breakdown */}
        {showBreakdown && (
          <div className="space-y-1 text-sm text-dt-text-sub border-t border-slate-700/50 pt-3">
            {breakdownItems.map((item, i) => (
              <div key={i} className="flex justify-between">
                <span>{item.label}</span>
                <span className="text-gm-accent-cyan">+{item.value}</span>
              </div>
            ))}
          </div>
        )}

        {/* Level up indicator */}
        {event.levelUp && (
          <div className="mt-3 pt-3 border-t border-slate-700/50 text-center">
            <div className="flex items-center justify-center gap-2 text-gm-accent-gold font-bold">
              <AnimatedEmoji emoji="Party" size="text-xl" />
              <span>Level Up! Lv.{event.oldLevel} → Lv.{event.newLevel}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
