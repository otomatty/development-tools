/**
 * Navigation Utility
 *
 * Provides navigation utilities for routing with @solidjs/router.
 * Integrates with navigation store for state management.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/135
 *   - Navigation Store: src/stores/navigationStore.ts
 */

import { useNavigate, useLocation } from '@solidjs/router';

export type AppPage =
  | 'home'
  | 'projects'
  | 'project-dashboard'
  | 'issues'
  | 'mock-server'
  | 'settings'
  | 'xp-history';

export const pagePaths: Record<AppPage, string> = {
  'home': '/',
  'projects': '/projects',
  'project-dashboard': '/projects/:id',
  'issues': '/issues',
  'mock-server': '/mock-server',
  'settings': '/settings',
  'xp-history': '/xp-history',
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

  const goTo = (page: AppPage, params?: Record<string, string>) => {
    let path = pagePaths[page];
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        path = path.replace(`:${key}`, value);
      });
    }
    navigate(path);
  };

  return { goTo, currentPath: location.pathname };
};

