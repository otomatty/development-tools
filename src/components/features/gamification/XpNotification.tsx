/**
 * XP Notification Component
 *
 * Solid.js implementation of XpNotification component.
 * Displays XP gain notifications with animation.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./xp_notification.rs
 */

import { Component, Show, For } from 'solid-js';
import { AnimatedEmoji } from '../../ui/animated-emoji';
import { useAnimation } from '../../../stores/animationStore';
import type { XpGainedEvent } from '../../../types';

interface XpNotificationProps {
  event: XpGainedEvent | null;
  onClose: () => void;
}

export const XpNotification: Component<XpNotificationProps> = (props) => {
  const animation = useAnimation();

  const visible = () => props.event !== null;
  const event = () => props.event;

  const breakdownItems = () => {
    if (!event()) return [];
    const bd = event()!.xpBreakdown;
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
  };

  const showBreakdown = () => breakdownItems().length > 0;

  return (
    <Show when={visible()}>
      <div
        class={`fixed top-4 right-4 z-50 ${
          animation.store.enabled ? 'animate-slide-in' : ''
        }`}
      >
        <div class="p-4 bg-gm-bg-card/95 backdrop-blur-sm rounded-xl border border-gm-accent-cyan/30 shadow-neon-cyan min-w-80">
          {/* Header */}
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <AnimatedEmoji emoji="Sparkles" size="text-2xl" />
              <span class="text-gm-accent-cyan font-gaming font-bold">XP Gained!</span>
            </div>
            <button
              class="text-dt-text-sub hover:text-white transition-colors"
              onClick={props.onClose}
            >
              ✕
            </button>
          </div>

          {/* XP amount */}
          <div class="text-center mb-3">
            <span
              class={`text-4xl font-gaming-mono font-bold text-gm-success ${
                animation.store.enabled ? 'animate-pulse' : ''
              }`}
            >
              +{event()!.xpGained} XP
            </span>
          </div>

          {/* Breakdown */}
          <Show when={showBreakdown()}>
            <div class="space-y-1 text-sm text-dt-text-sub border-t border-slate-700/50 pt-3">
              <For each={breakdownItems()}>
                {(item) => (
                  <div class="flex justify-between">
                    <span>{item.label}</span>
                    <span class="text-gm-accent-cyan">+{item.value}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>

          {/* Level up indicator */}
          <Show when={event()!.levelUp}>
            <div class="mt-3 pt-3 border-t border-slate-700/50 text-center">
              <div class="flex items-center justify-center gap-2 text-gm-accent-gold font-bold">
                <AnimatedEmoji emoji="Party" size="text-xl" />
                <span>Level Up! Lv.{event()!.oldLevel} → Lv.{event()!.newLevel}</span>
              </div>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};

