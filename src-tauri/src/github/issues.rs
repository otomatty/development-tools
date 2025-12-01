//! GitHub Issues API client
//!
//! Provides methods to interact with GitHub Issues and Labels API.
//! Related Issue: GitHub Issue #59 - GitHub Issue管理機能（Linear風カンバン）
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this module):
//!   ├─ src-tauri/src/github/mod.rs
//!   └─ src-tauri/src/commands/issues.rs
//! Dependencies:
//!   └─ src-tauri/src/database/models/project.rs

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::database::models::project::{IssueStatus, IssuePriority, LabelDefinition};

use super::client::{GitHubError, GitHubResult};

const GITHUB_API_URL: &str = "https://api.github.com";
const USER_AGENT_VALUE: &str = "development-tools/1.0";

/// GitHub Issue (detailed response from API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub labels: Vec<GitHubLabel>,
    pub assignee: Option<GitHubAssignee>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// GitHub Label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

/// GitHub Assignee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAssignee {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
}

/// GitHub Repository (for linking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: String,
    pub open_issues_count: i32,
    pub owner: GitHubOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
    pub id: i64,
}

/// Issues API client
pub struct IssuesClient {
    client: reqwest::Client,
    access_token: String,
}

impl IssuesClient {
    /// Create a new Issues client with an access token
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

    /// Make a GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> GitHubResult<T> {
        let response = self
            .client
            .get(url)
            .headers(self.build_headers())
            .send()
            .await?;

        match response.status() {
            status if status.is_success() => {
                let body = response.json().await?;
                Ok(body)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(GitHubError::Unauthorized),
            reqwest::StatusCode::NOT_FOUND => Err(GitHubError::NotFound(url.to_string())),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(GitHubError::ApiError(format!(
                    "Status {}: {}",
                    status, error_text
                )))
            }
        }
    }

    /// Make a POST request
    async fn post<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> GitHubResult<T> {
        let response = self
            .client
            .post(url)
            .headers(self.build_headers())
            .json(body)
            .send()
            .await?;

        match response.status() {
            status if status.is_success() => {
                let body = response.json().await?;
                Ok(body)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(GitHubError::Unauthorized),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                let error_text = response.text().await.unwrap_or_default();
                // Label already exists is not an error
                if error_text.contains("already_exists") {
                    Err(GitHubError::ApiError("already_exists".to_string()))
                } else {
                    Err(GitHubError::ApiError(format!(
                        "Validation failed: {}",
                        error_text
                    )))
                }
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

    /// Make a PATCH request
    async fn patch<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> GitHubResult<T> {
        let response = self
            .client
            .patch(url)
            .headers(self.build_headers())
            .json(body)
            .send()
            .await?;

        match response.status() {
            status if status.is_success() => {
                let body = response.json().await?;
                Ok(body)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(GitHubError::Unauthorized),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(GitHubError::ApiError(format!(
                    "Status {}: {}",
                    status, error_text
                )))
            }
        }
    }

    /// Make a DELETE request
    async fn delete(&self, url: &str) -> GitHubResult<()> {
        let response = self
            .client
            .delete(url)
            .headers(self.build_headers())
            .send()
            .await?;

        match response.status() {
            status if status.is_success() => Ok(()),
            reqwest::StatusCode::NO_CONTENT => Ok(()),
            reqwest::StatusCode::UNAUTHORIZED => Err(GitHubError::Unauthorized),
            reqwest::StatusCode::NOT_FOUND => Ok(()), // Label not found is OK for delete
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(GitHubError::ApiError(format!(
                    "Status {}: {}",
                    status, error_text
                )))
            }
        }
    }

    // ========================================================================
    // Repository methods
    // ========================================================================

    /// Get user's repositories
    pub async fn get_user_repositories(&self) -> GitHubResult<Vec<GitHubRepository>> {
        let url = format!(
            "{}/user/repos?sort=updated&per_page=100&affiliation=owner,collaborator",
            GITHUB_API_URL
        );
        self.get(&url).await
    }

    /// Get repository info
    pub async fn get_repository(&self, owner: &str, repo: &str) -> GitHubResult<GitHubRepository> {
        let url = format!("{}/repos/{}/{}", GITHUB_API_URL, owner, repo);
        self.get(&url).await
    }

    // ========================================================================
    // Issue methods
    // ========================================================================

    /// Get all issues for a repository
    pub async fn get_issues(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        per_page: i32,
        page: i32,
    ) -> GitHubResult<Vec<GitHubIssue>> {
        let url = format!(
            "{}/repos/{}/{}/issues?state={}&per_page={}&page={}&sort=updated&direction=desc",
            GITHUB_API_URL, owner, repo, state, per_page, page
        );
        self.get(&url).await
    }

    /// Get a single issue
    pub async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
    ) -> GitHubResult<GitHubIssue> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            GITHUB_API_URL, owner, repo, issue_number
        );
        self.get(&url).await
    }

    /// Create a new issue
    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: Option<&str>,
        labels: Vec<String>,
    ) -> GitHubResult<GitHubIssue> {
        let url = format!("{}/repos/{}/{}/issues", GITHUB_API_URL, owner, repo);
        let payload = serde_json::json!({
            "title": title,
            "body": body,
            "labels": labels,
        });
        self.post(&url, &payload).await
    }

    /// Update an issue
    pub async fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        title: Option<&str>,
        body: Option<&str>,
        state: Option<&str>,
        labels: Option<Vec<String>>,
    ) -> GitHubResult<GitHubIssue> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            GITHUB_API_URL, owner, repo, issue_number
        );

        let mut payload = serde_json::Map::new();
        if let Some(t) = title {
            payload.insert("title".to_string(), serde_json::json!(t));
        }
        if let Some(b) = body {
            payload.insert("body".to_string(), serde_json::json!(b));
        }
        if let Some(s) = state {
            payload.insert("state".to_string(), serde_json::json!(s));
        }
        if let Some(l) = labels {
            payload.insert("labels".to_string(), serde_json::json!(l));
        }

        self.patch(&url, &serde_json::Value::Object(payload)).await
    }

    /// Set labels on an issue
    pub async fn set_issue_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        labels: Vec<String>,
    ) -> GitHubResult<Vec<GitHubLabel>> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}/labels",
            GITHUB_API_URL, owner, repo, issue_number
        );
        let payload = serde_json::json!({ "labels": labels });
        
        // PUT replaces all labels
        let response = self
            .client
            .put(&url)
            .headers(self.build_headers())
            .json(&payload)
            .send()
            .await?;

        match response.status() {
            status if status.is_success() => {
                let body = response.json().await?;
                Ok(body)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(GitHubError::Unauthorized),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(GitHubError::ApiError(format!(
                    "Status {}: {}",
                    status, error_text
                )))
            }
        }
    }

    /// Add a label to an issue
    pub async fn add_issue_label(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        label: &str,
    ) -> GitHubResult<Vec<GitHubLabel>> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}/labels",
            GITHUB_API_URL, owner, repo, issue_number
        );
        let payload = serde_json::json!({ "labels": [label] });
        self.post(&url, &payload).await
    }

    /// Remove a label from an issue
    pub async fn remove_issue_label(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        label: &str,
    ) -> GitHubResult<()> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}/labels/{}",
            GITHUB_API_URL,
            owner,
            repo,
            issue_number,
            urlencoding::encode(label)
        );
        self.delete(&url).await
    }

    // ========================================================================
    // Label methods
    // ========================================================================

    /// Get all labels for a repository
    pub async fn get_labels(&self, owner: &str, repo: &str) -> GitHubResult<Vec<GitHubLabel>> {
        let url = format!(
            "{}/repos/{}/{}/labels?per_page=100",
            GITHUB_API_URL, owner, repo
        );
        self.get(&url).await
    }

    /// Create a label
    pub async fn create_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        color: &str,
        description: &str,
    ) -> GitHubResult<GitHubLabel> {
        let url = format!("{}/repos/{}/{}/labels", GITHUB_API_URL, owner, repo);
        let payload = serde_json::json!({
            "name": name,
            "color": color,
            "description": description,
        });
        self.post(&url, &payload).await
    }

    /// Create status and priority labels for a repository
    pub async fn create_status_labels(&self, owner: &str, repo: &str) -> GitHubResult<()> {
        let labels = LabelDefinition::all_labels();

        for label in labels {
            match self
                .create_label(owner, repo, &label.name, &label.color, &label.description)
                .await
            {
                Ok(_) => {
                    eprintln!("Created label: {}", label.name);
                }
                Err(GitHubError::ApiError(msg)) if msg == "already_exists" => {
                    eprintln!("Label already exists: {}", label.name);
                }
                Err(e) => {
                    eprintln!("Failed to create label {}: {:?}", label.name, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    // ========================================================================
    // Status update methods
    // ========================================================================

    /// Update issue status (removes old status label, adds new one)
    pub async fn update_issue_status(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        new_status: IssueStatus,
    ) -> GitHubResult<GitHubIssue> {
        // Get current issue to find existing status label
        let issue = self.get_issue(owner, repo, issue_number).await?;

        // Find and remove old status label
        let status_labels = IssueStatus::all_labels();
        for label in &issue.labels {
            if status_labels.contains(&label.name.as_str()) {
                let _ = self
                    .remove_issue_label(owner, repo, issue_number, &label.name)
                    .await;
            }
        }

        // Add new status label
        self.add_issue_label(owner, repo, issue_number, new_status.to_label())
            .await?;

        // If done, close the issue
        if new_status == IssueStatus::Done {
            self.update_issue(owner, repo, issue_number, None, None, Some("closed"), None)
                .await
        } else if new_status != IssueStatus::Cancelled && issue.state == "closed" {
            // Reopen if moving from Done/Cancelled back to active status
            self.update_issue(owner, repo, issue_number, None, None, Some("open"), None)
                .await
        } else {
            self.get_issue(owner, repo, issue_number).await
        }
    }

    /// Update issue priority (removes old priority label, adds new one)
    pub async fn update_issue_priority(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        priority: Option<IssuePriority>,
    ) -> GitHubResult<GitHubIssue> {
        // Get current issue
        let issue = self.get_issue(owner, repo, issue_number).await?;

        // Remove old priority labels
        let priority_labels = ["priority:high", "priority:medium", "priority:low"];
        for label in &issue.labels {
            if priority_labels.contains(&label.name.as_str()) {
                let _ = self
                    .remove_issue_label(owner, repo, issue_number, &label.name)
                    .await;
            }
        }

        // Add new priority label if set
        if let Some(p) = priority {
            self.add_issue_label(owner, repo, issue_number, p.to_label())
                .await?;
        }

        self.get_issue(owner, repo, issue_number).await
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    /// Extract status from issue labels
    pub fn extract_status(labels: &[GitHubLabel]) -> IssueStatus {
        for label in labels {
            if let Some(status) = IssueStatus::from_label(&label.name) {
                return status;
            }
        }
        IssueStatus::Backlog
    }

    /// Extract priority from issue labels
    pub fn extract_priority(labels: &[GitHubLabel]) -> Option<IssuePriority> {
        for label in labels {
            if let Some(priority) = IssuePriority::from_label(&label.name) {
                return Some(priority);
            }
        }
        None
    }
}

// ============================================================================
// GitHub Actions template
// ============================================================================

/// Generate GitHub Actions workflow YAML for automatic status updates
pub fn generate_actions_template() -> String {
    r#"name: Issue Status Sync

on:
  push:
    branches-ignore:
      - main
      - master
  pull_request:
    types: [opened, closed, reopened]

jobs:
  update-status:
    runs-on: ubuntu-latest
    steps:
      - name: Validate Branch Name & Extract Issue Number
        id: extract
        run: |
          BRANCH="${{ github.head_ref || github.ref_name }}"
          # ブランチ命名規則: type/issue番号-description
          if [[ ! "$BRANCH" =~ ^(feat|fix|docs|refactor|test|chore)/([0-9]+)-[a-zA-Z0-9_-]+$ ]]; then
            echo "Branch name does not match required pattern: type/<issue-number>-<description>"
            echo "Examples: feat/123-add-login, fix/456-fix-crash"
            echo "issue_number=" >> $GITHUB_OUTPUT
            exit 0
          fi
          ISSUE_NUMBER="${BASH_REMATCH[2]}"
          echo "issue_number=$ISSUE_NUMBER" >> $GITHUB_OUTPUT
          echo "Branch: $BRANCH, Issue: $ISSUE_NUMBER"

      - name: Skip if no issue number
        if: steps.extract.outputs.issue_number == ''
        run: echo "No valid issue number found, skipping status update"

      - name: Update Status on Push (In Progress)
        if: github.event_name == 'push' && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              // Skip if already done or cancelled
              const currentStatus = issue.labels.find(l => statusLabels.includes(l.name));
              if (currentStatus && (currentStatus.name === 'status:done' || currentStatus.name === 'status:cancelled')) {
                console.log(`Issue #${issueNumber} is already ${currentStatus.name}, skipping`);
                return;
              }
              
              // Remove old status labels
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              // Add in-progress label
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:in-progress']
              });
              
              console.log(`Updated issue #${issueNumber} to in-progress`);
            } catch (error) {
              console.log(`Failed to update issue #${issueNumber}: ${error.message}`);
            }

      - name: Update Status on PR Open (In Review)
        if: github.event_name == 'pull_request' && github.event.action == 'opened' && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:in-review']
              });
              
              console.log(`Updated issue #${issueNumber} to in-review`);
            } catch (error) {
              console.log(`Failed to update issue #${issueNumber}: ${error.message}`);
            }

      - name: Update Status on PR Merge (Done)
        if: github.event_name == 'pull_request' && github.event.action == 'closed' && github.event.pull_request.merged == true && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:done']
              });
              
              // Close the issue
              await github.rest.issues.update({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                state: 'closed',
                state_reason: 'completed'
              });
              
              console.log(`Updated issue #${issueNumber} to done and closed`);
            } catch (error) {
              console.log(`Failed to update issue #${issueNumber}: ${error.message}`);
            }

      - name: Revert Status on PR Close without Merge
        if: github.event_name == 'pull_request' && github.event.action == 'closed' && github.event.pull_request.merged == false && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              // Revert to in-progress
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:in-progress']
              });
              
              console.log(`Reverted issue #${issueNumber} to in-progress (PR closed without merge)`);
            } catch (error) {
              console.log(`Failed to revert issue #${issueNumber}: ${error.message}`);
            }
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_status() {
        let labels = vec![
            GitHubLabel {
                id: 1,
                name: "bug".to_string(),
                color: "red".to_string(),
                description: None,
            },
            GitHubLabel {
                id: 2,
                name: "status:in-progress".to_string(),
                color: "yellow".to_string(),
                description: None,
            },
        ];

        assert_eq!(
            IssuesClient::extract_status(&labels),
            IssueStatus::InProgress
        );
    }

    #[test]
    fn test_extract_status_default() {
        let labels = vec![GitHubLabel {
            id: 1,
            name: "bug".to_string(),
            color: "red".to_string(),
            description: None,
        }];

        assert_eq!(IssuesClient::extract_status(&labels), IssueStatus::Backlog);
    }

    #[test]
    fn test_extract_priority() {
        let labels = vec![
            GitHubLabel {
                id: 1,
                name: "priority:high".to_string(),
                color: "red".to_string(),
                description: None,
            },
            GitHubLabel {
                id: 2,
                name: "enhancement".to_string(),
                color: "green".to_string(),
                description: None,
            },
        ];

        assert_eq!(
            IssuesClient::extract_priority(&labels),
            Some(IssuePriority::High)
        );
    }
}
