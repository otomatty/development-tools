//! Home Page Component
//!
//! The main home page that displays user profile, stats, badges, and contribution graph.
//! This page component is responsible for layout and orchestrating feature components.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this page):
//!   ├─ src/components/pages/mod.rs
//!   └─ src/app.rs
//! Dependencies (Feature components used):
//!   ├─ features/auth/login_card.rs - LoginCard
//!   ├─ features/gamification/cache_indicator.rs - CacheIndicator
//!   ├─ features/gamification/dashboard_content.rs - DashboardContent
//!   ├─ features/gamification/sync_notifications.rs - handle_sync_result_notifications
//!   ├─ features/gamification/home_data_loader.rs - load_user_data
//!   ├─ features/gamification/xp_notification.rs - XpNotification, LevelUpModal, MultipleBadgesNotification
//!   └─ home/skeleton.rs - HomeSkeleton
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::components::features::auth::login_card::LoginState;
use crate::components::features::auth::LoginCard;
use crate::components::features::gamification::xp_notification::{
    LevelUpModal, MultipleBadgesNotification,
};
use crate::components::features::gamification::{
    handle_sync_result_notifications, load_user_data, CacheIndicator, DashboardContent,
    XpNotification,
};
use crate::components::home::HomeSkeleton;
use crate::components::network_status::use_is_online;
use crate::tauri_api;
use crate::types::{
    AppPage, AuthState, Badge, BadgeDefinition, DeviceTokenStatus, GitHubStats, LevelInfo,
    NewBadgeInfo, StatsDiffResult, UserSettings, UserStats, XpGainedEvent,
};

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
    let component_mounted = Arc::new(AtomicBool::new(true));
    let component_mounted_for_auto_sync = component_mounted.clone();
    let component_mounted_for_polling = component_mounted.clone();

    // Cleanup when component unmounts
    on_cleanup({
        let component_mounted = component_mounted.clone();
        let set_polling_active = set_polling_active.clone();
        move || {
            component_mounted.store(false, Ordering::SeqCst);
            set_polling_active.set(false);
            web_sys::console::log_1(&"HomePage: Component unmounted, cleanup triggered".into());
        }
    });

    // Load initial data
    spawn_local(async move {
        match tauri_api::get_auth_state().await {
            Ok(state) => {
                set_auth_state.set(state.clone());

                if state.is_logged_in {
                    match tauri_api::get_settings().await {
                        Ok(settings) => {
                            let should_sync_on_startup = settings.sync_on_startup;
                            set_notification_settings.set(Some(settings));

                            if should_sync_on_startup {
                                web_sys::console::log_1(
                                    &"Startup sync: Syncing GitHub stats...".into(),
                                );
                                if let Ok(sync_result) = tauri_api::sync_github_stats().await {
                                    set_user_stats.set(Some(sync_result.user_stats.clone()));
                                    set_stats_diff.set(sync_result.stats_diff.clone());
                                    handle_sync_result_notifications(
                                        &sync_result,
                                        notification_settings,
                                        set_xp_event,
                                        set_level_up_event,
                                        set_new_badges_event,
                                    );
                                }
                            }
                        }
                        Err(e) => {
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
                set_error.set(Some(e));
            }
        }

        if let Ok(defs) = tauri_api::get_badge_definitions().await {
            set_badge_definitions.set(defs);
        }

        set_loading.set(false);
    });

    // Online recovery sync
    {
        let is_online = use_is_online();
        Effect::new(move |prev_online: Option<bool>| {
            let current_online = is_online.get();
            if let Some(was_online) = prev_online {
                if !was_online && current_online && auth_state.get_untracked().is_logged_in {
                    spawn_local(async move {
                        if let Ok(sync_result) = tauri_api::sync_github_stats().await {
                            set_user_stats.set(Some(sync_result.user_stats.clone()));
                            set_stats_diff.set(sync_result.stats_diff.clone());
                            set_data_from_cache.set(false);
                            set_cache_timestamp.set(None);
                            handle_sync_result_notifications(
                                &sync_result,
                                notification_settings,
                                set_xp_event,
                                set_level_up_event,
                                set_new_badges_event,
                            );
                        }
                    });
                }
            }
            current_online
        });
    }

    // Setup auto-sync timer
    {
        let auth_state = auth_state.clone();
        let component_mounted = component_mounted_for_auto_sync;

        spawn_local(async move {
            // Wait for settings to load
            let mut waited = 0;
            while notification_settings.get_untracked().is_none() && waited < 5000 {
                if !component_mounted.load(Ordering::SeqCst) {
                    return;
                }
                if let Some(window) = web_sys::window() {
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let _ = window
                            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 200);
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    waited += 200;
                } else {
                    break;
                }
            }

            loop {
                if !component_mounted.load(Ordering::SeqCst) {
                    break;
                }

                let sync_interval_ms = notification_settings
                    .get_untracked()
                    .map(|s| {
                        if s.sync_interval_minutes == 0 {
                            0
                        } else {
                            s.sync_interval_minutes
                                .saturating_mul(60)
                                .saturating_mul(1000)
                        }
                    })
                    .unwrap_or(60 * 60 * 1000);

                if sync_interval_ms == 0 {
                    if let Some(window) = web_sys::window() {
                        let promise = js_sys::Promise::new(&mut |resolve, _| {
                            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                &resolve, 30000,
                            );
                        });
                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    }
                    continue;
                }

                if let Some(window) = web_sys::window() {
                    let auth = auth_state.get_untracked();
                    let background_sync_enabled = notification_settings
                        .get_untracked()
                        .map(|s| s.background_sync)
                        .unwrap_or(true);

                    if auth.is_logged_in && background_sync_enabled {
                        if let Ok(sync_result) = tauri_api::sync_github_stats().await {
                            set_user_stats.set(Some(sync_result.user_stats.clone()));
                            set_stats_diff.set(sync_result.stats_diff.clone());
                            handle_sync_result_notifications(
                                &sync_result,
                                notification_settings,
                                set_xp_event,
                                set_level_up_event,
                                set_new_badges_event,
                            );
                            if let Ok(info) = tauri_api::get_level_info().await {
                                set_level_info.set(info);
                            }
                            if let Ok(b) = tauri_api::get_badges().await {
                                set_badges.set(b);
                            }
                        }
                    }

                    if !component_mounted.load(Ordering::SeqCst) {
                        break;
                    }

                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            sync_interval_ms,
                        );
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                } else {
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

    // Handle opening verification URL and start polling
    let on_open_url = Callback::new(move |url: String| {
        let url_clone = url.clone();
        let component_mounted = component_mounted_for_polling.clone();

        spawn_local(async move {
            let _ = tauri_api::open_url(&url_clone).await;
        });

        set_login_state.set(LoginState::Polling);
        set_polling_active.set(true);

        spawn_local(async move {
            const POLL_INTERVAL_MS: i32 = 5000;
            loop {
                if !component_mounted.load(Ordering::SeqCst) || !polling_active.get_untracked() {
                    break;
                }

                match tauri_api::poll_device_token().await {
                    Ok(status) => match status {
                        DeviceTokenStatus::Pending => {}
                        DeviceTokenStatus::Success {
                            auth_state: new_auth_state,
                        } => {
                            set_auth_state.set(new_auth_state);
                            set_login_state.set(LoginState::Initial);
                            set_polling_active.set(false);
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
                            set_login_state.set(LoginState::Error(message));
                            set_polling_active.set(false);
                            break;
                        }
                    },
                    Err(e) => {
                        set_login_state.set(LoginState::Error(format!("Polling failed: {}", e)));
                        set_polling_active.set(false);
                        break;
                    }
                }

                if !component_mounted.load(Ordering::SeqCst) || !polling_active.get_untracked() {
                    break;
                }

                if let Some(window) = web_sys::window() {
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            POLL_INTERVAL_MS,
                        );
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                } else {
                    break;
                }
            }
        });
    });

    // Handle cancel login
    let on_cancel_login = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_polling_active.set(false);
        set_login_state.set(LoginState::Initial);
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
        // Notifications and modals
        <XpNotification event=xp_event on_close=on_close_xp />
        <LevelUpModal event=level_up_event on_close=on_close_level_up />
        <MultipleBadgesNotification badges=new_badges_event on_close=on_close_badges />

        <div class="flex-1 overflow-y-auto bg-gradient-to-br from-gm-bg-primary via-gm-bg-secondary to-gm-bg-primary min-h-full">
            <div class="max-w-6xl mx-auto p-6 space-y-6">
                // Error display
                <Show when=move || error.get().is_some()>
                    <div class="p-4 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                // Cache indicator
                <CacheIndicator data_from_cache=data_from_cache cache_timestamp=cache_timestamp />

                // Loading state
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
                        <DashboardContent
                            auth_state=auth_state
                            level_info=level_info
                            user_stats=user_stats
                            github_stats=github_stats
                            stats_diff=stats_diff
                            on_logout=on_logout
                            set_current_page=set_current_page
                        />
                    </Show>
                </Show>
            </div>
        </div>
    }
}
