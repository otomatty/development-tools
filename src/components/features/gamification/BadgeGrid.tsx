/**
 * Badge Grid Component
 *
 * Solid.js implementation of BadgeGrid component.
 * Displays earned badges and near-completion badges with progress.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./badge_grid.rs
 */

import { Component, createSignal, createResource, Show, For } from 'solid-js';
import { Modal, ModalHeader, ModalBody } from '../../ui/dialog';
import { gamification } from '../../../lib/tauri/commands';
import type { BadgeWithProgress } from '../../../types';

// Threshold for near-completion badges (percentage)
const NEAR_COMPLETION_THRESHOLD = 50;

// Individual earned badge item (compact)
const BadgeItem: Component<{ badge: BadgeWithProgress; onClick: () => void }> = (props) => {
  const rarityClass = () => {
    switch (props.badge.rarity) {
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
  };

  const title = `${props.badge.name}: ${props.badge.description}`;

  return (
    <div
      class={`relative p-3 rounded-xl border-2 ${rarityClass()} transition-all duration-200 hover:scale-105 cursor-pointer group`}
      title={title}
      onClick={props.onClick}
    >
      {/* Badge icon */}
      <div class="text-center">
        <span class="text-2xl">{props.badge.icon}</span>
      </div>

      {/* Badge name (on hover) */}
      <div class="absolute inset-0 flex items-center justify-center bg-gm-bg-card/90 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity p-2">
        <div class="text-center">
          <div class="text-xs font-bold text-white truncate">{props.badge.name}</div>
        </div>
      </div>
    </div>
  );
};

// Near completion badge item with progress bar
const NearCompletionBadgeItem: Component<{ badge: BadgeWithProgress; onClick: () => void }> = (props) => {
  const rarityClass = () => {
    switch (props.badge.rarity) {
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
  };

  const progressBarClass = () => {
    switch (props.badge.rarity) {
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
  };

  const progress = () => props.badge.progress;
  const progressPercent = () => progress()?.progressPercent ?? 0;
  const currentValue = () => progress()?.currentValue ?? 0;
  const targetValue = () => progress()?.targetValue ?? 0;

  return (
    <div
      class={`flex items-center gap-3 p-3 rounded-xl border ${rarityClass()} bg-slate-800/30 hover:bg-slate-800/50 transition-colors cursor-pointer`}
      onClick={props.onClick}
    >
      {/* Badge icon */}
      <div class="flex-shrink-0 opacity-60">
        <span class="text-2xl">{props.badge.icon}</span>
      </div>

      {/* Badge info and progress */}
      <div class="flex-1 min-w-0">
        <div class="flex items-center justify-between mb-1">
          <span class="text-sm font-medium text-white truncate">{props.badge.name}</span>
          <span class="text-xs text-dt-text-sub ml-2">
            {currentValue()}/{targetValue()}
          </span>
        </div>

        {/* Progress bar */}
        <div class="h-1.5 bg-slate-700 rounded-full overflow-hidden">
          <div
            class={`h-full ${progressBarClass()} transition-all duration-300`}
            style={{ width: `${Math.min(progressPercent(), 100)}%` }}
          />
        </div>
      </div>

      {/* Progress percentage */}
      <div class="flex-shrink-0 text-sm font-bold text-gm-accent-cyan">
        {Math.round(progressPercent())}%
      </div>
    </div>
  );
};

// Badge detail modal
const BadgeDetailModal: Component<{
  badge: BadgeWithProgress | null;
  onClose: () => void;
}> = (props) => {
  const visible = () => props.badge !== null;

  const textClass = () => {
    if (!props.badge) return 'text-slate-400';
    switch (props.badge.rarity) {
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
  };

  const progressBarClass = () => {
    if (!props.badge) return 'bg-slate-600';
    switch (props.badge.rarity) {
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
  };

  const borderClass = () => {
    if (!props.badge) return 'border border-slate-600';
    switch (props.badge.rarity) {
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
  };

  const categoryLabel = () => {
    if (!props.badge) return '📌 Other';
    switch (props.badge.badgeType) {
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
  };

  return (
    <Modal visible={visible} onClose={props.onClose} size="sm" borderClass={borderClass()}>
      <Show when={props.badge}>
        {(badge) => (
          <>
            <ModalHeader onClose={props.onClose}>
              <h3 class={`text-xl font-gaming font-bold ${textClass()}`}>{badge().name}</h3>
            </ModalHeader>
            <ModalBody class="text-center">
              <div class="space-y-4">
                {/* Badge icon */}
                <div class={badge().earned ? '' : 'opacity-50 grayscale'}>
                  <span class="text-7xl">{badge().icon}</span>
                </div>

                {/* Description */}
                <p class="text-dt-text-sub">{badge().description}</p>

                {/* Category and rarity */}
                <div class="flex justify-center gap-4 text-sm">
                  <span class="px-3 py-1 rounded-full bg-slate-800/50 text-dt-text-sub">
                    {categoryLabel()}
                  </span>
                  <span class={`px-3 py-1 rounded-full bg-slate-800/50 uppercase font-bold ${textClass()}`}>
                    {badge().rarity}
                  </span>
                </div>

                {/* Status and progress */}
                <div class="pt-2">
                  <Show
                    when={badge().earned}
                    fallback={
                      <Show
                        when={badge().progress}
                        fallback={<div class="text-dt-text-sub italic">Not yet unlocked</div>}
                      >
                        {(prog) => (
                          <div class="space-y-2">
                            <div class="flex justify-between text-sm">
                              <span class="text-dt-text-sub">Progress</span>
                              <span class="text-white font-medium">
                                {prog().currentValue}/{prog().targetValue}
                              </span>
                            </div>
                            <div class="h-2 bg-slate-700 rounded-full overflow-hidden">
                              <div
                                class={`h-full ${progressBarClass()} transition-all duration-500`}
                                style={{ width: `${Math.min(prog().progressPercent, 100)}%` }}
                              />
                            </div>
                            <div class="text-sm text-gm-accent-cyan font-bold">
                              {Math.round(prog().progressPercent)}% complete
                            </div>
                          </div>
                        )}
                      </Show>
                    }
                  >
                    <div class="flex items-center justify-center gap-2 text-gm-success">
                      <span>✓</span>
                      <span class="font-bold">Unlocked!</span>
                    </div>
                  </Show>
                </div>
              </div>
            </ModalBody>
          </>
        )}
      </Show>
    </Modal>
  );
};

export const BadgeGrid: Component = () => {
  const [selectedBadge, setSelectedBadge] = createSignal<BadgeWithProgress | null>(null);

  // Fetch badges with progress
  const [badges] = createResource(() => gamification.getBadgesWithProgress());

  const earned = () => badges()?.filter((b) => b.earned) ?? [];
  const nearCompletion = () =>
    badges()?.filter((b) => !b.earned && (b.progress?.progressPercent ?? 0) >= NEAR_COMPLETION_THRESHOLD) ?? [];
  const totalCount = () => badges()?.length ?? 0;
  const earnedCount = () => earned().length;

  return (
    <>
      <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-badge-gold/20">
        <Show
          when={!badges.loading}
          fallback={
            <div class="animate-pulse space-y-4">
              <div class="h-6 w-32 bg-slate-700 rounded"></div>
              <div class="grid grid-cols-4 gap-3">
                {Array.from({ length: 8 }).map(() => (
                  <div class="h-16 bg-slate-700 rounded-xl"></div>
                ))}
              </div>
            </div>
          }
        >
          <Show
            when={badges()}
            fallback={
              <div class="text-center py-4 text-gm-error">
                Failed to load badges: {badges.error?.message || 'Unknown error'}
              </div>
            }
          >
            {/* Header */}
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-xl font-gaming font-bold text-badge-gold">🏅 Badges</h3>
              <span class="text-sm text-dt-text-sub">
                {earnedCount()} / {totalCount()} unlocked
              </span>
            </div>

            {/* Earned badges section */}
            <Show when={earned().length > 0}>
              <div class="mb-6">
                <h4 class="text-sm font-bold text-gm-success mb-3 flex items-center gap-2">
                  <span>✓</span>
                  <span>Unlocked</span>
                  <span class="text-dt-text-sub font-normal">({earned().length})</span>
                </h4>
                <div class="grid grid-cols-5 gap-3">
                  <For each={earned()}>
                    {(badge) => (
                      <BadgeItem badge={badge} onClick={() => setSelectedBadge(badge)} />
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Near completion badges section */}
            <Show when={nearCompletion().length > 0}>
              <div>
                <h4 class="text-sm font-bold text-gm-accent-cyan mb-3 flex items-center gap-2">
                  <span>🎯</span>
                  <span>Almost There</span>
                  <span class="text-dt-text-sub font-normal">({nearCompletion().length})</span>
                </h4>
                <div class="space-y-2">
                  <For each={nearCompletion()}>
                    {(badge) => (
                      <NearCompletionBadgeItem badge={badge} onClick={() => setSelectedBadge(badge)} />
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Empty state */}
            <Show when={earned().length === 0 && nearCompletion().length === 0}>
              <div class="text-center py-8 text-dt-text-sub">
                <span class="text-4xl mb-2 block">🏅</span>
                <p>No badges yet. Keep coding to earn your first badge!</p>
              </div>
            </Show>
          </Show>
        </Show>
      </div>

      {/* Badge detail modal */}
      <BadgeDetailModal badge={selectedBadge()} onClose={() => setSelectedBadge(null)} />
    </>
  );
};

