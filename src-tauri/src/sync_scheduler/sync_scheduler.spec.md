# Sync Scheduler Specification

## Related Files

- Module: `src-tauri/src/sync_scheduler/mod.rs`
- State / types: `src-tauri/src/sync_scheduler/state.rs`
- Pure decision logic: `src-tauri/src/sync_scheduler/actions.rs`
- Side-effect runner: `src-tauri/src/sync_scheduler/runner.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/180
- Audit: `docs/02_research/2026_04/20260425_github_integration_audit.md` (§4.1, §9.4, §8 G-07)
- Cache strategy spec: `src-tauri/src/commands/cache_fallback.spec.md`

## Requirements

### 責務

GitHub 統計同期 (`sync_github_stats`) を、ユーザーの `user_settings` の値に基づいて自動的に起動・停止・スロットリングする。手動同期（`SyncSettings` の「今すぐ同期」ボタン）は引き続き利用できる。

### 入力（Inputs）

| 項目 | 由来 | 用途 |
| --- | --- | --- |
| `sync_on_startup`        | `user_settings`                  | 起動時に 1 回同期するか |
| `sync_interval_minutes`  | `user_settings`                  | 定期同期の間隔（0 = 手動のみ） |
| `background_sync`        | `user_settings`                  | OFF にするとタイマーを止める |
| `last_sync_at`           | `sync_metadata`                  | 経過時間判定 |
| `rate_limit_remaining`   | `sync_metadata`                  | レート制限のスロットリング |
| `rate_limit_reset_at`    | `sync_metadata`                  | レート制限の解除時刻 |
| `is_first_run`           | ループ自身が保持                  | startup-sync の発火タイミング |
| `now`                    | `chrono::Utc::now()`             | 経過時間計算 |

### 出力（Actions）

```text
RunSync                              - 同期を実行
Sleep { seconds }                    - 経過後に再評価
Idle  { reason }                     - 設定変更を待つ
RateLimited { reason, seconds }      - レート制限解除を待つ
```

### 判定アルゴリズム

優先度順に評価する：

1. `background_sync = false` かつ「初回 + sync_on_startup=true」でない → `Idle("background_sync_disabled")`
2. 初回 + `sync_on_startup = true` → `RunSync`
3. `sync_interval_minutes <= 0` → `Idle("manual_only")`
4. レート制限が閾値未満 (`remaining <= 50`) かつ `reset_at` が未来 → `RateLimited`
5. 経過時間が `interval` 以上 → `RunSync`
6. それ以外 → `Sleep`（最低 30 秒、最大 5 分にクランプ）

#### `last_sync_at = None` の扱い

履歴が無いケース（fresh install など）では：

- 初回 + `sync_on_startup = false` → `now` を baseline と見なし、Sleep する
  （`interval` 後に最初の自動同期）。ユーザーが明示的に「起動時同期しない」を
  選んでいるため、catch-up でその意図を上書きしない。
- それ以外（履歴無しの 2 回目以降など実質的に発生しないケース） → `RunSync`
  で復帰させる。

### 並行制御

`run_github_sync` は `AppState.sync_lock`（`tokio::sync::Mutex<()>`）を
取得してから実行する。手動同期（"今すぐ同期" ボタン）とスケジューラ同期が
重なっても、片方が完了するまでもう片方はブロックされる。

Mutex を取らないと、両者が同じ pre-sync snapshot
（`get_previous_github_stats`）を読んで XP / バッジ / チャレンジ進捗を
それぞれ適用してしまい、ユーザーの XP が二重加算される。

### スリープのクランプ理由

- 最低 30 秒: 高頻度チェックを避ける（ロードを下げる）
- 最大 5 分: 通知チャネル取りこぼしの安全網（設定変更が最遅でも 5 分で観測される）

### スキップ理由

DB (`sync_metadata.last_skipped_reason`) と in-memory `SchedulerStatus` の
両方に保存されるもの：

| 値                          | 意味 |
| --------------------------- | --- |
| `background_sync_disabled`  | バックグラウンド同期 OFF |
| `manual_only`               | 自動同期 OFF（`interval=0`） |
| `rate_limited`              | レート制限到達 |

in-memory `SchedulerStatus` のみ（DB には永続化しない）：

| 値                          | 意味 |
| --------------------------- | --- |
| `not_logged_in`             | 未ログイン。`set_status_logged_out` で UI 用に表示するだけで、`record_sync_skipped` は呼ばれない |

スキップを記録した直後にランナーは `RwLock<SchedulerStatus>` の
`last_skipped_reason` / `last_skipped_at` も同期更新する。これにより
`get_scheduler_status` が次のループ反復を待たずに最新のスキップ理由を返す。

### 同期成功時の `sync_metadata` 更新

`run_github_sync` が成功した時点で、以下を **同関数内で** 永続化する。これにより
手動同期（`sync_github_stats` コマンド）と自動同期（スケジューラ）の両経路で
同じ事後処理が走る。

1. `get_or_create_sync_metadata(user_id, "github_stats")` で行を確実に作成
2. `update_sync_metadata` で `last_sync_at = now()`
3. `clear_sync_skipped` で `last_skipped_*` をクリア
4. `clear_sync_rate_limit` で `rate_limit_*` をクリア

→ 新規ユーザーで「初回 RunSync 後も `last_sync_at` が `None` のままで
RunSync を連発する」現象を防ぐ。

### 同期失敗時の `sync_metadata` 更新

GitHub 側が `Rate limit exceeded. Resets at <unix_ts>` を返した場合、
ランナーは：

1. `parse_rate_limit_reset(err_msg)` でリセット時刻を抽出
2. `record_sync_rate_limit(user_id, sync_type, reset_at)` で
   `rate_limit_remaining=0` / `rate_limit_reset_at` を保存
3. `record_sync_skipped` で `last_skipped_reason="rate_limited"` を記録
4. `seconds_until(reset_at, now)` で算出した秒数 sleep
   （`MIN_FAILURE_SLEEP_SECONDS=60` 〜 `MAX_FAILURE_SLEEP_SECONDS=30 分` でクランプ）

リセット時刻が抽出できない場合でも、最低 60 秒は sleep する（タイトな再試行
ループの防止）。

### 設定変更の即時反映

`update_settings` / `reset_settings` コマンドは、保存後に
`SyncSchedulerHandle::notify_config_changed()` を呼び出してループの
`tokio::sync::Notify` を起こす。スリープ中・Idle 中ともに即座に再評価される。

### Idle 状態の再評価

Idle ブランチも `wait_for_change_or_timeout(&notify, IDLE_POLL_SECONDS)` で
最大 5 分の bounded wait としている。設定変更以外の状態遷移（ログアウト／
ログイン、アカウント切り替え、メタデータの外部変更など）でも、最大
`IDLE_POLL_SECONDS` 後にループが再評価される自己治癒設計。

### スケジューラの起動／停止

- 起動: `tauri::Builder::setup` 内で `start_scheduler(app.app_handle())` を呼び出して 1 回だけ生成する。
- 停止: 単独の停止 API は提供しない。アプリ終了時に Tauri のランタイムが停止する。`background_sync=false` でループは Idle 状態に入る。

### ロギング

`eprintln!` プレフィックス `Scheduler:` で出力。`logクレートに置換` の TODO は既存方針（`AGENTS.md` 参照）に従う。

## Test Cases

`actions.rs` の `#[cfg(test)] mod tests` に対応するテストを実装。

### TC-001: `sync_on_startup=true` runs immediately on first iteration

- Given: `is_first_run=true`, `sync_on_startup=true`, `background_sync=true`
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::RunSync`

### TC-002: First run without startup sync still runs when no history

- Given: `is_first_run=true`, `sync_on_startup=false`, `last_sync_at=None`
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::RunSync`

### TC-003: `background_sync=false` halts the scheduler

- Given: `background_sync=false`, `is_first_run=false`
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::Idle { reason: "background_sync_disabled" }`

### TC-004: `background_sync=false` still allows one startup sync

- Given: `background_sync=false`, `is_first_run=true`, `sync_on_startup=true`
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::RunSync`

### TC-005: `sync_interval_minutes=0` (manual only) idles

- Given: `sync_interval_minutes=0`
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::Idle { reason: "manual_only" }`

### TC-006: When the interval has elapsed since last sync, run

- Given: `sync_interval_minutes=5`, `last_sync_at = now - 10min`
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::RunSync`

### TC-007: When the interval has not elapsed, sleep for the remainder

- Given: `sync_interval_minutes=60`, `last_sync_at = now - 10min`
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::Sleep { seconds }` clamped between `MIN_SLEEP_SECONDS` and `MAX_SLEEP_SECONDS`

### TC-008: Rate limit critical with future reset → RateLimited

- Given: `rate_limit_remaining = 10`, `rate_limit_reset_at = now + 2min`, eligible by interval
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::RateLimited { reason: "rate_limited", seconds }`

### TC-009: Rate limit not critical → normal eligibility wins

- Given: `rate_limit_remaining = 4500`, eligible by interval
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::RunSync`

### TC-010: Rate limit critical but reset is in the past → not skipped

- Given: `rate_limit_remaining = 10`, `rate_limit_reset_at = now - 1min`, eligible by interval
- When: `decide_action(&inputs)`
- Then: `SchedulerAction::RunSync`

### TC-011: `next_sync_at` returns last+interval

- Given: `sync_interval_minutes=30`, `last_sync_at = now - 10min`
- When: `next_sync_at(&inputs)`
- Then: `Some(last + 30min)`

### TC-012: `next_sync_at` is None when scheduling is disabled

- Given: `sync_interval_minutes=0` OR `background_sync=false`
- When: `next_sync_at(&inputs)`
- Then: `None`

### TC-013: `classify_rate_limited` recognizes the GitHubError display

- Given: an error string formatted from `GitHubError::RateLimited(_)`
- When: `classify_rate_limited(msg)`
- Then: `true`

### TC-014: `parse_rate_limit_reset` extracts the unix timestamp

- Given: `"Rate limit exceeded. Resets at 1700000000"` (formatted from `GitHubError::RateLimited`)
- When: `parse_rate_limit_reset(msg)`
- Then: `Some(DateTime::from_timestamp(1_700_000_000, 0))`

### TC-015: `parse_rate_limit_reset` returns None when no timestamp

- Given: `"Rate limit"` (no trailing timestamp), `"network error"`
- When: `parse_rate_limit_reset(msg)`
- Then: `None`

### TC-016: `seconds_until` clamps past targets to MIN_FAILURE_SLEEP_SECONDS

- Given: target = now - 120s
- When: `seconds_until(target, now)`
- Then: `MIN_FAILURE_SLEEP_SECONDS` (60s)

### TC-017: `seconds_until` clamps far-future targets to MAX_FAILURE_SLEEP_SECONDS

- Given: target = now + 2 hours
- When: `seconds_until(target, now)`
- Then: `MAX_FAILURE_SLEEP_SECONDS` (30 minutes)

### TC-018: `seconds_until` passes through values within the window

- Given: target = now + 5 minutes
- When: `seconds_until(target, now)`
- Then: `300`

## DoD（Issue #180 完了条件）への対応

| Issue 完了条件 | 対応 |
| --- | --- |
| `sync_on_startup=true` でアプリを起動すると、ログイン済みなら 1 回 `sync_github_stats` が実行される | TC-001, runner の初回イテレーション |
| `sync_interval_minutes` を 5 分にすると、5 分ごとに同期が走る | TC-006/007, runner の Sleep ループ |
| `background_sync=false` ではタイマーが停止する | TC-003, runner の Idle |
| 単体テスト追加（スケジューラ判定ロジック） | TC-001〜TC-018 |
| レート制限到達時は同期をスキップし、UI でユーザーに通知する | TC-008/009/010, `last_skipped_reason` 永続化, `get_scheduler_status` で UI 表示 |
