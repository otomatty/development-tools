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
