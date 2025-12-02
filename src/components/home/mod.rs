//! Home page components
//!
//! This module contains all components for the gamification home page.

pub mod badge_grid;
pub mod challenge_card;
pub mod contribution_graph;
pub mod login_card;
pub mod profile_card;
pub mod skeleton;
pub mod stats_display;
pub mod xp_history;
pub mod xp_notification;

pub use badge_grid::BadgeGrid;
pub use challenge_card::ChallengeCard;
pub use contribution_graph::ContributionGraph;
pub use login_card::{LoginCard, LoginState};
pub use profile_card::ProfileCard;
pub use skeleton::HomeSkeleton;
pub use stats_display::StatsDisplay;
pub use xp_history::XpHistoryPage;
pub use xp_notification::{LevelUpModal, MultipleBadgesNotification, XpNotification};

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wasm_bindgen::JsCast;

use crate::components::network_status::use_is_online;
use crate::tauri_api;
use crate::types::{
    AppPage, AuthState, Badge, BadgeDefinition, DeviceTokenStatus, GitHubStats, LevelInfo,
    NewBadgeInfo, NotificationMethod, StatsDiffResult, SyncResult, UserSettings, UserStats,
    XpGainedEvent,
};

/// Handle sync result notifications
///
/// Shows app-internal notifications based on sync result and notification settings.
/// This function centralizes the notification logic to avoid duplication.
fn handle_sync_result_notifications(
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

/// Home page component
#[component]
pub fn HomePage(set_current_page: WriteSignal<AppPage>) -> impl IntoView {
    // State
    let (auth_state, set_auth_state) = signal(AuthState::default());
    let (loading, set_loading) = signal(true);
    let (github_stats, set_github_stats) = signal(Option::<GitHubStats>::None);
    let (level_info, set_level_info) = signal(Option::<LevelInfo>::None);
    let (user_stats, set_user_stats) = signal(Option::<UserStats>::None);
    let (badges, set_badges) = signal(Vec::<Badge>::new());
    let (badge_definitions, set_badge_definitions) = signal(Vec::<BadgeDefinition>::new());
    let (error, set_error) = signal(Option::<String>::None);

    // Stats diff for day-over-day comparison
    let (stats_diff, set_stats_diff) = signal(Option::<StatsDiffResult>::None);

    // Cache status tracking
    let (data_from_cache, set_data_from_cache) = signal(false);
    let (cache_timestamp, set_cache_timestamp) = signal(Option::<String>::None);

    // XP notification state
    let (xp_event, set_xp_event) = signal(Option::<XpGainedEvent>::None);
    let (level_up_event, set_level_up_event) = signal(Option::<XpGainedEvent>::None);

    // Badge notification state
    let (new_badges_event, set_new_badges_event) = signal(Vec::<NewBadgeInfo>::new());

    // Notification settings
    let (notification_settings, set_notification_settings) = signal(Option::<UserSettings>::None);

    // Device Flow login state
    let (login_state, set_login_state) = signal(LoginState::default());
    let (polling_active, set_polling_active) = signal(false);

    // Component lifecycle - used to cancel async loops when component unmounts
    // Using Arc<AtomicBool> to safely share across async boundaries
    let component_mounted = Arc::new(AtomicBool::new(true));
    let component_mounted_for_auto_sync = component_mounted.clone();
    let component_mounted_for_polling = component_mounted.clone();

    // Cleanup when component unmounts
    on_cleanup({
        let component_mounted = component_mounted.clone();
        let set_polling_active = set_polling_active.clone();
        move || {
            // Signal all async loops to stop
            component_mounted.store(false, Ordering::SeqCst);
            // Also stop polling if active
            set_polling_active.set(false);
            web_sys::console::log_1(&"HomePage: Component unmounted, cleanup triggered".into());
        }
    });

    // Load initial data
    spawn_local(async move {
        // Get auth state
        match tauri_api::get_auth_state().await {
            Ok(state) => {
                set_auth_state.set(state.clone());

                // If logged in, load additional data and settings
                if state.is_logged_in {
                    // Load notification/sync settings
                    match tauri_api::get_settings().await {
                        Ok(settings) => {
                            // Check sync_on_startup setting
                            let should_sync_on_startup = settings.sync_on_startup;

                            set_notification_settings.set(Some(settings));

                            // Perform startup sync if enabled
                            if should_sync_on_startup {
                                web_sys::console::log_1(
                                    &"Startup sync: Syncing GitHub stats...".into(),
                                );
                                match tauri_api::sync_github_stats().await {
                                    Ok(sync_result) => {
                                        set_user_stats.set(Some(sync_result.user_stats.clone()));
                                        set_stats_diff.set(sync_result.stats_diff.clone());

                                        // Handle notifications to provide consistent UX with other syncs
                                        handle_sync_result_notifications(
                                            &sync_result,
                                            notification_settings,
                                            set_xp_event,
                                            set_level_up_event,
                                            set_new_badges_event,
                                        );

                                        web_sys::console::log_1(
                                            &format!(
                                                "Startup sync: Completed, XP gained: {}",
                                                sync_result.xp_gained
                                            )
                                            .into(),
                                        );
                                    }
                                    Err(e) => {
                                        web_sys::console::error_1(
                                            &format!("Startup sync failed: {}", e).into(),
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to load settings: {}", e).into(),
                            );
                            set_error.set(Some(format!("設定の読み込みに失敗しました: {}", e)));
                        }
                    }

                    load_user_data(
                        set_github_stats,
                        set_level_info,
                        set_user_stats,
                        set_badges,
                        set_error,
                        set_data_from_cache,
                        set_cache_timestamp,
                    )
                    .await;
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get auth state: {}", e).into());
                set_error.set(Some(e));
            }
        }

        // Load badge definitions (doesn't require auth)
        if let Ok(defs) = tauri_api::get_badge_definitions().await {
            set_badge_definitions.set(defs);
        }

        set_loading.set(false);
    });

    // Online recovery sync - sync when coming back online
    // This is separate from the interval-based auto-sync
    {
        let is_online = use_is_online();

        // Track previous online state to detect transition
        Effect::new(move |prev_online: Option<bool>| {
            let current_online = is_online.get();

            // Check if we just came back online (was offline, now online)
            if let Some(was_online) = prev_online {
                if !was_online && current_online {
                    web_sys::console::log_1(
                        &"Network: Online recovery detected, triggering sync...".into(),
                    );

                    // Check if user is logged in before syncing
                    let auth = auth_state.get_untracked();
                    if auth.is_logged_in {
                        spawn_local(async move {
                            match tauri_api::sync_github_stats().await {
                                Ok(sync_result) => {
                                    set_user_stats.set(Some(sync_result.user_stats.clone()));
                                    set_stats_diff.set(sync_result.stats_diff.clone());

                                    // Clear cache indicator - we have fresh data now
                                    set_data_from_cache.set(false);
                                    set_cache_timestamp.set(None);

                                    // Handle notifications
                                    handle_sync_result_notifications(
                                        &sync_result,
                                        notification_settings,
                                        set_xp_event,
                                        set_level_up_event,
                                        set_new_badges_event,
                                    );

                                    web_sys::console::log_1(
                                        &format!(
                                            "Online recovery sync: Completed, XP gained: {}",
                                            sync_result.xp_gained
                                        )
                                        .into(),
                                    );
                                }
                                Err(e) => {
                                    web_sys::console::error_1(
                                        &format!("Online recovery sync failed: {}", e).into(),
                                    );
                                }
                            }
                        });
                    }
                }
            }

            current_online
        });
    }

    // Setup auto-sync timer - interval is determined by user settings
    {
        let auth_state = auth_state.clone();
        let component_mounted = component_mounted_for_auto_sync;

        spawn_local(async move {
            // Poll for settings with timeout (max 5 seconds, check every 200ms)
            let max_wait_ms = 5000;
            let poll_interval_ms = 200;
            let mut waited = 0;

            while notification_settings.get_untracked().is_none() && waited < max_wait_ms {
                if !component_mounted.load(Ordering::SeqCst) {
                    web_sys::console::log_1(
                        &"Auto-sync: Component unmounted while waiting for settings".into(),
                    );
                    return;
                }
                if let Some(window) = web_sys::window() {
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            poll_interval_ms,
                        );
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    waited += poll_interval_ms;
                } else {
                    break;
                }
            }

            web_sys::console::log_1(&"Auto-sync: Settings loaded, starting timer loop.".into());

            // Auto-sync loop - runs based on user settings while component is mounted
            loop {
                // Check if component is still mounted - exit if not
                if !component_mounted.load(Ordering::SeqCst) {
                    web_sys::console::log_1(
                        &"Auto-sync: Component unmounted, stopping loop".into(),
                    );
                    break;
                }

                // Get current sync interval from settings (use saturating_mul to avoid overflow)
                let sync_interval_ms = notification_settings
                    .get_untracked()
                    .map(|s| {
                        if s.sync_interval_minutes == 0 {
                            0 // Manual only
                        } else {
                            s.sync_interval_minutes
                                .saturating_mul(60)
                                .saturating_mul(1000) // Convert minutes to ms safely
                        }
                    })
                    .unwrap_or(60 * 60 * 1000); // Default: 1 hour

                // If sync interval is 0 (manual only), wait a bit and check again
                if sync_interval_ms == 0 {
                    if let Some(window) = web_sys::window() {
                        let promise = js_sys::Promise::new(&mut |resolve, _| {
                            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                &resolve, 30000, // Check every 30 seconds if settings changed
                            );
                        });
                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    }
                    continue;
                }

                if let Some(window) = web_sys::window() {
                    // Use get_untracked() since we're in an async context outside reactive tracking
                    // This is intentional - we poll these values periodically, not reactively
                    let auth = auth_state.get_untracked();

                    // Check background_sync setting
                    let background_sync_enabled = notification_settings
                        .get_untracked()
                        .map(|s| s.background_sync)
                        .unwrap_or(true);

                    if auth.is_logged_in && background_sync_enabled {
                        web_sys::console::log_1(
                            &format!(
                                "Auto-sync: Syncing GitHub stats (interval: {}ms)...",
                                sync_interval_ms
                            )
                            .into(),
                        );

                        match tauri_api::sync_github_stats().await {
                            Ok(sync_result) => {
                                set_user_stats.set(Some(sync_result.user_stats.clone()));
                                set_stats_diff.set(sync_result.stats_diff.clone());

                                // Handle notifications based on sync result
                                handle_sync_result_notifications(
                                    &sync_result,
                                    notification_settings,
                                    set_xp_event,
                                    set_level_up_event,
                                    set_new_badges_event,
                                );

                                // Reload level info and badges
                                if let Ok(info) = tauri_api::get_level_info().await {
                                    set_level_info.set(info);
                                }
                                if let Ok(b) = tauri_api::get_badges().await {
                                    set_badges.set(b);
                                }

                                web_sys::console::log_1(
                                    &format!(
                                        "Auto-sync: Completed, XP gained: {}",
                                        sync_result.xp_gained
                                    )
                                    .into(),
                                );
                            }
                            Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Auto-sync failed: {}", e).into(),
                                );
                            }
                        }
                    }

                    // Check again before waiting
                    if !component_mounted.load(Ordering::SeqCst) {
                        web_sys::console::log_1(
                            &"Auto-sync: Component unmounted during operation, stopping".into(),
                        );
                        break;
                    }

                    // Wait for next interval using a promise-based sleep
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            sync_interval_ms,
                        );
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                } else {
                    // No window available, exit loop
                    web_sys::console::log_1(
                        &"Auto-sync: No window available, stopping loop".into(),
                    );
                    break;
                }
            }
        });
    }

    // Handle login with Device Flow
    let on_login = Callback::new(move |_: leptos::ev::MouseEvent| {
        spawn_local(async move {
            set_login_state.set(LoginState::Starting);

            match tauri_api::start_device_flow().await {
                Ok(device_response) => {
                    web_sys::console::log_1(
                        &format!(
                            "Device Flow started. User code: {}, URL: {}",
                            device_response.user_code, device_response.verification_uri
                        )
                        .into(),
                    );

                    // Show the user code
                    set_login_state.set(LoginState::WaitingForCode {
                        user_code: device_response.user_code,
                        verification_uri: device_response.verification_uri,
                        expires_in: device_response.expires_in,
                    });
                }
                Err(e) => {
                    set_login_state.set(LoginState::Error(format!("Failed to start login: {}", e)));
                }
            }
        });
    });

    // Handle opening verification URL
    let on_open_url = Callback::new(move |url: String| {
        let url_clone = url.clone();
        let component_mounted = component_mounted_for_polling.clone();

        // Open URL in system browser using Tauri API
        spawn_local(async move {
            if let Err(e) = tauri_api::open_url(&url_clone).await {
                web_sys::console::error_1(&format!("Failed to open URL: {}", e).into());
            }
        });

        // Start polling for token
        set_login_state.set(LoginState::Polling);
        set_polling_active.set(true);

        spawn_local(async move {
            // Polling interval (5 seconds as recommended by GitHub)
            const POLL_INTERVAL_MS: i32 = 5000;

            loop {
                // Check if component is still mounted
                if !component_mounted.load(Ordering::SeqCst) {
                    web_sys::console::log_1(
                        &"Device Flow polling: Component unmounted, stopping".into(),
                    );
                    break;
                }

                // Check if polling is still active (user may have cancelled)
                if !polling_active.get_untracked() {
                    web_sys::console::log_1(
                        &"Device Flow polling: Polling cancelled, stopping".into(),
                    );
                    break;
                }

                match tauri_api::poll_device_token().await {
                    Ok(status) => {
                        match status {
                            DeviceTokenStatus::Pending => {
                                // Still waiting - continue polling
                                web_sys::console::log_1(
                                    &"Device Flow: Authorization pending...".into(),
                                );
                            }
                            DeviceTokenStatus::Success {
                                auth_state: new_auth_state,
                            } => {
                                // Success! User is logged in
                                web_sys::console::log_1(
                                    &"Device Flow: Authorization successful!".into(),
                                );
                                set_auth_state.set(new_auth_state);
                                set_login_state.set(LoginState::Initial);
                                set_polling_active.set(false);

                                // Load user data
                                load_user_data(
                                    set_github_stats,
                                    set_level_info,
                                    set_user_stats,
                                    set_badges,
                                    set_error,
                                    set_data_from_cache,
                                    set_cache_timestamp,
                                )
                                .await;
                                break;
                            }
                            DeviceTokenStatus::Error { message } => {
                                // Error occurred
                                set_login_state.set(LoginState::Error(message));
                                set_polling_active.set(false);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        set_login_state.set(LoginState::Error(format!("Polling failed: {}", e)));
                        set_polling_active.set(false);
                        break;
                    }
                }

                // Check again before waiting
                if !component_mounted.load(Ordering::SeqCst) || !polling_active.get_untracked() {
                    break;
                }

                // Wait before next poll
                if let Some(window) = web_sys::window() {
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            POLL_INTERVAL_MS,
                        );
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                } else {
                    // No window available, exit loop
                    break;
                }
            }
        });
    });

    // Handle cancel login
    let on_cancel_login = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_polling_active.set(false);
        set_login_state.set(LoginState::Initial);

        // Cancel the device flow on backend
        spawn_local(async move {
            let _ = tauri_api::cancel_device_flow().await;
        });
    });

    // Handle logout
    let on_logout = Callback::new(move |_: leptos::ev::MouseEvent| {
        spawn_local(async move {
            set_loading.set(true);
            if let Err(e) = tauri_api::logout().await {
                set_error.set(Some(format!("Logout failed: {}", e)));
            } else {
                set_auth_state.set(AuthState::default());
                set_github_stats.set(None);
                set_level_info.set(None);
                set_user_stats.set(None);
                set_badges.set(Vec::new());
            }
            set_loading.set(false);
        });
    });

    // Callbacks for closing notifications
    let on_close_xp = move || set_xp_event.set(None);
    let on_close_level_up = move || set_level_up_event.set(None);
    let on_close_badges = move || set_new_badges_event.set(Vec::new());

    view! {
        <div class="flex-1 overflow-y-auto bg-gradient-to-br from-gm-bg-primary via-gm-bg-secondary to-gm-bg-primary min-h-full">
            // XP Notification
            <XpNotification event=xp_event on_close=on_close_xp />

            // Level Up Modal
            <LevelUpModal event=level_up_event on_close=on_close_level_up />

            // Badge Notification
            <MultipleBadgesNotification badges=new_badges_event on_close=on_close_badges />

            <div class="max-w-6xl mx-auto p-6 space-y-6">
                // Error display
                <Show when=move || error.get().is_some()>
                    <div class="p-4 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                // Cache indicator - show when data is from cache (offline mode)
                <Show when=move || data_from_cache.get()>
                    <div class="p-3 bg-gm-warning/20 border border-gm-warning/50 rounded-lg flex items-center gap-3">
                        <svg class="w-5 h-5 text-gm-warning flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                        </svg>
                        <div class="flex-1">
                            <p class="text-gm-warning text-sm font-medium">
                                "キャッシュデータを表示中"
                            </p>
                            <p class="text-gm-text-secondary text-xs">
                                {move || {
                                    cache_timestamp.get()
                                        .map(|ts| format!("最終更新: {}", ts))
                                        .unwrap_or_else(|| "オンライン復帰時に自動更新されます".to_string())
                                }}
                            </p>
                        </div>
                    </div>
                </Show>

                // Loading state - show skeleton UI
                <Show when=move || loading.get()>
                    <HomeSkeleton />
                </Show>

                // Content
                <Show when=move || !loading.get()>
                    <Show
                        when=move || auth_state.get().is_logged_in
                        fallback=move || view! {
                            <LoginCard
                                login_state=login_state
                                on_login=on_login
                                on_cancel=on_cancel_login
                                on_open_url=on_open_url
                            />
                        }
                    >
                        // Profile Card
                        <ProfileCard
                            auth_state=auth_state
                            level_info=level_info
                            user_stats=user_stats
                            on_logout=move |e| on_logout.run(e)
                            on_settings=move |_| set_current_page.set(AppPage::Settings)
                        />

                        // Quick Actions
                        <div class="flex justify-end">
                            <button
                                class="flex items-center gap-2 px-4 py-2 text-sm text-dt-text-sub hover:text-gm-accent-cyan transition-colors"
                                on:click=move |_| set_current_page.set(AppPage::XpHistory)
                            >
                                <span>"📜"</span>
                                <span>"XP獲得履歴を見る"</span>
                                <span class="text-xs">"→"</span>
                            </button>
                        </div>

                        // Stats and Challenges Grid
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            // Stats Display
                            <StatsDisplay
                                github_stats=github_stats
                                user_stats=user_stats
                                stats_diff=stats_diff
                            />

                            // Challenges
                            <ChallengeCard />
                        </div>

                        // Badges Section
                        <BadgeGrid />

                        // Contribution Graph
                        <ContributionGraph
                            github_stats=github_stats
                        />
                    </Show>
                </Show>
            </div>
        </div>
    }
}

/// Load user data from API
async fn load_user_data(
    set_github_stats: WriteSignal<Option<GitHubStats>>,
    set_level_info: WriteSignal<Option<LevelInfo>>,
    set_user_stats: WriteSignal<Option<UserStats>>,
    set_badges: WriteSignal<Vec<Badge>>,
    set_error: WriteSignal<Option<String>>,
    set_data_from_cache: WriteSignal<bool>,
    set_cache_timestamp: WriteSignal<Option<String>>,
) {
    // Load GitHub stats with cache fallback
    match tauri_api::get_github_stats_with_cache().await {
        Ok(response) => {
            set_github_stats.set(Some(response.data));
            if response.from_cache {
                set_data_from_cache.set(true);
                set_cache_timestamp.set(response.cached_at);
                web_sys::console::log_1(&"GitHub stats loaded from cache (offline mode)".into());
            } else {
                // Fresh data - clear cache indicator
                set_data_from_cache.set(false);
                set_cache_timestamp.set(None);
            }
        }
        Err(e) => {
            web_sys::console::error_1(&format!("Failed to get GitHub stats: {}", e).into());
            set_error.set(Some(format!("Failed to load GitHub stats: {}", e)));
        }
    }

    // Load level info
    if let Ok(info) = tauri_api::get_level_info().await {
        set_level_info.set(info);
    }

    // Load user stats
    if let Ok(stats) = tauri_api::get_user_stats().await {
        set_user_stats.set(stats);
    }

    // Load badges
    if let Ok(b) = tauri_api::get_badges().await {
        set_badges.set(b);
    }
}
