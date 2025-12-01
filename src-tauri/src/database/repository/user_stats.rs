//! User stats repository operations

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

use crate::database::connection::{Database, DatabaseError, DbResult};
use crate::database::models::{level, UserStats};

/// User stats row from database
#[derive(Debug, FromRow)]
pub(crate) struct UserStatsRow {
    pub id: i64,
    pub user_id: i64,
    pub total_xp: i32,
    pub current_level: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_activity_date: Option<String>,
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_reviews: i32,
    pub total_issues: i32,
    pub updated_at: String,
}

impl TryFrom<UserStatsRow> for UserStats {
    type Error = DatabaseError;

    fn try_from(row: UserStatsRow) -> Result<Self, Self::Error> {
        Ok(UserStats {
            id: row.id,
            user_id: row.user_id,
            total_xp: row.total_xp,
            current_level: row.current_level,
            current_streak: row.current_streak,
            longest_streak: row.longest_streak,
            last_activity_date: row
                .last_activity_date
                .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            total_commits: row.total_commits,
            total_prs: row.total_prs,
            total_reviews: row.total_reviews,
            total_issues: row.total_issues,
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

/// User stats repository operations
impl Database {
    /// Create user stats (called automatically when creating user)
    pub(crate) async fn create_user_stats(&self, user_id: i64) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO user_stats (user_id, total_xp, current_level, current_streak, 
                                   longest_streak, total_commits, total_prs, 
                                   total_reviews, total_issues, updated_at)
            VALUES (?, 0, 1, 0, 0, 0, 0, 0, 0, ?)
            "#,
        )
        .bind(user_id)
        .bind(now)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Get user stats by user ID
    pub async fn get_user_stats(&self, user_id: i64) -> DbResult<Option<UserStats>> {
        let row: Option<UserStatsRow> =
            sqlx::query_as("SELECT * FROM user_stats WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    /// Update user XP and level
    pub async fn add_xp(&self, user_id: i64, xp_amount: i32) -> DbResult<UserStats> {
        let now = Utc::now().to_rfc3339();

        // Get current stats
        let current = self
            .get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found".to_string()))?;

        let new_total_xp = current.total_xp + xp_amount;
        let new_level = level::level_from_xp(new_total_xp);

        sqlx::query(
            r#"
            UPDATE user_stats 
            SET total_xp = ?, current_level = ?, updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(new_total_xp)
        .bind(new_level)
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found after update".to_string()))
    }

    /// Update streak
    pub async fn update_streak(
        &self,
        user_id: i64,
        activity_date: NaiveDate,
    ) -> DbResult<UserStats> {
        let now = Utc::now().to_rfc3339();

        let current = self
            .get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found".to_string()))?;

        let (new_streak, new_longest) = if let Some(last_date) = current.last_activity_date {
            let days_diff = (activity_date - last_date).num_days();
            if days_diff == 1 {
                // Consecutive day
                let new_streak = current.current_streak + 1;
                let new_longest = new_streak.max(current.longest_streak);
                (new_streak, new_longest)
            } else if days_diff == 0 {
                // Same day, no change
                (current.current_streak, current.longest_streak)
            } else {
                // Streak broken
                (1, current.longest_streak)
            }
        } else {
            // First activity
            (1, 1)
        };

        sqlx::query(
            r#"
            UPDATE user_stats 
            SET current_streak = ?, longest_streak = ?, last_activity_date = ?, updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(new_streak)
        .bind(new_longest)
        .bind(activity_date.to_string())
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found after update".to_string()))
    }

    /// Update streak from GitHub contribution calendar data
    ///
    /// This method takes pre-calculated streak information from the GitHub contribution
    /// calendar and updates the database accordingly. Unlike `update_streak`, this method
    /// uses the actual GitHub contribution history rather than app usage dates.
    ///
    /// Uses an atomic UPDATE with RETURNING to avoid race conditions and reduce
    /// database round-trips. The MAX() function ensures longest_streak never decreases.
    ///
    /// # Arguments
    /// * `user_id` - The user's database ID
    /// * `current_streak` - Current consecutive days with contributions (from GitHub)
    /// * `longest_streak` - Longest consecutive days ever (from GitHub)
    /// * `last_activity_date` - The last date with contributions (YYYY-MM-DD format)
    pub async fn update_streak_from_github(
        &self,
        user_id: i64,
        current_streak: i32,
        longest_streak: i32,
        last_activity_date: Option<&str>,
    ) -> DbResult<UserStats> {
        let now = Utc::now().to_rfc3339();

        // Use atomic UPDATE with RETURNING to avoid race conditions
        // MAX() ensures longest_streak never decreases
        let updated_row: UserStatsRow = sqlx::query_as(
            r#"
            UPDATE user_stats 
            SET 
                current_streak = ?,
                longest_streak = MAX(longest_streak, ?),
                last_activity_date = ?,
                updated_at = ?
            WHERE user_id = ?
            RETURNING *
            "#,
        )
        .bind(current_streak)
        .bind(longest_streak)
        .bind(last_activity_date)
        .bind(&now)
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        updated_row.try_into()
    }

    /// Increment activity counts
    pub async fn increment_activity_count(
        &self,
        user_id: i64,
        commits: i32,
        prs: i32,
        reviews: i32,
        issues: i32,
    ) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE user_stats 
            SET total_commits = total_commits + ?,
                total_prs = total_prs + ?,
                total_reviews = total_reviews + ?,
                total_issues = total_issues + ?,
                updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(commits)
        .bind(prs)
        .bind(reviews)
        .bind(issues)
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}
