/**
 * Tauri Command Wrappers
 *
 * Type-safe wrappers for all Tauri commands.
 * This file provides a unified API for invoking Tauri commands from the frontend.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/133
 *   - Commands: src-tauri/src/commands/
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  AuthState,
  UserInfo,
  DeviceCodeResponse,
  DeviceTokenStatus,
  UserSettings,
  UpdateSettingsRequest,
  DatabaseInfo,
  ClearCacheResult,
  AppInfo,
  SyncIntervalOption,
  Project,
  ProjectWithStats,
  RepositoryInfo,
  CachedIssue,
  KanbanBoard,
  ToolInfo,
  ToolConfig,
  MockServerState,
  MockServerConfig,
  UpdateConfigRequest,
  DirectoryMapping,
  CreateMappingRequest,
  UpdateMappingRequest,
  FileInfo,
  LevelInfo,
  Badge,
  BadgeDefinition,
  XpHistoryEntry,
  ChallengeInfo,
  CreateChallengeRequest,
  ChallengeStats,
  GitHubUser,
  GitHubStats,
  UserStats,
  SyncResult,
  CodeStatsSyncResult,
  CodeStatsResponse,
  RateLimitInfo,
  RateLimitDetailed,
  ContributionCalendar,
  BadgeWithProgress,
  CachedResponse,
  CacheStats,
} from '@/types';

// ============================================================================
// Auth Commands
// ============================================================================

export const auth = {
  /**
   * Get current authentication state
   */
  getState: (): Promise<AuthState> =>
    invoke<AuthState>('get_auth_state'),

  /**
   * Logout current user
   */
  logout: (): Promise<void> =>
    invoke<void>('logout'),

  /**
   * Get current user info
   */
  getCurrentUser: (): Promise<UserInfo | null> =>
    invoke<UserInfo | null>('get_current_user'),

  /**
   * Validate current token
   */
  validateToken: (): Promise<boolean> =>
    invoke<boolean>('validate_token'),

  /**
   * Start Device Flow authentication
   */
  startDeviceFlow: (): Promise<DeviceCodeResponse> =>
    invoke<DeviceCodeResponse>('start_device_flow'),

  /**
   * Poll for device token
   */
  pollDeviceToken: (): Promise<DeviceTokenStatus> =>
    invoke<DeviceTokenStatus>('poll_device_token'),

  /**
   * Cancel the current device flow
   */
  cancelDeviceFlow: (): Promise<void> =>
    invoke<void>('cancel_device_flow'),

  /**
   * Open a URL in the system's default browser
   */
  openUrl: (url: string): Promise<void> =>
    invoke<void>('open_url', { url }),
};

// ============================================================================
// Settings Commands
// ============================================================================

export const settings = {
  /**
   * Get user settings
   */
  get: (): Promise<UserSettings> =>
    invoke<UserSettings>('get_settings'),

  /**
   * Update user settings
   */
  update: (settings: UpdateSettingsRequest): Promise<UserSettings> =>
    invoke<UserSettings>('update_settings', { settings }),

  /**
   * Reset settings to defaults
   */
  reset: (): Promise<UserSettings> =>
    invoke<UserSettings>('reset_settings'),

  /**
   * Clear cache
   */
  clearCache: (): Promise<ClearCacheResult> =>
    invoke<ClearCacheResult>('clear_cache'),

  /**
   * Get database info
   */
  getDatabaseInfo: (): Promise<DatabaseInfo> =>
    invoke<DatabaseInfo>('get_database_info'),

  /**
   * Reset all user data
   */
  resetAllData: (): Promise<void> =>
    invoke<void>('reset_all_data'),

  /**
   * Get available sync interval options
   */
  getSyncIntervals: (): Promise<SyncIntervalOption[]> =>
    invoke<SyncIntervalOption[]>('get_sync_intervals'),

  /**
   * Export user data as JSON
   */
  exportData: (): Promise<string> =>
    invoke<string>('export_data'),

  /**
   * Get application information
   */
  getAppInfo: (): Promise<AppInfo> =>
    invoke<AppInfo>('get_app_info'),

  /**
   * Open URL in external browser
   */
  openExternalUrl: (url: string): Promise<void> =>
    invoke<void>('open_external_url', { url }),
};

// ============================================================================
// Project/Issue Commands
// ============================================================================

export const projects = {
  /**
   * Get all projects for the current user
   */
  list: (): Promise<ProjectWithStats[]> =>
    invoke<ProjectWithStats[]>('get_projects'),

  /**
   * Get a single project by ID
   */
  get: (project_id: number): Promise<Project> =>
    invoke<Project>('get_project', { project_id }),

  /**
   * Create a new project
   */
  create: (name: string, description?: string | null): Promise<Project> =>
    invoke<Project>('create_project', { name, description }),

  /**
   * Update a project
   */
  update: (project_id: number, name: string, description?: string | null): Promise<Project> =>
    invoke<Project>('update_project', { project_id, name, description }),

  /**
   * Delete a project
   */
  delete: (project_id: number): Promise<void> =>
    invoke<void>('delete_project', { project_id }),
};

export const repositories = {
  /**
   * Get user's repositories for linking
   */
  getUserRepositories: (): Promise<RepositoryInfo[]> =>
    invoke<RepositoryInfo[]>('get_user_repositories'),

  /**
   * Link a repository to a project
   */
  link: (project_id: number, owner: string, repo: string): Promise<Project> =>
    invoke<Project>('link_repository', { project_id, owner, repo }),
};

export const issues = {
  /**
   * Setup GitHub Actions for automatic status updates
   */
  setupGitHubActions: (project_id: number): Promise<string> =>
    invoke<string>('setup_github_actions', { project_id }),

  /**
   * Sync issues from GitHub to local cache
   */
  syncProjectIssues: (project_id: number): Promise<CachedIssue[]> =>
    invoke<CachedIssue[]>('sync_project_issues', { project_id }),

  /**
   * Get cached issues for a project
   */
  getProjectIssues: (project_id: number, status?: string | null): Promise<CachedIssue[]> =>
    invoke<CachedIssue[]>('get_project_issues', { project_id, status }),

  /**
   * Get issues as kanban board
   */
  getKanbanBoard: (project_id: number): Promise<KanbanBoard> =>
    invoke<KanbanBoard>('get_kanban_board', { project_id }),

  /**
   * Update issue status (also updates on GitHub)
   */
  updateStatus: (project_id: number, issue_number: number, new_status: string): Promise<CachedIssue> =>
    invoke<CachedIssue>('update_issue_status', { project_id, issue_number, new_status }),

  /**
   * Create a new issue (on GitHub and cache locally)
   */
  create: (
    project_id: number,
    title: string,
    body?: string | null,
    status?: string | null,
    priority?: string | null,
  ): Promise<CachedIssue> =>
    invoke<CachedIssue>('create_github_issue', { project_id, title, body, status, priority }),
};

// ============================================================================
// Tool Commands
// ============================================================================

export const tools = {
  /**
   * Get list of available tools
   */
  list: (): Promise<ToolInfo[]> =>
    invoke<ToolInfo[]>('list_tools'),

  /**
   * Get tool configuration
   */
  getConfig: (tool_name: string): Promise<ToolConfig> =>
    invoke<ToolConfig>('get_tool_config', { tool_name }),

  /**
   * Run a tool
   */
  run: (tool_name: string, options: Record<string, unknown>): Promise<void> =>
    invoke<void>('run_tool', { tool_name, options }),

  /**
   * Select a path using native dialog
   */
  selectPath: (
    path_type: string,
    title?: string | null,
    default_path?: string | null,
  ): Promise<string | null> =>
    invoke<string | null>('select_path', { path_type, title, default_path }),
};

// ============================================================================
// Mock Server Commands
// ============================================================================

export const mockServer = {
  /**
   * Get current Mock Server state
   */
  getState: (): Promise<MockServerState> =>
    invoke<MockServerState>('get_mock_server_state'),

  /**
   * Start the Mock Server
   */
  start: (): Promise<MockServerState> =>
    invoke<MockServerState>('start_mock_server'),

  /**
   * Stop the Mock Server
   */
  stop: (): Promise<MockServerState> =>
    invoke<MockServerState>('stop_mock_server'),

  /**
   * Get Mock Server configuration
   */
  getConfig: (): Promise<MockServerConfig> =>
    invoke<MockServerConfig>('get_mock_server_config'),

  /**
   * Update Mock Server configuration
   */
  updateConfig: (request: UpdateConfigRequest): Promise<MockServerConfig> =>
    invoke<MockServerConfig>('update_mock_server_config', { request }),

  /**
   * Get all directory mappings
   */
  getMappings: (): Promise<DirectoryMapping[]> =>
    invoke<DirectoryMapping[]>('get_mock_server_mappings'),

  /**
   * Create a new directory mapping
   */
  createMapping: (request: CreateMappingRequest): Promise<DirectoryMapping> =>
    invoke<DirectoryMapping>('create_mock_server_mapping', { request }),

  /**
   * Update a directory mapping
   */
  updateMapping: (request: UpdateMappingRequest): Promise<DirectoryMapping> =>
    invoke<DirectoryMapping>('update_mock_server_mapping', { request }),

  /**
   * Delete a directory mapping
   */
  deleteMapping: (id: number): Promise<void> =>
    invoke<void>('delete_mock_server_mapping', { id }),

  /**
   * List files in a directory
   */
  listDirectory: (path: string): Promise<FileInfo[]> =>
    invoke<FileInfo[]>('list_mock_server_directory', { path }),

  /**
   * Select a directory using native dialog
   */
  selectDirectory: (): Promise<string | null> =>
    invoke<string | null>('select_mock_server_directory'),
};

// ============================================================================
// Gamification Commands
// ============================================================================

export const gamification = {
  /**
   * Get level info for current user
   */
  getLevelInfo: (): Promise<LevelInfo | null> =>
    invoke<LevelInfo | null>('get_level_info'),

  /**
   * Add XP to current user (for testing/admin purposes)
   */
  addXp: (amount: number, action_type: string, description?: string | null): Promise<UserStats> =>
    invoke<UserStats>('add_xp', { amount, action_type, description }),

  /**
   * Get user's badges
   */
  getBadges: (): Promise<Badge[]> =>
    invoke<Badge[]>('get_badges'),

  /**
   * Award a badge to current user
   */
  awardBadge: (badge_type: string, badge_id: string): Promise<boolean> =>
    invoke<boolean>('award_badge', { badge_type, badge_id }),

  /**
   * Get recent XP history
   */
  getXpHistory: (limit?: number | null): Promise<XpHistoryEntry[]> =>
    invoke<XpHistoryEntry[]>('get_xp_history', { limit }),

  /**
   * Get all available badge definitions
   */
  getBadgeDefinitions: (): Promise<BadgeDefinition[]> =>
    invoke<BadgeDefinition[]>('get_badge_definitions'),
};

// ============================================================================
// Challenge Commands
// ============================================================================

export const challenges = {
  /**
   * Get all active challenges for current user
   */
  getActive: (): Promise<ChallengeInfo[]> =>
    invoke<ChallengeInfo[]>('get_active_challenges'),

  /**
   * Get all challenges (including completed and failed)
   */
  getAll: (): Promise<ChallengeInfo[]> =>
    invoke<ChallengeInfo[]>('get_all_challenges'),

  /**
   * Get challenges by type (daily/weekly)
   */
  getByType: (challenge_type: string): Promise<ChallengeInfo[]> =>
    invoke<ChallengeInfo[]>('get_challenges_by_type', { challenge_type }),

  /**
   * Create a custom challenge
   */
  create: (request: CreateChallengeRequest): Promise<ChallengeInfo> =>
    invoke<ChallengeInfo>('create_challenge', { request }),

  /**
   * Delete a challenge
   */
  delete: (challenge_id: number): Promise<void> =>
    invoke<void>('delete_challenge', { challenge_id }),

  /**
   * Update challenge progress manually (for testing/admin)
   */
  updateProgress: (challenge_id: number, current_value: number): Promise<ChallengeInfo> =>
    invoke<ChallengeInfo>('update_challenge_progress', { challenge_id, current_value }),

  /**
   * Get challenge completion stats
   */
  getStats: (): Promise<ChallengeStats> =>
    invoke<ChallengeStats>('get_challenge_stats'),
};

// ============================================================================
// GitHub Commands
// ============================================================================

export const github = {
  /**
   * Get GitHub user profile
   */
  getUser: (): Promise<GitHubUser> =>
    invoke<GitHubUser>('get_github_user'),

  /**
   * Get GitHub stats for the current user
   */
  getStats: (): Promise<GitHubStats> =>
    invoke<GitHubStats>('get_github_stats'),

  /**
   * Get local user stats (gamification data)
   */
  getUserStats: (): Promise<UserStats | null> =>
    invoke<UserStats | null>('get_user_stats'),

  /**
   * Sync GitHub stats to local database
   */
  syncStats: (): Promise<SyncResult> =>
    invoke<SyncResult>('sync_github_stats'),

  /**
   * Get contribution calendar
   */
  getContributionCalendar: (): Promise<ContributionCalendar> =>
    invoke<ContributionCalendar>('get_contribution_calendar'),

  /**
   * Get badges with progress information
   */
  getBadgesWithProgress: (): Promise<BadgeWithProgress[]> =>
    invoke<BadgeWithProgress[]>('get_badges_with_progress'),

  /**
   * Get badges that are close to being earned
   */
  getNearCompletionBadges: (threshold_percent?: number | null): Promise<BadgeWithProgress[]> =>
    invoke<BadgeWithProgress[]>('get_near_completion_badges', { threshold_percent }),

  /**
   * Sync code statistics from GitHub
   */
  syncCodeStats: (force_full_sync?: boolean | null): Promise<CodeStatsSyncResult> =>
    invoke<CodeStatsSyncResult>('sync_code_stats', { force_full_sync }),

  /**
   * Get code statistics summary for display
   */
  getCodeStatsSummary: (period?: string | null): Promise<CodeStatsResponse> =>
    invoke<CodeStatsResponse>('get_code_stats_summary', { period }),

  /**
   * Get detailed rate limit information
   */
  getRateLimitInfo: (): Promise<RateLimitDetailed> =>
    invoke<RateLimitDetailed>('get_rate_limit_info'),

  /**
   * Get GitHub stats with cache fallback
   */
  getStatsWithCache: (): Promise<CachedResponse<GitHubStats>> =>
    invoke<CachedResponse<GitHubStats>>('get_github_stats_with_cache'),

  /**
   * Get user stats with cache fallback
   */
  getUserStatsWithCache: (): Promise<CachedResponse<UserStats>> =>
    invoke<CachedResponse<UserStats>>('get_user_stats_with_cache'),
};

// ============================================================================
// Cache Management Commands
// ============================================================================

export const cache = {
  /**
   * Get cache statistics for the current user
   */
  getStats: (): Promise<CacheStats> =>
    invoke<CacheStats>('get_cache_stats'),

  /**
   * Clear all cache for the current user
   */
  clearUserCache: (): Promise<number> =>
    invoke<number>('clear_user_cache'),

  /**
   * Clear only expired cache entries (cleanup)
   */
  cleanupExpired: (): Promise<number> =>
    invoke<number>('cleanup_expired_cache'),
};

