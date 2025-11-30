//! Network status context and provider
//!
//! Provides network connectivity status (online/offline) detection
//! and context for the entire application.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/components/mod.rs
//!   - src/app.rs (NetworkStatusProvider)
//! Dependencies (Files this module imports):
//!   - src/types/network.rs (NetworkState)
//! Related Documentation:
//!   - Spec: ./network_status.spec.md
//!   - Issue: GitHub Issue #10

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::types::NetworkState;

/// ネットワーク状態コンテキスト
#[derive(Clone, Copy)]
pub struct NetworkStatusContext {
    /// 現在のネットワーク状態（読み取り専用）
    pub state: ReadSignal<NetworkState>,
    /// ネットワーク状態を更新（内部使用）
    set_state: WriteSignal<NetworkState>,
}

impl NetworkStatusContext {
    /// オンラインかどうかを取得
    pub fn is_online(&self) -> bool {
        self.state.get().is_online
    }

    /// オンライン状態のシグナルを取得
    pub fn is_online_signal(&self) -> Signal<bool> {
        let state = self.state;
        Signal::derive(move || state.get().is_online)
    }
}

/// ブラウザの navigator.onLine を取得
fn get_navigator_online() -> bool {
    web_sys::window()
        .map(|w| w.navigator().on_line())
        .unwrap_or(true) // 取得できない場合はオンラインと仮定
}

/// ネットワーク状態プロバイダー
///
/// アプリ全体でネットワーク状態を共有するためのコンテキストプロバイダー。
/// online/offline イベントを監視し、状態変化を自動検知する。
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <NetworkStatusProvider>
///         <App />
///     </NetworkStatusProvider>
/// }
/// ```
#[component]
pub fn NetworkStatusProvider(children: Children) -> impl IntoView {
    // 初期状態を navigator.onLine から取得
    let initial_online = get_navigator_online();
    let (state, set_state) = signal(NetworkState::new(initial_online));

    // イベントリスナーを設定
    Effect::new(move |_| {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        // online イベントハンドラー
        let set_state_online = set_state;
        let on_online: Closure<dyn Fn()> = Closure::new(move || {
            set_state_online.update(|s| s.set_online());
            web_sys::console::log_1(&"Network: Online".into());
        });

        // offline イベントハンドラー
        let set_state_offline = set_state;
        let on_offline: Closure<dyn Fn()> = Closure::new(move || {
            set_state_offline.update(|s| s.set_offline());
            web_sys::console::log_1(&"Network: Offline".into());
        });

        // イベントリスナーを登録
        let _ = window.add_event_listener_with_callback(
            "online",
            on_online.as_ref().unchecked_ref(),
        );
        let _ = window.add_event_listener_with_callback(
            "offline",
            on_offline.as_ref().unchecked_ref(),
        );

        // クロージャをリークして永続化（コンポーネントのライフタイム中は必要）
        on_online.forget();
        on_offline.forget();
    });

    // コンテキストを提供
    let context = NetworkStatusContext { state, set_state };
    provide_context(context);

    children()
}

/// ネットワーク状態コンテキストを取得
///
/// `NetworkStatusProvider` 内で使用する必要がある。
/// プロバイダー外で呼び出すとパニックする。
///
/// # Panics
///
/// `NetworkStatusProvider` 外で呼び出された場合
pub fn use_network_status() -> NetworkStatusContext {
    use_context::<NetworkStatusContext>()
        .expect("use_network_status must be used within NetworkStatusProvider")
}

/// ネットワーク状態コンテキストを取得（オプショナル版）
///
/// プロバイダー外で呼び出された場合は `None` を返す。
pub fn try_use_network_status() -> Option<NetworkStatusContext> {
    use_context::<NetworkStatusContext>()
}

/// オンライン状態のシグナルを取得するショートカット
///
/// コンテキストが利用可能な場合は実際の状態を、
/// 利用できない場合は常に `true` を返す。
pub fn use_is_online() -> Signal<bool> {
    match try_use_network_status() {
        Some(ctx) => ctx.is_online_signal(),
        None => Signal::derive(|| true), // フォールバック: 常にオンライン
    }
}

/// オフラインバナーコンポーネント
///
/// オフライン時に表示する警告バナー。
/// 最終オンライン時刻も表示する。
#[component]
pub fn OfflineBanner() -> impl IntoView {
    let network_ctx = try_use_network_status();
    
    view! {
        {move || {
            let ctx = network_ctx?;
            let state = ctx.state.get();
            
            if state.is_online {
                return None;
            }
            
            let last_online = state.last_online_at
                .as_ref()
                .map(|t| format!("最終オンライン: {}", format_timestamp(t)))
                .unwrap_or_default();
            
            Some(view! {
                <div class="bg-amber-500/90 text-amber-950 px-4 py-2 text-sm flex items-center justify-center gap-2">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                    <span>"⚠️ オフラインモード - キャッシュデータを表示中"</span>
                    <span class="text-amber-800 text-xs">{last_online}</span>
                </div>
            })
        }}
    }
}

/// ISO 8601 タイムスタンプを人間が読みやすい形式に変換
fn format_timestamp(iso_string: &str) -> String {
    // 簡易的な実装：ISO文字列から日時部分を抽出
    // 例: "2025-11-30T12:34:56.789Z" -> "12:34"
    iso_string
        .split('T')
        .nth(1)
        .and_then(|time_part| time_part.get(0..5))
        .map(|s| s.to_string())
        .unwrap_or_else(|| iso_string.to_string())
}
