/**
 * Tauri API Wrapper
 *
 * Unified, type-safe API for interacting with Tauri commands and events.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/133
 *   - Commands: ./commands.ts
 *   - Events: ./events.ts
 */

export * from './commands';
export * from './events';

// Re-export for convenience
export { auth, settings, projects, repositories, issues, tools, mockServer, gamification, challenges, github, cache } from './commands';
export { events } from './events';

