mod auth;
mod commands;
mod database;
mod github;
mod types;

use tauri::{Listener, Manager};

use commands::{
    // Tool commands
    get_tool_config, list_tools, run_tool,
    // Auth commands
    get_auth_state, get_current_user, handle_oauth_callback, logout, start_oauth_login,
    // GitHub commands
    get_contribution_calendar, get_github_stats, get_github_user, get_user_stats, sync_github_stats,
    // Gamification commands
    add_xp, award_badge, get_badge_definitions, get_badges, get_level_info, get_xp_history,
    // State
    AppState,
};

use auth::OAuthConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Initialize app state
            let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            
            let app_state = runtime.block_on(async {
                let mut state = AppState::new().await.expect("Failed to initialize app state");
                
                // Load OAuth config from environment if available
                if let (Ok(client_id), Ok(client_secret)) = (
                    std::env::var("GITHUB_CLIENT_ID"),
                    std::env::var("GITHUB_CLIENT_SECRET"),
                ) {
                    let oauth_config = OAuthConfig::new(client_id, client_secret);
                    state = state.with_oauth_config(oauth_config);
                }
                
                state
            });
            
            app.manage(app_state);
            
            // Setup deep link handler
            #[cfg(desktop)]
            {
                let _handle = app.handle().clone();
                app.listen("deep-link://new-url", move |event| {
                    // Handle the OAuth callback URL
                    eprintln!("Received deep link event: {:?}", event.payload());
                    // The frontend will handle parsing the URL
                });
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tool commands
            list_tools,
            get_tool_config,
            run_tool,
            // Auth commands
            get_auth_state,
            start_oauth_login,
            handle_oauth_callback,
            logout,
            get_current_user,
            // GitHub commands
            get_github_user,
            get_github_stats,
            get_user_stats,
            sync_github_stats,
            get_contribution_calendar,
            // Gamification commands
            get_level_info,
            add_xp,
            get_badges,
            award_badge,
            get_xp_history,
            get_badge_definitions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
