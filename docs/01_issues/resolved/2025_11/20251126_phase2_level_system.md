# Phase 2: レベルシステム実装

**Issue ID:** 7  
**Created:** 2025-11-26  
**Status:** Completed  
**Completed:** 2025-11-26  
**GitHub Issue:** [#7](https://github.com/otomatty/development-tools/issues/7)

## 概要

GitHub活動に基づく経験値・レベルシステムを実装する。

## 目標

- アクションごとにXPが正しく計算される
- 累計XPからレベルが正しく計算される
- 経験値バーがUIに表示される
- レベルアップ時にアニメーションが表示される
- XPデータがSQLiteに永続化される
- 全テストがパスする

## 実装タスク

### 1. 経験値（XP）計算ロジック

- [x] XP計算モジュールの実装（`XpActionType::xp_value()`）
- [x] 累計XP計算の実装
- [x] XP履歴の保存（SQLite）
- [x] GitHub統計からの差分XP付与機能

### 2. レベル計算ロジック

- [x] レベル計算式の実装（`level_from_xp()`）
- [x] 次レベルへの必要XP計算（`xp_to_next_level()`）
- [x] レベルアップ判定の実装
- [x] 最高レベル100の制限

### 3. 経験値バーUI

- [x] 経験値プログレスバーの実装
- [x] 現在XP / 次レベルXP の表示
- [x] レベル表示
- [x] レベルアップ時のアニメーション

### 4. XP付与イベント処理

- [x] GitHub統計の差分計算
- [x] リアルタイムXP付与
- [x] XP獲得通知

## XP獲得ルール

| アクション | XP |
|-----------|-----|
| コミット | +10 XP |
| PR作成 | +30 XP |
| PRマージ | +50 XP |
| Issue作成 | +15 XP |
| Issue解決 | +40 XP |
| PRレビュー | +25 XP |
| スター獲得 | +5 XP |
| 連続コミットボーナス | +20 XP |

## レベル計算式

```rust
// XP = 50 * (level - 1)^2
fn xp_for_level(level: u32) -> u32 {
    if level <= 1 { return 0; }
    50 * (level - 1).pow(2)
}

fn level_from_xp(total_xp: u32) -> u32 {
    let level = ((total_xp as f64 / 50.0).sqrt() + 1.0).floor() as u32;
    level.min(100) // Max level 100
}
```

## 関連ファイル

- Implementation: `src-tauri/src/database/models.rs`
- Commands: `src-tauri/src/commands/gamification.rs`
- Repository: `src-tauri/src/database/repository.rs`
- Frontend: `src/components/home/profile_card.rs`

