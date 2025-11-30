# Cache Management Specification (Step 4)

## Related Files

### Backend Implementation

- Commands: `src-tauri/src/commands/github.rs`
  - `CacheStats` struct (lines ~573-580)
  - `get_cache_stats` command
  - `clear_user_cache` command
  - `cleanup_expired_cache` command
- Startup cleanup: `src-tauri/src/lib.rs` (setup hook)
- Database: `src-tauri/src/database/models/activity_cache.rs`

### Frontend Implementation

- Types: `src/types/gamification.rs` (`CacheStats` struct)
- API: `src/tauri_api.rs`
  - `get_cache_stats()` function
  - `clear_user_cache()` function
  - `cleanup_expired_cache()` function

### Related Documentation

- Issue: GitHub Issue #10 "Phase 5: オフライン対応 & キャッシュ機能"
- Plan: `docs/03_plans/` (N/A for this issue)
- Related Specs:
  - Network Status: `src/components/network_status.spec.md`
  - Offline UI: `src/components/home/offline_ui.spec.md`

## Requirements

### 責務

- キャッシュデータの統計情報の取得
- ユーザーキャッシュの手動クリア
- 期限切れキャッシュの自動クリーンアップ
- アプリ起動時の期限切れキャッシュ削除

### 機能要件

#### R-CACHE-001: キャッシュ統計情報取得

- キャッシュの総サイズ（バイト数）を取得できる
- キャッシュエントリ数を取得できる
- 期限切れエントリ数を取得できる
- 最終クリーンアップ時刻を取得できる

#### R-CACHE-002: ユーザーキャッシュクリア

- 特定ユーザーのキャッシュのみをクリアできる
- クリア対象: GitHub 活動データ、チャレンジデータ等
- クリア後は次回アクセス時に API から再取得

#### R-CACHE-003: 期限切れキャッシュクリーンアップ

- `expires_at`が現在時刻より過去のエントリを削除
- 手動実行可能（設定画面から）
- 自動実行も可能（起動時）

#### R-CACHE-004: 起動時自動クリーンアップ

- アプリ起動時に非同期で期限切れキャッシュを削除
- メインスレッドをブロックしない
- エラー発生時もアプリ起動は継続

### データ構造

#### CacheStats

```rust
pub struct CacheStats {
    /// Total cache size in bytes
    pub total_size_bytes: u64,
    /// Number of cache entries
    pub entry_count: u64,
    /// Number of expired entries
    pub expired_count: u64,
    /// Last cleanup timestamp (ISO8601)
    pub last_cleanup_at: Option<String>,
}
```

### キャッシュ TTL 設定

| データ種別           | TTL     | 備考                         |
| -------------------- | ------- | ---------------------------- |
| GitHub 活動データ    | 30 分   | API 負荷軽減と鮮度のバランス |
| ユーザー統計         | 60 分   | ローカル DB 由来のため長めに |
| ユーザープロファイル | 24 時間 | 比較的変更頻度が低い         |
| バッジ定義           | 24 時間 | マスターデータ扱い           |

### API 設計

#### get_cache_stats

```rust
#[tauri::command]
pub async fn get_cache_stats(db_state: State<'_, DbState>) -> Result<CacheStats, String>
```

#### clear_user_cache

```rust
#[tauri::command]
pub async fn clear_user_cache(
    db_state: State<'_, DbState>,
    github_username: String,
) -> Result<(), String>
```

#### cleanup_expired_cache

```rust
#[tauri::command]
pub async fn cleanup_expired_cache(db_state: State<'_, DbState>) -> Result<u64, String>
```

- 戻り値: 削除されたエントリ数

## Test Cases

### TC-CACHE-001: キャッシュ統計取得

- Given: キャッシュにエントリが存在する
- When: `get_cache_stats`を呼び出す
- Then: `CacheStats`が正しい統計情報を含む

### TC-CACHE-002: 空のキャッシュ統計

- Given: キャッシュが空
- When: `get_cache_stats`を呼び出す
- Then: `entry_count=0`, `total_size_bytes=0`を返す

### TC-CACHE-003: ユーザーキャッシュクリア

- Given: ユーザー A のキャッシュが存在する
- When: `clear_user_cache("userA")`を呼び出す
- Then: ユーザー A のキャッシュのみが削除される
- And: 他のユーザーのキャッシュは残る

### TC-CACHE-004: 期限切れキャッシュクリーンアップ

- Given: 期限切れエントリと有効なエントリが混在
- When: `cleanup_expired_cache`を呼び出す
- Then: 期限切れエントリのみが削除される
- And: 有効なエントリは残る

### TC-CACHE-005: クリーンアップ削除数の返却

- Given: 期限切れエントリが 3 件存在
- When: `cleanup_expired_cache`を呼び出す
- Then: 戻り値が`3`である

### TC-CACHE-006: 起動時自動クリーンアップ

- Given: アプリ起動前に期限切れキャッシュが存在
- When: アプリを起動する
- Then: 期限切れキャッシュが非同期で削除される
- And: アプリ起動は即座に完了（ブロックしない）

### TC-CACHE-007: 起動時クリーンアップエラー耐性

- Given: データベースアクセスでエラーが発生
- When: アプリを起動する
- Then: エラーはログに出力されるがアプリは正常起動

## Implementation Notes

### 実装済み機能

- [x] `CacheStats`構造体（バックエンド）
- [x] `get_cache_stats`コマンド
- [x] `clear_user_cache`コマンド
- [x] `cleanup_expired_cache`コマンド
- [x] 起動時自動クリーンアップ（`lib.rs` setup hook）
- [x] フロントエンド`CacheStats`型
- [x] フロントエンド API 関数

### Step 5 実装済み機能

- [x] 設定画面でのキャッシュ管理 UI (`src/components/settings/data_management.rs`)
- [x] キャッシュ統計表示（サイズ、エントリ数、期限切れ数、オンライン/オフライン状態）
- [x] 期限切れクリーンアップボタン（期限切れエントリがある場合のみ有効）
- [x] 全キャッシュクリアボタン（エントリがある場合のみ有効）
- [x] オフラインインジケーター表示

### 備考

- SQLite の`LENGTH()`関数でデータサイズを計算
- 起動時クリーンアップは`tokio::spawn`で非同期実行
- エラーは`eprintln!`でログ出力（log crate は未使用）
