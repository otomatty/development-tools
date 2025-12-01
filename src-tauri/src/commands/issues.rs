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
//!   ├─ src-tauri/src/database/models/project.rs
//!   ├─ src-tauri/src/github/issues.rs
//!   └─ src-tauri/src/commands/auth.rs (for auth state)

use chrono::Utc;
use sqlx::Row;
use tauri::State;

use crate::commands::AppState;
use crate::database::models::project::{
    CachedIssue, IssueStatus, KanbanBoard, Project, ProjectWithStats, RepositoryInfo,
};
use crate::github::issues::{generate_actions_template, IssuesClient};

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
    state: State<'_, AppState>,
) -> Result<Vec<RepositoryInfo>, String> {
    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    let repos = client
        .get_user_repositories()
        .await
        .map_err(|e| format!("Failed to fetch repositories: {:?}", e))?;

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
    state: State<'_, AppState>,
    project_id: i64,
    owner: String,
    repo: String,
) -> Result<Project, String> {
    let user_id = get_current_user_id(&state).await?;
    let access_token = get_access_token(&state).await?;
    let client = IssuesClient::new(access_token);

    // Get repository info from GitHub
    let repo_info = client
        .get_repository(&owner, &repo)
        .await
        .map_err(|e| format!("Failed to fetch repository: {:?}", e))?;

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
        let issues = client
            .get_issues(&owner, &repo, "all", 100, page)
            .await
            .map_err(|e| format!("Failed to fetch issues: {:?}", e))?;

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
        let status = IssuesClient::extract_status(&issue.labels);
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
    let updated_issue = client
        .update_issue_status(&owner, &repo, issue_number, status)
        .await
        .map_err(|e| format!("Failed to update issue status: {:?}", e))?;

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
    let github_issue = client
        .create_issue(&owner, &repo, &title, body.as_deref(), labels)
        .await
        .map_err(|e| format!("Failed to create issue: {:?}", e))?;

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
