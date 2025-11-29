# Issue: バッジシステムの拡充

## 概要

バッジの種類を大幅に増やし、様々なパターン・レベルのバッジを用意する。また、ダッシュボードには取得済みバッジと取得間近のバッジのみを表示するよう変更する。

## 現状

- `src-tauri/src/database/models.rs` の `badge` モジュールに約 15 個のバッジが定義
- カテゴリ: milestone, streak, collaboration, quality
- レアリティ: bronze, silver, gold, platinum
- ダッシュボードに全バッジを表示

## 要件

### 機能要件

#### 1. 新規バッジカテゴリの追加

**時間帯バッジ (time_based)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| early_bird | Early Bird | 午前 6 時前にコミット | bronze | 早朝コミット 1 回 |
| night_owl | Night Owl | 深夜 0 時以降にコミット | bronze | 深夜コミット 1 回 |
| weekend_warrior | Weekend Warrior | 週末に 10 回コミット | silver | 週末コミット累計 10 回 |
| all_nighter | All Nighter | 24 時間以内に 10 コミット | gold | 24 時間で 10 コミット |

**言語マスターバッジ (language)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| polyglot_3 | Trilingual | 3 言語を使用 | bronze | 3 言語 |
| polyglot_5 | Polyglot | 5 言語を使用 | silver | 5 言語 |
| polyglot_10 | Language Master | 10 言語を使用 | gold | 10 言語 |

**レベル達成バッジ (level)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| level_5 | Rising Star | レベル 5 達成 | bronze | Lv.5 |
| level_10 | Skilled Dev | レベル 10 達成 | bronze | Lv.10 |
| level_25 | Expert | レベル 25 達成 | silver | Lv.25 |
| level_50 | Master | レベル 50 達成 | gold | Lv.50 |
| level_100 | Grandmaster | レベル 100 達成 | platinum | Lv.100 |

**継続バッジ (consistency)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| weekly_3 | Week Warrior | 3 週連続活動 | bronze | 3 週連続 |
| weekly_12 | Quarter Champion | 12 週連続活動 | silver | 12 週連続 |
| monthly_6 | Half Year Hero | 6 ヶ月連続活動 | gold | 6 ヶ月連続 |
| monthly_12 | Year Legend | 12 ヶ月連続活動 | platinum | 12 ヶ月連続 |

**コラボレーション拡張バッジ (collaboration)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| first_pr | Pull Request Rookie | 最初の PR 作成 | bronze | PR 1 件 |
| pr_10 | PR Contributor | 10 件の PR 作成 | bronze | PR 10 件 |
| pr_50 | PR Expert | 50 件の PR 作成 | silver | PR 50 件 |
| pr_100 | PR Master | 100 件の PR 作成 | gold | PR 100 件 |
| reviewer_100 | Code Sage | 100 件のレビュー | gold | レビュー 100 件 |

**スター獲得バッジ (stars)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| star_1 | First Star | 最初のスターを獲得 | bronze | 1 スター |
| star_10 | Rising Repository | 10 スター獲得 | bronze | 10 スター |
| star_50 | Popular Project | 50 スター獲得 | silver | 50 スター |
| star_100 | Star Magnet | 100 スター獲得 | gold | 100 スター |
| star_1000 | Open Source Hero | 1000 スター獲得 | platinum | 1000 スター |

**マイルストーン拡張バッジ (milestone)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| commits_500 | Half Thousand | 500 コミット達成 | silver | 500 コミット |
| commits_5000 | Five Thousand | 5000 コミット達成 | gold | 5000 コミット |

**ストリーク拡張バッジ (streak)**
| ID | 名前 | 説明 | レアリティ | 条件 |
|---|---|---|---|---|
| streak_14 | Two Weeks | 14 日連続 | bronze | 14 日ストリーク |
| streak_60 | Two Months | 60 日連続 | silver | 60 日ストリーク |
| streak_90 | Quarter Year | 90 日連続 | gold | 90 日ストリーク |
| streak_180 | Half Year | 180 日連続 | gold | 180 日ストリーク |

#### 2. ダッシュボード表示の改善

- **取得済みバッジ**: 全て表示（最新取得順）
- **取得間近バッジ**: 進捗が 50%以上のバッジを表示
- **ページネーションまたは「もっと見る」**: バッジが多い場合

### 技術要件

1. **`BadgeCondition` enum の拡張**

   ```rust
   pub enum BadgeCondition {
       // 既存
       Commits { threshold: i32 },
       Streak { days: i32 },
       Reviews { threshold: i32 },
       PrsMerged { threshold: i32 },
       IssuesClosed { threshold: i32 },
       PrMergeRate { min_rate: f32, min_prs: i32 },
       Languages { count: i32 },
       // 新規
       Level { level: i32 },
       Stars { threshold: i32 },
       PrsCreated { threshold: i32 },
       WeeklyStreak { weeks: i32 },
       MonthlyStreak { months: i32 },
       TimeOfDay { hour_start: i32, hour_end: i32, count: i32 },
       WeekendCommits { threshold: i32 },
   }
   ```

2. **バッジ評価ロジックの更新**

   - `BadgeEvalContext` に新しいフィールド追加
   - 各条件タイプの評価関数を実装

3. **進捗計算関数**

   - 各バッジの進捗率（0-100%）を計算する関数
   - ダッシュボードでの「取得間近」判定に使用

4. **フロントエンド `BadgeGrid` の改善**
   - 取得済み/取得間近の分類表示
   - 進捗バー表示（オプション）

## 影響範囲

### 修正ファイル

- `src-tauri/src/database/models.rs` - バッジ定義・条件の拡張
- `src-tauri/src/commands/github.rs` - バッジ評価ロジック
- `src/components/home/badge_grid.rs` - 表示ロジックの改善
- `src/types.rs` - 進捗情報の型追加

## テストケース

1. **TC-001**: 新規バッジ定義が全て取得できる
2. **TC-002**: 各条件タイプのバッジが正しく評価される
3. **TC-003**: 進捗が 50%以上のバッジが「取得間近」として表示される
4. **TC-004**: 取得済みバッジが最新順で表示される
5. **TC-005**: レベル達成バッジがレベルアップ時に付与される

## 優先度

中 - ゲーミフィケーション強化

## 関連 Issue

- `20251129_01_streak-calculation-based-on-github-contributions.md` - ストリークバッジの評価に影響
