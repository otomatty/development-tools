use wasm_bindgen::prelude::*;
use std::collections::HashMap;

use crate::types::{ToolConfig, ToolInfo};

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

