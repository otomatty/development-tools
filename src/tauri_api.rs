use wasm_bindgen::prelude::*;
use std::collections::HashMap;

use crate::types::{
    AuthState, Badge, BadgeDefinition, GitHubStats, GitHubUser, 
    LevelInfo, ToolConfig, ToolInfo, UserStats, XpHistoryEntry,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// ツール一覧を取得
pub async fn list_tools() -> Result<Vec<ToolInfo>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("list_tools", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse tools list: {:?}", e))
}

/// ツールの詳細設定を取得
pub async fn get_tool_config(tool_name: &str) -> Result<ToolConfig, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        tool_name: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { tool_name }).unwrap();
    let result = invoke("get_tool_config", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse tool config: {:?}", e))
}

/// ツールを実行
pub async fn run_tool(tool_name: &str, options: &HashMap<String, serde_json::Value>) -> Result<(), String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        tool_name: &'a str,
        options: &'a HashMap<String, serde_json::Value>,
    }

    let args = serde_wasm_bindgen::to_value(&Args { tool_name, options }).unwrap();
    let result = invoke("run_tool", args).await;
    
    // Check if result is an error
    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result.clone()) {
        Err(err)
    } else {
        Ok(())
    }
}

/// Tauriイベントリスナーをセットアップ
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

/// イベントリスナーの解除用
pub struct UnlistenFn {
    _unlisten: JsValue,
}

impl Drop for UnlistenFn {
    fn drop(&mut self) {
        // 解除処理（必要に応じて）
    }
}

/// ログイベントをリッスン
pub async fn listen_log_events<F>(mut callback: F) -> Result<UnlistenFn, String>
where
    F: FnMut(crate::types::LogEvent) + 'static,
{
    let closure = Closure::new(move |event: JsValue| {
        if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
            if let Ok(log_event) = serde_wasm_bindgen::from_value(payload) {
                callback(log_event);
            }
        }
    });

    let unlisten = listen("tool-log", &closure).await;
    closure.forget(); // リークさせてコールバックを維持

    Ok(UnlistenFn { _unlisten: unlisten })
}

/// ステータスイベントをリッスン
pub async fn listen_status_events<F>(mut callback: F) -> Result<UnlistenFn, String>
where
    F: FnMut(crate::types::ToolStatusEvent) + 'static,
{
    let closure = Closure::new(move |event: JsValue| {
        if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
            if let Ok(status_event) = serde_wasm_bindgen::from_value(payload) {
                callback(status_event);
            }
        }
    });

    let unlisten = listen("tool-status", &closure).await;
    closure.forget();

    Ok(UnlistenFn { _unlisten: unlisten })
}

// ============================================
// 認証関連API
// ============================================

/// 認証状態を取得
pub async fn get_auth_state() -> Result<AuthState, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_auth_state", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get auth state: {:?}", e))
}

/// OAuthログイン開始（認証URLを返す）
pub async fn start_oauth_login() -> Result<String, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("start_oauth_login", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to start OAuth login: {:?}", e))
}

/// OAuthコールバック処理
pub async fn handle_oauth_callback(code: &str, state: &str) -> Result<AuthState, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        code: &'a str,
        callback_state: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { code, callback_state: state }).unwrap();
    let result = invoke("handle_oauth_callback", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to handle OAuth callback: {:?}", e))
}

/// ログアウト
pub async fn logout() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("logout", args).await;
    
    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        Ok(())
    }
}

// ============================================
// GitHub関連API
// ============================================

/// GitHubユーザー情報を取得
pub async fn get_github_user() -> Result<GitHubUser, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_github_user", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get GitHub user: {:?}", e))
}

/// GitHub統計を取得
pub async fn get_github_stats() -> Result<GitHubStats, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_github_stats", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get GitHub stats: {:?}", e))
}

/// コントリビューションカレンダーを取得
pub async fn get_contribution_calendar() -> Result<serde_json::Value, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_contribution_calendar", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get contribution calendar: {:?}", e))
}

// ============================================
// ゲーミフィケーション関連API
// ============================================

/// ユーザー統計を取得
pub async fn get_user_stats() -> Result<Option<UserStats>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_user_stats", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get user stats: {:?}", e))
}

/// レベル情報を取得
pub async fn get_level_info() -> Result<Option<LevelInfo>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_level_info", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get level info: {:?}", e))
}

/// バッジ一覧を取得
pub async fn get_badges() -> Result<Vec<Badge>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_badges", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get badges: {:?}", e))
}

/// バッジ定義一覧を取得
pub async fn get_badge_definitions() -> Result<Vec<BadgeDefinition>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_badge_definitions", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get badge definitions: {:?}", e))
}

/// XP履歴を取得
pub async fn get_xp_history(limit: Option<i32>) -> Result<Vec<XpHistoryEntry>, String> {
    #[derive(serde::Serialize)]
    struct Args {
        limit: Option<i32>,
    }

    let args = serde_wasm_bindgen::to_value(&Args { limit }).unwrap();
    let result = invoke("get_xp_history", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get XP history: {:?}", e))
}

/// GitHub統計を同期
pub async fn sync_github_stats() -> Result<UserStats, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("sync_github_stats", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to sync GitHub stats: {:?}", e))
}

