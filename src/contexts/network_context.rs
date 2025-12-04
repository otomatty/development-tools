//! Network status context and provider
//!
//! ネットワーク接続状態（オンライン/オフライン）を検出し、
//! アプリケーション全体で共有するためのコンテキスト。
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/contexts/mod.rs
//!   - src/app.rs
//! Dependencies:
//!   - src/types/network.rs (NetworkState)
//! Related:
//!   - Hooks: src/hooks/use_network_status.rs
//!   - UI: src/components/ui/feedback/offline_banner.rs
//!   - Spec: src/components/network_status.spec.md

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
        let _ =
            window.add_event_listener_with_callback("online", on_online.as_ref().unchecked_ref());
        let _ =
            window.add_event_listener_with_callback("offline", on_offline.as_ref().unchecked_ref());

        // TODO: [BUG] イベントリスナーのメモリリーク（アプリルートでのみ使用のため影響軽微）
        // on_cleanupでremove_event_listenerを呼ぶべきだが、Closureのライフタイム管理が複雑
        // クロージャをリークして永続化（コンポーネントのライフタイム中は必要）
        on_online.forget();
        on_offline.forget();
    });

    // コンテキストを提供
    let context = NetworkStatusContext { state, set_state };
    provide_context(context);

    children()
}
