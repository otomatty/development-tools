# 認証ライフサイクル仕様

GitHub Device Flow による認証〜失効検知〜再ログイン誘導までの状態遷移とイ
ベント設計をまとめたドキュメント。Issue #181 の対応で追加。

## 関連ファイル

| 役割 | パス |
|------|------|
| バックエンド: トークン保管 | `src-tauri/src/auth/token.rs` |
| バックエンド: Device Flow | `src-tauri/src/auth/oauth.rs` |
| バックエンド: 401 共通ハンドラ | `src-tauri/src/auth/session.rs` |
| バックエンド: 認証コマンド | `src-tauri/src/commands/auth.rs` |
| バックエンド: Tauri 起動セットアップ | `src-tauri/src/lib.rs` |
| バックエンド: 同期スケジューラ | `src-tauri/src/sync_scheduler/runner.rs` |
| フロント: 認証ストア | `src/stores/authStore.ts` |
| フロント: イベント購読 | `src/lib/tauri/events.ts` |
| フロント: 再ログインバナー | `src/components/features/auth/SessionExpiredBanner.tsx` |

## 状態遷移

```
                         ┌──────────────────────┐
                         │  LoggedOut           │
                         │  (token なし)         │
                         └────────────┬─────────┘
                                      │
                          ユーザー操作 │  start_device_flow
                                      ▼
                         ┌──────────────────────┐
                         │  DeviceFlowPending   │
                         │  (認可待ち / poll)    │
                         └────────────┬─────────┘
                                      │
                       Polling 成功    │  poll_device_token Success
                                      ▼
                         ┌──────────────────────┐
                  ┌──────│  LoggedIn            │◀─────┐
                  │      │  (token 保持中)        │      │
                  │      └────────────┬─────────┘      │
                  │                   │                │
       手動ログアウト│       401 検出   │ auth-expired   │
        (logout)   │                   │                │
                  │                   ▼                │
                  │      ┌──────────────────────┐      │
                  │      │  SessionExpired      │      │
                  │      │  (token 失効済み・     │      │
                  │      │   バナー表示中)         │      │
                  │      └────────────┬─────────┘      │
                  │                   │                │
                  ▼                   │  再ログイン     │
       ┌──────────────────────┐      │   完了          │
       │  LoggedOut           │◀─────┘                 │
       └──────────┬───────────┘                        │
                  │                                    │
                  └────────────────────────────────────┘
                              再認証成功
```

## バックエンド側の責務

### トークン検証 (`TokenManager::validate_token`)

`GET /user` を Bearer 付きで叩いて以下を返す:

| GitHub レスポンス | 戻り値 | ハンドラ動作 |
|--------------------|--------|--------------|
| 2xx | `Ok(true)` | なにもしない（正常） |
| 401 Unauthorized | `Ok(false)` | 呼び出し側で `handle_unauthorized` を起動 |
| 5xx / 通信エラー | `Err(_)` | **強制ログアウトしない**（ネットワーク不調と区別） |

ネットワーク不調で勝手にログアウトされると UX が破壊されるため、`Err` を
受け取った呼び出し側はセッションをそのまま温存する。

### 共通 401 ハンドラ (`auth::session::handle_unauthorized`)

役割:

1. `TokenManager::logout` で DB のトークンを破棄
2. `auth-expired` イベントを `app.emit` で発火

冪等で、複数回呼ばれても害はない（保管済みトークンが既に空なら no-op、フロ
ント側のリスナーは重複イベントを無視する）。

### 401 検出ポイント

| 場所 | きっかけ | reason 値 |
|------|----------|-----------|
| GitHub コマンド全般 | `map_github_result` ヘルパー経由 | `github_unauthorized` |
| 起動時セッション検証 | `lib.rs::setup` → `run_startup_token_validation` | `startup_validation_failed` |
| 手動 `validate_token` コマンド | UI からの明示的検証 | `manual_validation_failed` |
| バックグラウンド同期 | `sync_scheduler::runner` の失敗時に `classify_unauthorized` 真 | `scheduler_unauthorized` |

`map_github_result` の使い方:

```rust
let github_stats =
    map_github_result(&app, state.inner(), client.get_user_stats(&user.username).await).await?;
```

これだけで「401 → トークン破棄 + イベント emit」までの導線が共通化される。
新しい GitHub API コマンドを追加するときも同じパターンで包む。

### 起動時検証 (`run_startup_token_validation`)

`lib.rs` の `setup` フック内で `tauri::async_runtime::spawn` から非同期で
呼ぶ。スプラッシュ表示や初回描画をブロックしない。

判定ロジック:

```
if no current user → no-op
if validate_token == Ok(true)  → no-op
if validate_token == Ok(false) → handle_unauthorized(STARTUP_VALIDATION_FAILED)
if validate_token == Err(_)    → no-op (ネットワーク不調)
```

### スケジューラとの統合

`sync_scheduler::runner::run_loop` は `run_github_sync` の戻り値文字列を
`classify_unauthorized` で判定する。マッチしたら:

1. `handle_unauthorized` を発火（`SCHEDULER_UNAUTHORIZED`）
2. 即座に `wait_for_change_or_timeout` でループのトップへ戻る
3. 次の反復で `get_current_user` が `None` を返すので「未ログイン」分岐に
   入り、ユーザーが再ログインするまで同期を停止する

## フロント側の責務

### `auth-expired` イベントの購読

`src/stores/authStore.ts` のモジュールロード時に
`events.onAuthExpired` を購読する。受信時:

```ts
useAuth.setState({
  state: { isLoggedIn: false, user: null },
  authExpired: event,
  isLoading: false,
});
```

これで:

- `isLoggedIn` が `false` になるため、各ページの「ログイン済みなら API を
  叩く」分岐が自動で停止する → 401 受信後の API 呼び出し抑止が実現する
- `Home` ページなどは自動的に `LoginCard` を再表示する
- グローバルバナー (`SessionExpiredBanner`) が表示され、ユーザーへ再ログイン
  を促す

### 再ログインフロー

1. `SessionExpiredBanner` の「再ログイン」ボタンをクリック
2. `dismiss()` でバナーを閉じ、`/` (Home) へルーティング
3. `Home` が `LoginCard` を表示
4. 通常の Device Flow → 成功 → `fetchAuthState` → `state.isLoggedIn = true`
5. `fetchAuthState` 内で `authExpired` を自動的に `null` にクリア

### キャッシュ読み取りの扱い

GitHub API 呼び出しは `isLoggedIn === false` で抑止される。ローカル DB の
キャッシュ (`get_github_stats_with_cache` 等) も同じく抑止対象だが、ユーザ
ーが再ログインすれば即座に最新化される。

## イベントペイロード

```ts
interface AuthExpiredEvent {
  /** マシン可読な理由コード */
  reason: string;
  /** UI に表示する日本語メッセージ */
  message: string;
}
```

`reason` は `src-tauri/src/auth/session.rs::reasons` モジュールに定数化さ
れている。新しい検出ポイントを追加する際は、まず定数を追加してから emit
する。

## 既知の制約

- GitHub Device Flow のアクセストークンは原則として失効しない。明示的に
  `https://github.com/settings/applications` から取り消されたケースが主な
  401 原因。
- 401 検出は呼び出しが行われるたびに評価されるため、長時間 API を叩かない
  ユーザーは「失効を検知できる契機がない」可能性がある。本実装では起動時
  の `run_startup_token_validation` と、バックグラウンド同期スケジューラ
  の周期実行でカバーしている。
- 通信障害（タイムアウト・5xx）は強制ログアウトの対象外。ネットワーク復
  旧後に自動で API が再開する。
