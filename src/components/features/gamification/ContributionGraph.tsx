/**
 * Contribution Graph Component
 *
 * Solid.js implementation of ContributionGraph component.
 * Displays GitHub-style contribution calendar (草グラフ) with hover cards.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./contribution_graph.rs
 */

import { Component, createSignal, createResource, Show, For, createEffect } from 'solid-js';
import { Icon } from '../../icons';
import { gamification } from '../../../lib/tauri/commands';
import type { GitHubStats, ContributionCalendar, ContributionDay } from '../../../types';

interface ContributionGraphProps {
  githubStats?: GitHubStats | null;
}

// Get intensity level for contribution count
const getIntensity = (count: number): number => {
  if (count === 0) return 0;
  if (count <= 3) return 1;
  if (count <= 6) return 2;
  if (count <= 10) return 3;
  return 4;
};

// Format date for display
const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleDateString('ja-JP', { month: 'short', day: 'numeric' });
};

export const ContributionGraph: Component<ContributionGraphProps> = (props) => {
  const [showCodeLines, setShowCodeLines] = createSignal(false);
  const [hoveredDate, setHoveredDate] = createSignal<string | null>(null);
  const [hoverPosition, setHoverPosition] = createSignal<[number, number]>([0, 0]);
  const [isSyncing, setIsSyncing] = createSignal(false);
  const [syncError, setSyncError] = createSignal<string | null>(null);

  // Fetch contribution calendar
  const [calendar] = createResource(() => {
    if (props.githubStats?.contributionCalendar) {
      return Promise.resolve(props.githubStats.contributionCalendar);
    }
    return gamification.getContributionCalendar();
  });

  const handleSync = async () => {
    if (isSyncing()) return;
    setIsSyncing(true);
    setSyncError(null);
    try {
      await gamification.syncCodeStats(false);
      // Refresh calendar after sync
      calendar.refetch();
    } catch (e) {
      setSyncError(String(e));
    } finally {
      setIsSyncing(false);
    }
  };

  const handleMouseEnter = (e: MouseEvent, date: string) => {
    setHoveredDate(date);
    setHoverPosition([e.pageX, e.pageY]);
  };

  const handleMouseLeave = () => {
    setHoveredDate(null);
  };

  const contributionCalendar = () => calendar();

  return (
    <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-success/20 relative">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-xl font-gaming font-bold text-gm-success">📈 Contribution Graph</h3>

        <div class="flex items-center gap-4">
          {/* Sync button */}
          <button
            class={`p-2 rounded-lg transition-all ${
              isSyncing()
                ? 'bg-gm-bg-secondary/30 text-gm-text-muted cursor-not-allowed'
                : 'bg-gm-bg-secondary/50 hover:bg-gm-bg-secondary text-gm-text-secondary hover:text-gm-text-primary'
            }`}
            onClick={handleSync}
            disabled={isSyncing()}
            title="コード統計を同期"
          >
            <Icon name="refresh-cw" class={`w-4 h-4 ${isSyncing() ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Error message */}
      <Show when={syncError()}>
        <div class="mb-4 p-3 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error text-sm">
          {syncError()}
        </div>
      </Show>

      {/* Loading state */}
      <Show when={calendar.loading}>
        <div class="animate-pulse space-y-4">
          <div class="h-6 w-32 bg-slate-700 rounded"></div>
          <div class="flex gap-1">
            {Array.from({ length: 53 }).map(() => (
              <div class="flex flex-col gap-1">
                {Array.from({ length: 7 }).map(() => (
                  <div class="w-3 h-3 bg-slate-700 rounded-sm"></div>
                ))}
              </div>
            ))}
          </div>
        </div>
      </Show>

      {/* Contribution calendar */}
      <Show when={contributionCalendar()}>
        {(cal) => {
          const weeks = cal().weeks;
          return (
            <>
              <div class="flex gap-1 min-w-fit overflow-x-auto pb-2">
                <For each={weeks}>
                  {(week) => (
                    <div class="flex flex-col gap-1">
                      <For each={week.contributionDays}>
                        {(day) => {
                          const intensity = getIntensity(day.contributionCount);
                          const bgClass = () => {
                            switch (intensity) {
                              case 0:
                                return 'bg-gm-bg-secondary';
                              case 1:
                                return 'bg-gm-success/20';
                              case 2:
                                return 'bg-gm-success/40';
                              case 3:
                                return 'bg-gm-success/60';
                              default:
                                return 'bg-gm-success';
                            }
                          };

                          return (
                            <div
                              class={`w-3 h-3 rounded-sm ${bgClass()} hover:ring-2 hover:ring-gm-accent-cyan transition-all cursor-pointer`}
                              onMouseEnter={(e) => handleMouseEnter(e, day.date)}
                              onMouseLeave={handleMouseLeave}
                              title={`${day.date}: ${day.contributionCount} contributions`}
                            />
                          );
                        }}
                      </For>
                    </div>
                  )}
                </For>
              </div>

              {/* Legend */}
              <div class="flex items-center justify-between mt-4">
                <div class="flex items-center gap-2 text-xs text-dt-text-sub">
                  <span>Less</span>
                  <div class="flex gap-1">
                    <div class="w-3 h-3 rounded-sm bg-gm-bg-secondary"></div>
                    <div class="w-3 h-3 rounded-sm bg-gm-success/20"></div>
                    <div class="w-3 h-3 rounded-sm bg-gm-success/40"></div>
                    <div class="w-3 h-3 rounded-sm bg-gm-success/60"></div>
                    <div class="w-3 h-3 rounded-sm bg-gm-success"></div>
                  </div>
                  <span>More</span>
                </div>
                <div class="text-xs text-dt-text-sub">
                  {cal().totalContributions} contributions in the last year
                </div>
              </div>
            </>
          );
        }}
      </Show>

      {/* Hover tooltip */}
      <Show when={hoveredDate()}>
        <div
          class="absolute z-50 px-3 py-2 bg-gm-bg-card/95 backdrop-blur-sm border border-gm-accent-cyan/20 rounded-lg shadow-lg pointer-events-none"
          style={{
            left: `${hoverPosition()[0] + 10}px`,
            top: `${hoverPosition()[1] - 10}px`,
          }}
        >
          <div class="text-sm text-white font-medium">{formatDate(hoveredDate()!)}</div>
          <div class="text-xs text-dt-text-sub">
            {(() => {
              const cal = contributionCalendar();
              if (!cal) return '0 contributions';
              const day = cal.weeks
                .flatMap((w) => w.contributionDays)
                .find((d) => d.date === hoveredDate());
              return `${day?.contributionCount ?? 0} contributions`;
            })()}
          </div>
        </div>
      </Show>
    </div>
  );
};

