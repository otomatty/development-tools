/**
 * Main App Component
 *
 * Sets up routing using @solidjs/router.
 * All pages are lazy-loaded for better performance.
 * Integrates with navigation store for state synchronization.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/135
 *   - Pages: src/pages/
 *   - Navigation Store: src/stores/navigationStore.ts
 */

import { Router, Route, Routes, useLocation, useParams } from '@solidjs/router';
import { lazy, createEffect } from 'solid-js';
import { syncNavigationFromUrl } from './stores/navigationStore';

// Lazy-load all pages for better performance
const Home = lazy(() => import('./pages/Home'));
const Projects = lazy(() => import('./pages/Projects'));
const ProjectDashboard = lazy(() => import('./pages/ProjectDashboard'));
const Issues = lazy(() => import('./pages/Issues'));
const MockServer = lazy(() => import('./pages/MockServer'));
const Settings = lazy(() => import('./pages/Settings'));
const XpHistory = lazy(() => import('./pages/XpHistory'));

/**
 * Router Sync Component
 *
 * Syncs navigation store with router URL changes.
 * This component must be inside Router context.
 */
function RouterSync() {
  const location = useLocation();
  const params = useParams();

  // Sync navigation store with URL changes
  createEffect(() => {
    syncNavigationFromUrl(location.pathname, params());
  });

  return null;
}

const App = () => {
  return (
    <Router>
      <RouterSync />
      <Routes>
        <Route path="/" component={Home} />
        <Route path="/projects" component={Projects} />
        <Route path="/projects/:id" component={ProjectDashboard} />
        <Route path="/issues" component={Issues} />
        <Route path="/mock-server" component={MockServer} />
        <Route path="/settings" component={Settings} />
        <Route path="/xp-history" component={XpHistory} />
        <Route
          path="*"
          element={
            <div class="min-h-screen bg-dt-bg text-dt-text flex items-center justify-center">
              <div class="text-center">
                <h1 class="text-4xl font-bold text-gm-accent-cyan">404</h1>
                <p class="mt-4 text-dt-text-sub">Page Not Found</p>
              </div>
            </div>
          }
        />
      </Routes>
    </Router>
  );
};

export default App;
