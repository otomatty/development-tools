//! GitHub API client
//!
//! Provides methods to interact with the GitHub REST and GraphQL APIs.

use chrono::{Datelike, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use thiserror::Error;

use super::types::*;

const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";
const USER_AGENT_VALUE: &str = "development-tools/1.0";

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Rate limit exceeded. Resets at {0}")]
    RateLimited(i64),

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("GraphQL error: {0}")]
    GraphQL(String),
}

pub type GitHubResult<T> = Result<T, GitHubError>;

/// GitHub API client
pub struct GitHubClient {
    client: reqwest::Client,
    access_token: String,
}

impl GitHubClient {
    /// Create a new GitHub client with an access token
    pub fn new(access_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token,
        }
    }

    /// Build default headers for API requests
    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.access_token))
                .expect("Invalid token format"),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers
    }

    /// Check and handle rate limiting
    async fn check_rate_limit(&self, response: &reqwest::Response) -> GitHubResult<()> {
        if response.status() == reqwest::StatusCode::FORBIDDEN {
            if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
                if remaining.to_str().unwrap_or("1") == "0" {
                    let reset = response
                        .headers()
                        .get("x-ratelimit-reset")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(0);
                    return Err(GitHubError::RateLimited(reset));
                }
            }
        }
        Ok(())
    }

    /// Make a GET request to the GitHub REST API
    async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> GitHubResult<T> {
        let url = format!("{}{}", GITHUB_API_URL, endpoint);
        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        self.check_rate_limit(&response).await?;

        match response.status() {
            status if status.is_success() => {
                let body = response.json().await?;
                Ok(body)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(GitHubError::Unauthorized),
            reqwest::StatusCode::NOT_FOUND => {
                Err(GitHubError::NotFound(endpoint.to_string()))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(GitHubError::ApiError(format!(
                    "Status {}: {}",
                    status, error_text
                )))
            }
        }
    }

    /// Make a GraphQL query
    async fn graphql<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> GitHubResult<T> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables.unwrap_or(serde_json::json!({}))
        });

        let response = self
            .client
            .post(GITHUB_GRAPHQL_URL)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await?;

        self.check_rate_limit(&response).await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError(error_text));
        }

        let gql_response: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = gql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            return Err(GitHubError::GraphQL(error_messages.join(", ")));
        }

        gql_response
            .data
            .ok_or_else(|| GitHubError::GraphQL("No data in response".to_string()))
    }

    /// Get the authenticated user's profile
    pub async fn get_user(&self) -> GitHubResult<GitHubUser> {
        self.get("/user").await
    }

    /// Get the authenticated user's repositories
    pub async fn get_repositories(&self, per_page: i32, page: i32) -> GitHubResult<Vec<Repository>> {
        self.get(&format!(
            "/user/repos?sort=updated&per_page={}&page={}",
            per_page, page
        ))
        .await
    }

    /// Get recent events for the authenticated user
    pub async fn get_user_events(
        &self,
        username: &str,
        per_page: i32,
        page: i32,
    ) -> GitHubResult<Vec<ActivityEvent>> {
        self.get(&format!(
            "/users/{}/events?per_page={}&page={}",
            username, per_page, page
        ))
        .await
    }

    /// Get pull requests created by the user
    pub async fn get_user_pull_requests(
        &self,
        username: &str,
        state: &str,
        per_page: i32,
    ) -> GitHubResult<Vec<serde_json::Value>> {
        self.get(&format!(
            "/search/issues?q=type:pr+author:{}+state:{}&per_page={}",
            username, state, per_page
        ))
        .await
    }

    /// Get issues created by the user
    pub async fn get_user_issues(
        &self,
        username: &str,
        state: &str,
        per_page: i32,
    ) -> GitHubResult<Vec<serde_json::Value>> {
        self.get(&format!(
            "/search/issues?q=type:issue+author:{}+state:{}&per_page={}",
            username, state, per_page
        ))
        .await
    }

    /// Get contribution calendar using GraphQL
    pub async fn get_contribution_calendar(
        &self,
        username: &str,
    ) -> GitHubResult<ContributionsCollection> {
        let query = r#"
            query($login: String!) {
                user(login: $login) {
                    contributionsCollection {
                        contributionCalendar {
                            totalContributions
                            weeks {
                                contributionDays {
                                    contributionCount
                                    date
                                    weekday
                                }
                            }
                        }
                        totalCommitContributions
                        totalPullRequestContributions
                        totalIssueContributions
                        totalPullRequestReviewContributions
                    }
                }
            }
        "#;

        let variables = serde_json::json!({ "login": username });
        let response: ContributionCollectionResponse =
            self.graphql(query, Some(variables)).await?;

        response
            .user
            .map(|u| u.contributions_collection)
            .ok_or_else(|| GitHubError::NotFound(format!("User {} not found", username)))
    }

    /// Get count of merged PRs for the user
    /// 
    /// Note: This uses GitHub's Search API which has stricter rate limits
    /// (30 requests/minute for authenticated users). Call sequentially
    /// and handle rate limit errors gracefully.
    pub async fn get_merged_prs_count(&self, username: &str) -> GitHubResult<i32> {
        let query = format!(
            "/search/issues?q=type:pr+author:{}+is:merged&per_page=1",
            username
        );
        let response: serde_json::Value = self.get(&query).await?;
        Ok(response
            .get("total_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32)
    }

    /// Get count of closed issues for the user
    /// 
    /// Note: This uses GitHub's Search API which has stricter rate limits
    /// (30 requests/minute for authenticated users). Call sequentially
    /// and handle rate limit errors gracefully.
    pub async fn get_closed_issues_count(&self, username: &str) -> GitHubResult<i32> {
        // Issues created by user that are closed
        let query = format!(
            "/search/issues?q=type:issue+author:{}+is:closed&per_page=1",
            username
        );
        let response: serde_json::Value = self.get(&query).await?;
        Ok(response
            .get("total_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32)
    }

    /// Get unique programming languages used across user's repositories
    pub async fn get_languages_count(&self, username: &str) -> GitHubResult<i32> {
        let repos = self.get_repositories(100, 1).await?;
        let languages: std::collections::HashSet<&str> = repos
            .iter()
            .filter_map(|r| r.language.as_deref())
            .collect();
        Ok(languages.len() as i32)
    }

    /// Get count of all PRs (open + closed) for the user
    /// 
    /// Note: This uses GitHub's Search API which has stricter rate limits
    /// (30 requests/minute for authenticated users). Call sequentially
    /// and handle rate limit errors gracefully.
    pub async fn get_total_prs_count(&self, username: &str) -> GitHubResult<i32> {
        let query = format!(
            "/search/issues?q=type:pr+author:{}&per_page=1",
            username
        );
        let response: serde_json::Value = self.get(&query).await?;
        Ok(response
            .get("total_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32)
    }

    /// Get current rate limit status
    pub async fn get_rate_limit(&self) -> GitHubResult<RateLimit> {
        let response: RateLimitResponse = self.get("/rate_limit").await?;
        Ok(response.rate)
    }

    /// Calculate streak from contribution calendar
    /// 
    /// Returns StreakInfo containing:
    /// - current_streak: consecutive days with contributions up to today/yesterday
    /// - longest_streak: longest consecutive days with contributions ever
    /// - last_activity_date: the most recent date with contributions
    pub fn calculate_streak(calendar: &ContributionCalendar) -> StreakInfo {
        let mut current_streak = 0;
        let mut longest_streak = 0;
        let mut temp_streak = 0;
        let mut last_activity_date: Option<String> = None;

        // Flatten all days and sort by date
        let mut all_days: Vec<&ContributionDay> = calendar
            .weeks
            .iter()
            .flat_map(|w| w.contribution_days.iter())
            .collect();

        all_days.sort_by(|a, b| a.date.cmp(&b.date));

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let yesterday = (Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        for (i, day) in all_days.iter().enumerate() {
            if day.contribution_count > 0 {
                temp_streak += 1;
                longest_streak = longest_streak.max(temp_streak);
                last_activity_date = Some(day.date.clone());

                // Check if this could be current streak
                if day.date == today || day.date == yesterday {
                    current_streak = temp_streak;
                }
            } else {
                // Check if today has no contributions yet but yesterday did
                if day.date == today && i > 0 {
                    if let Some(prev) = all_days.get(i - 1) {
                        if prev.contribution_count > 0 && prev.date == yesterday {
                            current_streak = temp_streak;
                        }
                    }
                }
                temp_streak = 0;
            }
        }

        StreakInfo {
            current_streak,
            longest_streak,
            last_activity_date,
        }
    }

    /// Calculate streak from contribution calendar (legacy tuple return)
    /// 
    /// This is kept for backward compatibility. Use calculate_streak for new code.
    #[deprecated(note = "Use calculate_streak which returns StreakInfo")]
    pub fn calculate_streak_tuple(calendar: &ContributionCalendar) -> (i32, i32) {
        let info = Self::calculate_streak(calendar);
        (info.current_streak, info.longest_streak)
    }

    /// Calculate weekly and monthly streaks from contribution calendar
    /// 
    /// Returns (weekly_streak, monthly_streak) where:
    /// - weekly_streak: consecutive weeks with at least one contribution
    /// - monthly_streak: consecutive months with at least one contribution
    /// 
    /// Grace period: If current week/month has no contributions, streak calculation
    /// starts from the previous week/month (similar to daily streak behavior).
    pub fn calculate_weekly_monthly_streak(calendar: &ContributionCalendar) -> (i32, i32) {
        use std::collections::HashSet;
        
        // Collect all contribution days
        let all_days: Vec<_> = calendar
            .weeks
            .iter()
            .flat_map(|w| w.contribution_days.iter())
            .collect();
        
        // Group contributions by week (year-week) and month (year-month)
        let mut weeks_with_contributions: HashSet<String> = HashSet::new();
        let mut months_with_contributions: HashSet<String> = HashSet::new();
        
        for day in &all_days {
            if day.contribution_count > 0 {
                // Parse date string YYYY-MM-DD
                let parts: Vec<&str> = day.date.split('-').collect();
                if parts.len() == 3 {
                    // Use if let for explicit error handling
                    if let (Ok(year), Ok(month), Ok(day_num)) = (
                        parts[0].parse::<i32>(),
                        parts[1].parse::<u32>(),
                        parts[2].parse::<u32>(),
                    ) {
                        // Calculate ISO week number
                        if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day_num) {
                            let iso_week = date.iso_week();
                            let week_key = format!("{}-W{:02}", iso_week.year(), iso_week.week());
                            weeks_with_contributions.insert(week_key);
                            
                            let month_key = format!("{}-{:02}", year, month);
                            months_with_contributions.insert(month_key);
                        }
                    }
                    // Silently skip malformed dates - this is expected for edge cases
                }
            }
        }
        
        // Calculate current weekly streak with grace period
        let now = Utc::now();
        let current_iso = now.iso_week();
        let mut weekly_streak = 0;
        let mut check_year = current_iso.year();
        let mut check_week = current_iso.week();
        
        // Check if current week has contributions
        let current_week_key = format!("{}-W{:02}", check_year, check_week);
        let has_current_week = weeks_with_contributions.contains(&current_week_key);
        
        // If no contributions this week, start checking from previous week (grace period)
        if !has_current_week {
            if check_week == 1 {
                check_year -= 1;
                check_week = chrono::NaiveDate::from_ymd_opt(check_year, 12, 28)
                    .map(|d| d.iso_week().week())
                    .unwrap_or(52);
            } else {
                check_week -= 1;
            }
        }
        
        loop {
            let week_key = format!("{}-W{:02}", check_year, check_week);
            if weeks_with_contributions.contains(&week_key) {
                weekly_streak += 1;
                
                // Move to previous week
                if check_week == 1 {
                    check_year -= 1;
                    // Get the last week of the previous year
                    check_week = chrono::NaiveDate::from_ymd_opt(check_year, 12, 28)
                        .map(|d| d.iso_week().week())
                        .unwrap_or(52);
                } else {
                    check_week -= 1;
                }
            } else {
                break;
            }
        }
        
        // Calculate current monthly streak with grace period
        let mut monthly_streak = 0;
        let mut check_month = now.month();
        let mut check_year_m = now.year();
        
        // Check if current month has contributions
        let current_month_key = format!("{}-{:02}", check_year_m, check_month);
        let has_current_month = months_with_contributions.contains(&current_month_key);
        
        // If no contributions this month, start checking from previous month (grace period)
        if !has_current_month {
            if check_month == 1 {
                check_year_m -= 1;
                check_month = 12;
            } else {
                check_month -= 1;
            }
        }
        
        loop {
            let month_key = format!("{}-{:02}", check_year_m, check_month);
            if months_with_contributions.contains(&month_key) {
                monthly_streak += 1;
                
                // Move to previous month
                if check_month == 1 {
                    check_year_m -= 1;
                    check_month = 12;
                } else {
                    check_month -= 1;
                }
            } else {
                break;
            }
        }
        
        (weekly_streak, monthly_streak)
    }

    /// Get aggregated user stats
    /// 
    /// This method fetches stats from multiple API endpoints, including the
    /// Search API which has stricter rate limits. If rate limits are hit,
    /// fallback values from GraphQL/REST endpoints are used.
    pub async fn get_user_stats(&self, username: &str) -> GitHubResult<GitHubStats> {
        // Get contribution calendar (uses GraphQL - higher rate limit)
        let contributions = self.get_contribution_calendar(username).await?;
        let streak_info = Self::calculate_streak(&contributions.contribution_calendar);

        // Get total stars received and languages count (uses REST API)
        let repos = self.get_repositories(100, 1).await?;
        let total_stars: i32 = repos.iter().map(|r| r.stargazers_count).sum();
        let languages: std::collections::HashSet<&str> = repos
            .iter()
            .filter_map(|r| r.language.as_deref())
            .collect();

        // Get detailed PR and issue counts using Search API
        // IMPORTANT: Search API has stricter rate limits (30 req/min authenticated)
        // We call these sequentially and use fallback values if rate limited
        
        // Total PRs - fallback to GraphQL contributions if rate limited
        let total_prs = match self.get_total_prs_count(username).await {
            Ok(count) => count,
            Err(GitHubError::RateLimited(reset)) => {
                eprintln!(
                    "Rate limited fetching total PRs, using GraphQL fallback. Resets at {}",
                    reset
                );
                contributions.total_pull_request_contributions
            }
            Err(e) => {
                eprintln!("Error fetching total PRs: {}, using GraphQL fallback", e);
                contributions.total_pull_request_contributions
            }
        };

        // Merged PRs - fallback to 0 if rate limited
        let total_prs_merged = match self.get_merged_prs_count(username).await {
            Ok(count) => count,
            Err(GitHubError::RateLimited(reset)) => {
                eprintln!(
                    "Rate limited fetching merged PRs, using fallback (0). Resets at {}",
                    reset
                );
                0
            }
            Err(e) => {
                eprintln!("Error fetching merged PRs: {}, using fallback (0)", e);
                0
            }
        };

        // Closed issues - fallback to 0 if rate limited
        let total_issues_closed = match self.get_closed_issues_count(username).await {
            Ok(count) => count,
            Err(GitHubError::RateLimited(reset)) => {
                eprintln!(
                    "Rate limited fetching closed issues, using fallback (0). Resets at {}",
                    reset
                );
                0
            }
            Err(e) => {
                eprintln!("Error fetching closed issues: {}, using fallback (0)", e);
                0
            }
        };

        // Calculate weekly and monthly streaks
        let (weekly_streak, monthly_streak) = 
            Self::calculate_weekly_monthly_streak(&contributions.contribution_calendar);

        Ok(GitHubStats {
            total_commits: contributions.total_commit_contributions,
            total_prs,
            total_prs_merged,
            total_issues: contributions.total_issue_contributions,
            total_issues_closed,
            total_reviews: contributions.total_pull_request_review_contributions,
            total_stars_received: total_stars,
            total_contributions: contributions.contribution_calendar.total_contributions,
            contribution_calendar: Some(contributions.contribution_calendar),
            current_streak: streak_info.current_streak,
            longest_streak: streak_info.longest_streak,
            weekly_streak,
            monthly_streak,
            languages_count: languages.len() as i32,
            streak_info: Some(streak_info),
        })
    }

    // ========================================================================
    // Code Statistics Methods (for Issue #74)
    // ========================================================================

    /// Get code statistics (additions/deletions) for user's repositories
    /// 
    /// Uses a GraphQL batch query to fetch commit history with additions/deletions
    /// from the user's most recently pushed repositories.
    /// 
    /// # Arguments
    /// * `username` - GitHub username
    /// * `since` - ISO 8601 timestamp (e.g., "2025-01-01T00:00:00Z")
    /// * `max_repos` - Maximum number of repositories to query (default: 100)
    /// 
    /// # Returns
    /// HashMap of date -> DailyCodeStatsAggregated
    pub async fn get_code_stats(
        &self,
        username: &str,
        since: &str,
        max_repos: i32,
    ) -> GitHubResult<Vec<DailyCodeStatsAggregated>> {
        let query = r#"
            query($login: String!, $since: GitTimestamp!, $maxRepos: Int!) {
                user(login: $login) {
                    repositories(first: $maxRepos, orderBy: {field: PUSHED_AT, direction: DESC}) {
                        nodes {
                            nameWithOwner
                            defaultBranchRef {
                                target {
                                    ... on Commit {
                                        history(first: 100, since: $since) {
                                            nodes {
                                                additions
                                                deletions
                                                committedDate
                                                oid
                                            }
                                            pageInfo {
                                                hasNextPage
                                                endCursor
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        pageInfo {
                            hasNextPage
                            endCursor
                        }
                    }
                }
                rateLimit {
                    limit
                    cost
                    remaining
                    resetAt
                }
            }
        "#;

        let variables = serde_json::json!({
            "login": username,
            "since": since,
            "maxRepos": max_repos
        });

        let response: CodeStatsQueryResponse = self.graphql(query, Some(variables)).await?;

        // Aggregate commits by date across all repositories
        let mut daily_stats: std::collections::HashMap<String, DailyCodeStatsAggregated> = 
            std::collections::HashMap::new();

        if let Some(user) = response.user {
            for repo in user.repositories.nodes {
                let repo_name = repo.name_with_owner.clone();
                
                if let Some(branch_ref) = repo.default_branch_ref {
                    if let Some(target) = branch_ref.target {
                        if let Some(history) = target.history {
                            for commit in history.nodes {
                                // Parse the date (take YYYY-MM-DD part)
                                let date = commit.committed_date
                                    .split('T')
                                    .next()
                                    .unwrap_or(&commit.committed_date)
                                    .to_string();
                                
                                let entry = daily_stats
                                    .entry(date.clone())
                                    .or_insert_with(|| DailyCodeStatsAggregated {
                                        date: date.clone(),
                                        additions: 0,
                                        deletions: 0,
                                        commits_count: 0,
                                        repositories: vec![],
                                    });
                                
                                entry.additions += commit.additions;
                                entry.deletions += commit.deletions;
                                entry.commits_count += 1;
                                
                                if !entry.repositories.contains(&repo_name) {
                                    entry.repositories.push(repo_name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Convert to sorted vector
        let mut result: Vec<DailyCodeStatsAggregated> = daily_stats.into_values().collect();
        result.sort_by(|a, b| b.date.cmp(&a.date)); // Sort by date descending
        
        Ok(result)
    }

    /// Get detailed rate limit information for all API types
    pub async fn get_detailed_rate_limit(&self) -> GitHubResult<RateLimitDetailed> {
        // Get REST API rate limits
        let rest_limits = self.get_rate_limit().await?;

        // Get GraphQL rate limit using a minimal query
        let graphql_query = r#"
            query {
                rateLimit {
                    limit
                    cost
                    remaining
                    resetAt
                }
            }
        "#;

        let graphql_response: GraphQLRateLimitResponse = 
            self.graphql(graphql_query, None).await?;

        let graphql_limit = graphql_response.rate_limit.as_ref().map(|r| r.limit).unwrap_or(0);
        let graphql_remaining = graphql_response.rate_limit.as_ref().map(|r| r.remaining).unwrap_or(0);
        let graphql_reset = graphql_response.rate_limit
            .as_ref()
            .and_then(|r| chrono::DateTime::parse_from_rfc3339(&r.reset_at).ok())
            .map(|dt| dt.timestamp())
            .unwrap_or(0);

        // Search API limits (we can't query this directly, using defaults)
        // Authenticated users: 30 requests per minute
        let search_limit = RateLimit {
            limit: 30,
            remaining: 30, // We don't track this precisely
            reset: Utc::now().timestamp() + 60,
            used: 0,
        };

        Ok(RateLimitDetailed {
            core: rest_limits,
            search: search_limit,
            graphql: RateLimit {
                limit: graphql_limit,
                remaining: graphql_remaining,
                reset: graphql_reset,
                used: graphql_limit - graphql_remaining,
            },
        })
    }

    /// Check if rate limit is critical (below 20% remaining)
    pub fn is_rate_limit_critical(rate_limit: &RateLimitDetailed) -> bool {
        let core_critical = rate_limit.core.limit > 0 
            && (rate_limit.core.remaining as f32 / rate_limit.core.limit as f32) < 0.2;
        let graphql_critical = rate_limit.graphql.limit > 0 
            && (rate_limit.graphql.remaining as f32 / rate_limit.graphql.limit as f32) < 0.2;
        let search_critical = rate_limit.search.limit > 0 
            && (rate_limit.search.remaining as f32 / rate_limit.search.limit as f32) < 0.2;
        
        core_critical || graphql_critical || search_critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_streak_empty() {
        let calendar = ContributionCalendar {
            total_contributions: 0,
            weeks: vec![],
        };

        let streak_info = GitHubClient::calculate_streak(&calendar);
        assert_eq!(streak_info.current_streak, 0);
        assert_eq!(streak_info.longest_streak, 0);
        assert_eq!(streak_info.last_activity_date, None);
    }

    #[test]
    fn test_calculate_streak_with_contributions() {
        let calendar = ContributionCalendar {
            total_contributions: 10,
            weeks: vec![ContributionWeek {
                contribution_days: vec![
                    ContributionDay {
                        contribution_count: 5,
                        date: "2024-01-01".to_string(),
                        weekday: 1,
                    },
                    ContributionDay {
                        contribution_count: 3,
                        date: "2024-01-02".to_string(),
                        weekday: 2,
                    },
                    ContributionDay {
                        contribution_count: 0,
                        date: "2024-01-03".to_string(),
                        weekday: 3,
                    },
                    ContributionDay {
                        contribution_count: 2,
                        date: "2024-01-04".to_string(),
                        weekday: 4,
                    },
                ],
            }],
        };

        let streak_info = GitHubClient::calculate_streak(&calendar);
        assert_eq!(streak_info.longest_streak, 2); // 2 consecutive days at the start
        assert_eq!(streak_info.last_activity_date, Some("2024-01-04".to_string()));
    }

    // ============================================================
    // Tests for calculate_weekly_monthly_streak
    // ============================================================

    #[test]
    fn test_weekly_monthly_streak_empty_calendar() {
        let calendar = ContributionCalendar {
            total_contributions: 0,
            weeks: vec![],
        };

        let (weekly, monthly) = GitHubClient::calculate_weekly_monthly_streak(&calendar);
        assert_eq!(weekly, 0);
        assert_eq!(monthly, 0);
    }

    #[test]
    fn test_weekly_monthly_streak_consecutive_weeks() {
        // Create contributions for consecutive weeks
        // Week 1 of current year and Week 52 of previous year
        let now = chrono::Utc::now();
        let current_week = now.iso_week().week();
        let current_year = now.iso_week().year();
        
        // Build contribution days for current week and previous weeks
        let mut weeks = Vec::new();
        
        // Add contributions for current week and 2 previous weeks (3 consecutive weeks)
        for week_offset in 0..3 {
            let mut check_year = current_year;
            let mut check_week = current_week as i32 - week_offset;
            
            // Handle year boundary
            if check_week <= 0 {
                check_year -= 1;
                check_week = 52 + check_week; // Approximate
            }
            
            // Find a date in that week
            if let Some(date) = chrono::NaiveDate::from_isoywd_opt(check_year, check_week as u32, chrono::Weekday::Mon) {
                weeks.push(ContributionWeek {
                    contribution_days: vec![ContributionDay {
                        contribution_count: 1,
                        date: date.format("%Y-%m-%d").to_string(),
                        weekday: 1,
                    }],
                });
            }
        }
        
        let calendar = ContributionCalendar {
            total_contributions: 3,
            weeks,
        };

        let (weekly, _monthly) = GitHubClient::calculate_weekly_monthly_streak(&calendar);
        assert!(weekly >= 3, "Expected at least 3 consecutive weeks, got {}", weekly);
    }

    #[test]
    fn test_weekly_monthly_streak_non_consecutive_weeks() {
        // Create contributions with a gap (week 1 and week 3, missing week 2)
        let now = chrono::Utc::now();
        let current_week = now.iso_week().week();
        let current_year = now.iso_week().year();
        
        let mut weeks = Vec::new();
        
        // Current week
        if let Some(date) = chrono::NaiveDate::from_isoywd_opt(current_year, current_week, chrono::Weekday::Mon) {
            weeks.push(ContributionWeek {
                contribution_days: vec![ContributionDay {
                    contribution_count: 1,
                    date: date.format("%Y-%m-%d").to_string(),
                    weekday: 1,
                }],
            });
        }
        
        // Skip one week (week_offset = 1), add week_offset = 2
        let week_offset = 2i32;
        let mut check_year = current_year;
        let mut check_week = current_week as i32 - week_offset;
        if check_week <= 0 {
            check_year -= 1;
            check_week = 52 + check_week;
        }
        
        if let Some(date) = chrono::NaiveDate::from_isoywd_opt(check_year, check_week as u32, chrono::Weekday::Mon) {
            weeks.push(ContributionWeek {
                contribution_days: vec![ContributionDay {
                    contribution_count: 1,
                    date: date.format("%Y-%m-%d").to_string(),
                    weekday: 1,
                }],
            });
        }
        
        let calendar = ContributionCalendar {
            total_contributions: 2,
            weeks,
        };

        let (weekly, _monthly) = GitHubClient::calculate_weekly_monthly_streak(&calendar);
        // Should be 1 because there's a gap (missing last week)
        assert_eq!(weekly, 1, "Expected streak of 1 due to gap");
    }

    #[test]
    fn test_weekly_monthly_streak_year_boundary() {
        // Test handling of year boundary (December to January)
        // This is a simplified test to check that year boundaries don't crash
        let calendar = ContributionCalendar {
            total_contributions: 2,
            weeks: vec![
                ContributionWeek {
                    contribution_days: vec![ContributionDay {
                        contribution_count: 1,
                        date: "2024-12-30".to_string(), // Week 1 of 2025 (ISO week)
                        weekday: 1,
                    }],
                },
                ContributionWeek {
                    contribution_days: vec![ContributionDay {
                        contribution_count: 1,
                        date: "2024-12-23".to_string(), // Week 52 of 2024
                        weekday: 1,
                    }],
                },
            ],
        };

        // Should not panic on year boundary
        let (weekly, monthly) = GitHubClient::calculate_weekly_monthly_streak(&calendar);
        // The actual values depend on current date, but it should not crash
        assert!(weekly >= 0);
        assert!(monthly >= 0);
    }

    #[test]
    fn test_weekly_monthly_streak_grace_period() {
        // Test grace period: if current week has no contributions,
        // streak should still count from previous week
        let now = chrono::Utc::now();
        let current_week = now.iso_week().week();
        let current_year = now.iso_week().year();
        
        // Only add contributions for previous week (not current week)
        let mut check_year = current_year;
        let mut check_week = current_week as i32 - 1;
        if check_week <= 0 {
            check_year -= 1;
            check_week = 52;
        }
        
        let mut weeks = Vec::new();
        
        // Add 2 consecutive weeks starting from previous week
        for offset in 0..2 {
            let mut y = check_year;
            let mut w = check_week - offset;
            if w <= 0 {
                y -= 1;
                w = 52 + w;
            }
            
            if let Some(date) = chrono::NaiveDate::from_isoywd_opt(y, w as u32, chrono::Weekday::Mon) {
                weeks.push(ContributionWeek {
                    contribution_days: vec![ContributionDay {
                        contribution_count: 1,
                        date: date.format("%Y-%m-%d").to_string(),
                        weekday: 1,
                    }],
                });
            }
        }
        
        let calendar = ContributionCalendar {
            total_contributions: 2,
            weeks,
        };

        let (weekly, _monthly) = GitHubClient::calculate_weekly_monthly_streak(&calendar);
        // With grace period, should count the streak from previous weeks
        assert!(weekly >= 2, "Expected streak of at least 2 with grace period, got {}", weekly);
    }
}

