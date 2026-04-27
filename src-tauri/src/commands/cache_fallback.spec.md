# Cache Fallback Specification

オフライン時にキャッシュされたデータを返すためのフォールバック機能仕様。

## Related Files

- Implementation: `src-tauri/src/commands/github.rs`
- Cache Repository: `src-tauri/src/database/repository/cache.rs`
- Cache Models: `src-tauri/src/database/models/cache.rs`
- Frontend Integration: `src/components/home/mod.rs`

## Related Documentation

- Network Status: `src/components/network_status.spec.md`
- Database Schema: `docs/database/SCHEMA.md`
- Issue: GitHub Issue #10 (Phase 5: Offline Support & Caching)

---

## Requirements

### 責務

1. **API 呼び出し成功時**: レスポンスをキャッシュに保存し、新鮮なデータを返す
2. **API 呼び出し失敗時（オフライン）**: キャッシュされたデータを返し、キャッシュ由来であることを示す
3. **キャッシュなし + オフライン**: エラーを返す

### データ構造

#### CachedResponse<T>

API 呼び出し結果とキャッシュ情報を含むレスポンス型。

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedResponse<T> {
    /// 実際のデータ
    pub data: T,
    /// キャッシュから取得したかどうか
    pub from_cache: bool,
    /// キャッシュされた日時（ISO8601形式）
    pub cached_at: Option<String>,
    /// キャッシュの有効期限（ISO8601形式）
    pub expires_at: Option<String>,
}
```

### キャッシュ対象

| データタイプ | cache_type キー | 有効期限 | 説明                       |
| ------------ | --------------- | -------- | -------------------------- |
| GitHub Stats | `github_stats`  | 30 分    | ユーザーの GitHub 統計情報 |
| User Stats   | `user_stats`    | 60 分    | ゲーミフィケーション統計   |
| Level Info   | `level_info`    | 60 分    | レベル情報                 |

### コマンド変更

#### get_github_stats_with_cache

既存の `get_github_stats` を拡張し、キャッシュフォールバックを追加。

```rust
#[command]
pub async fn get_github_stats_with_cache(
    state: State<'_, AppState>
) -> Result<CachedResponse<GitHubStats>, String>
```

**動作フロー:**

1. GitHubStats API を呼び出し
2. 成功 → キャッシュに保存 → `CachedResponse { data, from_cache: false, ... }` を返す
3. 失敗 → キャッシュから取得を試行
   - キャッシュあり → `CachedResponse { data, from_cache: true, cached_at, ... }` を返す
   - キャッシュなし → エラーを返す

#### get_user_stats_with_cache

既存の `get_user_stats` を拡張。

```rust
#[command]
pub async fn get_user_stats_with_cache(
    state: State<'_, AppState>
) -> Result<CachedResponse<UserStats>, String>
```

---

## Test Cases

### TC-001: API 成功時のキャッシュ保存

- **Given**: オンライン状態
- **When**: `get_github_stats_with_cache` を呼び出す
- **Then**:
  - `from_cache` が `false`
  - キャッシュに新しいデータが保存される
  - `cached_at` が現在時刻に近い

### TC-002: API 失敗時のキャッシュフォールバック

- **Given**: オフライン状態、キャッシュにデータあり
- **When**: `get_github_stats_with_cache` を呼び出す
- **Then**:
  - `from_cache` が `true`
  - `cached_at` にキャッシュ保存時刻が含まれる
  - データがキャッシュされた内容と一致

### TC-003: API 失敗 + キャッシュなし

- **Given**: オフライン状態、キャッシュにデータなし
- **When**: `get_github_stats_with_cache` を呼び出す
- **Then**: エラーが返される（"データがキャッシュされていません"）

### TC-004: キャッシュの更新

- **Given**: 古いキャッシュが存在
- **When**: `get_github_stats_with_cache` を呼び出し、API が成功
- **Then**:
  - 古いキャッシュが新しいデータで上書きされる
  - `from_cache` が `false`

### TC-005: UserStats のキャッシュフォールバック

- **Given**: オフライン状態、UserStats キャッシュあり
- **When**: `get_user_stats_with_cache` を呼び出す
- **Then**:
  - `from_cache` が `true`
  - UserStats データが返される

---

## DEPENDENCY MAP

```
Parents (このファイルを使用するファイル):
  └─ src/components/home/mod.rs

Dependencies (このファイルが使用するファイル):
  ├─ src-tauri/src/database/repository/cache.rs
  ├─ src-tauri/src/database/models/cache.rs
  └─ src-tauri/src/github/mod.rs
```

---

## Implementation Notes

### キャッシュキー命名規則

```
{data_type}_{user_id}
```

例: `github_stats` (user_id は DB クエリで使用)

### エラーハンドリング

- ネットワークエラー → キャッシュフォールバック試行
- 認証エラー → キャッシュフォールバック試行しない（エラーをそのまま返す）
- キャッシュ取得エラー → 元のエラーを返す

### フロントエンド表示

キャッシュデータ使用時は以下を表示:

- 「キャッシュデータを表示中」のインジケーター
- 最終更新日時（cached_at）

---

## 同期スケジューラとの関係

GitHub 統計の自動同期は `crate::sync_scheduler` が担当する。スケジューラはユーザー設定（`sync_on_startup` / `sync_interval_minutes` / `background_sync`）に応じて
バックグラウンドで `sync_github_stats` を駆動し、結果は本キャッシュ層と同様に DB
に保存される。

| トリガー | 担当 | 備考 |
| --- | --- | --- |
| 起動時同期 (`sync_on_startup=true`) | `sync_scheduler::runner` | 初回ループで `RunSync` |
| 定期同期 (`sync_interval_minutes>0`) | `sync_scheduler::runner` | 経過時間で `RunSync` を発火 |
| バックグラウンド OFF (`background_sync=false`) | `sync_scheduler::runner` | `Idle` 状態に入り、設定変更を待つ |
| 手動同期 | `SyncSettings` の「今すぐ同期」 | 直接 `sync_github_stats` を呼ぶ |
| キャッシュフォールバック | 本仕様 (`*_with_cache`) | API 失敗時のみ |

スケジューラ側の詳細仕様: `src-tauri/src/sync_scheduler/sync_scheduler.spec.md`

レート制限到達時は、スケジューラが `sync_metadata.last_skipped_reason = "rate_limited"`
を保存し、`get_scheduler_status` 経由で `SyncSettings` 画面に表示される。本キャッシュ層は
レート制限を含むネットワーク障害でフォールバックを発火する点で、両者は補完関係にある。
