//! Issue management Tauri commands
//!
//! This module provides Tauri commands for the GitHub Issue management feature.
//! Related Issue: GitHub Issue #59 - GitHub Issue管理機能（Linear風カンバン）
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this module):
//!   └─ src-tauri/src/lib.rs
//! Dependencies:
//!   ├─ src-tauri/src/auth/session.rs       (handle_unauthorized, map_github_result, reasons)
//!   ├─ src-tauri/src/commands/auth.rs      (for auth state)
//!   ├─ src-tauri/src/commands/github.rs    (CachedResponse envelope reused by *_with_cache)
//!   ├─ src-tauri/src/database/models/cache.rs (cache_types / cache_durations for Issue #183)
//!   ├─ src-tauri/src/database/models/project.rs
//!   ├─ src-tauri/src/database/repository/cache.rs (save_cache / get_any_cache)
//!   ├─ src-tauri/src/github/client.rs      (GitHubError variants)
//!   └─ src-tauri/src/github/issues.rs      (IssuesClient + Search API types)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::{AppHandle, State};

use crate::auth::{handle_unauthorized, map_github_result, reasons};
use crate::commands::github::CachedResponse;
use crate::commands::AppState;
use crate::database::models::project::{
    CachedIssue, IssueStatus, KanbanBoard, Project, ProjectWithStats, RepositoryInfo,
};
use crate::github::client::GitHubError;
use crate::github::issues::{generate_actions_template, GitHubSearchItem, IssuesClient};
use crate::github::{GitHubClient, PrProgress};

/// Get all projects for the current user
#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<ProjectWithStats>, String> {
    let user_id = get_current_user_id(&state).await?;

    let projects: Vec<Project> = sqlx::query_as(
        r#"
        SELECT id, user_id, name, description, github_repo_id, repo_owner, repo_name,
               repo_full_name, is_actions_setup, last_synced_at, created_at, updated_at
        FROM projects
        WHERE user_id = ?
        ORDER BY updated_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| format!("Failed to fetch projects: {}", e))?;

    // Get issue counts for each project
    let mut projects_with_stats = Vec::new();
    for project in projects {
        let counts: (i32, i32) = sqlx::query_as(
            r#"
            SELECT 
                CAST(SUM(CASE WHEN state = 'open' THEN 1 ELSE 0 END) AS INTEGER) as open_count,
                CAST(COUNT(*) AS INTEGER) as total_count
            FROM cached_issues
            WHERE project_id = ?
            "#,
        )
        .bind(project.id)
        .fetch_one(state.db.pool())
        .await
        .unwrap_or((0, 0));

        projects_with_stats.push(ProjectWithStats {
            project,
            open_issues_count: counts.0,
            total_issues_count: counts.1,
        });
    }

    Ok(projects_with_stats)
}

/// Get a single project by ID
#[tauri::command]
pub async fn get_project(state: State<'_, AppState>, project_id: i64) -> Result<Project, String> {
    let user_id = get_current_user_id(&state).await?;

    sqlx::query_as(
        r#"
        SELECT id, user_id, name, description, github_repo_id, repo_owner, repo_name,
               repo_full_name, is_actions_setup, last_synced_at, created_at, updated_at
        FROM projects
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| format!("Failed to fetch project: {}", e))?
    .ok_or_else(|| "Project not found".to_string())
}

/// Create a new project
#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<Project, String> {
    let user_id = get_current_user_id(&state).await?;
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"
        INSERT INTO projects (user_id, name, description, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&name)
    .bind(&description)
    .bind(&now)
    .bind(&now)
    .execute(state.db.pool())
    .await
    .map_err(|e| format!("Failed to create project: {}", e))?;

    get_project(state, result.last_insert_rowid()).await
}

/// Update a project
#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    project_id: i64,
    name: String,
    description: Option<String>,
) -> Result<Project, String> {
    let user_id = get_current_user_id(&state).await?;
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE projects
        SET name = ?, description = ?, updated_at = ?
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&now)
    .bind(project_id)
    .bind(user_id)
    .execute(state.db.pool())
    .await
    .map_err(|e| format!("Failed to update project: {}", e))?;

    get_project(state, project_id).await
}

/// Delete a project
#[tauri::command]
pub async fn delete_project(state: State<'_, AppState>, project_id: i64) -> Result<(), String> {
    let user_id = get_current_user_id(&state).await?;

    // Delete cached issues first (foreign key constraint)
    sqlx::query("DELETE FROM cached_issues WHERE project_id = ?")
        .bind(project_id)
        .execute(state.db.pool())
        .await
        .map_err(|e| format!("Failed to delete cached issues: {}", e))?;

    // Delete project
    let result = sqlx::query("DELETE FROM projects WHERE id = ? AND user_id = ?")
        .bind(project_id)
        .bind(user_id)
        .execute(state.db.pool())
        .await
        .map_err(|e| format!("Failed to delete project: {}", e))?;

    if result.rows_affected() == 0 {
        return Err("Project not found".to_string());
    }

    Ok(())
}

/// Get user's repositories for linking
#[tauri::command]
pub async fn get_user_repositories(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<RepositoryInfo>, String> {
    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    let repos =
        map_github_result(&app, state.inner(), client.get_user_repositories().await).await?;

    Ok(repos
        .into_iter()
        .map(|r| RepositoryInfo {
            id: r.id,
            name: r.name,
            full_name: r.full_name,
            owner: r.owner.login,
            description: r.description,
            html_url: r.html_url,
            private: r.private,
            open_issues_count: r.open_issues_count,
        })
        .collect())
}

/// Link a repository to a project
#[tauri::command]
pub async fn link_repository(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: i64,
    owner: String,
    repo: String,
) -> Result<Project, String> {
    let user_id = get_current_user_id(&state).await?;
    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    // Get repository info from GitHub
    let repo_info = map_github_result(
        &app,
        state.inner(),
        client.get_repository(&owner, &repo).await,
    )
    .await?;

    let now = Utc::now().to_rfc3339();
    let full_name = format!("{}/{}", owner, repo);

    // Update project with repository info
    sqlx::query(
        r#"
        UPDATE projects
        SET github_repo_id = ?, repo_owner = ?, repo_name = ?, repo_full_name = ?, updated_at = ?
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(repo_info.id)
    .bind(&owner)
    .bind(&repo)
    .bind(&full_name)
    .bind(&now)
    .bind(project_id)
    .bind(user_id)
    .execute(state.db.pool())
    .await
    .map_err(|e| format!("Failed to link repository: {}", e))?;

    // Create status labels in the repository
    if let Err(e) = client.create_status_labels(&owner, &repo).await {
        eprintln!("Warning: Failed to create status labels: {:?}", e);
        // Don't fail the link operation if label creation fails
    }

    get_project(state, project_id).await
}

/// Setup GitHub Actions for automatic status updates
#[tauri::command]
pub async fn setup_github_actions(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<String, String> {
    let project = get_project(state.clone(), project_id).await?;

    let owner = project.repo_owner.ok_or("Repository not linked")?;
    let repo = project.repo_name.ok_or("Repository not linked")?;

    // Generate the workflow content
    let workflow_content = generate_actions_template();

    // Return instructions for the user to create the workflow
    // (Creating files via GitHub API requires a separate implementation)
    Ok(format!(
        r#"To enable automatic status updates, create the following file in your repository:

File: .github/workflows/issue-status-sync.yml

Content:
{}

You can also create this file manually or through a Pull Request.

Repository: {}/{}
"#,
        workflow_content, owner, repo
    ))
}

/// Sync issues from GitHub to local cache
#[tauri::command]
pub async fn sync_project_issues(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<CachedIssue>, String> {
    let project = get_project(state.clone(), project_id).await?;

    let owner = project.repo_owner.ok_or("Repository not linked")?;
    let repo = project.repo_name.ok_or("Repository not linked")?;

    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    // Fetch all issues (open and closed)
    let mut all_issues = Vec::new();
    let mut page = 1;

    loop {
        let issues = map_github_result(
            &app,
            state.inner(),
            client.get_issues(&owner, &repo, "all", 100, page).await,
        )
        .await?;

        if issues.is_empty() {
            break;
        }

        all_issues.extend(issues);
        page += 1;

        // Limit to 1000 issues for safety
        if all_issues.len() >= 1000 {
            break;
        }
    }

    // Update cache
    let now = Utc::now().to_rfc3339();

    for issue in &all_issues {
        // Use extract_status_with_state to properly handle closed issues
        let status = IssuesClient::extract_status_with_state(
            &issue.labels,
            &issue.state,
            issue.state_reason.as_deref(),
        );
        let priority = IssuesClient::extract_priority(&issue.labels);
        let labels_json =
            serde_json::to_string(&issue.labels.iter().map(|l| &l.name).collect::<Vec<_>>())
                .unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            r#"
            INSERT INTO cached_issues (
                project_id, github_issue_id, number, title, body, state, status, priority,
                assignee_login, assignee_avatar_url, labels_json, html_url,
                github_created_at, github_updated_at, cached_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(project_id, github_issue_id) DO UPDATE SET
                number = excluded.number,
                title = excluded.title,
                body = excluded.body,
                state = excluded.state,
                status = excluded.status,
                priority = excluded.priority,
                assignee_login = excluded.assignee_login,
                assignee_avatar_url = excluded.assignee_avatar_url,
                labels_json = excluded.labels_json,
                html_url = excluded.html_url,
                github_created_at = excluded.github_created_at,
                github_updated_at = excluded.github_updated_at,
                cached_at = excluded.cached_at
            "#,
        )
        .bind(project_id)
        .bind(issue.id)
        .bind(issue.number)
        .bind(&issue.title)
        .bind(&issue.body)
        .bind(&issue.state)
        .bind(status.to_string())
        .bind(priority.map(|p| p.to_string()))
        .bind(issue.assignee.as_ref().map(|a| &a.login))
        .bind(issue.assignee.as_ref().map(|a| &a.avatar_url))
        .bind(&labels_json)
        .bind(&issue.html_url)
        .bind(issue.created_at.to_rfc3339())
        .bind(issue.updated_at.to_rfc3339())
        .bind(&now)
        .execute(state.db.pool())
        .await
        .map_err(|e| format!("Failed to cache issue: {}", e))?;
    }

    // Update last synced timestamp
    sqlx::query("UPDATE projects SET last_synced_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(project_id)
        .execute(state.db.pool())
        .await
        .map_err(|e| format!("Failed to update project: {}", e))?;

    get_project_issues(state, project_id, None).await
}

/// Get cached issues for a project
#[tauri::command]
pub async fn get_project_issues(
    state: State<'_, AppState>,
    project_id: i64,
    status: Option<String>,
) -> Result<Vec<CachedIssue>, String> {
    // Verify user is logged in
    let _user_id = get_current_user_id(&state).await?;

    // Verify project belongs to user
    let _project = get_project(state.clone(), project_id).await?;

    let issues = if let Some(s) = status {
        sqlx::query_as(
            r#"
            SELECT id, project_id, github_issue_id, number, title, body, state, status, priority,
                   assignee_login, assignee_avatar_url, labels_json, html_url,
                   github_created_at, github_updated_at, cached_at
            FROM cached_issues
            WHERE project_id = ? AND status = ?
            ORDER BY number DESC
            "#,
        )
        .bind(project_id)
        .bind(s)
        .fetch_all(state.db.pool())
        .await
    } else {
        sqlx::query_as(
            r#"
            SELECT id, project_id, github_issue_id, number, title, body, state, status, priority,
                   assignee_login, assignee_avatar_url, labels_json, html_url,
                   github_created_at, github_updated_at, cached_at
            FROM cached_issues
            WHERE project_id = ?
            ORDER BY number DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(state.db.pool())
        .await
    };

    issues.map_err(|e| format!("Failed to fetch issues: {}", e))
}

/// Get issues as kanban board
#[tauri::command]
pub async fn get_kanban_board(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<KanbanBoard, String> {
    let issues = get_project_issues(state, project_id, None).await?;
    Ok(KanbanBoard::from_issues(issues))
}

/// Update issue status (also updates on GitHub)
#[tauri::command]
pub async fn update_issue_status(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: i64,
    issue_number: i32,
    new_status: String,
) -> Result<CachedIssue, String> {
    let project = get_project(state.clone(), project_id).await?;

    let owner = project.repo_owner.ok_or("Repository not linked")?;
    let repo = project.repo_name.ok_or("Repository not linked")?;

    let status: IssueStatus = new_status
        .parse()
        .map_err(|_| format!("Invalid status: {}", new_status))?;

    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    // Update status on GitHub
    let updated_issue = map_github_result(
        &app,
        state.inner(),
        client
            .update_issue_status(&owner, &repo, issue_number, status)
            .await,
    )
    .await?;

    // Update local cache
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE cached_issues SET status = ?, state = ?, cached_at = ? WHERE project_id = ? AND number = ?",
    )
    .bind(status.to_string())
    .bind(&updated_issue.state)
    .bind(&now)
    .bind(project_id)
    .bind(issue_number)
    .execute(state.db.pool())
    .await
    .map_err(|e| format!("Failed to update cache: {}", e))?;

    // Fetch and return updated issue
    sqlx::query_as(
        r#"
        SELECT id, project_id, github_issue_id, number, title, body, state, status, priority,
               assignee_login, assignee_avatar_url, labels_json, html_url,
               github_created_at, github_updated_at, cached_at
        FROM cached_issues
        WHERE project_id = ? AND number = ?
        "#,
    )
    .bind(project_id)
    .bind(issue_number)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| format!("Failed to fetch issue: {}", e))?
    .ok_or_else(|| "Issue not found".to_string())
}

/// Create a new issue (on GitHub and cache locally)
#[tauri::command]
pub async fn create_github_issue(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: i64,
    title: String,
    body: Option<String>,
    status: Option<String>,
    priority: Option<String>,
) -> Result<CachedIssue, String> {
    let project = get_project(state.clone(), project_id).await?;

    let owner = project.repo_owner.ok_or("Repository not linked")?;
    let repo = project.repo_name.ok_or("Repository not linked")?;

    let issue_status: IssueStatus = status
        .as_ref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(IssueStatus::Backlog);

    let issue_priority: Option<crate::database::models::project::IssuePriority> =
        priority.as_ref().and_then(|p| p.parse().ok());

    // Build labels
    let mut labels = vec![issue_status.to_label().to_string()];
    if let Some(p) = &issue_priority {
        labels.push(p.to_label().to_string());
    }

    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    // Create issue on GitHub
    let github_issue = map_github_result(
        &app,
        state.inner(),
        client
            .create_issue(&owner, &repo, &title, body.as_deref(), labels)
            .await,
    )
    .await?;

    // Cache the new issue
    let now = Utc::now().to_rfc3339();
    let labels_json = serde_json::to_string(
        &github_issue
            .labels
            .iter()
            .map(|l| &l.name)
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"
        INSERT INTO cached_issues (
            project_id, github_issue_id, number, title, body, state, status, priority,
            assignee_login, assignee_avatar_url, labels_json, html_url,
            github_created_at, github_updated_at, cached_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(project_id)
    .bind(github_issue.id)
    .bind(github_issue.number)
    .bind(&github_issue.title)
    .bind(&github_issue.body)
    .bind(&github_issue.state)
    .bind(issue_status.to_string())
    .bind(issue_priority.map(|p| p.to_string()))
    .bind(github_issue.assignee.as_ref().map(|a| &a.login))
    .bind(github_issue.assignee.as_ref().map(|a| &a.avatar_url))
    .bind(&labels_json)
    .bind(&github_issue.html_url)
    .bind(github_issue.created_at.to_rfc3339())
    .bind(github_issue.updated_at.to_rfc3339())
    .bind(&now)
    .execute(state.db.pool())
    .await
    .map_err(|e| format!("Failed to cache issue: {}", e))?;

    // Fetch and return the cached issue
    sqlx::query_as(
        r#"
        SELECT id, project_id, github_issue_id, number, title, body, state, status, priority,
               assignee_login, assignee_avatar_url, labels_json, html_url,
               github_created_at, github_updated_at, cached_at
        FROM cached_issues
        WHERE project_id = ? AND number = ?
        "#,
    )
    .bind(project_id)
    .bind(github_issue.number)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| format!("Failed to fetch issue: {}", e))?
    .ok_or_else(|| "Issue not found".to_string())
}

// ============================================================================
// Cross-repository "Today / Inbox" (Issue #183)
// ============================================================================

/// One row in the cross-repository inbox.
///
/// Field naming uses camelCase on the wire for parity with the rest of the
/// `commands::*` API surface; the frontend type lives in
/// `src/types/issue.ts::MyOpenWorkItem`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyOpenWorkItem {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub repo_full_name: String,
    /// `"issue"` (assigned to me) or `"pull_request"` (review requested).
    pub kind: String,
    /// Source query, used by the UI to split tabs:
    /// `"assigned"` or `"review_requested"`.
    pub source: String,
    pub priority: Option<String>,
    pub labels: Vec<String>,
    pub assignee_login: Option<String>,
    pub assignee_avatar_url: Option<String>,
    pub author_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Aggregated payload returned by [`get_my_open_work_with_cache`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyOpenWork {
    pub assigned: Vec<MyOpenWorkItem>,
    pub review_requested: Vec<MyOpenWorkItem>,
}

fn convert_search_item(item: GitHubSearchItem, source: &str) -> MyOpenWorkItem {
    let (owner, repo) = item
        .owner_and_repo()
        .unwrap_or_else(|| ("".to_string(), "".to_string()));
    let repo_full_name = if !owner.is_empty() && !repo.is_empty() {
        format!("{}/{}", owner, repo)
    } else {
        item.repository_url.clone()
    };

    let priority = IssuesClient::extract_priority(&item.labels).map(|p| p.to_string());
    let kind = if item.is_pull_request() {
        "pull_request"
    } else {
        "issue"
    };

    MyOpenWorkItem {
        id: item.id,
        number: item.number,
        title: item.title,
        state: item.state,
        html_url: item.html_url,
        repo_owner: owner,
        repo_name: repo,
        repo_full_name,
        kind: kind.to_string(),
        source: source.to_string(),
        priority,
        labels: item.labels.into_iter().map(|l| l.name).collect(),
        assignee_login: item.assignee.as_ref().map(|a| a.login.clone()),
        assignee_avatar_url: item.assignee.as_ref().map(|a| a.avatar_url.clone()),
        author_login: item.user.as_ref().map(|u| u.login.clone()),
        created_at: item.created_at.to_rfc3339(),
        updated_at: item.updated_at.to_rfc3339(),
    }
}

fn is_network_or_rate_limit_error(error: &GitHubError) -> bool {
    // `Incomplete` (Search API server-side timeout) is included here so the
    // command falls back to the previous cache instead of caching a partial
    // inbox as a successful refresh.
    matches!(
        error,
        GitHubError::HttpRequest(_) | GitHubError::RateLimited(_) | GitHubError::Incomplete(_)
    )
}

/// Fetch the cross-repository inbox (assigned Open Issues + Review Requested
/// PRs) with a 5-minute SQLite cache.
///
/// Behaviour mirrors `get_github_stats_with_cache`:
/// - Both Search API queries succeed → cache + return `from_cache: false`.
/// - Network / rate-limit error → fall back to last cached payload (any age)
///   if available, else surface the error.
/// - 401 → trigger the central auth-expired flow; do not fall back to cache.
///
/// Search API budget: 30 req/min for authenticated users. Two requests per
/// refresh and a 5-minute TTL keep us well under that even with focus
/// revalidation.
#[tauri::command]
pub async fn get_my_open_work_with_cache(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CachedResponse<MyOpenWork>, String> {
    let user_id = get_current_user_id(&state).await?;
    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    // Fire both queries in parallel — they're independent and the latency
    // win matters more than the single request we'd save by short-circuiting
    // the second on failure of the first. Search API budget is 30 req/min;
    // even pathological "always fails" loops can't approach that with the
    // 5-minute TTL gating refreshes.
    let (assigned_result, reviews_result) = tokio::join!(
        client.search_assigned_issues(),
        client.search_review_requested(),
    );

    match (assigned_result, reviews_result) {
        (Ok(assigned), Ok(reviews)) => {
            let payload = MyOpenWork {
                assigned: assigned
                    .into_iter()
                    .map(|i| convert_search_item(i, "assigned"))
                    .collect(),
                review_requested: reviews
                    .into_iter()
                    .map(|i| convert_search_item(i, "review_requested"))
                    .collect(),
            };

            let payload_json = serde_json::to_string(&payload)
                .map_err(|e| format!("Failed to serialize my_open_work: {}", e))?;

            let now = Utc::now();
            let expires_at =
                now + chrono::Duration::minutes(crate::database::cache_durations::MY_OPEN_WORK);

            // Best effort: a cache failure shouldn't block the response.
            let _ = state
                .db
                .save_cache(
                    user_id,
                    crate::database::cache_types::MY_OPEN_WORK,
                    &payload_json,
                    expires_at,
                )
                .await;

            Ok(CachedResponse {
                data: payload,
                from_cache: false,
                cached_at: Some(now.to_rfc3339()),
                expires_at: Some(expires_at.to_rfc3339()),
            })
        }
        (assigned, reviews) => {
            // 401 must take precedence regardless of which call surfaced it.
            // Without this, a pattern like `assigned = network err` +
            // `reviews = 401` would have fallen through to cache fallback,
            // leaving a revoked token in place.
            if matches!(&assigned, Err(GitHubError::Unauthorized))
                || matches!(&reviews, Err(GitHubError::Unauthorized))
            {
                handle_unauthorized(&app, state.inner(), reasons::GITHUB_UNAUTHORIZED).await;
                return Err(GitHubError::Unauthorized.to_string());
            }

            // Mixed-failure handling: a hard (non-fallback-eligible) error
            // from *either* query takes priority over a transient one.
            // Without this, `(network err, 500 from API)` would silently
            // serve stale cache and hide the real backend problem from
            // the user.
            let err_a = assigned.as_ref().err();
            let err_b = reviews.as_ref().err();

            let hard = err_a
                .filter(|e| !is_network_or_rate_limit_error(e))
                .or_else(|| err_b.filter(|e| !is_network_or_rate_limit_error(e)));
            if let Some(e) = hard {
                return Err(format!("GitHub Search API error: {}", e));
            }

            // Every error remaining is network / rate-limit eligible — the
            // 401 case was already handled above. Use either one as the
            // representative for logs and the no-cache error message.
            // Both-Ok was handled in the outer match.
            let representative = err_a.or(err_b).expect("at least one Err in this branch");

            // Network / rate-limit error → cache fallback.
            eprintln!(
                "GitHub Search API error, attempting cache fallback: {}",
                representative
            );

            // Surface DB failures rather than masking them as "no cache":
            // when both the network and the local cache are broken, the
            // user deserves the actual root cause, not a misleading
            // "no cached data" message.
            let cache_result = state
                .db
                .get_any_cache(user_id, crate::database::cache_types::MY_OPEN_WORK)
                .await
                .map_err(|e| format!("Failed to read my_open_work cache: {}", e))?;

            match cache_result {
                Some((data_json, cached_at, expires_at)) => {
                    let payload: MyOpenWork = serde_json::from_str(&data_json)
                        .map_err(|e| format!("Failed to parse cached data: {}", e))?;
                    Ok(CachedResponse {
                        data: payload,
                        from_cache: true,
                        cached_at: Some(cached_at),
                        expires_at: Some(expires_at),
                    })
                }
                None => Err(format!(
                    "Search APIにアクセスできず、キャッシュもありません: {}",
                    representative
                )),
            }
        }
    }
}

// ============================================================================
// PR Progress dashboard panel (Issue #185)
// ============================================================================

/// Fetch the authenticated user's open PRs with mergeable / checks /
/// reviewDecision aggregated, with cache fallback.
///
/// Behaviour mirrors `get_my_open_work_with_cache`:
/// - Success → cache + return `from_cache: false`.
/// - Network / rate-limit error → fall back to last cached payload (any age)
///   if available, else surface the error.
/// - 401 → trigger the central auth-expired flow; do not fall back to cache.
///
/// Cost: GraphQL 5000-points/hour budget. Each PR node costs ~5 points and
/// the query is capped at 200 PRs (4 pages × 50). The 5-minute SQLite TTL
/// keeps us well under the budget even with revalidate-on-focus.
#[tauri::command]
pub async fn get_my_pr_progress_with_cache(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CachedResponse<PrProgress>, String> {
    let user_id = get_current_user_id(&state).await?;
    let access_token = get_access_token(&state).await?;
    let client = GitHubClient::new(access_token);

    match client.get_my_pr_progress().await {
        Ok(payload) => {
            let payload_json = serde_json::to_string(&payload)
                .map_err(|e| format!("Failed to serialize pr_progress: {}", e))?;

            let now = Utc::now();
            let expires_at =
                now + chrono::Duration::minutes(crate::database::cache_durations::MY_PR_PROGRESS);

            // Best-effort cache write — a failure here shouldn't block the
            // response. Mirrors `get_my_open_work_with_cache`.
            let _ = state
                .db
                .save_cache(
                    user_id,
                    crate::database::cache_types::MY_PR_PROGRESS,
                    &payload_json,
                    expires_at,
                )
                .await;

            Ok(CachedResponse {
                data: payload,
                from_cache: false,
                cached_at: Some(now.to_rfc3339()),
                expires_at: Some(expires_at.to_rfc3339()),
            })
        }
        Err(GitHubError::Unauthorized) => {
            handle_unauthorized(&app, state.inner(), reasons::GITHUB_UNAUTHORIZED).await;
            Err(GitHubError::Unauthorized.to_string())
        }
        Err(api_error) => {
            // Only network / rate-limit errors warrant cache fallback —
            // surface anything else (GraphQL schema error, JSON parse
            // failure, etc.) so we don't mask real bugs with stale data.
            if !matches!(
                &api_error,
                GitHubError::HttpRequest(_) | GitHubError::RateLimited(_)
            ) {
                return Err(format!("GitHub GraphQL error: {}", api_error));
            }

            eprintln!(
                "PR progress fetch failed, attempting cache fallback: {}",
                api_error
            );

            let cache_result = state
                .db
                .get_any_cache(user_id, crate::database::cache_types::MY_PR_PROGRESS)
                .await
                .map_err(|e| format!("Failed to read pr_progress cache: {}", e))?;

            match cache_result {
                Some((data_json, cached_at, expires_at)) => {
                    let payload: PrProgress = serde_json::from_str(&data_json)
                        .map_err(|e| format!("Failed to parse cached data: {}", e))?;
                    Ok(CachedResponse {
                        data: payload,
                        from_cache: true,
                        cached_at: Some(cached_at),
                        expires_at: Some(expires_at),
                    })
                }
                None => Err(format!(
                    "GitHub APIにアクセスできず、キャッシュもありません: {}",
                    api_error
                )),
            }
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Get the current user's ID from the database
async fn get_current_user_id(state: &State<'_, AppState>) -> Result<i64, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| format!("Failed to get user: {}", e))?
        .ok_or("Not logged in")?;

    let row = sqlx::query("SELECT id FROM users WHERE github_id = ?")
        .bind(user.github_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("User not found in database")?;

    Ok(row.get::<i64, _>("id"))
}

/// Get the current user's access token
async fn get_access_token(state: &State<'_, AppState>) -> Result<String, String> {
    state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| format!("Failed to get token: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::issues::{GitHubAssignee, GitHubLabel, GitHubSearchItem};
    use chrono::TimeZone;

    fn make_search_item(
        repository_url: &str,
        pull_request: Option<serde_json::Value>,
        labels: Vec<&str>,
    ) -> GitHubSearchItem {
        let created = Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap();
        let updated = Utc.with_ymd_and_hms(2026, 4, 2, 12, 0, 0).unwrap();
        GitHubSearchItem {
            id: 42,
            number: 7,
            title: "example".into(),
            body: None,
            state: "open".into(),
            state_reason: None,
            html_url: "https://github.com/octo/test/issues/7".into(),
            repository_url: repository_url.into(),
            labels: labels
                .into_iter()
                .enumerate()
                .map(|(i, name)| GitHubLabel {
                    id: i as i64,
                    name: name.to_string(),
                    color: "ededed".into(),
                    description: None,
                })
                .collect(),
            assignee: Some(GitHubAssignee {
                id: 1,
                login: "alice".into(),
                avatar_url: "https://avatars.githubusercontent.com/u/1".into(),
            }),
            assignees: None,
            user: Some(GitHubAssignee {
                id: 2,
                login: "bob".into(),
                avatar_url: "https://avatars.githubusercontent.com/u/2".into(),
            }),
            created_at: created,
            updated_at: updated,
            closed_at: None,
            pull_request,
        }
    }

    // TC-001: aggregating logic — verify the issue / source / labels / metadata
    // mapping for an `assigned` row.
    #[test]
    fn convert_search_item_maps_assigned_issue() {
        let item = make_search_item(
            "https://api.github.com/repos/octo/test",
            None,
            vec!["bug", "priority:high"],
        );

        let row = convert_search_item(item, "assigned");

        assert_eq!(row.kind, "issue");
        assert_eq!(row.source, "assigned");
        assert_eq!(row.repo_owner, "octo");
        assert_eq!(row.repo_name, "test");
        assert_eq!(row.repo_full_name, "octo/test");
        assert_eq!(row.priority.as_deref(), Some("high"));
        assert_eq!(
            row.labels,
            vec!["bug".to_string(), "priority:high".to_string()]
        );
        assert_eq!(row.assignee_login.as_deref(), Some("alice"));
        assert_eq!(row.author_login.as_deref(), Some("bob"));
    }

    // TC-001 + TC-006: PR detection through `is_pull_request` flows into
    // `kind = "pull_request"` for review-requested rows.
    #[test]
    fn convert_search_item_maps_review_requested_pull_request() {
        let item = make_search_item(
            "https://api.github.com/repos/octo/test",
            Some(serde_json::json!({"url": "..."})),
            vec![],
        );

        let row = convert_search_item(item, "review_requested");

        assert_eq!(row.kind, "pull_request");
        assert_eq!(row.source, "review_requested");
        assert!(row.priority.is_none());
        assert!(row.labels.is_empty());
    }

    // TC-007: a GHES (or any unexpected host) `repository_url` must not be
    // mis-parsed into a wrong owner/repo. Falling back to the raw URL keeps
    // the link clickable without inventing a bogus repo path.
    #[test]
    fn convert_search_item_falls_back_to_url_for_unexpected_host() {
        let item = make_search_item("https://ghe.example.com/api/v3/repos/o/r", None, vec![]);

        let row = convert_search_item(item, "assigned");

        assert_eq!(row.repo_owner, "");
        assert_eq!(row.repo_name, "");
        assert_eq!(
            row.repo_full_name,
            "https://ghe.example.com/api/v3/repos/o/r"
        );
    }

    // TC-002 / TC-005: rate-limit and incomplete-results errors must be
    // classified as transient so the command falls back to cached data.
    #[test]
    fn classifies_transient_errors_as_network_or_rate_limit() {
        assert!(is_network_or_rate_limit_error(&GitHubError::RateLimited(0)));
        assert!(is_network_or_rate_limit_error(&GitHubError::Incomplete(
            "search timeout".into()
        )));
    }

    // TC-004: 401 must NOT be classified as transient — the command needs to
    // fire the auth-expired flow instead of serving stale cache.
    // ApiError / NotFound / GraphQL / JsonParse likewise are surfaced rather
    // than masked: a malformed Search API payload is a real bug, not a
    // transient hiccup, and serving stale cache would hide it.
    #[test]
    fn classifies_hard_errors_as_non_transient() {
        assert!(!is_network_or_rate_limit_error(&GitHubError::Unauthorized));
        assert!(!is_network_or_rate_limit_error(&GitHubError::ApiError(
            "500 internal".into()
        )));
        assert!(!is_network_or_rate_limit_error(&GitHubError::NotFound(
            "x".into()
        )));
        assert!(!is_network_or_rate_limit_error(&GitHubError::GraphQL(
            "y".into()
        )));
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        assert!(!is_network_or_rate_limit_error(&GitHubError::JsonParse(
            json_err
        )));
    }
}
