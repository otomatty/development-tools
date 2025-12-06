/**
 * Tauri Event Listeners
 *
 * Type-safe wrappers for Tauri event listeners.
 * This file provides a unified API for listening to Tauri events from the frontend.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/133
 *   - Events: src-tauri/src/commands/ (check for app.emit calls)
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AuthState,
  ToolStatusEvent,
  LogEvent,
  XpGainedEvent,
  StreakMilestoneEvent,
  BadgeEarnedEvent,
  AccessLogEntry,
} from '@/types';

// ============================================================================
// Auth Events
// ============================================================================

export const events = {
  /**
   * Listen for authentication state changes
   */
  onAuthStateChange: (callback: (state: AuthState) => void): Promise<UnlistenFn> =>
    listen<AuthState>('auth-state-change', (event) => callback(event.payload)),

  // ============================================================================
  // Tool Events
  // ============================================================================

  /**
   * Listen for tool status changes
   */
  onToolStatus: (callback: (event: ToolStatusEvent) => void): Promise<UnlistenFn> =>
    listen<ToolStatusEvent>('tool-status', (event) => callback(event.payload)),

  /**
   * Listen for tool log output
   */
  onToolLog: (callback: (event: LogEvent) => void): Promise<UnlistenFn> =>
    listen<LogEvent>('tool-log', (event) => callback(event.payload)),

  // ============================================================================
  // Gamification Events
  // ============================================================================

  /**
   * Listen for XP gained events
   */
  onXpGained: (callback: (event: XpGainedEvent) => void): Promise<UnlistenFn> =>
    listen<XpGainedEvent>('xp-gained', (event) => callback(event.payload)),

  /**
   * Listen for level up events
   * Note: level-up event uses the same payload as xp-gained
   */
  onLevelUp: (callback: (event: XpGainedEvent) => void): Promise<UnlistenFn> =>
    listen<XpGainedEvent>('level-up', (event) => callback(event.payload)),

  /**
   * Listen for streak milestone events
   */
  onStreakMilestone: (callback: (event: StreakMilestoneEvent) => void): Promise<UnlistenFn> =>
    listen<StreakMilestoneEvent>('streak-milestone', (event) => callback(event.payload)),

  /**
   * Listen for badge earned events
   */
  onBadgeEarned: (callback: (event: BadgeEarnedEvent) => void): Promise<UnlistenFn> =>
    listen<BadgeEarnedEvent>('badge-earned', (event) => callback(event.payload)),

  // ============================================================================
  // Mock Server Events
  // ============================================================================

  /**
   * Listen for mock server log events
   */
  onMockServerLog: (callback: (event: AccessLogEntry) => void): Promise<UnlistenFn> =>
    listen<AccessLogEntry>('mock-server-log', (event) => callback(event.payload)),
};

