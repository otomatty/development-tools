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

```
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
5. `last_sync_at` が無い、または経過時間が `interval` 以上 → `RunSync`
6. それ以外 → `Sleep`（最低 30 秒、最大 5 分にクランプ）

### スリープのクランプ理由

- 最低 30 秒: 高頻度チェックを避ける（ロードを下げる）
- 最大 5 分: 通知チャネル取りこぼしの安全網（設定変更が最遅でも 5 分で観測される）

### スキップ理由（`sync_metadata.last_skipped_reason`）

| 値                          | 意味 |
| --------------------------- | --- |
| `background_sync_disabled`  | バックグラウンド同期 OFF |
| `manual_only`               | 自動同期 OFF（`interval=0`） |
| `rate_limited`              | レート制限到達 |
| `not_logged_in`             | 未ログイン |

### 設定変更の即時反映

`update_settings` コマンドは、保存後に `SyncSchedulerHandle::notify_config_changed()` を呼び出し、ループの `tokio::sync::Notify` を起こす。スリープ中・Idle 中ともに即座に再評価される。

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

## DoD（Issue #180 完了条件）への対応

| Issue 完了条件 | 対応 |
| --- | --- |
| `sync_on_startup=true` でアプリを起動すると、ログイン済みなら 1 回 `sync_github_stats` が実行される | TC-001, runner の初回イテレーション |
| `sync_interval_minutes` を 5 分にすると、5 分ごとに同期が走る | TC-006/007, runner の Sleep ループ |
| `background_sync=false` ではタイマーが停止する | TC-003, runner の Idle |
| 単体テスト追加（スケジューラ判定ロジック） | TC-001〜TC-013 |
| レート制限到達時は同期をスキップし、UI でユーザーに通知する | TC-008/009/010, `last_skipped_reason` 永続化, `get_scheduler_status` で UI 表示 |
