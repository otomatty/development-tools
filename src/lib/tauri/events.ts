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
  AuthExpiredEvent,
  XpGainedEvent,
  StreakMilestoneEvent,
  BadgeEarnedEvent,
  NotificationsUpdatedEvent,
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

  /**
   * Listen for authentication-expired events.
   *
   * Emitted by the backend whenever the stored GitHub token is rejected (401)
   * — for example, the user revoked it on github.com between launches, or the
   * background sync scheduler observed a 401. The backend has already cleared
   * the credential by the time this fires; the frontend's job is to surface
   * the re-login prompt and stop firing API requests.
   *
   * See `src-tauri/src/auth/session.rs` for the emitter and reason codes.
   */
  onAuthExpired: (callback: (event: AuthExpiredEvent) => void): Promise<UnlistenFn> =>
    listen<AuthExpiredEvent>('auth-expired', (event) => callback(event.payload)),

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
  // GitHub Notifications Events (Issue #186)
  // ============================================================================

  /**
   * Listen for changes to the authenticated user's GitHub notifications.
   *
   * Emitted by the background scheduler whenever a poll observes a
   * non-304 response. The frontend should treat this as a cue to re-fetch
   * the notifications list.
   */
  onNotificationsUpdated: (
    callback: (event: NotificationsUpdatedEvent) => void,
  ): Promise<UnlistenFn> =>
    listen<NotificationsUpdatedEvent>('notifications-updated', (event) =>
      callback(event.payload),
    ),
};

