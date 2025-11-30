# Network Status Component Specification

## Related Files

- Implementation: `src/components/network_status.rs`
- Types: `src/types/network.rs`
- Tests: N/A (WASM component - tested via integration)

## Related Documentation

- Issue: GitHub Issue #10 - [Phase 5] オフライン対応 & キャッシュ機能
- Parent Issue: GitHub Issue #4 - ホーム画面（GitHub 連携ゲーミフィケーション）機能実装
- PRD: `docs/prd/home-gamification.md` Section 3.7

## Requirements

### 責務

- ブラウザのネットワーク接続状態（オンライン/オフライン）を検出する
- ネットワーク状態の変化をリアルタイムで監視する
- アプリ全体でネットワーク状態を共有するコンテキストを提供する
- オフライン時のフォールバック動作を可能にする

### 状態構造

```rust
/// ネットワーク状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkState {
    /// オンラインかどうか
    pub is_online: bool,
    /// 最終確認時刻 (RFC3339形式)
    pub last_checked_at: Option<String>,
    /// 最後にオンラインになった時刻
    pub last_online_at: Option<String>,
}
```

### 機能

1. **初期状態の取得**

   - `navigator.onLine` から初期オンライン状態を取得
   - 取得できない場合はオンラインと仮定（フォールバック）

2. **状態変化の監視**

   - `online` イベントでオンライン復帰を検知
   - `offline` イベントでオフライン移行を検知
   - 状態変化時にコンテキストを更新

3. **コンテキストの提供**

   - `NetworkStatusContext` を通じてアプリ全体で状態を共有
   - `use_network_status()` フックで現在の状態を取得
   - `use_is_online()` フックでオンライン状態のみを取得（簡易版）

4. **クリーンアップ**
   - コンポーネントアンマウント時にイベントリスナーを解除

### API

```rust
/// ネットワーク状態コンテキスト
pub struct NetworkStatusContext {
    pub state: ReadSignal<NetworkState>,
    pub set_state: WriteSignal<NetworkState>,
}

/// ネットワーク状態プロバイダー
#[component]
pub fn NetworkStatusProvider(children: Children) -> impl IntoView

/// ネットワーク状態を取得するフック
pub fn use_network_status() -> NetworkStatusContext

/// オンライン状態のみを取得するフック（簡易版）
pub fn use_is_online() -> Signal<bool>

/// オンライン時のみ実行するユーティリティ
pub fn when_online<F, Fut>(f: F) where F: Fn() -> Fut, Fut: Future<Output = ()>
```

## Test Cases

### TC-001: 初期状態の取得

- Given: アプリ起動時
- When: `navigator.onLine` が `true` を返す
- Then: `NetworkState.is_online` が `true` になる

### TC-002: 初期状態の取得（オフライン）

- Given: アプリ起動時
- When: `navigator.onLine` が `false` を返す
- Then: `NetworkState.is_online` が `false` になる

### TC-003: オフラインへの移行

- Given: オンライン状態
- When: `offline` イベントが発火
- Then: `NetworkState.is_online` が `false` になる

### TC-004: オンラインへの復帰

- Given: オフライン状態
- When: `online` イベントが発火
- Then: `NetworkState.is_online` が `true` になる
- And: `last_online_at` が現在時刻に更新される

### TC-005: 状態変化時刻の記録

- Given: 任意の状態
- When: 状態が変化
- Then: `last_checked_at` が現在時刻で更新される

### TC-006: コンテキストの共有

- Given: `NetworkStatusProvider` で囲まれたコンポーネント
- When: `use_network_status()` を呼び出す
- Then: 現在のネットワーク状態が取得できる

### TC-007: フォールバック動作

- Given: `navigator.onLine` が利用できない環境
- When: 初期化
- Then: `is_online` が `true` と仮定される（楽観的フォールバック）

## Implementation Notes

### ブラウザ API 使用

```javascript
// 初期状態
navigator.onLine;

// イベントリスナー
window.addEventListener("online", handler);
window.addEventListener("offline", handler);
```

### Leptos でのイベントリスナー登録

```rust
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

// クロージャ作成
let on_online: Closure<dyn Fn()> = Closure::new(move || {
    set_state.update(|s| s.is_online = true);
});

// イベント登録
window.add_event_listener_with_callback("online", on_online.as_ref().unchecked_ref())?;

// リーク防止
on_online.forget(); // または on_cleanup で解除
```

### 状態変化時のコールバック

オンライン復帰時に自動同期などを行いたい場合は、`Effect` を使用：

```rust
Effect::new(move |_| {
    if is_online.get() {
        // オンライン復帰時の処理
        spawn_local(async move {
            let _ = sync_github_stats().await;
        });
    }
});
```
