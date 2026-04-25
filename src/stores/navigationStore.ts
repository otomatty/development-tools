/**
 * Navigation Store
 *
 * Manages application navigation state (current page) using zustand.
 * Integrates with React Router for URL synchronization.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/134, #135
 *   - Types: src/types/index.ts (AppPage, AppPageType, ProjectDetailPage)
 *   - Navigation Utility: src/lib/navigation.ts
 */

import { create } from 'zustand';
import { AppPage, type AppPageType } from '@/types';

interface NavigationStore {
  currentPage: AppPageType;
  setCurrentPage: (page: AppPageType) => void;
}

export const useNavigation = create<NavigationStore>((set) => ({
  currentPage: AppPage.Home,
  setCurrentPage: (page: AppPageType) => set({ currentPage: page }),
}));

/**
 * Parse route path and parameters to AppPageType
 * Handles both static routes and dynamic routes with parameters.
 */
export function parseRouteToPageType(pathname: string, params?: Record<string, string>): AppPageType {
  if (pathname === '/') return AppPage.Home;
  if (pathname === '/projects') return AppPage.Projects;
  if (pathname.startsWith('/projects/') && params?.id) {
    const projectId = parseInt(params.id, 10);
    if (!isNaN(projectId)) return { type: 'ProjectDetail', projectId };
    console.warn(`[navigationStore] Invalid project ID in URL: "${params.id}" (parsed as NaN)`);
    return AppPage.NotFound;
  }
  if (pathname === '/issues') return AppPage.Issues;
  if (pathname === '/settings') return AppPage.Settings;
  if (pathname === '/xp-history') return AppPage.XpHistory;
  return AppPage.NotFound;
}

/**
 * Update navigation store from URL
 * This should be called from within Router context
 */
export function syncNavigationFromUrl(pathname: string, params?: Record<string, string>) {
  const pageType = parseRouteToPageType(pathname, params);
  useNavigation.getState().setCurrentPage(pageType);
}
