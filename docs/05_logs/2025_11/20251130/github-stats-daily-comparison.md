# GitHub 統計 前日比表示機能 実装ログ

## 日付

2025-11-30

## 関連 Issue

- GitHub Issue #35: GitHub 統計の前日比表示機能

## 実装概要

GitHub 統計の日次スナップショットを保存し、前日との差分を計算・表示する機能を実装しました。

## 変更ファイル一覧

### バックエンド（src-tauri/）

1. **新規作成**

   - `src/database/models/github_stats_snapshot.rs` - スナップショットモデルと差分計算
   - `src/database/models/github_stats_snapshot.spec.md` - 仕様書
   - `src/database/repository/github_stats_snapshot.rs` - リポジトリ層

2. **修正**
   - `src/database/migrations.rs` - version 6 追加（github_stats_snapshots テーブル）
   - `src/database/models/mod.rs` - モジュール追加
   - `src/database/repository/mod.rs` - モジュール追加
   - `src/commands/github.rs` - SyncResult に stats_diff 追加、スナップショット保存・差分計算ロジック

### フロントエンド（src/）

1. **修正**
   - `src/types/gamification.rs` - StatsDiffResult 型追加、SyncResult 更新
   - `src/components/home/stats_display.rs` - StatCard に差分表示機能追加
   - `src/components/home/mod.rs` - stats_diff シグナル追加、sync 結果から diff 取得

## データベーススキーマ

```sql
CREATE TABLE github_stats_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    total_commits INTEGER NOT NULL DEFAULT 0,
    total_prs INTEGER NOT NULL DEFAULT 0,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    total_issues INTEGER NOT NULL DEFAULT 0,
    total_stars_received INTEGER NOT NULL DEFAULT 0,
    total_contributions INTEGER NOT NULL DEFAULT 0,
    snapshot_date TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(user_id, snapshot_date)
);
```

## 主要な型定義

### GitHubStatsSnapshot

- 日次の統計スナップショットを保持
- user_id, snapshot_date でユニーク制約

### StatsDiff

- 2 つのスナップショット間の差分を表現
- commits_diff, prs_diff, reviews_diff, issues_diff, stars_diff, contributions_diff

### StatsDiffResult（フロントエンド用）

- バックエンドの StatsDiff をフロントエンドで使用する形式

## テスト結果

- バックエンドテスト: 169 passed
- フロントエンドコンパイル: OK

## UI 変更

StatCard コンポーネントに差分表示を追加:

- 増加: ↑ (緑色)
- 減少: ↓ (赤色)
- 変化なし: → (グレー)

## 今後の課題

- [ ] 週間・月間の差分表示（オプション）
- [ ] 差分のトレンドグラフ表示
- [ ] 古いスナップショットの自動クリーンアップ
