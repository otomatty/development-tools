// Issue management types for frontend

/// Issue status (maps to GitHub labels)
export type IssueStatus = 'backlog' | 'todo' | 'in-progress' | 'in-review' | 'done' | 'cancelled';

/// Get display name
export function issueStatusDisplayName(status: IssueStatus): string {
  switch (status) {
    case 'backlog':
      return 'Backlog';
    case 'todo':
      return 'Todo';
    case 'in-progress':
      return 'In Progress';
    case 'in-review':
      return 'In Review';
    case 'done':
      return 'Done';
    case 'cancelled':
      return 'Cancelled';
  }
}

/// Get CSS color class
export function issueStatusColorClass(status: IssueStatus): string {
  switch (status) {
    case 'backlog':
      return 'bg-gray-400';
    case 'todo':
      return 'bg-blue-500';
    case 'in-progress':
      return 'bg-yellow-500';
    case 'in-review':
      return 'bg-purple-500';
    case 'done':
      return 'bg-green-500';
    case 'cancelled':
      return 'bg-gray-500';
  }
}

/// Get all statuses in order
export function getAllIssueStatuses(): IssueStatus[] {
  return ['backlog', 'todo', 'in-progress', 'in-review', 'done', 'cancelled'];
}

/// Get visible statuses (for kanban board display)
export function getVisibleIssueStatuses(): IssueStatus[] {
  return ['backlog', 'todo', 'in-progress', 'in-review', 'done', 'cancelled'];
}

/// Issue priority
export type IssuePriority = 'high' | 'medium' | 'low';

/// Get display name
export function issuePriorityDisplayName(priority: IssuePriority): string {
  switch (priority) {
    case 'high':
      return 'High';
    case 'medium':
      return 'Medium';
    case 'low':
      return 'Low';
  }
}

/// Get emoji indicator
export function issuePriorityEmoji(priority: IssuePriority): string {
  switch (priority) {
    case 'high':
      return '🔴';
    case 'medium':
      return '🟡';
    case 'low':
      return '🟢';
  }
}

/// Get CSS color class
export function issuePriorityColorClass(priority: IssuePriority): string {
  switch (priority) {
    case 'high':
      return 'text-red-500';
    case 'medium':
      return 'text-yellow-500';
    case 'low':
      return 'text-green-500';
  }
}

/// Project model (1 project = 1 repository)
export interface Project {
  id: number;
  userId: number;
  name: string;
  description: string | null;
  githubRepoId: number | null;
  repoOwner: string | null;
  repoName: string | null;
  repoFullName: string | null;
  isActionsSetup: boolean;
  lastSyncedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

/// Check if repository is linked
export function isLinked(project: Project): boolean {
  return project.githubRepoId !== null;
}

/// Get repository display name
export function repoDisplayName(project: Project): string | null {
  return project.repoFullName;
}

/// Project with additional stats
export interface ProjectWithStats {
  id: number;
  userId: number;
  name: string;
  description: string | null;
  githubRepoId: number | null;
  repoOwner: string | null;
  repoName: string | null;
  repoFullName: string | null;
  isActionsSetup: boolean;
  lastSyncedAt: string | null;
  createdAt: string;
  updatedAt: string;
  openIssuesCount: number;
  totalIssuesCount: number;
}

/// Cached issue model
export interface CachedIssue {
  id: number;
  projectId: number;
  githubIssueId: number;
  number: number;
  title: string;
  body: string | null;
  state: string;
  status: string;
  priority: string | null;
  assigneeLogin: string | null;
  assigneeAvatarUrl: string | null;
  labelsJson: string | null;
  htmlUrl: string | null;
  githubCreatedAt: string | null;
  githubUpdatedAt: string | null;
  cachedAt: string;
}

/// Get parsed status
export function getIssueStatus(issue: CachedIssue): IssueStatus {
  switch (issue.status) {
    case 'backlog':
      return 'backlog';
    case 'todo':
      return 'todo';
    case 'in-progress':
      return 'in-progress';
    case 'in-review':
      return 'in-review';
    case 'done':
      return 'done';
    case 'cancelled':
      return 'cancelled';
    default:
      return 'backlog';
  }
}

/// Get parsed priority
export function getIssuePriority(issue: CachedIssue): IssuePriority | null {
  if (!issue.priority) {
    return null;
  }
  switch (issue.priority) {
    case 'high':
      return 'high';
    case 'medium':
      return 'medium';
    case 'low':
      return 'low';
    default:
      return null;
  }
}

/// Get parsed labels
export function getIssueLabels(issue: CachedIssue): string[] {
  if (!issue.labelsJson) {
    return [];
  }
  try {
    const parsed = JSON.parse(issue.labelsJson);
    if (Array.isArray(parsed) && parsed.every((item) => typeof item === 'string')) {
      return parsed;
    }
    console.error('Parsed labelsJson is not a string array:', parsed);
    return [];
  } catch (error) {
    console.error('Failed to parse labelsJson', error);
    return [];
  }
}

/// Check if issue is open
export function isIssueOpen(issue: CachedIssue): boolean {
  return issue.state === 'open';
}

/// Check if issue was updated within the specified number of days
export function isIssueUpdatedWithinDays(issue: CachedIssue, days: number): boolean {
  const updatedAt = issue.githubUpdatedAt;
  if (!updatedAt) {
    return true; // If no date, show it (conservative approach)
  }

  // Parse RFC3339 date
  const updatedMs = new Date(updatedAt).getTime();
  if (isNaN(updatedMs)) {
    return true; // If parse fails, show it
  }

  const nowMs = Date.now();
  const daysMs = days * 24 * 60 * 60 * 1000;

  return (nowMs - updatedMs) <= daysMs;
}

/// Check if issue is a completed status (Done or Cancelled)
export function isCompletedStatus(issue: CachedIssue): boolean {
  return issue.status === 'done' || issue.status === 'cancelled';
}

/// Issues grouped by status for kanban display
export interface KanbanBoard {
  backlog: CachedIssue[];
  todo: CachedIssue[];
  inProgress: CachedIssue[];
  inReview: CachedIssue[];
  done: CachedIssue[];
  cancelled: CachedIssue[];
}

/// Create kanban board from issues list
export function createKanbanBoard(issues: CachedIssue[]): KanbanBoard {
  const board: KanbanBoard = {
    backlog: [],
    todo: [],
    inProgress: [],
    inReview: [],
    done: [],
    cancelled: [],
  };

  for (const issue of issues) {
    const status = getIssueStatus(issue);
    switch (status) {
      case 'backlog':
        board.backlog.push(issue);
        break;
      case 'todo':
        board.todo.push(issue);
        break;
      case 'in-progress':
        board.inProgress.push(issue);
        break;
      case 'in-review':
        board.inReview.push(issue);
        break;
      case 'done':
        board.done.push(issue);
        break;
      case 'cancelled':
        board.cancelled.push(issue);
        break;
    }
  }

  return board;
}

/// Get issues for a specific status
export function getIssuesByStatus(board: KanbanBoard, status: IssueStatus): CachedIssue[] {
  switch (status) {
    case 'backlog':
      return board.backlog;
    case 'todo':
      return board.todo;
    case 'in-progress':
      return board.inProgress;
    case 'in-review':
      return board.inReview;
    case 'done':
      return board.done;
    case 'cancelled':
      return board.cancelled;
  }
}

/// Get count for a specific status
export function getStatusCount(board: KanbanBoard, status: IssueStatus): number {
  return getIssuesByStatus(board, status).length;
}

/// Get total count
export function getTotalCount(board: KanbanBoard): number {
  return (
    board.backlog.length +
    board.todo.length +
    board.inProgress.length +
    board.inReview.length +
    board.done.length +
    board.cancelled.length
  );
}

/// GitHub repository info for linking
export interface RepositoryInfo {
  id: number;
  name: string;
  fullName: string;
  owner: string;
  description: string | null;
  htmlUrl: string;
  private: boolean;
  openIssuesCount: number;
}

