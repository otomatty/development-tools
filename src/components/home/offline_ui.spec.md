# Offline UI Adjustments Specification

## Related Files

### Implementation Files

- `src/components/home/login_card.rs` - ログインカードのオフライン対応
- `src/components/home/challenge_card.rs` - チャレンジカードのオフライン対応
- `src/components/network_status.rs` - ネットワーク状態プロバイダー

### Related Documentation

- Parent Issue: GitHub Issue #10 (Phase 5: Offline Support & Caching)
- Step 1 Spec: `src/components/network_status.spec.md`
- Step 2 Spec: `src-tauri/src/commands/cache_fallback.spec.md`

## Requirements

### Step 3: オフライン時の UI 調整

#### 1. ログインボタンの無効化（LoginCard）

- オフライン時、ログインボタンを無効化（disabled）する
- 無効化時のスタイリング：
  - 背景色をグレーに変更
  - カーソルを `not-allowed` に設定
  - 不透明度を下げる
- ホバー時にツールチップを表示：「⚠️ オフラインのためログインできません」

#### 2. リトライボタンの無効化（LoginCard ErrorView）

- 認証エラー画面のリトライボタンも同様に無効化
- ツールチップ：「⚠️ オフラインのため再試行できません」

#### 3. チャレンジリフレッシュボタンの無効化（ChallengeCard）

- オフライン時、リフレッシュボタンを無効化
- 無効化時のスタイリング：
  - 背景色を薄く変更
  - テキスト色をミュートに変更
  - カーソルを `not-allowed` に設定
- title 属性でオフライン状態を表示
- ホバー時にツールチップを表示：「⚠️ オフライン」

### 共通仕様

#### ネットワーク状態の取得

- `use_is_online()` フックを使用
- `NetworkStatusProvider` コンテキスト内で動作
- コンテキスト外ではフォールバック（常にオンライン）

#### UI のリアクティブ性

- ネットワーク状態の変化に即座に反応
- オンライン復帰時、ボタンが自動的に有効化

#### ツールチップのスタイリング

- 背景: `bg-gm-bg-dark/95`
- テキスト色: `text-gm-warning`
- ボーダー: `border border-gm-warning/30`
- 丸角: `rounded-lg`
- 表示: ホバー時にフェードイン（`opacity-0 group-hover:opacity-100`）

## Test Cases

### TC-001: ログインボタンのオフライン無効化

- Given: アプリがオフライン状態
- When: ログインカードを表示
- Then: ログインボタンが無効化され、グレーアウトされている

### TC-002: ログインボタンのオフラインツールチップ

- Given: アプリがオフライン状態でログインカードを表示
- When: 無効化されたログインボタンにホバー
- Then: 「オフラインのためログインできません」ツールチップが表示される

### TC-003: ログインボタンのオンライン有効化

- Given: アプリがオフライン状態でログインボタンが無効
- When: ネットワーク接続が復帰（オンラインに変化）
- Then: ログインボタンが自動的に有効化される

### TC-004: リトライボタンのオフライン無効化

- Given: 認証エラー画面が表示されている状態でオフライン
- When: リトライボタンを確認
- Then: リトライボタンが無効化されている

### TC-005: チャレンジリフレッシュボタンのオフライン無効化

- Given: アプリがオフライン状態
- When: チャレンジカードを表示
- Then: リフレッシュボタンが無効化され、スタイルが変更されている

### TC-006: チャレンジリフレッシュボタンのツールチップ

- Given: アプリがオフライン状態でチャレンジカードを表示
- When: 無効化されたリフレッシュボタンにホバー
- Then: 「オフライン」ツールチップが表示される

### TC-007: オンライン時のボタン動作

- Given: アプリがオンライン状態
- When: ログインボタンまたはリフレッシュボタンをクリック
- Then: 通常通りアクションが実行される

## Implementation Notes

### ボタン無効化のパターン

```rust
// オフライン時のボタン無効化パターン
let is_online = use_is_online();

view! {
    <div class="relative group">
        <button
            class=move || {
                let online = is_online.get();
                if online {
                    "normal-style"
                } else {
                    "disabled-style cursor-not-allowed opacity-50"
                }
            }
            on:click=move |e| {
                if is_online.get() {
                    // 実際のアクション
                }
            }
            disabled=move || !is_online.get()
        >
            "Button Text"
        </button>

        // オフライン時のみツールチップ表示
        <Show when=move || !is_online.get()>
            <div class="tooltip">
                "⚠️ オフラインのため..."
            </div>
        </Show>
    </div>
}
```

### 今後の拡張候補

- プロフィールカードのログアウトボタン（ログアウトはオフラインでも可能だが、検討の余地あり）
- 設定画面の同期ボタン
- バッジ詳細の取得ボタン（もし追加する場合）
