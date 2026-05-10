# Repository Archive Cleanup Specification

GitHub 側でリポジトリ削除や名前変更（リダイレクト無し）が発生したとき、
`sync_project_issues` が 404 で停止して他プロジェクトの同期まで巻き添えに
するのを防ぐための仕様。

監査レポート §7.3 / §8 G-09 への対応。

## Related Files

- Backend Implementation: `src-tauri/src/commands/issues.rs`
  (`sync_project_issues`, `sync_all_projects`, `relink_repository`,
  `mark_project_repository_gone`)
- Error Type: `src-tauri/src/github/client.rs`
  (`GitHubError::RepositoryGone`, `GitHubError::NotFound`)
- Issue Client: `src-tauri/src/github/issues.rs` (`IssuesClient::get_issues`)
- Models: `src-tauri/src/database/models/project.rs`
  (`Project.is_archived`, `Project.archived_at`, `Project.archived_reason`,
  `archive_reasons::REPOSITORY_GONE`, `CachedIssue.is_archived`)
- Migration: `src-tauri/src/database/migrations.rs` (version 13,
  `add_project_archive_columns`)
- Tauri Wrapper: `src/lib/tauri/commands.ts`
  (`issues.syncProjectIssues`, `issues.syncAllProjects`,
  `repositories.relink`)
- Frontend Types: `src/types/issue.ts`
  (`Project.isArchived`, `ProjectArchivedReason`,
  `SyncAllProjectsResult`, `isProjectRepositoryGone`)

## Related Documentation

- Cache Fallback Specification: `src-tauri/src/commands/cache_fallback.spec.md`
- Issue: GitHub Issue #190
- 監査レポート §7.3, §8 G-09

---

## Requirements

### 責務

1. **404 の局所化**: 単一プロジェクトのリポジトリが消えても、他プロジェクトの
   同期と全体スケジューラの動作を止めない。
2. **状態の保全**: アーカイブされたプロジェクトと cached_issues は物理削除
   せず、`is_archived = 1` でフラグするだけにする。再リンク後に履歴を
   失わないため。
3. **再リンク導線**: ユーザーが新しい owner/repo を選ぶだけで archive を
   解除できるようにする。

### データモデル

#### `projects` テーブル（v13 で追加）

| カラム             | 型       | 説明                                                                |
| ----------------- | -------- | ------------------------------------------------------------------- |
| `is_archived`     | INTEGER  | 1 = リポジトリが 404、0 = 正常                                       |
| `archived_at`     | DATETIME | はじめて 404 を観測した時刻（後続の同期では更新しない）                |
| `archived_reason` | TEXT     | `repository_gone` 固定（将来別の理由を増やすときの拡張ポイント）       |

`repo_*` カラムは archive 中も保持する。これは:
- 再リンク UI に「以前は X/Y にリンクされていた」と表示するため
- 監査ログとして履歴的価値があるため

#### `cached_issues` テーブル（v13 で追加）

| カラム         | 型       | 説明                                            |
| ------------- | -------- | ----------------------------------------------- |
| `is_archived` | INTEGER  | 親プロジェクトの状態をミラー                       |
| `archived_at` | DATETIME | 親プロジェクトと同じタイムスタンプ                  |

物理削除しないのは、リネームの場合 `html_url` が新しい URL にリダイレクト
することがあるため。kanban で「読み取り専用」表示にすればよい。

### 同期フロー（`sync_project_issues`）

```text
sync_project_issues(project_id) -> SyncProjectIssuesResponse
  ├─ get_issues(owner, repo, page=1)
  │   ├─ Ok(issues)            → 通常パスへ（cached_issues を更新）
  │   ├─ Err(NotFound)         → confirm_repository_gone(/repos/owner/repo)
  │   │     ├─ Confirmed       → mark_project_repository_gone
  │   │     │                     → cache を返し archived=true で Ok 終了
  │   │     ├─ RepoExists      → archive せず Err（トークン権限不足の旨）
  │   │     └─ Indeterminate   → archive せず元エラーを伝播（次回再試行）
  │   ├─ Err(Unauthorized)     → 既存の auth-expired フローへ委譲
  │   └─ その他のエラー         → そのまま map_github_result でエスカレート
  ├─ ...
  └─ 全ページ成功時:
        UPDATE projects
        SET last_synced_at = now,
            is_archived = 0, archived_at = NULL, archived_reason = NULL
        WHERE id = project_id
        → archived=false で Ok 終了
```

#### 404 確認フロー (`confirm_repository_gone`)

`/repos/{owner}/{repo}/issues` の 404 だけを根拠にアーカイブすると、
トークンスコープ不足や Repo の private 化など本来回復可能な状態でも
プロジェクトを `is_archived = 1` にしてしまう。`sync_all_projects` は
`is_archived = 0` でフィルタするため、誤検知のアーカイブは「自動同期から
永続的に外れる」スティッキーな副作用を持つ。

そこで `IssuesClient::get_repository` を 1 回だけ追加で呼び、結果に
よって挙動を分岐する:

| `/repos/{owner}/{repo}` の戻り値 | 判定                | 挙動                          |
| -------------------------------- | ------------------- | ----------------------------- |
| `Err(NotFound)`                  | `Confirmed`         | アーカイブ実行                 |
| `Ok(_)`                          | `RepoExists`        | アーカイブせず `Err` で報告     |
| その他 `Err(...)`                | `Indeterminate(e)`  | アーカイブせず元エラーを伝播    |

戻り値は `SyncProjectIssuesResponse { issues, archived }`。404 を `Err`
にしないのは `sync_all_projects` のループ中断を避けるためだが、単発の
呼び出し元（プロジェクトダッシュボード等）が「成功 + 古いキャッシュ」
として扱ってしまう問題（PR #213 レビュー指摘 P1）があったので、
`archived` フラグで明示的にシグナルを返す。UI は `archived=true`
を受けたら再リンクバナーを表示する。

成功時に archive フラグもリセットすることで、一過性の 404
（権限が一瞬剥がれた、GHE の indexing ラグ等）が起きても次回同期で
自動回復する。

### 一括同期（`sync_all_projects`）

```rust
pub struct SyncAllProjectsResult {
    pub synced:   Vec<i64>,      // 正常に同期されたプロジェクト
    pub archived: Vec<i64>,      // 今回 404 でアーカイブされたプロジェクト
    pub failed:   Vec<SyncFailure>,
}
```

- 既に `is_archived = 1` のプロジェクトはスキップ（クエリで除外）。
  → 一度 gone と判定されたリポジトリで毎回 404 を打たない。
- 各プロジェクトの 404 は `sync_project_issues` 内で吸収されるので、
  `sync_all_projects` から見ると `Ok` で返ってくる。アーカイブ判定は
  同期後にもう一度 `is_archived` を読み直して `archived` バケットに
  振り分ける。

### 再リンク（`relink_repository` / `link_repository`）

`link_repository` を再利用する。同一トランザクション内で:

1. `projects` の repo_* と archive フラグを更新
2. 「異なるリポジトリへの再リンク」（旧 `github_repo_id` ≠ 新 `repo_info.id`）
   の場合のみ `DELETE FROM cached_issues WHERE project_id = ?` を実行

```sql
BEGIN;
UPDATE projects
SET github_repo_id = ?, repo_owner = ?, repo_name = ?, repo_full_name = ?,
    is_archived = 0, archived_at = NULL, archived_reason = NULL,
    updated_at = ?
WHERE id = ? AND user_id = ?;
-- 異なるリポジトリへの再リンク時のみ:
DELETE FROM cached_issues WHERE project_id = ?;
COMMIT;
```

stale な cached_issues を保持しない理由（PR #213 P1 レビュー）:

- kanban が旧リポジトリの issue 番号を表示し続ける
- `update_issue_status` は `(project_id, number)` で cached_issues を
  特定し、GitHub には **新しい** owner/repo + その番号で API を叩く
- 結果、ドラッグで偶然同じ番号の無関係な issue を変更してしまう

最初に同じリポジトリを再リンクするケース（権限変更後の確認等）では
キャッシュを保持する。次回 sync で UPSERT が `is_archived = 0` を
上書きする。

### エラー型

- `GitHubError::NotFound(url)` … 任意の 404。`get_issues` ／その他汎用。
- `GitHubError::RepositoryGone(repo)` … 上位レイヤで 404 を「リポジトリが
  消えた」と確定的に判定したい場合に使う型。現状は `NotFound` を直接
  `mark_project_repository_gone` にルーティングしているのでパブリック
  API 上の必須ではないが、将来 webhook / push 通知でリポジトリ削除を
  検知したときに同じ型でハンドラを書けるよう先行追加してある。

## Out of Scope

- リネーム（リダイレクトあり）への自動追従。GitHub REST API は
  301 を返すが、`/repos/{owner}/{repo}/issues` は 404 を返すケースが多く
  確実な検出が難しいため、当面はユーザー操作による再リンクのみとする。
- `delete_project` への自動移行。データ消失リスクを避けるため、
  自動削除は行わずユーザーアクション（「削除」ボタン）に委ねる。

## Test Coverage

`src-tauri/src/commands/issues.rs` の `#[cfg(test)] mod tests`:

- `mark_project_repository_gone_*` … archive フラグが立つ／二重実行が
  no-op／cached_issues も連動する。
- `sync_project_issues_archives_on_404` … 404 観測でプロジェクトが
  archived 状態に遷移し、他のプロジェクトには影響しない。
- `link_repository_clears_archive_flags` … 再リンクで archive フラグが
  解除される。
