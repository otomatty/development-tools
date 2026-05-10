# Badge Evaluation Specification

## Related Files

- Model & evaluator: `src-tauri/src/database/models/badge.rs`
- Tauri commands: `src-tauri/src/commands/github.rs`
  (`get_badges_with_progress`, `get_near_completion_badges`,
  `refresh_badges_progress`, `run_github_sync`)
- Repository: `src-tauri/src/database/repository/user_stats.rs`
  (`update_github_aggregates`, `UserStatsGitHubAggregates`)
- Schema: `src-tauri/src/database/migrations.rs` (migration v14)
- Frontend: `src/components/features/gamification/BadgeGrid.tsx`,
  `src/lib/tauri/commands.ts` (`getBadgesWithProgress`,
  `getNearCompletionBadges`, `refreshBadgesProgress`)

## Related Documentation

- 監査レポート: `docs/02_research/2026_04/20260425_github_integration_audit.md`
  §6.3 / §9.1
- GitHub Issue: #191（バッジ進捗計算の DB 完結化）

## 責務

1. ユーザーが獲得したバッジ・獲得目前のバッジを返す
   (`get_badges_with_progress` / `get_near_completion_badges`)。
2. これらの呼び出しが GitHub API を叩かないことを保証する。
3. 「最新化したい」ユーザー操作のために
   `refresh_badges_progress` を提供する。`run_github_sync` も
   `client.get_user_stats` を叩くが、バッジ表示系コマンドの中で
   直接 GitHub 取得を行うのはこの `refresh_badges_progress` だけ。

## データフロー

```text
sync_github_stats          ── get_user_stats(REST 4 + GraphQL 1) ──▶ GitHubStats
        │
        │ 1) update_github_aggregates(user.id, &agg)
        ▼
   user_stats (DB)  ◀────── (default DB-only writer)
        ▲
        │ 2) badge_context_from_user_stats(&user_stats)
        │
get_badges_with_progress / get_near_completion_badges
        │  → BadgeEvalContext (no API calls)
        ▼
     UI (BadgeGrid)
```

`refresh_badges_progress` は同じ `update_github_aggregates`
パスを通り、呼び出し直後の最新値で
`get_badges_with_progress` 相当の応答を返す。

## `BadgeEvalContext` のフィールド対応

| `BadgeEvalContext` フィールド | DB 列 (`user_stats.*`)        | 書き込みタイミング |
| ----------------------------- | ----------------------------- | ------------------ |
| `total_commits`               | `total_commits`               | sync               |
| `current_streak`              | `current_streak`              | sync (streak)      |
| `longest_streak`              | `longest_streak`              | sync (streak)      |
| `weekly_streak`               | `weekly_streak` *(v14)*       | sync               |
| `monthly_streak`              | `monthly_streak` *(v14)*      | sync               |
| `total_reviews`               | `total_reviews`               | sync               |
| `total_prs`                   | `total_prs`                   | sync               |
| `total_prs_merged`            | `total_prs_merged` *(v14)*    | sync               |
| `total_issues_closed`         | `total_issues_closed` *(v14)* | sync               |
| `languages_count`             | `languages_count` *(v14)*     | sync               |
| `current_level`               | derived from `total_xp`        | (read-only)        |
| `total_stars_received`        | `total_stars_received` *(v14)* | sync               |

`*(v14)*` は migration v14
(`add_user_stats_badge_eval_fields`) で追加された列。

## 不変条件 (Invariants)

- `get_badges_with_progress` / `get_near_completion_badges`
  は **GitHub API を呼ばない**。
- `update_github_aggregates` は XP・current_level・current_streak・
  longest_streak・last_activity_date を **書き換えない**
  （これらは XP / streak の専用パスが書く）。
- `refresh_badges_progress` は GitHub から取得した
  `streak_info.current_streak` / `longest_streak` を
  返り値の `BadgeEvalContext` には反映するが、
  `user_stats` には **書き戻さない**。`run_github_sync` は
  `user_stats.current_streak` を XP ストリークボーナスの
  `old_streak` 基準値として読むため、refresh が DB に
  書いてしまうと次回 sync で `(old_streak == new_streak)` と
  なり日次／マイルストーンボーナスが恒久的に取り逃される。
  ストリークの永続化は `run_github_sync` の単一責任。
- 既存ユーザーは migration v14 直後、新規 6 列が `0` で初期化される。
  最初の `sync_github_stats` 実行で正しい値に上書きされる。
  それまでバッジ進捗は「不足」側に倒れるが、誤って「達成」側に
  なることはない（v14 直後にいきなり満たされる badge は無い）。

## 完了条件 (DoD) — Issue #191

- [x] バッジ表示時に GitHub API が叩かれない
- [x] レート制限消費が顕著に減る
  (Search 30 req/min を消費する `get_user_stats` を
  badge UI から取り除いたため)

## テスト観点

- `database::repository::tests::test_update_github_aggregates_persists_badge_eval_fields`
  — `UserStatsGitHubAggregates` の全フィールドが
  `user_stats` に正しく書き込まれ、再フェッチでも保持されること。
- `database::repository::tests::test_update_github_aggregates_preserves_xp_and_streak`
  — 集計フィールドの書き込みが XP / streak / level を破壊しないこと。
- `database::migrations::tests::test_migrations_run_successfully` /
  `test_migrations_are_idempotent` — v14 を含むスキーマ全体が
  繰り返し適用されても整合すること。
- 既存の `database::models::badge::badge::tests::*`
  — `BadgeEvalContext` ベースの評価ロジック（badge
  evaluator 自体は本変更で改変していない）。
