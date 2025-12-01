use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::types::{
    AppInfo, AuthState, Badge, BadgeDefinition, BadgeWithProgress, ClearCacheResult, DatabaseInfo,
    DeviceCodeResponse, DeviceTokenStatus, GitHubStats, GitHubUser, LevelInfo, SyncIntervalOption,
    SyncResult, ToolConfig, ToolInfo, UpdateSettingsRequest, UserSettings, UserStats,
    XpGainedEvent, XpHistoryEntry,
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
pub async fn run_tool(
    tool_name: &str,
    options: &HashMap<String, serde_json::Value>,
) -> Result<(), String> {
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

/// ファイルまたはディレクトリを選択するダイアログを表示
///
/// # Arguments
/// * `path_type` - 選択するパスの種類 ("file", "directory", "any")
/// * `title` - ダイアログのタイトル (オプション)
/// * `default_path` - デフォルトのパス (オプション)
pub async fn select_path(
    path_type: &str,
    title: Option<&str>,
    default_path: Option<&str>,
) -> Result<Option<String>, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        path_type: &'a str,
        title: Option<&'a str>,
        default_path: Option<&'a str>,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        path_type,
        title,
        default_path,
    })
    .unwrap();
    let result = invoke("select_path", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to select path: {:?}", e))
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

    Ok(UnlistenFn {
        _unlisten: unlisten,
    })
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

    Ok(UnlistenFn {
        _unlisten: unlisten,
    })
}

// ============================================
// 認証関連API（Device Flow）
// ============================================

/// 認証状態を取得
pub async fn get_auth_state() -> Result<AuthState, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_auth_state", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get auth state: {:?}", e))
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

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to validate token: {:?}", e))
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

/// GitHub統計を取得（キャッシュフォールバック付き）
///
/// オフライン時はキャッシュされたデータを返します。
pub async fn get_github_stats_with_cache(
) -> Result<crate::types::CachedResponse<GitHubStats>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_github_stats_with_cache", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get GitHub stats with cache: {:?}", e))
}

// ============================================
// キャッシュ管理API
// ============================================

/// キャッシュ統計を取得
pub async fn get_cache_stats() -> Result<crate::types::CacheStats, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_cache_stats", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get cache stats: {:?}", e))
}

/// ユーザーのすべてのキャッシュをクリア
pub async fn clear_user_cache() -> Result<u64, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("clear_user_cache", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to clear user cache: {:?}", e))
}

/// 期限切れキャッシュのクリーンアップ
pub async fn cleanup_expired_cache() -> Result<u64, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("cleanup_expired_cache", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to cleanup expired cache: {:?}", e))
}

/// コントリビューションカレンダーを取得
pub async fn get_contribution_calendar() -> Result<serde_json::Value, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_contribution_calendar", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get contribution calendar: {:?}", e))
}

// ============================================
// コード統計関連API
// ============================================

/// コード統計を同期（GitHubから取得してDBに保存）
pub async fn sync_code_stats() -> Result<crate::types::CodeStatsSyncResult, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("sync_code_stats", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to sync code stats: {:?}", e))
}

/// コード統計サマリーを取得
pub async fn get_code_stats_summary(
    period: &str,
) -> Result<crate::types::CodeStatsResponse, String> {
    #[derive(serde::Serialize)]
    struct Args<'a> {
        period: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { period }).unwrap();
    let result = invoke("get_code_stats_summary", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get code stats summary: {:?}", e))
}

/// レート制限情報を取得
pub async fn get_rate_limit_info() -> Result<crate::types::RateLimitInfo, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_rate_limit_info", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get rate limit info: {:?}", e))
}

// ============================================
// ゲーミフィケーション関連API
// ============================================

/// ユーザー統計を取得
pub async fn get_user_stats() -> Result<Option<UserStats>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_user_stats", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get user stats: {:?}", e))
}

/// ユーザー統計を取得（キャッシュフォールバック付き）
///
/// データベースエラー時はキャッシュされたデータを返します。
pub async fn get_user_stats_with_cache() -> Result<crate::types::CachedResponse<UserStats>, String>
{
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_user_stats_with_cache", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get user stats with cache: {:?}", e))
}

/// レベル情報を取得
pub async fn get_level_info() -> Result<Option<LevelInfo>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_level_info", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get level info: {:?}", e))
}

/// バッジ一覧を取得
pub async fn get_badges() -> Result<Vec<Badge>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_badges", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get badges: {:?}", e))
}

/// バッジ定義一覧を取得
pub async fn get_badge_definitions() -> Result<Vec<BadgeDefinition>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_badge_definitions", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get badge definitions: {:?}", e))
}

/// 進捗情報付きバッジ一覧を取得
pub async fn get_badges_with_progress() -> Result<Vec<BadgeWithProgress>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_badges_with_progress", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get badges with progress: {:?}", e))
}

/// 取得間近のバッジを取得
pub async fn get_near_completion_badges(
    threshold_percent: Option<f32>,
) -> Result<Vec<BadgeWithProgress>, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        threshold_percent: Option<f32>,
    }

    let args = serde_wasm_bindgen::to_value(&Args { threshold_percent }).unwrap();
    let result = invoke("get_near_completion_badges", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get near completion badges: {:?}", e))
}

/// XP履歴を取得
pub async fn get_xp_history(limit: Option<i32>) -> Result<Vec<XpHistoryEntry>, String> {
    #[derive(serde::Serialize)]
    struct Args {
        limit: Option<i32>,
    }

    let args = serde_wasm_bindgen::to_value(&Args { limit }).unwrap();
    let result = invoke("get_xp_history", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get XP history: {:?}", e))
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

    Ok(UnlistenFn {
        _unlisten: unlisten,
    })
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

    Ok(UnlistenFn {
        _unlisten: unlisten,
    })
}

// ============================================
// 設定関連API
// ============================================

/// ユーザー設定を取得
pub async fn get_settings() -> Result<UserSettings, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_settings", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get settings: {:?}", e))
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
    #[derive(serde::Serialize)]
    struct Args<'a> {
        settings: &'a UpdateSettingsRequest,
    }

    let args = serde_wasm_bindgen::to_value(&Args { settings }).unwrap();
    let result = invoke("update_settings", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update settings: {:?}", e))
}

/// 設定をリセット
pub async fn reset_settings() -> Result<UserSettings, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("reset_settings", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to reset settings: {:?}", e))
}

/// キャッシュをクリア
pub async fn clear_cache() -> Result<ClearCacheResult, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("clear_cache", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to clear cache: {:?}", e))
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

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to export data: {:?}", e))
}

/// アプリケーション情報を取得
pub async fn get_app_info() -> Result<AppInfo, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_app_info", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get app info: {:?}", e))
}

/// 外部URLをブラウザで開く
pub async fn open_external_url(url: &str) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Args<'a> {
        url: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { url }).unwrap();
    let result = invoke("open_external_url", args).await;

    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        // Unexpected return value should be treated as an error
        Err("Unexpected return value from Tauri command".to_string())
    }
}

// ============================================
// チャレンジ関連のAPI
// ============================================

use crate::types::{ChallengeInfo, ChallengeStats, CreateChallengeRequest};

/// アクティブなチャレンジを取得
pub async fn get_active_challenges() -> Result<Vec<ChallengeInfo>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_active_challenges", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get active challenges: {:?}", e))
}

/// 全チャレンジを取得（完了・失敗を含む）
pub async fn get_all_challenges() -> Result<Vec<ChallengeInfo>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_all_challenges", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get all challenges: {:?}", e))
}

/// タイプ別チャレンジを取得
pub async fn get_challenges_by_type(challenge_type: &str) -> Result<Vec<ChallengeInfo>, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        challenge_type: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { challenge_type }).unwrap();
    let result = invoke("get_challenges_by_type", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get challenges by type: {:?}", e))
}

/// チャレンジを作成
pub async fn create_challenge(request: &CreateChallengeRequest) -> Result<ChallengeInfo, String> {
    #[derive(serde::Serialize)]
    struct Args<'a> {
        request: &'a CreateChallengeRequest,
    }

    let args = serde_wasm_bindgen::to_value(&Args { request }).unwrap();
    let result = invoke("create_challenge", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to create challenge: {:?}", e))
}

/// チャレンジを削除
pub async fn delete_challenge(challenge_id: i64) -> Result<(), String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        challenge_id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { challenge_id }).unwrap();
    let result = invoke("delete_challenge", args).await;

    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        Ok(())
    }
}

/// チャレンジの進捗を更新
pub async fn update_challenge_progress(
    challenge_id: i64,
    current_value: i32,
) -> Result<ChallengeInfo, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        challenge_id: i64,
        current_value: i32,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        challenge_id,
        current_value,
    })
    .unwrap();
    let result = invoke("update_challenge_progress", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update challenge progress: {:?}", e))
}

/// チャレンジ統計を取得
pub async fn get_challenge_stats() -> Result<ChallengeStats, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_challenge_stats", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get challenge stats: {:?}", e))
}

// ============================================
// Mock Server API
// ============================================

use crate::types::{
    AccessLogEntry, CreateMappingRequest, DirectoryMapping, FileInfo, MockServerConfig,
    MockServerState, UpdateConfigRequest, UpdateMappingRequest,
};

/// Get Mock Server state
pub async fn get_mock_server_state() -> Result<MockServerState, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_mock_server_state", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get mock server state: {:?}", e))
}

/// Start Mock Server
pub async fn start_mock_server() -> Result<MockServerState, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("start_mock_server", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to start mock server: {:?}", e))
}

/// Stop Mock Server
pub async fn stop_mock_server() -> Result<MockServerState, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("stop_mock_server", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to stop mock server: {:?}", e))
}

/// Get Mock Server configuration
pub async fn get_mock_server_config() -> Result<MockServerConfig, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_mock_server_config", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get mock server config: {:?}", e))
}

/// Update Mock Server configuration
pub async fn update_mock_server_config(
    request: UpdateConfigRequest,
) -> Result<MockServerConfig, String> {
    let args = serde_wasm_bindgen::to_value(&request).unwrap();
    let result = invoke("update_mock_server_config", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update mock server config: {:?}", e))
}

/// Get all directory mappings
pub async fn get_mock_server_mappings() -> Result<Vec<DirectoryMapping>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_mock_server_mappings", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get mock server mappings: {:?}", e))
}

/// Create a new directory mapping
pub async fn create_mock_server_mapping(
    request: CreateMappingRequest,
) -> Result<DirectoryMapping, String> {
    let args = serde_wasm_bindgen::to_value(&request).unwrap();
    let result = invoke("create_mock_server_mapping", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to create mock server mapping: {:?}", e))
}

/// Update a directory mapping
pub async fn update_mock_server_mapping(
    request: UpdateMappingRequest,
) -> Result<DirectoryMapping, String> {
    let args = serde_wasm_bindgen::to_value(&request).unwrap();
    let result = invoke("update_mock_server_mapping", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update mock server mapping: {:?}", e))
}

/// Delete a directory mapping
pub async fn delete_mock_server_mapping(id: i64) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Args {
        id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { id }).unwrap();
    let result = invoke("delete_mock_server_mapping", args).await;

    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        Ok(())
    }
}

// =============================================================================
// Issue Management API
// =============================================================================

use crate::types::issue::{CachedIssue, KanbanBoard, Project, RepositoryInfo};

/// Get all projects for current user
pub async fn get_projects() -> Result<Vec<Project>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_projects", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get projects: {:?}", e))
}

/// Get a single project by ID
pub async fn get_project(project_id: i64) -> Result<Project, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        project_id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { project_id }).unwrap();
    let result = invoke("get_project", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to get project: {:?}", e))
}

/// Create a new project
pub async fn create_project(name: &str, description: Option<&str>) -> Result<Project, String> {
    #[derive(serde::Serialize)]
    struct Args<'a> {
        name: &'a str,
        description: Option<&'a str>,
    }

    let args = serde_wasm_bindgen::to_value(&Args { name, description }).unwrap();
    let result = invoke("create_project", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to create project: {:?}", e))
}

/// Update a project
pub async fn update_project(
    project_id: i64,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<Project, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        project_id: i64,
        name: Option<&'a str>,
        description: Option<&'a str>,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        project_id,
        name,
        description,
    })
    .unwrap();
    let result = invoke("update_project", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update project: {:?}", e))
}

/// Delete a project
pub async fn delete_project(project_id: i64) -> Result<(), String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        project_id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { project_id }).unwrap();
    let result = invoke("delete_project", args).await;

    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        Ok(())
    }
}

/// Get user's GitHub repositories
pub async fn get_user_repositories() -> Result<Vec<RepositoryInfo>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("get_user_repositories", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get repositories: {:?}", e))
}

/// Link a repository to a project
pub async fn link_repository(
    project_id: i64,
    owner: &str,
    repo: &str,
) -> Result<Project, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        project_id: i64,
        owner: &'a str,
        repo: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        project_id,
        owner,
        repo,
    })
    .unwrap();
    let result = invoke("link_repository", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to link repository: {:?}", e))
}

/// Setup GitHub Actions for a project
pub async fn setup_github_actions(project_id: i64) -> Result<String, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        project_id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { project_id }).unwrap();
    let result = invoke("setup_github_actions", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to setup GitHub Actions: {:?}", e))
}

/// Sync issues from GitHub for a project
pub async fn sync_project_issues(project_id: i64) -> Result<Vec<CachedIssue>, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        project_id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { project_id }).unwrap();
    let result = invoke("sync_project_issues", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to sync issues: {:?}", e))
}

/// Get cached issues for a project
pub async fn get_project_issues(project_id: i64) -> Result<Vec<CachedIssue>, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        project_id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { project_id }).unwrap();
    let result = invoke("get_project_issues", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get issues: {:?}", e))
}

/// Get kanban board for a project
pub async fn get_kanban_board(project_id: i64) -> Result<KanbanBoard, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        project_id: i64,
    }

    let args = serde_wasm_bindgen::to_value(&Args { project_id }).unwrap();
    let result = invoke("get_kanban_board", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to get kanban board: {:?}", e))
}

/// Update issue status
pub async fn update_issue_status(
    project_id: i64,
    issue_number: i32,
    new_status: &str,
) -> Result<(), String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        project_id: i64,
        issue_number: i32,
        new_status: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        project_id,
        issue_number,
        new_status,
    })
    .unwrap();
    let result = invoke("update_issue_status", args).await;

    if result.is_null() || result.is_undefined() {
        Ok(())
    } else if let Ok(err) = serde_wasm_bindgen::from_value::<String>(result) {
        Err(err)
    } else {
        Ok(())
    }
}

/// Create a new issue on GitHub
pub async fn create_github_issue(
    project_id: i64,
    title: &str,
    body: Option<&str>,
    status: Option<&str>,
    priority: Option<&str>,
) -> Result<CachedIssue, String> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        project_id: i64,
        title: &'a str,
        body: Option<&'a str>,
        status: Option<&'a str>,
        priority: Option<&'a str>,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        project_id,
        title,
        body,
        status,
        priority,
    })
    .unwrap();
    let result = invoke("create_github_issue", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to create issue: {:?}", e))
}

/// List files in a directory
pub async fn list_mock_server_directory(path: &str) -> Result<Vec<FileInfo>, String> {
    #[derive(serde::Serialize)]
    struct Args<'a> {
        path: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { path }).unwrap();
    let result = invoke("list_mock_server_directory", args).await;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to list directory: {:?}", e))
}

/// Select a directory using native dialog
pub async fn select_mock_server_directory() -> Result<Option<String>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let result = invoke("select_mock_server_directory", args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to select directory: {:?}", e))
}
