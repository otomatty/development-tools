mod auth;
mod commands;
mod database;
mod github;
mod sync_scheduler;
mod utils;

use tauri::Manager;

use commands::auth::run_startup_token_validation;
use commands::{
    // Gamification commands
    add_xp,
    award_badge,
    // Auth commands (Device Flow)
    cancel_device_flow,
    // Cache management commands
    cleanup_expired_cache,
    // Settings commands
    clear_cache,
    clear_user_cache,
    // Challenge commands
    create_challenge,
    // Issue management commands (Issue #59)
    create_github_issue,
    create_project,
    delete_challenge,
    delete_project,
    export_data,
    get_active_challenges,
    get_all_challenges,
    get_app_info,
    get_auth_state,
    get_badge_definitions,
    get_badges,
    // GitHub commands
    get_badges_with_progress,
    get_cache_stats,
    get_challenge_stats,
    get_challenges_by_type,
    // Code Statistics commands (Issue #74)
    get_code_stats_summary,
    get_contribution_calendar,
    get_current_user,
    get_database_info,
    get_github_stats,
    // Cache fallback commands
    get_github_stats_with_cache,
    get_github_user,
    get_kanban_board,
    get_level_info,
    // Cross-repository "Today / Inbox" command (Issue #183)
    get_my_open_work_with_cache,
    get_near_completion_badges,
    get_project,
    get_project_issues,
    get_projects,
    get_rate_limit_info,
    get_scheduler_status,
    get_settings,
    get_sync_intervals,
    get_user_repositories,
    get_user_stats,
    get_user_stats_with_cache,
    get_xp_history,
    link_repository,
    logout,
    open_external_url,
    open_url,
    poll_device_token,
    reset_all_data,
    reset_settings,
    setup_github_actions,
    start_device_flow,
    sync_code_stats,
    sync_github_stats,
    sync_project_issues,
    update_challenge_progress,
    update_issue_status,
    update_project,
    update_settings,
    validate_token,
    // State
    AppState,
};

use auth::DeviceFlowConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file from multiple possible locations (for development)
    // Try current directory first, then parent directory (for when running from src-tauri)
    if dotenvy::dotenv().is_err() {
        // Try parent directory (project root when running from src-tauri)
        let _ = dotenvy::from_filename("../.env");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Initialize app state
            let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

            let app_state = runtime.block_on(async {
                let mut state = AppState::new()
                    .await
                    .expect("Failed to initialize app state");

                // Load GitHub Client ID from environment for Device Flow
                if let Ok(client_id) = std::env::var("GITHUB_CLIENT_ID") {
                    eprintln!(
                        "GitHub Client ID loaded: {}...",
                        &client_id[..8.min(client_id.len())]
                    );
                    let device_flow_config = DeviceFlowConfig::new(client_id);
                    state = state.with_device_flow_config(device_flow_config);
                } else {
                    eprintln!("Warning: GITHUB_CLIENT_ID not set. GitHub login will not work.");
                }

                state
            });

            app.manage(app_state.db.clone());

            // Clean up expired cache on startup
            let db_for_cleanup = app_state.db.clone();
            tauri::async_runtime::spawn(async move {
                match db_for_cleanup.clear_expired_cache().await {
                    Ok(deleted) if deleted > 0 => {
                        // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
                        eprintln!("Startup: Cleaned up {} expired cache entries", deleted);
                    }
                    Ok(_) => {
                        // No expired cache entries to clean up (silent)
                    }
                    Err(e) => {
                        // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
                        eprintln!("Startup: Failed to clean up expired cache: {}", e);
                    }
                }
            });

            app.manage(app_state);

            // Probe the persisted GitHub token in the background so we can
            // surface a re-login prompt if the user revoked it externally
            // between launches. Runs after `app.manage(app_state)` because the
            // helper resolves AppState via the app handle.
            //
            // Issue #181 — see `commands::auth::run_startup_token_validation`
            // for the policy: only a confirmed 401 clears the session;
            // transport errors leave it intact.
            let app_for_auth_check = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_for_auth_check.state::<AppState>();
                run_startup_token_validation(app_for_auth_check.clone(), state.inner()).await;
            });

            // Start the background sync scheduler. Must run *after* AppState is
            // managed because the scheduler resolves it via app.state::<AppState>().
            let scheduler_handle = sync_scheduler::start_scheduler(app.handle().clone());
            app.manage(scheduler_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth commands (Device Flow)
            get_auth_state,
            logout,
            get_current_user,
            validate_token,
            start_device_flow,
            poll_device_token,
            cancel_device_flow,
            open_url,
            // GitHub commands
            get_github_user,
            get_github_stats,
            get_user_stats,
            sync_github_stats,
            get_contribution_calendar,
            get_badges_with_progress,
            get_near_completion_badges,
            // Cache fallback commands
            get_github_stats_with_cache,
            get_user_stats_with_cache,
            // Cache management commands
            get_cache_stats,
            clear_user_cache,
            cleanup_expired_cache,
            // Code Statistics commands (Issue #74)
            sync_code_stats,
            get_code_stats_summary,
            get_rate_limit_info,
            // Gamification commands
            get_level_info,
            add_xp,
            get_badges,
            award_badge,
            get_xp_history,
            get_badge_definitions,
            // Challenge commands
            get_active_challenges,
            get_all_challenges,
            get_challenges_by_type,
            create_challenge,
            delete_challenge,
            update_challenge_progress,
            get_challenge_stats,
            // Settings commands
            get_settings,
            update_settings,
            reset_settings,
            clear_cache,
            get_database_info,
            reset_all_data,
            export_data,
            get_sync_intervals,
            get_app_info,
            open_external_url,
            // Sync scheduler commands
            get_scheduler_status,
            // Issue management commands (Issue #59)
            get_projects,
            get_project,
            create_project,
            update_project,
            delete_project,
            get_user_repositories,
            link_repository,
            setup_github_actions,
            sync_project_issues,
            get_project_issues,
            get_kanban_board,
            update_issue_status,
            create_github_issue,
            get_my_open_work_with_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
