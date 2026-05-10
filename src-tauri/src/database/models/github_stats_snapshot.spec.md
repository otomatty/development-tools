# GitHubStatsSnapshot Model Specification

## Related Files

- Implementation: `src-tauri/src/database/models/github_stats_snapshot.rs`
- Migration: `src-tauri/src/database/migrations.rs` (version 6 = テーブル作成 / version 12 = `prs_merged`・`issues_closed` 追加 + 旧 `previous_github_stats` KV 廃止)
- Repository: `src-tauri/src/database/repository/github_stats_snapshot.rs`
- Caller: `src-tauri/src/commands/github.rs` (`run_github_sync`)
- Model Tests: `src-tauri/src/database/models/github_stats_snapshot.rs` (tests module)
- Repository Tests: `src-tauri/src/database/repository/github_stats_snapshot.rs` (tests module)

## Related Documentation

- Issue: `docs/01_issues/open/2025_11/20251129_02_github-stats-daily-comparison.md`
- GitHub Issue: #35（初版）, #189（旧 `previous_github_stats` KV との統合）
- 監査レポート: `docs/02_research/2026_04/20260425_github_integration_audit.md` §9.2

## Requirements

### 責務

- GitHub 統計の日次スナップショットを保存・管理
- 前日比（差分）計算のための基準データを提供（UI 表示）
- **同期間の XP 差分計算の基準データを提供**（Issue #189 で `previous_github_stats` KV から統合された責務）
- ユーザーごとの統計履歴を追跡

### 「前回値」の単一情報源 (SSoT)

Issue #189 までは「前回値」を 2 箇所に保持していた：

- `previous_github_stats`（`activity_cache` 内の KV、JSON 文字列） — XP 算出用
- `github_stats_snapshots`（本テーブル、日次 unique） — 前日比表示用

整合性事故の温床になっていたため、Issue #189 で本テーブルに統合した。
KV は migration v12 で削除済み。

XP 差分の取得方法は呼び出し側で 2 通りに分かれる:

- **XP 算出用ベース**: `get_latest_github_stats_snapshot(user_id)` — 日付に関係なく最新行を返す。同日中に複数回 sync しても 2 回目以降は同日の更新済み行と差分を取るため、二重加算を防ぐ。
- **前日比表示用ベース**: `get_previous_github_stats_snapshot(user_id, today)` — 「今日より前」の最新行を返す。同日中に何度 sync しても表示は「昨日との差分」で安定する。

### 状態構造

#### GitHubStatsSnapshot

| フィールド            | 型     | 説明                                                                             |
| --------------------- | ------ | -------------------------------------------------------------------------------- |
| id                    | i64    | プライマリキー                                                                   |
| user_id               | i64    | ユーザー ID（外部キー）                                                          |
| total_commits         | i32    | 累計コミット数                                                                   |
| total_prs             | i32    | 累計 PR 数                                                                       |
| total_prs_merged      | i32    | 累計マージ済み PR 数 — XP 算出用 (Issue #189)。前日比 UI には出ない              |
| total_reviews         | i32    | 累計レビュー数                                                                   |
| total_issues          | i32    | 累計 Issue 数                                                                    |
| total_issues_closed   | i32    | 累計クローズ済み Issue 数 — XP 算出用 (Issue #189)。前日比 UI には出ない         |
| total_stars_received  | i32    | 累計獲得スター数                                                                 |
| total_contributions   | i32    | 累計コントリビューション数                                                       |
| snapshot_date         | String | スナップショット日付 (YYYY-MM-DD)                                                |
| created_at            | String | 作成日時                                                                         |

#### StatsDiff

| フィールド         | 型             | 説明                         |
| ------------------ | -------------- | ---------------------------- |
| commits_diff       | i32            | コミット数の差分             |
| prs_diff           | i32            | PR 数の差分                  |
| reviews_diff       | i32            | レビュー数の差分             |
| issues_diff        | i32            | Issue 数の差分               |
| stars_diff         | i32            | スター数の差分               |
| contributions_diff | i32            | コントリビューション数の差分 |
| comparison_date    | Option<String> | 比較対象の日付               |

### メソッド

#### GitHubStatsSnapshot

- `new(user_id, total_commits, total_prs, total_prs_merged, total_reviews, total_issues, total_issues_closed, total_stars_received, total_contributions, snapshot_date)`: 各統計値からスナップショットを作成
- `calculate_diff(previous)`: 前のスナップショットとの差分（前日比 UI 用）を計算

#### Repository (`Database`)

- `save_github_stats_snapshot(snapshot)`: UPSERT で日次スナップショットを保存
- `get_previous_github_stats_snapshot(user_id, before_date)`: 指定日より前の最新行を返す（前日比表示用）
- `get_latest_github_stats_snapshot(user_id)`: 日付に関係なく最新行を返す（XP 算出用）
- `get_github_stats_snapshot_for_date(user_id, date)`: 特定日のスナップショットを取得

#### StatsDiff

- `default()`: 全て 0 の差分を作成
- `is_positive()`: 全体的に増加傾向かどうか
- `has_changes()`: 何らかの変化があるかどうか

## Test Cases

### TC-001: GitHubStatsSnapshot new

- **Given**: user_id、各統計値、日付
- **When**: `GitHubStatsSnapshot::new(...)` を実行
- **Then**: 正しい値でスナップショットが作成される

### TC-002: calculate_diff with previous snapshot

- **Given**: 2 つのスナップショット（現在と前日）
- **When**: `current.calculate_diff(Some(&previous))` を実行
- **Then**: 各フィールドの差分が正しく計算される

### TC-003: calculate_diff without previous snapshot

- **Given**: 現在のスナップショットのみ
- **When**: `current.calculate_diff(None)` を実行
- **Then**: StatsDiff::default() が返される（全て 0）

### TC-004: StatsDiff has_changes

- **Given**: 差分が 0 でない StatsDiff
- **When**: `diff.has_changes()` を実行
- **Then**: true が返される

### TC-005: StatsDiff has_changes with no changes

- **Given**: 全て 0 の StatsDiff
- **When**: `diff.has_changes()` を実行
- **Then**: false が返される
