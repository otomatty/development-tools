# GitHubStatsSnapshot Model Specification

## Related Files

- Implementation: `src-tauri/src/database/models/github_stats_snapshot.rs`
- Migration: `src-tauri/src/database/migrations.rs` (version 6)
- Repository: `src-tauri/src/database/repository/github_stats_snapshot.rs`
- Tests: `src-tauri/src/database/repository/tests.rs` (snapshot tests)

## Related Documentation

- Issue: `docs/01_issues/open/2025_11/20251129_02_github-stats-daily-comparison.md`
- GitHub Issue: #35

## Requirements

### 責務

- GitHub 統計の日次スナップショットを保存・管理
- 前日比（差分）計算のための基準データを提供
- ユーザーごとの統計履歴を追跡

### 状態構造

#### GitHubStatsSnapshot

| フィールド           | 型     | 説明                              |
| -------------------- | ------ | --------------------------------- |
| id                   | i64    | プライマリキー                    |
| user_id              | i64    | ユーザー ID（外部キー）           |
| total_commits        | i32    | 累計コミット数                    |
| total_prs            | i32    | 累計 PR 数                        |
| total_reviews        | i32    | 累計レビュー数                    |
| total_issues         | i32    | 累計 Issue 数                     |
| total_stars_received | i32    | 累計獲得スター数                  |
| total_contributions  | i32    | 累計コントリビューション数        |
| snapshot_date        | String | スナップショット日付 (YYYY-MM-DD) |
| created_at           | String | 作成日時                          |

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

- `from_github_stats(user_id, stats, date)`: GitHubStats からスナップショットを作成
- `calculate_diff(previous)`: 前のスナップショットとの差分を計算

#### StatsDiff

- `default()`: 全て 0 の差分を作成
- `is_positive()`: 全体的に増加傾向かどうか
- `has_changes()`: 何らかの変化があるかどうか

## Test Cases

### TC-001: GitHubStatsSnapshot from_github_stats

- **Given**: GitHubStats と user_id、日付
- **When**: `from_github_stats(user_id, stats, date)` を実行
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
