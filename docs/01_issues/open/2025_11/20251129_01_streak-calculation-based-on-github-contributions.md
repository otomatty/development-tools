# Issue: ストリーク計算を GitHub コントリビューションベースに修正

## 概要

現在のストリーク計算がアプリ使用開始日を基準にしているため、GitHub のコントリビューショングラフに記録されている活動をベースにストリークをカウントするように修正する。

## 現状

- `src-tauri/src/database/repository.rs` の `update_streak` メソッドがアプリの使用開始日を基準にストリークを計算
- `src-tauri/src/github/client.rs` の `calculate_streak` 関数が GitHub コントリビューションカレンダーからストリークを計算しているが、DB のストリーク管理とは連携していない
- ユーザーがアプリを使い始める前の GitHub 活動がストリークに反映されない

## 要件

### 機能要件

1. **活動日の定義変更**

   - GitHub のコントリビューショングラフに 1 つ以上のコントリビューションがある日を「活動した日」とする
   - コントリビューションの種類（コミット、PR、Issue、レビュー等）は問わない

2. **ストリーク計算ロジック**

   - 連続した活動日をストリークとしてカウント
   - 今日または昨日に活動があれば、そこからさかのぼって連続活動日数を計算
   - 今日活動がなくても、昨日まで連続していればストリークは維持

3. **DB ベースの管理継続**
   - `UserStats` テーブルの `current_streak`, `longest_streak`, `last_activity_date` は引き続き使用
   - GitHub 同期時にコントリビューションカレンダーを元に値を更新

### 技術要件

1. **`update_streak` メソッドの修正**

   - GitHub コントリビューションカレンダーデータを引数として受け取る
   - カレンダーデータからストリークを計算して更新

2. **`sync_github_stats` コマンドの修正**
   - コントリビューションカレンダーを取得後、`update_streak` に渡す

## 影響範囲

### 修正ファイル

- `src-tauri/src/database/repository.rs` - `update_streak` メソッド
- `src-tauri/src/commands/github.rs` - `sync_github_stats` コマンド
- `src-tauri/src/github/client.rs` - `calculate_streak` 関数（必要に応じて）

### 影響コンポーネント

- `src/components/home/stats_display.rs` - ストリーク表示（変更不要）
- `src/components/home/profile_card.rs` - プロフィールカードのストリーク表示（変更不要）

## テストケース

1. **TC-001**: GitHub で 7 日間連続コントリビューションがある場合、ストリークが 7 と表示される
2. **TC-002**: 昨日まで連続コントリビューションがあり今日はまだない場合、ストリークが維持される
3. **TC-003**: 2 日前で途切れている場合、ストリークが 0 にリセットされる
4. **TC-004**: アプリ使用開始前のコントリビューションもストリークにカウントされる
5. **TC-005**: longest_streak が正しく更新される

## 優先度

高 - ゲーミフィケーションの核となる機能

## 関連ドキュメント

- `docs/03_plans/level-system/20251126_implementation_plan.md`
