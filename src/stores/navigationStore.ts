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
import { AppPage, type AppPageType, isProjectDetailPage } from '@/types';

interface NavigationStore {
  currentPage: AppPageType;
}

const [navigationStore, setNavigationStore] = createStore<NavigationStore>({
  currentPage: AppPage.Home,
});

/**
 * Convert URL path to AppPageType
 */
export function pathToPageType(pathname: string, params?: Record<string, string>): AppPageType {
  if (pathname === '/') {
    return AppPage.Home;
  } else if (pathname === '/projects') {
    return AppPage.Projects;
  } else if (pathname.startsWith('/projects/') && params?.id) {
    const projectId = parseInt(params.id, 10);
    if (!isNaN(projectId)) {
      return { type: 'ProjectDetail', projectId };
    }
  } else if (pathname === '/issues') {
    return AppPage.Projects; // TODO: Add Issues page type when implemented
  } else if (pathname === '/mock-server') {
    return AppPage.MockServer;
  } else if (pathname === '/settings') {
    return AppPage.Settings;
  } else if (pathname === '/xp-history') {
    return AppPage.XpHistory;
  }
  // Default to Home
  return AppPage.Home;
}

/**
 * Update navigation store from URL
 * This should be called from within Router context
 */
export function syncNavigationFromUrl(pathname: string, params?: Record<string, string>) {
  const pageType = pathToPageType(pathname, params);
  setNavigationStore('currentPage', pageType);
}

/**
 * Navigation hook
 *
 * Provides current page state and method to navigate to a different page.
 */
export const useNavigation = () => {
  const setPage = (page: AppPageType) => {
    setNavigationStore('currentPage', page);
  };

  return {
    store: navigationStore,
    setPage,
  };
};

