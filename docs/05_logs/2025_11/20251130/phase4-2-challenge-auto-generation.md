# Phase 4-2: チャレンジ自動生成ロジック実装ログ

## 日付
2025-11-30

## 概要
Issue #9 Phase 4-2として、チャレンジの自動生成ロジックとGitHub同期への統合を実装した。

## 実装内容

### 1. challenge.rs モジュールの作成
**ファイル**: `src-tauri/src/database/challenge.rs`

新規作成したモジュールで、以下の機能を実装：

- `ChallengeStats`: GitHubの現在の統計情報を保持する構造体（commits, prs, reviews, issues）
- `ChallengeGeneratorConfig`: チャレンジ生成の設定（目標の倍率、最小値など）
- `RecommendedTargets`: 過去のアクティビティに基づく推奨ターゲット値
- `ChallengeTemplate`: チャレンジ生成用のテンプレート
- `HistoricalStats`: 過去4週間の統計情報

主要な関数：
- `calculate_recommended_targets()`: 履歴に基づいて推奨ターゲットを計算
- `generate_weekly_challenges()`: 週間チャレンジを生成
- `generate_daily_challenges()`: 日次チャレンジを生成
- `calculate_reward_xp()`: メトリクスと目標値からXP報酬を計算
- `calculate_challenge_period()`: チャレンジの開始・終了日時を計算
- `should_generate_daily_challenges()`: 新しい日次チャレンジが必要か判定
- `should_generate_weekly_challenges()`: 新しい週間チャレンジが必要か判定

### 2. マイグレーションの追加
**ファイル**: `src-tauri/src/database/migrations.rs`

新規マイグレーション（version 3）を追加：
- `start_stats_json` カラムをchallengesテーブルに追加
- チャレンジ開始時のGitHub統計を保存し、進捗計算に使用

### 3. repository.rsへのメソッド追加
**ファイル**: `src-tauri/src/database/repository.rs`

新規メソッド：
- `create_challenge_with_stats()`: 開始統計付きでチャレンジを作成
- `get_challenge_start_stats()`: チャレンジの開始統計を取得
- `get_last_daily_challenge_date()`: 最後の日次チャレンジの日付を取得
- `get_last_weekly_challenge_date()`: 最後の週間チャレンジの日付を取得

### 4. GitHub同期への統合
**ファイル**: `src-tauri/src/commands/github.rs`

`sync_github_stats`関数に以下のロジックを追加：

1. **チャレンジ統計の構築**
   - GitHubStatsからChallengeStatsを作成
   - JSON形式でシリアライズ

2. **日次チャレンジの自動生成**
   - 前回の日次チャレンジ日を確認
   - 新しい日の場合、日次チャレンジを生成
   - 現在のGitHub統計を開始統計として保存

3. **週間チャレンジの自動生成**
   - 前回の週間チャレンジ日を確認
   - 新しい週（月曜日以降）の場合、週間チャレンジを生成
   - 現在のGitHub統計を開始統計として保存

4. **アクティブチャレンジの進捗更新**
   - すべてのアクティブチャレンジを取得
   - 各チャレンジの開始統計と現在の統計の差分を計算
   - データベースの進捗を更新

5. **期限切れチャレンジの処理**
   - 期限切れのチャレンジを失敗状態に変更

## テスト結果
18個のチャレンジ関連テストが全て通過：
- `test_calculate_recommended_targets`
- `test_calculate_reward_xp`
- `test_calculate_progress_for_metric`
- `test_calculate_challenge_period_daily`
- `test_calculate_challenge_period_weekly`
- `test_should_generate_daily_challenges`
- `test_generate_default_weekly_challenges`
- その他repository関連テスト

## 次のステップ
Phase 4-3: フロントエンドのチャレンジUI実装
- チャレンジリスト表示コンポーネント
- 進捗表示（プログレスバー）
- チャレンジ詳細モーダル

## 関連ファイル
- `src-tauri/src/database/challenge.rs` (新規)
- `src-tauri/src/database/mod.rs` (更新)
- `src-tauri/src/database/migrations.rs` (更新)
- `src-tauri/src/database/repository.rs` (更新)
- `src-tauri/src/commands/github.rs` (更新)
