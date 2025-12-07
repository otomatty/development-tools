/**
 * Navigation Utility
 *
 * Provides navigation utilities for routing with @solidjs/router.
 * Integrates with navigation store for state management.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/135
 *   - Navigation Store: src/stores/navigationStore.ts
 *   - Types: src/types/index.ts (AppPage, AppPageType, ProjectDetailPage)
 */

import { useNavigate, useLocation } from '@solidjs/router';
import { AppPage, type AppPageType, type ProjectDetailPage, isProjectDetailPage } from '@/types';

export const pagePaths: Record<AppPage, string> = {
  [AppPage.Home]: '/',
  [AppPage.Projects]: '/projects',
  [AppPage.Issues]: '/issues',
  [AppPage.MockServer]: '/mock-server',
  [AppPage.Settings]: '/settings',
  [AppPage.XpHistory]: '/xp-history',
  [AppPage.NotFound]: '/404',
  // Tools is not used in routing (legacy)
  [AppPage.Tools]: '/tools',
};

/**
 * App Navigation Hook
 *
 * Provides navigation utilities for routing.
 * Use this hook to navigate between pages programmatically.
 *
 * @returns Navigation utilities including goTo function and currentPath
 */
export const useAppNavigation = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const goTo = (page: AppPageType, params?: Record<string, string>) => {
    // Handle ProjectDetailPage
    if (isProjectDetailPage(page)) {
      navigate(`/projects/${page.projectId}`);
      return;
    }

    // Handle AppPage enum
    const path = pagePaths[page];
    if (!path) {
      console.warn(`[useAppNavigation] Unknown page: ${page}`);
      navigate('/');
      return;
    }

    // Replace params in path if provided
    let finalPath = path;
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        finalPath = finalPath.replace(`:${key}`, value);
      });
    }

    navigate(finalPath);
  };

  return { goTo, currentPath: location.pathname };
};

