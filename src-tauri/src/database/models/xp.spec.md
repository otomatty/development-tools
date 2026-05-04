# XP Model Specification

## Related Files

- Implementation: `src-tauri/src/database/models/xp.rs`
- Caller: `src-tauri/src/commands/github.rs` (`sync_github_stats`)
- Repository: `src-tauri/src/database/repository/xp_history.rs`
- Frontend types: `src/types/gamification.rs`
- Tests: `src-tauri/src/database/models/xp.rs` (tests module)

## Related Documentation

- 公式 XP 仕様: `docs/prd/home-gamification.md` §3.3.2 経験値（XP）テーブル
- 監査レポート: `docs/02_research/2026_04/20260425_github_integration_audit.md` §6.1 / §8 G-14
- GitHub Issue: #184

## 責務

- アクション種別ごとの XP 値を **単一の真実 (single source of truth)** として保持する。
- GitHub 同期時に `XpBreakdown::calculate` で活動ごとの XP 内訳を算出する。
- ストリーク継続による XP ボーナスを計算する。

## XP ルール（公式）

XP 値は本ファイル (`xp.rs`) の `pub const` を **唯一の定義** とし、
`XpBreakdown::calculate` を含むすべての計算箇所はこの定数を参照すること。
ハードコード（例: `commits * 10`）は禁止。

| アクション   | 定数              | XP   | 備考                                         |
| ------------ | ----------------- | ---- | -------------------------------------------- |
| コミット     | `COMMIT_XP`       | +10  | 基本的な開発活動                             |
| PR 作成      | `PR_XP`           | +30  | まとまった作業の完了                         |
| PR マージ    | `PR_MERGED_XP`    | +50  | 品質を満たした成果（PR 作成 XP に加算される） |
| Issue 作成   | `ISSUE_XP`        | +15  | 問題発見・提案                               |
| Issue 解決   | `ISSUE_CLOSED_XP` | +40  | 問題解決（Issue 作成 XP に加算される）       |
| レビュー     | `REVIEW_XP`       | +25  | コラボレーション                             |
| スター獲得   | `STAR_XP`         | +5   | 他者からの評価                               |
| デイリーログイン | `DAILY_LOGIN_XP` | +5   | 起動ボーナス                                 |

### ストリークボーナス

ストリークボーナスは `XpBreakdown::calculate` に **一元化** されている。

| 定数                    | 値 | 意味                                                                                                    |
| ----------------------- | -- | ------------------------------------------------------------------------------------------------------- |
| `STREAK_BONUS_CAP_DAYS` | 10 | ボーナス計算に反映されるストリーク日数の上限。1 日あたり +1%、上限到達で base_total の +10% が加算される。 |

`XpBreakdown::calculate` のストリークボーナスは、

```text
streak_bonus_xp = base_total * min(streak, STREAK_BONUS_CAP_DAYS) / 100
```

で算出される（最大 +10%）。

> **note**: 旧来の `with_streak_bonus(base, streak)`（1 日 +10%、最大 +100%）と
> `STREAK_BONUS_PERCENT / MAX_STREAK_BONUS_PERCENT` 定数は本仕様から削除した。
> プロダクション側で呼び出されておらず、`XpBreakdown::calculate` と倍率が異なるため、
> 公開 API として残しておくと誤用の温床になる。

別途、ストリーク連続日数のマイルストーンによる**追加**ボーナスは
`database::models::streak::calculate_streak_bonus` 側で計算され、
`commands/github.rs` で別 XP 行として `xp_history` に記録される（`total_xp` には含めない）。
`XpBreakdown::calculate` の `streak_bonus_xp` とは別系統。

## 公開 API

### 定数

上記の `pub const` 群（`COMMIT_XP` ほか）。

### 型

- `XpSource` — XP の発生源を示す enum（`commit / pull_request / review / issue / streak_bonus / challenge_complete / badge_earned / daily_login`）
- `XpActionType` — DB の `xp_history.action_type` 文字列にマッピングされる enum
- `XpHistoryEntry` — 履歴 1 行のドメインモデル
- `XpBreakdown` — sync 結果に含まれる XP 内訳
  - `XpBreakdown::calculate(commits, prs_created, prs_merged, issues_created, issues_closed, reviews, stars, streak)` で生成

## マイグレーション方針

- 既存ユーザーの累積 XP（`user_stats.total_xp`）は **そのまま維持** する。
- 過去の `xp_history` 行も再計算しない。
- 新仕様は **次回の `sync_github_stats` 以降の差分計算** から適用される。
- 旧仕様（PR=25, Review=15 など）で計算済みの XP はレベル算出にもそのまま流用される。
  これにより、既存ユーザーがレベルダウンする等の破壊的変化は起きない。

## バッジ条件への影響

- バッジ評価 (`database::models::badge::evaluate_badges`) は **コミット / PR / レビュー / Issue
  などの累計カウント** を参照しており、XP 値そのものには依存していない。
- したがって、本変更によるバッジ獲得条件の動作変更はない。

## テスト観点

- `tests::test_xp_constants_match_spec` — 定数値が仕様表と一致することを保証
- `tests::test_breakdown_zero_streak` — 各カウント 1 件ずつのときの XP 内訳と合計（175）
- `tests::test_breakdown_with_streak` — ストリーク 5 日のときのボーナス計算
- `tests::test_breakdown_streak_capped_at_10_days` — ストリーク 10 日超でも 10 日でキャップ
- `tests::test_breakdown_uses_constants_not_hardcoded` — `XpBreakdown::calculate` がハードコードでなく定数を参照することを保証
- `tests::test_breakdown_saturates_on_overflow` — `i32::MAX` 入力でラップアラウンドせず `i32::MAX` に飽和することを保証
