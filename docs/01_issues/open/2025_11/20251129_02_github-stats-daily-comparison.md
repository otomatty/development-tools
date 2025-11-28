# Issue: GitHub 統計の前日比表示機能

## 概要

ダッシュボードの GitHub 統計に前日比（差分）を表示し、日々の成長を可視化する。

## 現状

- `GitHubStats` には累計値のみが保存されている
- 前回同期時の値を保持していないため、差分計算ができない
- ユーザーは数値の変化を把握しにくい

## 要件

### 機能要件

1. **前日比の表示**

   - コミット数、PR 数、レビュー数、Issue 数、スター数の前日比を表示
   - 増加は緑色（+N）、減少は赤色（-N）、変化なしはグレー（±0）で表示
   - 矢印アイコン（↑↓→）で視覚的に変化を示す

2. **比較対象**
   - 前回の同期時点（24 時間以上前）との比較
   - 同日内の複数同期は最新値のみ更新し、比較基準は変えない

### 技術要件

1. **新規テーブル `github_stats_snapshots`**

   ```sql
   CREATE TABLE github_stats_snapshots (
       id INTEGER PRIMARY KEY,
       user_id INTEGER NOT NULL,
       total_commits INTEGER NOT NULL,
       total_prs INTEGER NOT NULL,
       total_reviews INTEGER NOT NULL,
       total_issues INTEGER NOT NULL,
       total_stars_received INTEGER NOT NULL,
       total_contributions INTEGER NOT NULL,
       snapshot_date DATE NOT NULL,
       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
       FOREIGN KEY (user_id) REFERENCES users(id),
       UNIQUE(user_id, snapshot_date)
   );
   ```

2. **スナップショット保存ロジック**

   - 同期時に当日のスナップショットがなければ作成
   - 既存スナップショットがあれば更新

3. **差分計算 API**

   - 前日のスナップショットを取得
   - 現在値との差分を計算して返却

4. **フロントエンド表示**
   - `StatsDisplay` コンポーネントに差分表示を追加
   - `StatCard` コンポーネントに差分プロパティを追加

## 影響範囲

### 新規ファイル

- なし（既存ファイルに追加）

### 修正ファイル

- `src-tauri/src/database/migrations.rs` - マイグレーション追加
- `src-tauri/src/database/repository.rs` - スナップショット保存・取得メソッド
- `src-tauri/src/database/models.rs` - `GitHubStatsSnapshot` モデル
- `src-tauri/src/commands/github.rs` - スナップショット保存・差分計算
- `src/types.rs` - フロントエンド用型定義
- `src/components/home/stats_display.rs` - 差分表示 UI

## UI/UX デザイン

```
┌─────────────────────┐
│ 📝 Total Commits    │
│     1,234  ↑ +12    │
│            (green)  │
└─────────────────────┘

┌─────────────────────┐
│ 🔀 Pull Requests    │
│       56   → ±0     │
│            (gray)   │
└─────────────────────┘
```

## テストケース

1. **TC-001**: 初回同期時はスナップショットが作成され、差分は表示されない（または 0）
2. **TC-002**: 2 回目以降の同期で前日との差分が正しく計算される
3. **TC-003**: 同日の複数同期で比較基準が変わらない
4. **TC-004**: 増加・減少・変化なしが正しい色とアイコンで表示される
5. **TC-005**: 前日のスナップショットがない場合は直近のスナップショットと比較

## 優先度

中 - UX 向上機能

## 関連 Issue

- なし
