use wasm_bindgen::prelude::*;
use std::collections::HashMap;

use crate::types::{
    AuthState, Badge, BadgeDefinition, ClearCacheResult, DatabaseInfo, DeviceCodeResponse,
    DeviceTokenStatus, GitHubStats, GitHubUser, LevelInfo, SyncIntervalOption, SyncResult,
    ToolConfig, ToolInfo, UpdateSettingsRequest, UserSettings, UserStats, XpGainedEvent,
    XpHistoryEntry,
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
// 認証関連API（Device Flow）
// ============================================

/// 認証状態を取得
pub async fn get_auth_state() -> Result<AuthState, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_auth_state", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get auth state: {:?}", e))
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

/// 現在のトークンの有効性を確認
/// 
/// 注意: GitHub Device Flowのトークンは期限切れしませんが、
/// ユーザーがGitHubで手動で無効化した場合に検証できます。
pub async fn validate_token() -> Result<bool, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("validate_token", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to validate token: {:?}", e))
}

/// システムのデフォルトブラウザでURLを開く
pub async fn open_url(url: &str) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Args<'a> {
        url: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { url }).unwrap();
    let result = invoke("open_url", args).await;
    
    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        Ok(())
    }
}

/// Device Flow開始 - user_code と verification_uri を返す
/// 
/// ユーザーは以下を実行する:
/// 1. verification_uri (https://github.com/login/device) にアクセス
/// 2. 表示された user_code を入力
/// 3. アプリ側で poll_device_token() を繰り返し呼び出して認証完了を待つ
pub async fn start_device_flow() -> Result<DeviceCodeResponse, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("start_device_flow", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to start device flow: {:?}", e))
}

/// Device Flowトークンポーリング - 認証完了を待つ
/// 
/// 返り値:
/// - DeviceTokenStatus::Pending - ユーザーがまだ認証を完了していない
/// - DeviceTokenStatus::Success - 認証完了、ログイン成功
/// - DeviceTokenStatus::Error - エラー発生
pub async fn poll_device_token() -> Result<DeviceTokenStatus, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("poll_device_token", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to poll device token: {:?}", e))
}

/// Device Flowをキャンセル
pub async fn cancel_device_flow() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("cancel_device_flow", args).await;
    
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

/// GitHub統計を同期（差分XP付与付き）
pub async fn sync_github_stats() -> Result<SyncResult, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("sync_github_stats", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to sync GitHub stats: {:?}", e))
}

// ============================================
// イベントリスナー
// ============================================

/// XP獲得イベントをリッスン
pub async fn listen_xp_gained_events<F>(mut callback: F) -> Result<UnlistenFn, String>
where
    F: FnMut(XpGainedEvent) + 'static,
{
    let closure = Closure::new(move |event: JsValue| {
        if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
            if let Ok(xp_event) = serde_wasm_bindgen::from_value(payload) {
                callback(xp_event);
            }
        }
    });

    let unlisten = listen("xp-gained", &closure).await;
    closure.forget();

    Ok(UnlistenFn { _unlisten: unlisten })
}

/// レベルアップイベントをリッスン
pub async fn listen_level_up_events<F>(mut callback: F) -> Result<UnlistenFn, String>
where
    F: FnMut(XpGainedEvent) + 'static,
{
    let closure = Closure::new(move |event: JsValue| {
        if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
            if let Ok(level_event) = serde_wasm_bindgen::from_value(payload) {
                callback(level_event);
            }
        }
    });

    let unlisten = listen("level-up", &closure).await;
    closure.forget();

    Ok(UnlistenFn { _unlisten: unlisten })
}

// ============================================
// 設定関連API
// ============================================

/// ユーザー設定を取得
pub async fn get_settings() -> Result<UserSettings, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_settings", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get settings: {:?}", e))
}

/// 同期間隔の選択肢を取得
pub async fn get_sync_intervals() -> Result<Vec<SyncIntervalOption>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_sync_intervals", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get sync intervals: {:?}", e))
}

/// ユーザー設定を更新
pub async fn update_settings(settings: &UpdateSettingsRequest) -> Result<UserSettings, String> {
    let args = serde_wasm_bindgen::to_value(settings).unwrap();
    let result = invoke("update_settings", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update settings: {:?}", e))
}

/// 設定をリセット
pub async fn reset_settings() -> Result<UserSettings, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("reset_settings", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to reset settings: {:?}", e))
}

/// キャッシュをクリア
pub async fn clear_cache() -> Result<ClearCacheResult, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("clear_cache", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to clear cache: {:?}", e))
}

/// データベース情報を取得
pub async fn get_database_info() -> Result<DatabaseInfo, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_database_info", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get database info: {:?}", e))
}

/// 全データをリセット
pub async fn reset_all_data() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("reset_all_data", args).await;
    
    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        Ok(())
    }
}

/// データをエクスポート
pub async fn export_data() -> Result<String, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("export_data", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to export data: {:?}", e))
}

