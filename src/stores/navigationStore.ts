/**
 * Navigation Store
 *
 * Manages application navigation state (current page) using Solid.js stores.
 * Integrates with @solidjs/router for URL synchronization.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134, #135
 *   - Types: src/types/index.ts (AppPage, AppPageType, ProjectDetailPage)
 *   - Navigation Utility: src/lib/navigation.ts
 */

import { createStore } from 'solid-js/store';
import { AppPage, type AppPageType } from '@/types';

interface NavigationStore {
  currentPage: AppPageType;
}

const [navigationStore, setNavigationStore] = createStore<NavigationStore>({
  currentPage: AppPage.Home,
});

/**
 * Parse route path and parameters to AppPageType
 * Handles both static routes and dynamic routes with parameters.
 */
export function parseRouteToPageType(pathname: string, params?: Record<string, string>): AppPageType {
  if (pathname === '/') {
    return AppPage.Home;
  } else if (pathname === '/projects') {
    return AppPage.Projects;
  } else if (pathname.startsWith('/projects/') && params?.id) {
    const projectId = parseInt(params.id, 10);
    if (!isNaN(projectId)) {
      return { type: 'ProjectDetail', projectId };
    } else {
      console.warn(`[navigationStore] Invalid project ID in URL: "${params.id}" (parsed as NaN)`);
      // Invalid project ID - treat as 404
      return AppPage.NotFound;
    }
  } else if (pathname === '/issues') {
    return AppPage.Issues;
  } else if (pathname === '/mock-server') {
    return AppPage.MockServer;
  } else if (pathname === '/settings') {
    return AppPage.Settings;
  } else if (pathname === '/xp-history') {
    return AppPage.XpHistory;
  }
  // Unknown path - return NotFound
  return AppPage.NotFound;
}

/**
 * Update navigation store from URL
 * This should be called from within Router context
 */
export function syncNavigationFromUrl(pathname: string, params?: Record<string, string>) {
  const pageType = parseRouteToPageType(pathname, params);
  setNavigationStore('currentPage', pageType);
}

/**
 * Navigation hook
 *
 * Provides current page state.
 * NOTE: We do not expose a setPage method here to prevent direct manipulation
 * of navigation state without updating the URL. Always use router-based navigation
 * (e.g., useAppNavigation().goTo() from lib/navigation.ts) to ensure state and URL stay in sync.
 */
export const useNavigation = () => {
  return {
    store: navigationStore,
  };
};

