//! GitHub API types
//!
//! Data structures for GitHub API responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub public_repos: i32,
    pub followers: i32,
    pub following: i32,
    pub created_at: DateTime<Utc>,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: i32,
    pub forks_count: i32,
    pub open_issues_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: Option<DateTime<Utc>>,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub author: Option<CommitAuthor>,
    pub committer: Option<CommitAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub date: DateTime<Utc>,
}

/// Pull request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub html_url: String,
}

/// Issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub html_url: String,
}

/// Review information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: i64,
    pub state: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub html_url: String,
}

/// Contribution calendar (from GraphQL API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionCalendar {
    pub total_contributions: i32,
    pub weeks: Vec<ContributionWeek>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionWeek {
    pub contribution_days: Vec<ContributionDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionDay {
    pub contribution_count: i32,
    pub date: String,
    pub weekday: i32,
}

/// User activity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    pub repo: EventRepo,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRepo {
    pub id: i64,
    pub name: String,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64,
    pub used: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    pub rate: RateLimit,
}

/// GraphQL response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

/// Contribution collection response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionCollectionResponse {
    pub user: Option<UserContributions>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserContributions {
    pub contributions_collection: ContributionsCollection,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionsCollection {
    pub contribution_calendar: ContributionCalendar,
    pub total_commit_contributions: i32,
    pub total_pull_request_contributions: i32,
    pub total_issue_contributions: i32,
    pub total_pull_request_review_contributions: i32,
}

/// Streak information calculated from contribution calendar
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StreakInfo {
    /// Current consecutive days with contributions
    pub current_streak: i32,
    /// Longest consecutive days with contributions ever
    pub longest_streak: i32,
    /// Date of last activity (YYYY-MM-DD format)
    pub last_activity_date: Option<String>,
}

/// User statistics aggregated from various sources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GitHubStats {
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_prs_merged: i32,
    pub total_issues: i32,
    pub total_issues_closed: i32,
    pub total_reviews: i32,
    pub total_stars_received: i32,
    pub total_contributions: i32,
    pub contribution_calendar: Option<ContributionCalendar>,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub weekly_streak: i32,
    pub monthly_streak: i32,
    pub languages_count: i32,
    /// Detailed streak information from contribution calendar
    pub streak_info: Option<StreakInfo>,
}

// ============================================================================
// Code Statistics Types (for Issue #74 - Code Lines Tracking)
// ============================================================================

/// Commit statistics with additions/deletions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitStats {
    pub sha: String,
    pub additions: i32,
    pub deletions: i32,
    pub committed_date: String,
}

/// Repository with commit history for code stats
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWithCommits {
    pub name_with_owner: String,
    pub default_branch_ref: Option<DefaultBranchRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultBranchRef {
    pub target: Option<CommitTarget>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitTarget {
    pub history: Option<CommitHistory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitHistory {
    pub nodes: Vec<CommitNode>,
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitNode {
    pub additions: i32,
    pub deletions: i32,
    pub committed_date: String,
    pub oid: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

/// GraphQL response for code stats batch query
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeStatsQueryResponse {
    pub user: Option<CodeStatsUser>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeStatsUser {
    pub repositories: RepositoriesConnection,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoriesConnection {
    pub nodes: Vec<RepositoryWithCommits>,
    pub page_info: Option<PageInfo>,
}

/// Aggregated daily code statistics from all repositories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyCodeStatsAggregated {
    pub date: String,
    pub additions: i32,
    pub deletions: i32,
    pub commits_count: i32,
    pub repositories: Vec<String>,
}

/// Rate limit information with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitDetailed {
    pub core: RateLimit,
    pub search: RateLimit,
    pub graphql: RateLimit,
}

/// GraphQL rate limit response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLRateLimitResponse {
    pub rate_limit: Option<GraphQLRateLimit>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLRateLimit {
    pub limit: i32,
    pub cost: i32,
    pub remaining: i32,
    pub reset_at: String,
}

// ============================================================================
// PR Progress Types (Issue #185 — G-04 PR progress dashboard panel)
// ============================================================================

/// Aggregated GraphQL response for the viewer's open PRs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressQueryResponse {
    pub viewer: Option<PrProgressViewer>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressViewer {
    pub pull_requests: PrProgressConnection,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressConnection {
    pub nodes: Vec<PrProgressNode>,
    pub page_info: Option<PageInfo>,
    pub total_count: Option<i32>,
}

/// Raw PR node returned by the `viewer { pullRequests }` query.
///
/// `mergeable` is GraphQL's `MergeableState` (`MERGEABLE` / `CONFLICTING` /
/// `UNKNOWN`); `review_decision` is `PullRequestReviewDecision`
/// (`APPROVED` / `CHANGES_REQUESTED` / `REVIEW_REQUIRED`) — both are
/// surfaced verbatim so the frontend can render its own labels.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressNode {
    pub id: String,
    pub number: i32,
    pub title: String,
    pub url: String,
    pub is_draft: bool,
    pub mergeable: String,
    pub review_decision: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub repository: PrProgressRepository,
    pub commits: Option<PrProgressCommits>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressRepository {
    pub name_with_owner: String,
    pub url: String,
}

/// `commits(last: 1)` is the documented way to get the rollup state for the
/// PR's head commit — `statusCheckRollup` lives on `Commit`, not on the PR
/// itself.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressCommits {
    pub nodes: Vec<PrProgressCommitNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressCommitNode {
    pub commit: PrProgressCommit,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressCommit {
    pub status_check_rollup: Option<PrProgressStatusCheckRollup>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressStatusCheckRollup {
    /// `SUCCESS` / `FAILURE` / `PENDING` / `ERROR` / `EXPECTED`.
    pub state: String,
}

/// Flattened PR progress row returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrProgressItem {
    pub id: String,
    pub number: i32,
    pub title: String,
    pub url: String,
    pub repo_full_name: String,
    pub repo_url: String,
    pub is_draft: bool,
    /// GraphQL `MergeableState` — `MERGEABLE` / `CONFLICTING` / `UNKNOWN`.
    pub mergeable: String,
    /// GraphQL `PullRequestReviewDecision`. `None` means the PR has no
    /// review requirement / no reviewers yet — UI distinguishes this from
    /// `REVIEW_REQUIRED`.
    pub review_decision: Option<String>,
    /// Last commit's `statusCheckRollup.state`, or `None` if there are no
    /// checks configured / the rollup hasn't computed yet.
    pub checks_state: Option<String>,
    /// ISO8601.
    pub created_at: String,
    /// ISO8601.
    pub updated_at: String,
}

/// Top-level summary of all open PRs the viewer authored.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrProgress {
    pub items: Vec<PrProgressItem>,
    pub total_count: i32,
    /// True when GraphQL pagination cut us off before exhausting the list.
    /// The UI uses this to render a "showing first N of M" hint so users
    /// with very large PR queues aren't misled.
    pub truncated: bool,
}
