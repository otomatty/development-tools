# レベルシステム実装計画

**Date:** 2025-11-26  
**Issue:** [#7](https://github.com/otomatty/development-tools/issues/7)

## 実装ステップ

### Step 1: XP計算ロジックの強化

1. `sync_github_stats`の改善
   - 前回の統計との差分を計算
   - 差分に基づいてXPを付与
   - XP履歴に記録

### Step 2: レベルアップイベント検知

1. `add_xp`時にレベルアップを検知
2. レベルアップイベントをフロントエンドに通知
3. Tauriイベントシステムを使用

### Step 3: フロントエンドアニメーション

1. レベルアップモーダル/通知の作成
2. XP獲得アニメーション
3. プログレスバーのアニメーション強化

### Step 4: テスト追加

1. XP計算のテスト追加
2. 差分計算のテスト
3. レベルアップ検知のテスト

## ファイル変更一覧

### バックエンド (src-tauri/src/)

| ファイル | 変更内容 |
|---------|---------|
| `commands/gamification.rs` | レベルアップ検知、イベント発行 |
| `commands/github.rs` | sync時の差分XP付与 |
| `database/repository.rs` | 前回統計の保存・取得 |
| `database/models.rs` | テスト追加 |

### フロントエンド (src/)

| ファイル | 変更内容 |
|---------|---------|
| `components/home/profile_card.rs` | アニメーション追加 |
| `components/home/mod.rs` | イベントリスニング |
| `tauri_api.rs` | イベントリスナー追加 |
| `types.rs` | 新規型追加 |

## 完了条件

- [x] GitHub統計同期時に差分XPが付与される
- [x] レベルアップ時にイベントが発行される
- [x] フロントエンドでレベルアップアニメーションが表示される
- [x] XP獲得通知が表示される
- [x] 全テストがパスする（35テスト）

## 実装完了日

2025-11-26

