//! XP-related models
//!
//! Note: XPやカウント系の値は意味的にはu32（符号なし整数）が適切ですが、
//! SQLiteのINTEGER型が符号あり整数であり、sqlxがi32としてマッピングするため、
//! DB層との整合性を保つためにi32で統一しています。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// XP source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XpSource {
    Commit,
    PullRequest,
    Review,
    Issue,
    StreakBonus,
    ChallengeComplete,
    BadgeEarned,
    DailyLogin,
}

impl std::fmt::Display for XpSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XpSource::Commit => write!(f, "commit"),
            XpSource::PullRequest => write!(f, "pull_request"),
            XpSource::Review => write!(f, "review"),
            XpSource::Issue => write!(f, "issue"),
            XpSource::StreakBonus => write!(f, "streak_bonus"),
            XpSource::ChallengeComplete => write!(f, "challenge_complete"),
            XpSource::BadgeEarned => write!(f, "badge_earned"),
            XpSource::DailyLogin => write!(f, "daily_login"),
        }
    }
}

impl From<String> for XpSource {
    fn from(s: String) -> Self {
        match s.as_str() {
            "commit" => XpSource::Commit,
            "pull_request" => XpSource::PullRequest,
            "review" => XpSource::Review,
            "issue" => XpSource::Issue,
            "streak_bonus" => XpSource::StreakBonus,
            "challenge_complete" => XpSource::ChallengeComplete,
            "badge_earned" => XpSource::BadgeEarned,
            "daily_login" => XpSource::DailyLogin,
            _ => XpSource::Commit,
        }
    }
}

/// XP history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XpHistoryEntry {
    pub id: i64,
    pub user_id: i64,
    pub action_type: String,
    pub xp_amount: i32,
    pub description: Option<String>,
    pub github_event_id: Option<String>,
    pub breakdown: Option<XpBreakdown>,
    pub created_at: DateTime<Utc>,
}

/// XP action types for database
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpActionType {
    Commit,
    PullRequest,
    PullRequestMerged,
    Review,
    Issue,
    IssueClosed,
    StreakBonus,
    Star,
}

impl XpActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            XpActionType::Commit => "commit",
            XpActionType::PullRequest => "pull_request",
            XpActionType::PullRequestMerged => "pull_request_merged",
            XpActionType::Review => "review",
            XpActionType::Issue => "issue",
            XpActionType::IssueClosed => "issue_closed",
            XpActionType::StreakBonus => "streak_bonus",
            XpActionType::Star => "star",
        }
    }
}

impl std::fmt::Display for XpActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// 公式 XP ルール（単一の真実）
//
// 仕様の根拠: `docs/prd/home-gamification.md` §3.3.2 経験値テーブル
// および Issue #184 で確定された XP ルール。
//
// ここで定義した定数のみが XP 計算に使われる。`XpBreakdown::calculate` も
// すべてこれらの定数を参照すること（ハードコード禁止）。
// =============================================================================

/// XP for a commit
pub const COMMIT_XP: i32 = 10;
/// XP for creating a pull request
pub const PR_XP: i32 = 30;
/// XP for getting a pull request merged
pub const PR_MERGED_XP: i32 = 50;
/// XP for creating an issue
pub const ISSUE_XP: i32 = 15;
/// XP for closing/resolving an issue
pub const ISSUE_CLOSED_XP: i32 = 40;
/// XP for performing a code review
pub const REVIEW_XP: i32 = 25;
/// XP for receiving a star on a repository
pub const STAR_XP: i32 = 5;
/// XP for daily login
pub const DAILY_LOGIN_XP: i32 = 5;
/// Maximum streak days that contribute to `XpBreakdown::calculate` のストリークボーナス。
/// `min(streak, STREAK_BONUS_CAP_DAYS)` 日まで反映され、上限到達時は base_total の +10% となる。
pub const STREAK_BONUS_CAP_DAYS: i32 = 10;

/// XP breakdown for sync result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XpBreakdown {
    pub commits_xp: i32,
    pub prs_created_xp: i32,
    pub prs_merged_xp: i32,
    pub issues_created_xp: i32,
    pub issues_closed_xp: i32,
    pub reviews_xp: i32,
    pub stars_xp: i32,
    pub streak_bonus_xp: i32,
    pub total_xp: i32,
}

impl XpBreakdown {
    /// 同期で得た差分活動量から XP 内訳を算出する。
    ///
    /// # 引数の型について
    ///
    /// 各カウント引数は意味的に「非負の差分」であり、`u64` を要求する。
    /// 呼び出し側（`run_github_sync`）は `u64::saturating_sub` で前回値からの
    /// 差分を取り、累計値が同期間に減少したケース（例: スターを失う、
    /// repository 削除）でも 0 にクランプする。これにより
    /// `XpBreakdown` の各フィールドが常に非負になる（Issue #189 / DoD）。
    /// 旧 `i32` シグネチャの時代は `XpBreakdown` 内部に負値が現れる可能性が
    /// 残っており、`if xp_gained > 0` の DB 加算ガードに依存していた。
    ///
    /// # 戻り値の `total_xp` について
    ///
    /// `total_xp = (各活動 XP の合計) + streak_bonus_xp` であり、**ここで計算する
    /// 活動ベースのストリークボーナス（1 日あたり +1%、最大 `STREAK_BONUS_CAP_DAYS`%）は
    /// すでに `total_xp` に含まれている**。
    ///
    /// 呼び出し側でマイルストーンボーナス等、別系統のストリークボーナスを加算する場合は
    /// `total_xp` ではなく、内訳との整合を保つために以下のいずれかにすること:
    /// - 別 XP 行として `xp_history` に記録する（現行の `commands/github.rs` の方式）
    /// - 表示用合計を作る際は `total_xp + 外部ボーナス` と明示的に組み立てる
    ///
    /// 二重計上を避けるため、本関数の戻り値の `streak_bonus_xp` を再度足さないこと。
    #[allow(clippy::too_many_arguments)]
    pub fn calculate(
        commits: u64,
        prs_created: u64,
        prs_merged: u64,
        issues_created: u64,
        issues_closed: u64,
        reviews: u64,
        stars: u64,
        streak: i32,
    ) -> Self {
        // 入力カウントが極端に大きいケースでも個別積・合計でラップアラウンド
        // しないよう、内部計算はすべて u64 + saturating で行う。
        // 構造体フィールドは i32 のため、最後に i32::MAX で飽和して格納する。
        let saturate_u64_to_i32 = |v: u64| {
            if v > i32::MAX as u64 {
                i32::MAX
            } else {
                v as i32
            }
        };

        // XP 定数は正の小整数なので u64 へキャストしても情報落ちはない。
        let commits_xp = commits.saturating_mul(COMMIT_XP as u64);
        let prs_created_xp = prs_created.saturating_mul(PR_XP as u64);
        let prs_merged_xp = prs_merged.saturating_mul(PR_MERGED_XP as u64);
        let issues_created_xp = issues_created.saturating_mul(ISSUE_XP as u64);
        let issues_closed_xp = issues_closed.saturating_mul(ISSUE_CLOSED_XP as u64);
        let reviews_xp = reviews.saturating_mul(REVIEW_XP as u64);
        let stars_xp = stars.saturating_mul(STAR_XP as u64);

        let base_total = commits_xp
            .saturating_add(prs_created_xp)
            .saturating_add(prs_merged_xp)
            .saturating_add(issues_created_xp)
            .saturating_add(issues_closed_xp)
            .saturating_add(reviews_xp)
            .saturating_add(stars_xp);

        // ストリークボーナス: `base_total * min(streak, STREAK_BONUS_CAP_DAYS) / 100`。
        // 1 日あたり +1%、上限 `STREAK_BONUS_CAP_DAYS`% (= 10%)。
        // streak は `streak.max(0)` 相当に正規化したうえで掛け算する。
        let capped_streak_days = streak.clamp(0, STREAK_BONUS_CAP_DAYS) as u64;
        let streak_bonus_xp = base_total
            .saturating_mul(capped_streak_days)
            / 100;

        let total_xp = base_total.saturating_add(streak_bonus_xp);

        Self {
            commits_xp: saturate_u64_to_i32(commits_xp),
            prs_created_xp: saturate_u64_to_i32(prs_created_xp),
            prs_merged_xp: saturate_u64_to_i32(prs_merged_xp),
            issues_created_xp: saturate_u64_to_i32(issues_created_xp),
            issues_closed_xp: saturate_u64_to_i32(issues_closed_xp),
            reviews_xp: saturate_u64_to_i32(reviews_xp),
            stars_xp: saturate_u64_to_i32(stars_xp),
            streak_bonus_xp: saturate_u64_to_i32(streak_bonus_xp),
            total_xp: saturate_u64_to_i32(total_xp),
        }
    }
}

/// XP values module (for backward compatibility)
pub mod xp {
    pub use super::{
        XpActionType, XpBreakdown, COMMIT_XP, DAILY_LOGIN_XP, ISSUE_CLOSED_XP, ISSUE_XP,
        PR_MERGED_XP, PR_XP, REVIEW_XP, STAR_XP, STREAK_BONUS_CAP_DAYS,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_constants_match_spec() {
        // 公式仕様（docs/prd/home-gamification.md §3.3.2 / Issue #184）と
        // 定数値が一致していることを保証する。
        assert_eq!(COMMIT_XP, 10);
        assert_eq!(PR_XP, 30);
        assert_eq!(PR_MERGED_XP, 50);
        assert_eq!(ISSUE_XP, 15);
        assert_eq!(ISSUE_CLOSED_XP, 40);
        assert_eq!(REVIEW_XP, 25);
        assert_eq!(STAR_XP, 5);
        assert_eq!(DAILY_LOGIN_XP, 5);
    }

    #[test]
    fn test_breakdown_zero_streak() {
        // 1 commit / 1 PR 作成 / 1 PR マージ / 1 Issue 作成 / 1 Issue 解決 / 1 レビュー / 1 スター
        // 期待値: 10 + 30 + 50 + 15 + 40 + 25 + 5 = 175
        let bd = XpBreakdown::calculate(1, 1, 1, 1, 1, 1, 1, 0);
        assert_eq!(bd.commits_xp, 10);
        assert_eq!(bd.prs_created_xp, 30);
        assert_eq!(bd.prs_merged_xp, 50);
        assert_eq!(bd.issues_created_xp, 15);
        assert_eq!(bd.issues_closed_xp, 40);
        assert_eq!(bd.reviews_xp, 25);
        assert_eq!(bd.stars_xp, 5);
        assert_eq!(bd.streak_bonus_xp, 0);
        assert_eq!(bd.total_xp, 175);
    }

    #[test]
    fn test_breakdown_with_streak() {
        // base = 175, streak = 5 → bonus = 175 * 5 / 100 = 8 (i32 切り捨て)
        let bd = XpBreakdown::calculate(1, 1, 1, 1, 1, 1, 1, 5);
        assert_eq!(bd.streak_bonus_xp, 8);
        assert_eq!(bd.total_xp, 183);
    }

    #[test]
    fn test_breakdown_streak_capped_at_10_days() {
        // streak = 10 → bonus = 175 * 10 / 100 = 17
        let bd_10 = XpBreakdown::calculate(1, 1, 1, 1, 1, 1, 1, 10);
        // streak = 100 → 同じく 10 日にキャップされるので bonus = 17
        let bd_100 = XpBreakdown::calculate(1, 1, 1, 1, 1, 1, 1, 100);
        assert_eq!(bd_10.streak_bonus_xp, 17);
        assert_eq!(bd_10.total_xp, 192);
        assert_eq!(bd_100.streak_bonus_xp, bd_10.streak_bonus_xp);
        assert_eq!(bd_100.total_xp, bd_10.total_xp);
    }

    #[test]
    fn test_breakdown_uses_constants_not_hardcoded() {
        // 個別カウント × 定数 で計算されることをチェック
        let bd = XpBreakdown::calculate(3, 2, 1, 4, 2, 5, 6, 0);
        assert_eq!(bd.commits_xp, 3 * COMMIT_XP);
        assert_eq!(bd.prs_created_xp, 2 * PR_XP);
        assert_eq!(bd.prs_merged_xp, 1 * PR_MERGED_XP);
        assert_eq!(bd.issues_created_xp, 4 * ISSUE_XP);
        assert_eq!(bd.issues_closed_xp, 2 * ISSUE_CLOSED_XP);
        assert_eq!(bd.reviews_xp, 5 * REVIEW_XP);
        assert_eq!(bd.stars_xp, 6 * STAR_XP);
    }

    #[test]
    fn test_breakdown_saturates_on_overflow() {
        // i32::MAX 個のコミット × COMMIT_XP(=10) は i32 範囲を大きく超えるが、
        // 内部 u64 計算 → 末端 saturating cast によってラップアラウンドせず i32::MAX に丸まる。
        let bd = XpBreakdown::calculate(i32::MAX as u64, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(bd.commits_xp, i32::MAX);
        assert_eq!(bd.total_xp, i32::MAX);
    }

    // Issue #189 / DoD: `XpBreakdown` の各フィールドが非負であることを保証する。
    // u64 シグネチャに変更したため、入力レベルで負値が表現できず、
    // 内部計算（`saturating_mul` / `saturating_add`）も非負を保つ。
    #[test]
    fn test_breakdown_fields_are_non_negative() {
        // ゼロ入力
        let zero = XpBreakdown::calculate(0, 0, 0, 0, 0, 0, 0, 0);
        for field in [
            zero.commits_xp,
            zero.prs_created_xp,
            zero.prs_merged_xp,
            zero.issues_created_xp,
            zero.issues_closed_xp,
            zero.reviews_xp,
            zero.stars_xp,
            zero.streak_bonus_xp,
            zero.total_xp,
        ] {
            assert!(field >= 0, "XpBreakdown field must be >= 0 (got {})", field);
        }

        // 中間的な入力
        let typical = XpBreakdown::calculate(7, 3, 2, 5, 4, 6, 1, 5);
        for field in [
            typical.commits_xp,
            typical.prs_created_xp,
            typical.prs_merged_xp,
            typical.issues_created_xp,
            typical.issues_closed_xp,
            typical.reviews_xp,
            typical.stars_xp,
            typical.streak_bonus_xp,
            typical.total_xp,
        ] {
            assert!(field >= 0, "XpBreakdown field must be >= 0 (got {})", field);
        }

        // 飽和入力でも非負
        let saturated = XpBreakdown::calculate(u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, 100);
        for field in [
            saturated.commits_xp,
            saturated.prs_created_xp,
            saturated.prs_merged_xp,
            saturated.issues_created_xp,
            saturated.issues_closed_xp,
            saturated.reviews_xp,
            saturated.stars_xp,
            saturated.streak_bonus_xp,
            saturated.total_xp,
        ] {
            assert!(field >= 0, "XpBreakdown field must be >= 0 (got {})", field);
        }
    }

    // Issue #189: 負のストリーク値が混入しても 0 にクランプされ、
    // streak_bonus_xp が負にならない。
    #[test]
    fn test_breakdown_negative_streak_clamped_to_zero_bonus() {
        let bd = XpBreakdown::calculate(1, 1, 1, 1, 1, 1, 1, -42);
        assert_eq!(bd.streak_bonus_xp, 0);
        assert_eq!(bd.total_xp, 175);
    }
}
