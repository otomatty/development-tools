//! Sync Notification Utilities
//!
//! Handles displaying notifications based on sync results.
//! This module centralizes notification logic for XP gains, level ups, and badges.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/home_page.rs
//! Dependencies:
//!   └─ src/types/gamification.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::types::{NewBadgeInfo, NotificationMethod, SyncResult, UserSettings, XpGainedEvent};

/// Handle sync result notifications
///
/// Shows app-internal notifications based on sync result and notification settings.
/// This function centralizes the notification logic to avoid duplication.
pub fn handle_sync_result_notifications(
    sync_result: &SyncResult,
    notification_settings: ReadSignal<Option<UserSettings>>,
    set_xp_event: WriteSignal<Option<XpGainedEvent>>,
    set_level_up_event: WriteSignal<Option<XpGainedEvent>>,
    set_new_badges_event: WriteSignal<Vec<NewBadgeInfo>>,
) {
    // Show notification if XP gained (check notification settings)
    // Use get_untracked() since this is called from event handlers (non-reactive context)
    if sync_result.xp_gained > 0 {
        let should_show_app_notification = notification_settings
            .get_untracked()
            .map(|s| {
                let method = NotificationMethod::from_str(&s.notification_method);
                method != NotificationMethod::None && method != NotificationMethod::OsOnly
            })
            .unwrap_or(true); // Default to showing if settings not loaded

        if should_show_app_notification {
            let event = XpGainedEvent {
                xp_gained: sync_result.xp_gained,
                total_xp: sync_result.user_stats.total_xp,
                old_level: sync_result.old_level,
                new_level: sync_result.new_level,
                level_up: sync_result.level_up,
                xp_breakdown: sync_result.xp_breakdown.clone(),
                streak_bonus: sync_result.streak_bonus.clone(),
            };

            if sync_result.level_up {
                // Check if level up notifications are enabled
                let should_show_level_up = notification_settings
                    .get_untracked()
                    .map(|s| s.notify_level_up)
                    .unwrap_or(true);

                if should_show_level_up {
                    set_level_up_event.set(Some(event));
                }
            } else {
                // Check if XP gain notifications are enabled
                let should_show_xp = notification_settings
                    .get_untracked()
                    .map(|s| s.notify_xp_gain)
                    .unwrap_or(true);

                if should_show_xp {
                    set_xp_event.set(Some(event));

                    // Auto-hide after 5 seconds
                    if let Some(window) = web_sys::window() {
                        let closure = wasm_bindgen::closure::Closure::once(move || {
                            set_xp_event.set(None);
                        });
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            5000,
                        );
                        closure.forget();
                    }
                }
            }
        }
    }

    // Show badge notifications if any (check notification settings)
    if !sync_result.new_badges.is_empty() {
        let should_show_badge_notification = notification_settings
            .get_untracked()
            .map(|s| {
                let method = NotificationMethod::from_str(&s.notification_method);
                (method != NotificationMethod::None && method != NotificationMethod::OsOnly)
                    && s.notify_badge_earned
            })
            .unwrap_or(true); // Default to showing if settings not loaded

        if should_show_badge_notification {
            set_new_badges_event.set(sync_result.new_badges.clone());
        }
    }
}
