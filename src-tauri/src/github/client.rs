//! GitHub API client
//!
//! Provides methods to interact with the GitHub REST and GraphQL APIs.

use chrono::Utc;
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
    pub fn calculate_streak(calendar: &ContributionCalendar) -> (i32, i32) {
        let mut current_streak = 0;
        let mut longest_streak = 0;
        let mut temp_streak = 0;

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

        (current_streak, longest_streak)
    }

    /// Get aggregated user stats
    /// 
    /// This method fetches stats from multiple API endpoints, including the
    /// Search API which has stricter rate limits. If rate limits are hit,
    /// fallback values from GraphQL/REST endpoints are used.
    pub async fn get_user_stats(&self, username: &str) -> GitHubResult<GitHubStats> {
        // Get contribution calendar (uses GraphQL - higher rate limit)
        let contributions = self.get_contribution_calendar(username).await?;
        let (current_streak, longest_streak) =
            Self::calculate_streak(&contributions.contribution_calendar);

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
            current_streak,
            longest_streak,
            languages_count: languages.len() as i32,
        })
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

        let (current, longest) = GitHubClient::calculate_streak(&calendar);
        assert_eq!(current, 0);
        assert_eq!(longest, 0);
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

        let (_, longest) = GitHubClient::calculate_streak(&calendar);
        assert_eq!(longest, 2); // 2 consecutive days at the start
    }
}

