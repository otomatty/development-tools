# Cross-repository "Today / Inbox" Specification

GitHub Search API を用いて、自分にアサインされた Open Issue とレビュー依頼が
来ている Pull Request を**全リポジトリ横断**で取得・表示する機能の仕様。

監査レポート §1 ギャップ表 / §8 G-02 + G-03 への対応。UX 再設計 (#177) の
ファーストビューとして必須。

## Related Files

- Backend Implementation: `src-tauri/src/commands/issues.rs`
  (`get_my_open_work_with_cache`, `MyOpenWorkItem`, `MyOpenWork`)
- Search API Client: `src-tauri/src/github/issues.rs`
  (`IssuesClient::search_assigned_issues`, `IssuesClient::search_review_requested`,
  `GitHubSearchItem`, `GitHubSearchResponse`)
- Cache Repository: `src-tauri/src/database/repository/cache.rs`
- Cache Models: `src-tauri/src/database/models/cache.rs`
  (`cache_types::MY_OPEN_WORK`, `cache_durations::MY_OPEN_WORK`)
- Tauri Wrapper: `src/lib/tauri/commands.ts` (`issues.getMyOpenWorkWithCache`)
- Frontend Page: `src/pages/Issues/Issues.tsx`
  (with `InboxFilters.tsx`, `InboxItemRow.tsx`)
- Frontend Types: `src/types/issue.ts` (`MyOpenWork`, `MyOpenWorkItem`)

## Related Documentation

- Cache Fallback Specification: `src-tauri/src/commands/cache_fallback.spec.md`
- Issue: GitHub Issue #183
- 関連: #176 (現状調査), #177 (UX 再設計), #104 (Issue 管理: フィルタ／検索)

---

## Requirements

### 責務

1. **取得**: GitHub Search API で `assignee:@me` と `review-requested:@me` の
   Open Issue / PR を集計し、リポジトリ横断で `MyOpenWork` として返す。
2. **キャッシュ**: 5 分の TTL で SQLite (`activity_cache`) に保存し、ネットワーク
   障害時はフォールバック表示する。
3. **レート制限**: Search API は 30 req/min（認証あり）が上限。
   1 回の更新で 2 リクエストしか発行せず、`useCachedFetch` の `staleTime` は
   バックエンドの TTL と同じ 5 分にそろえることで、フォーカス復帰のたびに
   API を叩かないようにする。

### データ構造

#### `MyOpenWork`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyOpenWork {
    pub assigned: Vec<MyOpenWorkItem>,
    pub review_requested: Vec<MyOpenWorkItem>,
}
```

#### `MyOpenWorkItem`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyOpenWorkItem {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub repo_full_name: String,
    pub kind: String,    // "issue" | "pull_request"
    pub source: String,  // "assigned" | "review_requested"
    pub priority: Option<String>,         // "high" | "medium" | "low"
    pub labels: Vec<String>,
    pub assignee_login: Option<String>,
    pub assignee_avatar_url: Option<String>,
    pub author_login: Option<String>,
    pub created_at: String,  // ISO8601
    pub updated_at: String,  // ISO8601
}
```

### キャッシュ対象

| データタイプ           | cache_type キー | 有効期限 | 説明                                          |
| ---------------------- | --------------- | -------- | --------------------------------------------- |
| Today / Inbox payload  | `my_open_work`  | 5 分     | アサイン Issue + レビュー依頼 PR の集約結果   |

定数は `cache_types::MY_OPEN_WORK` / `cache_durations::MY_OPEN_WORK` で集中管理。

### コマンド

#### `get_my_open_work_with_cache`

```rust
#[tauri::command]
pub async fn get_my_open_work_with_cache(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CachedResponse<MyOpenWork>, String>
```

`CachedResponse<T>` は `commands::github` で定義されている既存の envelope を再利用。

**動作フロー:**

1. `IssuesClient::search_assigned_issues` を呼ぶ（`is:open is:issue assignee:@me`）。
2. 1 が成功した場合のみ続けて `IssuesClient::search_review_requested` を呼ぶ
   （`is:open is:pr review-requested:@me`）。最初の呼び出しで失敗していれば
   2 回目はスキップ（不要な API 消費を避けるため）。
3. 両方成功 → `MyOpenWork` を組み立て、`activity_cache` に上書き保存
   → `CachedResponse { from_cache: false }` を返す。
4. どちらかが失敗:
   - **401 Unauthorized**: `handle_unauthorized` を発火し、エラーをそのまま返す（キャッシュは触らない）。
   - **ネットワーク / レート制限エラー**: `get_any_cache` で過去キャッシュを取得し、
     - キャッシュあり → `CachedResponse { from_cache: true, cached_at, expires_at }` を返す
     - キャッシュなし → エラーを返す
   - **その他の API エラー**: フォールバックせずエラーを返す

### Search API クエリ

| 種別             | クエリ                                                  |
| ---------------- | ------------------------------------------------------- |
| Assigned Issues  | `is:open is:issue assignee:@me archived:false`          |
| Review Requested | `is:open is:pr review-requested:@me archived:false`     |

`archived:false` を付けることで、ユーザーがウォッチしていないアーカイブ済み
リポジトリのノイズを除外する。

### `repository_url` のパース

Search API の応答には `repository` ネストがなく、`repository_url`
（`https://api.github.com/repos/{owner}/{repo}`）しか含まれない。
`GitHubSearchItem::owner_and_repo` がこれを `(owner, repo)` に分解する。
GHES など想定外ホストの場合は `None` を返し、UI には URL をそのまま表示する。

---

## Test Cases

### TC-001: アサイン Issue とレビュー依頼の集計

- **Given**: GitHub API が assigned に 2 件、review_requested に 1 件返す
- **When**: `get_my_open_work_with_cache` を呼ぶ
- **Then**:
  - `assigned.len() == 2` / `review_requested.len() == 1`
  - 各アイテムの `kind` がそれぞれ `"issue"` / `"pull_request"` で一致
  - `from_cache == false`、`cached_at` がほぼ現在時刻

### TC-002: ネットワークエラー時のキャッシュフォールバック

- **Given**: 過去のキャッシュあり、現在は Search API がネットワークエラー
- **When**: `get_my_open_work_with_cache` を呼ぶ
- **Then**:
  - `from_cache == true` で過去のペイロードが返る
  - `cached_at` / `expires_at` がキャッシュ時刻
  - 401 ハンドラは発火しない

### TC-003: ネットワークエラー + キャッシュなし

- **Given**: キャッシュ未保存、Search API がネットワークエラー
- **When**: コマンド呼び出し
- **Then**: エラー文字列が返る（"Search APIにアクセスできず、キャッシュもありません"）

### TC-004: 401 はフォールバックしない

- **Given**: キャッシュあり、Search API が 401 を返す
- **When**: コマンド呼び出し
- **Then**:
  - エラーが返る（キャッシュは返らない）
  - `handle_unauthorized` 経由でトークンがクリアされ `auth-expired` イベントが emit される

### TC-005: レート制限はフォールバックする

- **Given**: 過去のキャッシュあり、Search API が `RateLimited` を返す
- **When**: コマンド呼び出し
- **Then**: ネットワーク障害と同様にキャッシュフォールバックが発火する
- **Rationale**: 30 req/min の Search API バジェットを超過したケースでも、
  ユーザーに古いデータを見せ続けたほうが UX としてはマシ。

### TC-006: PR と Issue の判別

- **Given**: Search 応答の 1 件が `pull_request` フィールドを持つ
- **When**: `GitHubSearchItem::is_pull_request` を呼ぶ
- **Then**: `true` を返す（持たない応答に対しては `false`）

### TC-007: 想定外ホストの URL

- **Given**: `repository_url` が `https://ghe.example.com/...` のような GHES URL
- **When**: `GitHubSearchItem::owner_and_repo` を呼ぶ
- **Then**: `None` を返し、`repo_full_name()` は元の URL をそのまま返す
- **Rationale**: 誤った owner/repo に推測でリンクするより、URL を見せたほうが安全

---

## DEPENDENCY MAP

```
Parents (このファイルを使用するファイル):
  └─ src/pages/Issues/Issues.tsx (via issues.getMyOpenWorkWithCache)

Dependencies (このファイルが使用するファイル):
  ├─ src-tauri/src/github/issues.rs        (Search API client)
  ├─ src-tauri/src/database/repository/cache.rs  (save_cache / get_any_cache)
  ├─ src-tauri/src/database/models/cache.rs       (cache_types / cache_durations)
  ├─ src-tauri/src/auth/session.rs                (handle_unauthorized)
  └─ src-tauri/src/commands/github.rs             (CachedResponse 型を再利用)
```

---

## Implementation Notes

### レート制限予算

- Search API: 30 req/min（認証あり）
- 1 回の更新につき 2 リクエスト（assigned + review_requested）
- TTL 5 分なので、ユーザー 1 名あたり最悪でも約 24 req/hour に抑えられる
- `useCachedFetch` の `staleTime` をバックエンドと同じ 5 分にそろえているため、
  フォーカス復帰やネットワーク再接続のたびに API を叩くことはない

### 既存 `cached_issues` との関係

- `cached_issues` は **プロジェクト単位**（1 プロジェクト = 1 リポジトリ）の
  Kanban 用キャッシュであり、TTL を持たない（`sync_project_issues` が手動で
  上書きする）。
- 本機能は **クロスリポジトリ**かつ TTL 付きで、別レイヤとして
  `activity_cache` に格納する。両者は独立して運用される。

### フロントエンド表示

`src/pages/Issues/Issues.tsx` が SWR ライクに本コマンドをラップ。
タブ（アサイン / レビュー依頼）、テキスト検索、リポジトリセレクタ、
優先度セレクタ、ソート（優先度 / 更新 / 作成）を提供する。
キャッシュ状態は `CacheStatusBanner` を再利用して通知する。

各行クリックで Tauri opener (`auth.openUrl`) 経由で GitHub の HTML URL を
システムブラウザで開く。
