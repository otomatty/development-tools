mod auth;
mod commands;
mod database;
mod github;
mod mock_server;
mod types;
mod utils;

use tauri::Manager;

use commands::{
    // Tool commands
    get_tool_config, list_tools, run_tool, select_path,
    // Auth commands (Device Flow)
    cancel_device_flow, get_auth_state, get_current_user, logout, open_url, poll_device_token,
    start_device_flow, validate_token,
    // GitHub commands
    get_badges_with_progress, get_contribution_calendar, get_github_stats, get_github_user,
    get_near_completion_badges, get_user_stats, sync_github_stats,
    // Cache fallback commands
    get_github_stats_with_cache, get_user_stats_with_cache,
    // Cache management commands
    cleanup_expired_cache, clear_user_cache, get_cache_stats,
    // Code Statistics commands (Issue #74)
    get_code_stats_summary, get_rate_limit_info, sync_code_stats,
    // Gamification commands
    add_xp, award_badge, get_badge_definitions, get_badges, get_level_info, get_xp_history,
    // Challenge commands
    create_challenge, delete_challenge, get_active_challenges, get_all_challenges,
    get_challenge_stats, get_challenges_by_type, update_challenge_progress,
    // Settings commands
    clear_cache, export_data, get_app_info, get_database_info, get_settings, get_sync_intervals,
    open_external_url, reset_all_data, reset_settings, update_settings,
    // Mock Server commands
    create_mock_server_mapping, delete_mock_server_mapping, get_mock_server_config,
    get_mock_server_mappings, get_mock_server_state, list_mock_server_directory,
    select_mock_server_directory, start_mock_server, stop_mock_server,
    update_mock_server_config, update_mock_server_mapping, MockServerManager,
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
            let runtime =
                tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

            let app_state = runtime.block_on(async {
                let mut state = AppState::new()
                    .await
                    .expect("Failed to initialize app state");

                // Load GitHub Client ID from environment for Device Flow
                if let Ok(client_id) = std::env::var("GITHUB_CLIENT_ID") {
                    eprintln!("GitHub Client ID loaded: {}...", &client_id[..8.min(client_id.len())]);
                    let device_flow_config = DeviceFlowConfig::new(client_id);
                    state = state.with_device_flow_config(device_flow_config);
                } else {
                    eprintln!("Warning: GITHUB_CLIENT_ID not set. GitHub login will not work.");
                }

                state
            });

            // Register Database as managed state (needed by mock_server commands)
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
            
            // Initialize Mock Server manager
            app.manage(MockServerManager::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tool commands
            list_tools,
            get_tool_config,
            run_tool,
            select_path,
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
            // Mock Server commands
            get_mock_server_state,
            start_mock_server,
            stop_mock_server,
            get_mock_server_config,
            update_mock_server_config,
            get_mock_server_mappings,
            create_mock_server_mapping,
            update_mock_server_mapping,
            delete_mock_server_mapping,
            list_mock_server_directory,
            select_mock_server_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
