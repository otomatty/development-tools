//! Repository tests

use crate::database::connection::Database;
use chrono::Utc;

async fn setup_test_db() -> Database {
    Database::in_memory()
        .await
        .expect("Failed to create test database")
}

#[tokio::test]
async fn test_create_and_get_user() {
    let db = setup_test_db().await;

    let user = db
        .create_user(
            12345,
            "testuser",
            Some("https://avatar.url"),
            "encrypted_token",
            None,
            None,
        )
        .await
        .expect("Should create user");

    assert_eq!(user.github_id, 12345);
    assert_eq!(user.username, "testuser");

    let fetched = db
        .get_user_by_github_id(12345)
        .await
        .expect("Should fetch user")
        .expect("User should exist");

    assert_eq!(fetched.id, user.id);
}

#[tokio::test]
async fn test_user_stats_created_with_user() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let stats = db
        .get_user_stats(user.id)
        .await
        .expect("Should get stats")
        .expect("Stats should exist");

    assert_eq!(stats.total_xp, 0);
    assert_eq!(stats.current_level, 1);
}

#[tokio::test]
async fn test_add_xp() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let stats = db.add_xp(user.id, 100).await.expect("Should add XP");

    assert_eq!(stats.total_xp, 100);
    assert_eq!(stats.current_level, 2); // 100 XP = level 2
}

#[tokio::test]
async fn test_streak_tracking() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let today = Utc::now().date_naive();

    let stats = db
        .update_streak(user.id, today)
        .await
        .expect("Should update streak");

    assert_eq!(stats.current_streak, 1);
    assert_eq!(stats.longest_streak, 1);
}

#[tokio::test]
async fn test_badge_operations() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    db.award_badge(user.id, "milestone", "first_blood")
        .await
        .expect("Should award badge");

    let has_badge = db
        .has_badge(user.id, "first_blood")
        .await
        .expect("Should check badge");

    assert!(has_badge);

    let badges = db
        .get_user_badges(user.id)
        .await
        .expect("Should get badges");

    assert_eq!(badges.len(), 1);
    assert_eq!(badges[0].badge_id, "first_blood");
}

#[tokio::test]
async fn test_cache_operations() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let expires = Utc::now() + chrono::Duration::hours(1);
    db.save_cache(user.id, "test_data", r#"{"test": true}"#, expires)
        .await
        .expect("Should save cache");

    let cached = db
        .get_valid_cache(user.id, "test_data")
        .await
        .expect("Should get cache")
        .expect("Cache should exist");

    assert_eq!(cached, r#"{"test": true}"#);
}

#[tokio::test]
async fn test_create_challenge() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    let challenge = db
        .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge");

    assert_eq!(challenge.user_id, user.id);
    assert_eq!(challenge.challenge_type, "weekly");
    assert_eq!(challenge.target_metric, "commits");
    assert_eq!(challenge.target_value, 10);
    assert_eq!(challenge.current_value, 0);
    assert_eq!(challenge.reward_xp, 100);
    assert_eq!(challenge.status, "active");
}

#[tokio::test]
async fn test_get_active_challenges() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    db.create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge 1");

    db.create_challenge(
        user.id,
        "daily",
        "prs",
        2,
        50,
        start,
        start + chrono::Duration::days(1),
    )
    .await
    .expect("Should create challenge 2");

    let challenges = db
        .get_active_challenges(user.id)
        .await
        .expect("Should get active challenges");

    assert_eq!(challenges.len(), 2);
}

#[tokio::test]
async fn test_update_challenge_progress() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    let challenge = db
        .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge");

    let updated = db
        .update_challenge_progress(challenge.id, 5)
        .await
        .expect("Should update progress");

    assert_eq!(updated.current_value, 5);
    assert_eq!(updated.status, "active");
}

#[tokio::test]
async fn test_complete_challenge() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    let challenge = db
        .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge");

    let completed = db
        .complete_challenge(challenge.id)
        .await
        .expect("Should complete challenge");

    assert_eq!(completed.status, "completed");
    assert!(completed.completed_at.is_some());
}

#[tokio::test]
async fn test_fail_challenge() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    let challenge = db
        .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge");

    let failed = db
        .fail_challenge(challenge.id)
        .await
        .expect("Should fail challenge");

    assert_eq!(failed.status, "failed");
}

#[tokio::test]
async fn test_has_active_challenge() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    // No challenge yet
    let has = db
        .has_active_challenge(user.id, "weekly", "commits")
        .await
        .expect("Should check");
    assert!(!has);

    // Create challenge
    db.create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge");

    // Now should have one
    let has = db
        .has_active_challenge(user.id, "weekly", "commits")
        .await
        .expect("Should check");
    assert!(has);
}

#[tokio::test]
async fn test_delete_challenge() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    let challenge = db
        .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge");

    db.delete_challenge(challenge.id)
        .await
        .expect("Should delete challenge");

    let challenges = db
        .get_active_challenges(user.id)
        .await
        .expect("Should get challenges");

    assert_eq!(challenges.len(), 0);
}

#[tokio::test]
async fn test_challenge_completion_count() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    let challenge1 = db
        .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge 1");

    let challenge2 = db
        .create_challenge(
            user.id,
            "daily",
            "prs",
            2,
            50,
            start,
            start + chrono::Duration::days(1),
        )
        .await
        .expect("Should create challenge 2");

    // Complete one challenge
    db.complete_challenge(challenge1.id)
        .await
        .expect("Should complete challenge 1");

    let count = db
        .get_challenge_completion_count(user.id)
        .await
        .expect("Should get count");

    assert_eq!(count, 1);

    // Complete second challenge
    db.complete_challenge(challenge2.id)
        .await
        .expect("Should complete challenge 2");

    let count = db
        .get_challenge_completion_count(user.id)
        .await
        .expect("Should get count");

    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_create_challenge_with_stats() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);
    let start_stats = r#"{"commits":100,"prs":10,"reviews":5,"issues":3}"#;

    let challenge = db
        .create_challenge_with_stats(
            user.id,
            "weekly",
            "commits",
            10,
            100,
            start,
            end,
            start_stats,
        )
        .await
        .expect("Should create challenge with stats");

    assert_eq!(challenge.target_metric, "commits");
    assert_eq!(challenge.target_value, 10);

    // Verify start stats can be retrieved
    let retrieved_stats = db
        .get_challenge_start_stats(challenge.id)
        .await
        .expect("Should get start stats");

    assert!(retrieved_stats.is_some());
    assert_eq!(retrieved_stats.unwrap(), start_stats);
}

#[tokio::test]
async fn test_get_last_daily_challenge_date() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    // Initially no challenges
    let last_date = db
        .get_last_daily_challenge_date(user.id)
        .await
        .expect("Should check last date");
    assert!(last_date.is_none());

    // Create a daily challenge
    let start = Utc::now();
    let end = start + chrono::Duration::days(1);

    db.create_challenge(user.id, "daily", "commits", 5, 50, start, end)
        .await
        .expect("Should create daily challenge");

    // Now should have a date
    let last_date = db
        .get_last_daily_challenge_date(user.id)
        .await
        .expect("Should check last date");
    assert!(last_date.is_some());
}

#[tokio::test]
async fn test_get_last_weekly_challenge_date() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    // Initially no challenges
    let last_date = db
        .get_last_weekly_challenge_date(user.id)
        .await
        .expect("Should check last date");
    assert!(last_date.is_none());

    // Create a weekly challenge
    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    db.create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create weekly challenge");

    // Now should have a date
    let last_date = db
        .get_last_weekly_challenge_date(user.id)
        .await
        .expect("Should check last date");
    assert!(last_date.is_some());
}

#[tokio::test]
async fn test_update_github_aggregates_persists_badge_eval_fields() {
    use crate::database::UserStatsGitHubAggregates;

    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    // Defaults are all zero on a freshly-created stats row.
    let initial = db
        .get_user_stats(user.id)
        .await
        .expect("Should get stats")
        .expect("Stats should exist");
    assert_eq!(initial.weekly_streak, 0);
    assert_eq!(initial.monthly_streak, 0);
    assert_eq!(initial.total_prs_merged, 0);
    assert_eq!(initial.total_issues_closed, 0);
    assert_eq!(initial.languages_count, 0);
    assert_eq!(initial.total_stars_received, 0);

    let agg = UserStatsGitHubAggregates {
        total_commits: 250,
        total_prs: 40,
        total_reviews: 12,
        total_issues: 8,
        total_prs_merged: 30,
        total_issues_closed: 5,
        weekly_streak: 4,
        monthly_streak: 2,
        languages_count: 6,
        total_stars_received: 75,
    };

    let updated = db
        .update_github_aggregates(user.id, &agg)
        .await
        .expect("Should update aggregates");

    // Aggregates from GitHub mirror onto user_stats so the badge UI can
    // read them on subsequent calls without re-fetching from GitHub.
    assert_eq!(updated.total_commits, 250);
    assert_eq!(updated.total_prs, 40);
    assert_eq!(updated.total_reviews, 12);
    assert_eq!(updated.total_issues, 8);
    assert_eq!(updated.total_prs_merged, 30);
    assert_eq!(updated.total_issues_closed, 5);
    assert_eq!(updated.weekly_streak, 4);
    assert_eq!(updated.monthly_streak, 2);
    assert_eq!(updated.languages_count, 6);
    assert_eq!(updated.total_stars_received, 75);

    // Round-trips through SELECT — the `RETURNING *` value matches a
    // fresh fetch.
    let refetched = db
        .get_user_stats(user.id)
        .await
        .expect("Should re-fetch")
        .expect("Stats should exist");
    assert_eq!(refetched.total_prs_merged, 30);
    assert_eq!(refetched.total_stars_received, 75);
    assert_eq!(refetched.languages_count, 6);
}

#[tokio::test]
async fn test_update_github_aggregates_preserves_xp_and_streak() {
    use crate::database::UserStatsGitHubAggregates;

    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    // Seed XP and streak — these must not be touched by the aggregates
    // mirror, which is responsible only for the GitHub-derived counts.
    db.add_xp(user.id, 500).await.expect("Should add XP");
    db.update_streak_from_github(user.id, 7, 14, Some("2026-04-26"))
        .await
        .expect("Should update streak from GitHub");

    let before = db
        .get_user_stats(user.id)
        .await
        .expect("fetch")
        .expect("stats exist");

    db.update_github_aggregates(
        user.id,
        &UserStatsGitHubAggregates {
            total_commits: 99,
            total_prs: 11,
            total_reviews: 4,
            total_issues: 3,
            total_prs_merged: 9,
            total_issues_closed: 2,
            weekly_streak: 3,
            monthly_streak: 1,
            languages_count: 4,
            total_stars_received: 21,
        },
    )
    .await
    .expect("Should update aggregates");

    let after = db
        .get_user_stats(user.id)
        .await
        .expect("fetch")
        .expect("stats exist");

    assert_eq!(after.total_xp, before.total_xp);
    assert_eq!(after.current_level, before.current_level);
    assert_eq!(after.current_streak, before.current_streak);
    assert_eq!(after.longest_streak, before.longest_streak);
    assert_eq!(after.last_activity_date, before.last_activity_date);
}

// ---------------------------------------------------------------------------
// Issue #194: xp_history.source coexistence (recalculation vs live entries).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_record_xp_gain_defaults_to_live_source() {
    let db = setup_test_db().await;
    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    db.record_xp_gain(user.id, "github_sync", 50, Some("first sync"), None, None)
        .await
        .expect("Should record xp gain");

    let history = db
        .get_recent_xp_history(user.id, 10)
        .await
        .expect("Should fetch history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].source, "live");
    assert_eq!(history[0].xp_amount, 50);
}

#[tokio::test]
async fn test_record_xp_recalculation_is_separate_from_live() {
    let db = setup_test_db().await;
    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    db.record_xp_gain(user.id, "github_sync", 100, Some("live"), None, None)
        .await
        .expect("live row");
    db.record_xp_recalculation(user.id, 80, Some("recalc"), None)
        .await
        .expect("recalc row");

    // user_stats.total_xp must NOT include the recalculation row.
    // record_xp_gain / record_xp_recalculation only write xp_history;
    // adding to user_stats.total_xp is the live caller's responsibility.
    let stats = db
        .get_user_stats(user.id)
        .await
        .expect("Should get stats")
        .expect("Stats should exist");
    assert_eq!(stats.total_xp, 0);

    let history = db
        .get_recent_xp_history(user.id, 10)
        .await
        .expect("Should fetch history");
    let live_count = history.iter().filter(|e| e.source == "live").count();
    let recalc_count = history.iter().filter(|e| e.source == "recalculated").count();
    assert_eq!(live_count, 1);
    assert_eq!(recalc_count, 1);
}

#[tokio::test]
async fn test_get_xp_total_in_range_filters_by_source_and_window() {
    let db = setup_test_db().await;
    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    // Two live rows ~ now, plus a recalc row that must NOT be counted.
    db.record_xp_gain(user.id, "github_sync", 30, None, None, None)
        .await
        .unwrap();
    db.record_xp_gain(user.id, "streak_bonus", 20, None, None, None)
        .await
        .unwrap();
    db.record_xp_recalculation(user.id, 999, None, None)
        .await
        .unwrap();

    let yesterday = Utc::now() - chrono::Duration::days(1);
    let tomorrow = Utc::now() + chrono::Duration::days(1);
    let live_total = db
        .get_xp_total_in_range(user.id, yesterday, tomorrow, "live")
        .await
        .expect("Should fetch live total");
    let recalc_total = db
        .get_xp_total_in_range(user.id, yesterday, tomorrow, "recalculated")
        .await
        .expect("Should fetch recalc total");

    assert_eq!(live_total, 50);
    assert_eq!(recalc_total, 999);

    // Window entirely in the future returns 0 (no rows in [tomorrow, day-after]).
    let day_after = Utc::now() + chrono::Duration::days(2);
    let future_total = db
        .get_xp_total_in_range(user.id, tomorrow, day_after, "live")
        .await
        .expect("Should fetch future total");
    assert_eq!(future_total, 0);

    // Upper-bound clamp: a window that ends before "now" must exclude
    // the just-inserted rows even though they pass the lower bound.
    let two_days_ago = Utc::now() - chrono::Duration::days(2);
    let past_window_total = db
        .get_xp_total_in_range(user.id, two_days_ago, yesterday, "live")
        .await
        .expect("Should fetch past-window total");
    assert_eq!(past_window_total, 0);
}

/// Regression for PR #217 P1 review: comparing RFC3339 `since` against
/// legacy `YYYY-MM-DD HH:MM:SS` `created_at` values must not drop rows.
///
/// We seed a legacy-format row directly so `datetime(created_at)`
/// normalisation in `get_xp_total_in_range` /
/// `get_last_recalculation_at` is exercised on the format that older
/// (pre-Issue #194) installs actually have.
#[tokio::test]
async fn test_get_xp_total_in_range_handles_legacy_timestamp_format() {
    let db = setup_test_db().await;
    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    // Bypass record_xp_gain so the inserted row keeps SQLite's
    // CURRENT_TIMESTAMP default ("YYYY-MM-DD HH:MM:SS"). The wall
    // time used here ('2025-12-01 10:00:00') is intentionally chosen
    // so that the boundary RFC3339 `since` ('2025-12-01T00:00:00+00:00')
    // sorts *after* it lexicographically — the bug being regressed.
    sqlx::query(
        "INSERT INTO xp_history (user_id, action_type, xp_amount, source, created_at) \
         VALUES (?, 'github_sync', 42, 'live', '2025-12-01 10:00:00')",
    )
    .bind(user.id)
    .execute(db.pool())
    .await
    .expect("Insert legacy row");

    let since: chrono::DateTime<Utc> = "2025-12-01T00:00:00+00:00".parse().unwrap();
    let until: chrono::DateTime<Utc> = "2025-12-02T00:00:00+00:00".parse().unwrap();
    let total = db
        .get_xp_total_in_range(user.id, since, until, "live")
        .await
        .expect("Should fetch legacy-row total");
    assert_eq!(total, 42, "legacy YYYY-MM-DD HH:MM:SS row must be included");

    // A recalc row in the same format must satisfy the rate-limit guard.
    sqlx::query(
        "INSERT INTO xp_history (user_id, action_type, xp_amount, source, created_at) \
         VALUES (?, 'recalculation', 0, 'recalculated', '2025-12-01 11:00:00')",
    )
    .bind(user.id)
    .execute(db.pool())
    .await
    .expect("Insert legacy recalc row");
    let last = db
        .get_last_recalculation_at(user.id)
        .await
        .expect("Should fetch latest recalc");
    assert!(last.is_some(), "legacy recalc row must be discoverable");
}

#[tokio::test]
async fn test_get_last_recalculation_at_returns_latest_recalc_only() {
    let db = setup_test_db().await;
    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    // No recalc rows yet → None.
    let initial = db
        .get_last_recalculation_at(user.id)
        .await
        .expect("Should query empty");
    assert!(initial.is_none());

    // Live row alone must not satisfy the rate-limit guard.
    db.record_xp_gain(user.id, "github_sync", 10, None, None, None)
        .await
        .unwrap();
    assert!(db
        .get_last_recalculation_at(user.id)
        .await
        .unwrap()
        .is_none());

    db.record_xp_recalculation(user.id, 1, None, None)
        .await
        .unwrap();
    let latest = db.get_last_recalculation_at(user.id).await.unwrap();
    assert!(latest.is_some(), "Should return the recalc timestamp");
}

#[tokio::test]
async fn test_progress_capped_at_target() {
    let db = setup_test_db().await;

    let user = db
        .create_user(12345, "testuser", None, "token", None, None)
        .await
        .expect("Should create user");

    let start = Utc::now();
    let end = start + chrono::Duration::days(7);

    let challenge = db
        .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
        .await
        .expect("Should create challenge");

    // Update progress to more than target
    let updated = db
        .update_challenge_progress(challenge.id, 15)
        .await
        .expect("Should update progress");

    // Progress should be capped at target
    assert_eq!(updated.current_value, 10);
    assert_eq!(updated.status, "completed");
}
