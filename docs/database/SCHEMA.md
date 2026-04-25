# データベーススキーマ

Development Tools で使用する SQLite データベースのスキーマ仕様です。

---

## 📋 目次

- [概要](#概要)
- [テーブル一覧](#テーブル一覧)
- [ER 図](#er図)
- [テーブル詳細](#テーブル詳細)
- [インデックス](#インデックス)
- [マイグレーション](#マイグレーション)

---

## 概要

### データベースファイルの場所

| OS      | パス                                                                       |
| ------- | -------------------------------------------------------------------------- |
| macOS   | `~/Library/Application Support/com.development-tools/development_tools.db` |
| Linux   | `~/.local/share/com.development-tools/development_tools.db`                |
| Windows | `%APPDATA%\com.development-tools\development_tools.db`                     |

### 技術仕様

- **DBMS**: SQLite 3
- **ORM/ドライバ**: sqlx (Rust)
- **モード**: WAL (Write-Ahead Logging)

---

## テーブル一覧

| テーブル名             | 説明                       | マイグレーション |
| ---------------------- | -------------------------- | ---------------- |
| `_migrations`          | マイグレーション追跡       | -                |
| `users`                | ユーザー情報               | v1               |
| `user_stats`           | ユーザー統計（XP、レベル） | v1               |
| `badges`               | 獲得バッジ                 | v1               |
| `challenges`           | チャレンジ                 | v1, v3           |
| `xp_history`           | XP 履歴                    | v1               |
| `activity_cache`       | API レスポンスキャッシュ   | v1               |
| `app_settings`         | アプリ設定                 | v1               |
| `user_settings`        | ユーザー設定               | v2               |
| `daily_code_stats`     | 日次コード統計             | v5               |
| `sync_metadata`        | 同期メタデータ             | v5               |

---

## ER 図

```
┌─────────────────┐
│     users       │
├─────────────────┤
│ id (PK)         │
│ github_id       │
│ username        │
│ avatar_url      │
│ access_token_*  │
│ refresh_token_* │
│ ...             │
└────────┬────────┘
         │
         │ 1:1
         ▼
┌─────────────────┐       ┌─────────────────┐
│   user_stats    │       │  user_settings  │
├─────────────────┤       ├─────────────────┤
│ user_id (FK)    │       │ user_id (FK)    │
│ total_xp        │       │ notification_*  │
│ current_level   │       │ sync_*          │
│ current_streak  │       │ animations_*    │
│ ...             │       │ ...             │
└─────────────────┘       └─────────────────┘

         │
         │ 1:N
         ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│     badges      │       │   xp_history    │       │   challenges    │
├─────────────────┤       ├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │       │ id (PK)         │
│ user_id (FK)    │       │ user_id (FK)    │       │ user_id (FK)    │
│ badge_type      │       │ action_type     │       │ challenge_type  │
│ badge_id        │       │ xp_amount       │       │ target_metric   │
│ earned_at       │       │ description     │       │ target_value    │
└─────────────────┘       │ ...             │       │ current_value   │
                          └─────────────────┘       │ reward_xp       │
                                                    │ status          │
                                                    └─────────────────┘

         │
         │ 1:N
         ▼
┌─────────────────┐       ┌─────────────────┐
│ daily_code_stats│       │ activity_cache  │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ user_id (FK)    │       │ user_id (FK)    │
│ date            │       │ data_type       │
│ additions       │       │ data_json       │
│ deletions       │       │ expires_at      │
│ ...             │       └─────────────────┘
└─────────────────┘

┌─────────────────┐
│  app_settings   │
├─────────────────┤
│ key (PK)        │
│ value           │
└─────────────────┘
```

---

## テーブル詳細

### `_migrations`

マイグレーションの適用履歴を追跡するシステムテーブル。

| カラム       | 型       | 制約                      | 説明                       |
| ------------ | -------- | ------------------------- | -------------------------- |
| `version`    | INTEGER  | PRIMARY KEY               | マイグレーションバージョン |
| `name`       | TEXT     | NOT NULL                  | マイグレーション名         |
| `applied_at` | DATETIME | DEFAULT CURRENT_TIMESTAMP | 適用日時                   |

---

### `users`

GitHub ユーザー情報を保存。

| カラム                    | 型       | 制約                      | 説明                       |
| ------------------------- | -------- | ------------------------- | -------------------------- |
| `id`                      | INTEGER  | PRIMARY KEY AUTOINCREMENT | 内部 ID                    |
| `github_id`               | INTEGER  | UNIQUE NOT NULL           | GitHub ID                  |
| `username`                | TEXT     | NOT NULL                  | GitHub ユーザー名          |
| `avatar_url`              | TEXT     | -                         | アバター URL               |
| `access_token_encrypted`  | TEXT     | NOT NULL                  | 暗号化アクセストークン     |
| `refresh_token_encrypted` | TEXT     | -                         | 暗号化リフレッシュトークン |
| `token_expires_at`        | DATETIME | -                         | トークン有効期限           |
| `created_at`              | DATETIME | DEFAULT CURRENT_TIMESTAMP | 作成日時                   |
| `updated_at`              | DATETIME | DEFAULT CURRENT_TIMESTAMP | 更新日時                   |

---

### `user_stats`

ユーザーのゲーミフィケーション統計。

| カラム               | 型       | 制約                      | 説明           |
| -------------------- | -------- | ------------------------- | -------------- |
| `id`                 | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID             |
| `user_id`            | INTEGER  | UNIQUE NOT NULL, FK       | ユーザー ID    |
| `total_xp`           | INTEGER  | DEFAULT 0                 | 累計 XP        |
| `current_level`      | INTEGER  | DEFAULT 1                 | 現在レベル     |
| `current_streak`     | INTEGER  | DEFAULT 0                 | 現在ストリーク |
| `longest_streak`     | INTEGER  | DEFAULT 0                 | 最長ストリーク |
| `last_activity_date` | DATE     | -                         | 最終活動日     |
| `total_commits`      | INTEGER  | DEFAULT 0                 | 累計コミット数 |
| `total_prs`          | INTEGER  | DEFAULT 0                 | 累計 PR 数     |
| `total_reviews`      | INTEGER  | DEFAULT 0                 | 累計レビュー数 |
| `total_issues`       | INTEGER  | DEFAULT 0                 | 累計 Issue 数  |
| `updated_at`         | DATETIME | DEFAULT CURRENT_TIMESTAMP | 更新日時       |

---

### `badges`

ユーザーが獲得したバッジ。

| カラム       | 型       | 制約                      | 説明        |
| ------------ | -------- | ------------------------- | ----------- |
| `id`         | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID          |
| `user_id`    | INTEGER  | NOT NULL, FK              | ユーザー ID |
| `badge_type` | TEXT     | NOT NULL                  | バッジ種別  |
| `badge_id`   | TEXT     | NOT NULL                  | バッジ ID   |
| `earned_at`  | DATETIME | DEFAULT CURRENT_TIMESTAMP | 獲得日時    |

**ユニーク制約**: `(user_id, badge_id)`

---

### `challenges`

デイリー/ウィークリーチャレンジ。

| カラム             | 型       | 制約                      | 説明                             |
| ------------------ | -------- | ------------------------- | -------------------------------- |
| `id`               | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID                               |
| `user_id`          | INTEGER  | NOT NULL, FK              | ユーザー ID                      |
| `challenge_type`   | TEXT     | NOT NULL                  | "daily" or "weekly"              |
| `target_metric`    | TEXT     | NOT NULL                  | 目標メトリクス                   |
| `target_value`     | INTEGER  | NOT NULL                  | 目標値                           |
| `current_value`    | INTEGER  | DEFAULT 0                 | 現在値                           |
| `reward_xp`        | INTEGER  | NOT NULL                  | 報酬 XP                          |
| `start_date`       | DATETIME | NOT NULL                  | 開始日時                         |
| `end_date`         | DATETIME | NOT NULL                  | 終了日時                         |
| `status`           | TEXT     | DEFAULT 'active'          | "active", "completed", "expired" |
| `completed_at`     | DATETIME | -                         | 完了日時                         |
| `start_stats_json` | TEXT     | -                         | 開始時の GitHub 統計（JSON）     |

---

### `xp_history`

XP 獲得履歴。

| カラム            | 型       | 制約                      | 説明               |
| ----------------- | -------- | ------------------------- | ------------------ |
| `id`              | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID                 |
| `user_id`         | INTEGER  | NOT NULL, FK              | ユーザー ID        |
| `action_type`     | TEXT     | NOT NULL                  | アクション種別     |
| `xp_amount`       | INTEGER  | NOT NULL                  | XP 量              |
| `description`     | TEXT     | -                         | 説明               |
| `github_event_id` | TEXT     | -                         | GitHub イベント ID |
| `created_at`      | DATETIME | DEFAULT CURRENT_TIMESTAMP | 作成日時           |

---

### `activity_cache`

GitHub API レスポンスのキャッシュ。

| カラム       | 型       | 制約                      | 説明        |
| ------------ | -------- | ------------------------- | ----------- |
| `id`         | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID          |
| `user_id`    | INTEGER  | NOT NULL, FK              | ユーザー ID |
| `data_type`  | TEXT     | NOT NULL                  | データ種別  |
| `data_json`  | TEXT     | NOT NULL                  | JSON データ |
| `fetched_at` | DATETIME | DEFAULT CURRENT_TIMESTAMP | 取得日時    |
| `expires_at` | DATETIME | NOT NULL                  | 有効期限    |

**ユニーク制約**: `(user_id, data_type)`

---

### `app_settings`

アプリケーション全体の設定（キーバリュー形式）。

| カラム       | 型       | 制約                      | 説明     |
| ------------ | -------- | ------------------------- | -------- |
| `key`        | TEXT     | PRIMARY KEY               | 設定キー |
| `value`      | TEXT     | NOT NULL                  | 設定値   |
| `updated_at` | DATETIME | DEFAULT CURRENT_TIMESTAMP | 更新日時 |

---

### `user_settings`

ユーザー個別の設定。

| カラム                    | 型       | 制約                      | 説明                         |
| ------------------------- | -------- | ------------------------- | ---------------------------- |
| `id`                      | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID                           |
| `user_id`                 | INTEGER  | UNIQUE NOT NULL, FK       | ユーザー ID                  |
| `notification_method`     | TEXT     | DEFAULT 'both'            | 通知方法                     |
| `notify_xp_gain`          | INTEGER  | DEFAULT 1                 | XP 獲得通知                  |
| `notify_level_up`         | INTEGER  | DEFAULT 1                 | レベルアップ通知             |
| `notify_badge_earned`     | INTEGER  | DEFAULT 1                 | バッジ獲得通知               |
| `notify_streak_update`    | INTEGER  | DEFAULT 1                 | ストリーク更新通知           |
| `notify_streak_milestone` | INTEGER  | DEFAULT 1                 | ストリークマイルストーン通知 |
| `sync_interval_minutes`   | INTEGER  | DEFAULT 60                | 同期間隔（分）               |
| `background_sync`         | INTEGER  | DEFAULT 1                 | バックグラウンド同期         |
| `sync_on_startup`         | INTEGER  | DEFAULT 1                 | 起動時同期                   |
| `animations_enabled`      | INTEGER  | DEFAULT 1                 | アニメーション有効           |
| `created_at`              | DATETIME | DEFAULT CURRENT_TIMESTAMP | 作成日時                     |
| `updated_at`              | DATETIME | DEFAULT CURRENT_TIMESTAMP | 更新日時                     |

---

### `daily_code_stats`

日次のコード統計（additions/deletions）。

| カラム              | 型       | 制約                      | 説明                        |
| ------------------- | -------- | ------------------------- | --------------------------- |
| `id`                | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID                          |
| `user_id`           | INTEGER  | NOT NULL, FK              | ユーザー ID                 |
| `date`              | DATE     | NOT NULL                  | 日付                        |
| `additions`         | INTEGER  | NOT NULL DEFAULT 0        | 追加行数                    |
| `deletions`         | INTEGER  | NOT NULL DEFAULT 0        | 削除行数                    |
| `commits_count`     | INTEGER  | NOT NULL DEFAULT 0        | コミット数                  |
| `repositories_json` | TEXT     | -                         | リポジトリ一覧（JSON 配列） |
| `created_at`        | DATETIME | DEFAULT CURRENT_TIMESTAMP | 作成日時                    |
| `updated_at`        | DATETIME | DEFAULT CURRENT_TIMESTAMP | 更新日時                    |

**ユニーク制約**: `(user_id, date)`

---

### `sync_metadata`

増分同期のメタデータ。

| カラム                 | 型       | 制約                      | 説明                   |
| ---------------------- | -------- | ------------------------- | ---------------------- |
| `id`                   | INTEGER  | PRIMARY KEY AUTOINCREMENT | ID                     |
| `user_id`              | INTEGER  | NOT NULL, FK              | ユーザー ID            |
| `sync_type`            | TEXT     | NOT NULL                  | 同期種別               |
| `last_sync_at`         | DATETIME | -                         | 最終同期日時           |
| `last_sync_cursor`     | TEXT     | -                         | GraphQL カーソル       |
| `etag`                 | TEXT     | -                         | ETag ヘッダー          |
| `rate_limit_remaining` | INTEGER  | -                         | レート制限残数         |
| `rate_limit_reset_at`  | DATETIME | -                         | レート制限リセット日時 |
| `created_at`           | DATETIME | DEFAULT CURRENT_TIMESTAMP | 作成日時               |
| `updated_at`           | DATETIME | DEFAULT CURRENT_TIMESTAMP | 更新日時               |

**ユニーク制約**: `(user_id, sync_type)`

---

## インデックス

### パフォーマンス用インデックス

```sql
-- badges
CREATE INDEX idx_badges_user_id ON badges(user_id);

-- challenges
CREATE INDEX idx_challenges_user_id ON challenges(user_id);
CREATE INDEX idx_challenges_status ON challenges(status);

-- xp_history
CREATE INDEX idx_xp_history_user_id ON xp_history(user_id);
CREATE INDEX idx_xp_history_created_at ON xp_history(created_at);

-- activity_cache
CREATE INDEX idx_activity_cache_expires ON activity_cache(expires_at);
CREATE INDEX idx_activity_cache_user_type ON activity_cache(user_id, data_type);

-- user_settings
CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);

-- daily_code_stats
CREATE INDEX idx_daily_code_stats_user_date ON daily_code_stats(user_id, date DESC);
CREATE INDEX idx_daily_code_stats_summary ON daily_code_stats(user_id, date, additions, deletions);

-- sync_metadata
CREATE INDEX idx_sync_metadata_user_type ON sync_metadata(user_id, sync_type);
```

---

## マイグレーション

### マイグレーション履歴

| Version | Name                                       | 説明                                                                                            |
| ------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| 1       | `initial_schema`                           | 初期スキーマ（users, user_stats, badges, challenges, xp_history, activity_cache, app_settings） |
| 2       | `add_user_settings`                        | ユーザー設定テーブル追加                                                                        |
| 3       | `add_challenge_start_stats`                | チャレンジに開始時統計カラム追加                                                                |
| 5       | `add_code_stats_tables`                    | コード統計テーブル追加                                                                          |
| 6       | `add_github_stats_snapshots`               | GitHub 統計スナップショットテーブル追加                                                         |
| 7       | `add_issue_management_tables`              | Issue 管理テーブル追加（projects, cached_issues）                                               |
| 8       | `add_xp_history_breakdown`                 | xp_history に詳細内訳カラム追加                                                                 |
| 9       | `drop_legacy_static_file_server_tables`    | 旧静的ファイルサーバー関連テーブル・インデックスを削除（Issue #175）                            |

### マイグレーションの仕組み

1. アプリ起動時に`run_migrations()`が実行される
2. `_migrations`テーブルで適用済みバージョンを確認
3. 未適用のマイグレーションを順番に実行
4. 適用後、`_migrations`テーブルに記録

### 新規マイグレーションの追加方法

`src-tauri/src/database/migrations.rs`に新しい`Migration`を追加：

```rust
Migration {
    version: 6,
    name: "add_new_feature",
    sql: r#"
-- SQL statements here
CREATE TABLE IF NOT EXISTS new_table (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ...
);
"#,
},
```

---

## データ型の注意点

### BOOLEAN

SQLite には BOOLEAN 型がないため、INTEGER（0/1）で代用。

```sql
-- 0 = false, 1 = true
animations_enabled INTEGER DEFAULT 1
```

### DATETIME

ISO 8601 形式の文字列で保存。

```sql
-- 例: "2025-11-30T12:34:56Z"
created_at DATETIME DEFAULT CURRENT_TIMESTAMP
```

### JSON

TEXT 型に JSON 文字列として保存。

```sql
-- 例: '["repo1", "repo2"]'
repositories_json TEXT
```

---

## バックアップと復元

### バックアップ

```bash
# データベースファイルをコピー
cp ~/Library/Application\ Support/com.development-tools/development_tools.db backup.db
```

### 復元

```bash
# バックアップから復元
cp backup.db ~/Library/Application\ Support/com.development-tools/development_tools.db
```

### データエクスポート（アプリ内）

設定ページの「データ管理」→「エクスポート」から JSON ファイルとしてエクスポート可能。
