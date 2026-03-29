/**
 * Contribution Graph Component
 *
 * React implementation of ContributionGraph component.
 * Displays GitHub-style contribution calendar with hover cards.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ./contribution_graph.rs
 */

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { Icon } from '../../icons';
import { github } from '../../../lib/tauri/commands';
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

export const ContributionGraph: React.FC<ContributionGraphProps> = ({ githubStats }) => {
  const [hoveredDate, setHoveredDate] = useState<string | null>(null);
  const [hoverPosition, setHoverPosition] = useState<[number, number]>([0, 0]);
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncError, setSyncError] = useState<string | null>(null);

  // Fetch contribution calendar
  const [calendar, setCalendar] = useState<ContributionCalendar | null>(null);
  const [calendarLoading, setCalendarLoading] = useState(true);
  const [calendarError, setCalendarError] = useState<string | null>(null);

  const fetchCalendar = useCallback(async () => {
    setCalendarLoading(true);
    setCalendarError(null);
    try {
      if (githubStats?.contributionCalendar) {
        setCalendar(githubStats.contributionCalendar);
      } else {
        const data = await github.getContributionCalendar();
        setCalendar(data);
      }
    } catch (e) {
      setCalendarError(String(e));
    } finally {
      setCalendarLoading(false);
    }
  }, [githubStats]);

  useEffect(() => {
    fetchCalendar();
  }, [fetchCalendar]);

  const handleSync = async () => {
    if (isSyncing) return;
    setIsSyncing(true);
    setSyncError(null);
    try {
      await github.syncCodeStats(false);
      // Refresh calendar after sync
      await fetchCalendar();
    } catch (e) {
      setSyncError(String(e));
    } finally {
      setIsSyncing(false);
    }
  };

  const handleMouseEnter = (e: React.MouseEvent, date: string) => {
    setHoveredDate(date);
    setHoverPosition([e.clientX, e.clientY]);
  };

  const handleMouseLeave = () => {
    setHoveredDate(null);
  };

  // Create a memoized map for efficient date lookup
  const contributionMap = useMemo(() => {
    if (!calendar) return new Map<string, ContributionDay>();

    const map = new Map<string, ContributionDay>();
    for (const week of calendar.weeks) {
      for (const day of week.contributionDays) {
        map.set(day.date, day);
      }
    }
    return map;
  }, [calendar]);

  const getBgClass = (intensity: number): string => {
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
    <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-success/20 relative">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-xl font-gaming font-bold text-gm-success">📈 Contribution Graph</h3>

        <div className="flex items-center gap-4">
          {/* Sync button */}
          <button
            className={`p-2 rounded-lg transition-all ${
              isSyncing
                ? 'bg-gm-bg-secondary/30 text-gm-text-muted cursor-not-allowed'
                : 'bg-gm-bg-secondary/50 hover:bg-gm-bg-secondary text-gm-text-secondary hover:text-gm-text-primary'
            }`}
            onClick={handleSync}
            disabled={isSyncing}
            title="コード統計を同期"
          >
            <Icon name="refresh-cw" className={`w-4 h-4 ${isSyncing ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Error message */}
      {syncError && (
        <div className="mb-4 p-3 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error text-sm">
          {syncError}
        </div>
      )}

      {/* Loading state */}
      {calendarLoading && (
        <div className="animate-pulse space-y-4">
          <div className="h-6 w-32 bg-slate-700 rounded"></div>
          <div className="flex gap-1">
            {Array.from({ length: 53 }).map((_, wi) => (
              <div key={wi} className="flex flex-col gap-1">
                {Array.from({ length: 7 }).map((_, di) => (
                  <div key={di} className="w-3 h-3 bg-slate-700 rounded-sm"></div>
                ))}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Contribution calendar */}
      {calendar && !calendarLoading && (
        <>
          <div className="flex gap-1 min-w-fit overflow-x-auto pb-2">
            {calendar.weeks.map((week, wi) => (
              <div key={wi} className="flex flex-col gap-1">
                {week.contributionDays.map((day, di) => {
                  const intensity = getIntensity(day.contributionCount);
                  const bgClass = getBgClass(intensity);

                  return (
                    <div
                      key={di}
                      className={`w-3 h-3 rounded-sm ${bgClass} hover:ring-2 hover:ring-gm-accent-cyan transition-all cursor-pointer`}
                      onMouseEnter={(e) => handleMouseEnter(e, day.date)}
                      onMouseLeave={handleMouseLeave}
                      title={`${day.date}: ${day.contributionCount} contributions`}
                    />
                  );
                })}
              </div>
            ))}
          </div>

          {/* Legend */}
          <div className="flex items-center justify-between mt-4">
            <div className="flex items-center gap-2 text-xs text-dt-text-sub">
              <span>Less</span>
              <div className="flex gap-1">
                <div className="w-3 h-3 rounded-sm bg-gm-bg-secondary"></div>
                <div className="w-3 h-3 rounded-sm bg-gm-success/20"></div>
                <div className="w-3 h-3 rounded-sm bg-gm-success/40"></div>
                <div className="w-3 h-3 rounded-sm bg-gm-success/60"></div>
                <div className="w-3 h-3 rounded-sm bg-gm-success"></div>
              </div>
              <span>More</span>
            </div>
            <div className="text-xs text-dt-text-sub">
              {calendar.totalContributions} contributions in the last year
            </div>
          </div>
        </>
      )}

      {/* Hover tooltip */}
      {hoveredDate && (
        <div
          className="absolute z-50 px-3 py-2 bg-gm-bg-card/95 backdrop-blur-sm border border-gm-accent-cyan/20 rounded-lg shadow-lg pointer-events-none"
          style={{
            left: `${hoverPosition[0] + 10}px`,
            top: `${hoverPosition[1] - 10}px`,
          }}
        >
          <div className="text-sm text-white font-medium">{formatDate(hoveredDate)}</div>
          <div className="text-xs text-dt-text-sub">
            {contributionMap.get(hoveredDate)?.contributionCount ?? 0} contributions
          </div>
        </div>
      )}
    </div>
  );
};
