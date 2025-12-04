//! Network status hooks
//!
//! ネットワーク状態コンテキストを使用するためのカスタムフック。
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/hooks/mod.rs
//!   - src/components/ (various components)
//! Dependencies:
//!   - src/contexts/network_context.rs

use leptos::prelude::*;

use crate::contexts::NetworkStatusContext;

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
