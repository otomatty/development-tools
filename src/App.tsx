/**
 * Main App Component
 *
 * Sets up routing using @solidjs/router v0.15.x.
 * All pages are lazy-loaded for better performance.
 * Integrates with navigation store for state synchronization.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/135
 *   - Pages: src/pages/
 *   - Navigation Store: src/stores/navigationStore.ts
 */

import { Router, Route, useLocation, useParams } from '@solidjs/router';
import { lazy, createEffect, Suspense } from 'solid-js';
import type { Component, ParentComponent } from 'solid-js';
import { syncNavigationFromUrl } from './stores/navigationStore';
import { MainLayout } from './components/layouts';

// Loading fallback component
const PageLoading: Component = () => (
  <div class="flex-1 flex items-center justify-center">
    <div class="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-cyan border-t-transparent"></div>
  </div>
);

// Lazy-load all pages for better performance
const Home = lazy(() => import('./pages/Home'));
const Projects = lazy(() => import('./pages/Projects'));
const ProjectDashboard = lazy(() => import('./pages/ProjectDashboard'));
const Issues = lazy(() => import('./pages/Issues'));
const MockServer = lazy(() => import('./pages/MockServer'));
const Settings = lazy(() => import('./pages/Settings'));
const XpHistory = lazy(() => import('./pages/XpHistory'));
const NotFound = lazy(() => import('./pages/NotFound'));

/**
 * Root Layout Component
 *
 * Wraps all routes with MainLayout and Suspense.
 * Also handles navigation state synchronization.
 */
const RootLayout: ParentComponent = (props) => {
  const location = useLocation();
  const params = useParams();

  // Sync navigation store with URL changes
  createEffect(() => {
    const safeParams: Record<string, string> = {};
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined) {
        safeParams[key] = value;
      }
    }
    syncNavigationFromUrl(location.pathname, safeParams);
  });

  return (
    <MainLayout>
      <Suspense fallback={<PageLoading />}>
        {props.children}
      </Suspense>
    </MainLayout>
  );
};

const App: Component = () => {
  return (
    <Router root={RootLayout}>
      <Route path="/" component={Home} />
      <Route path="/projects" component={Projects} />
      <Route path="/projects/:id" component={ProjectDashboard} />
      <Route path="/issues" component={Issues} />
      <Route path="/mock-server" component={MockServer} />
      <Route path="/settings" component={Settings} />
      <Route path="/xp-history" component={XpHistory} />
      <Route path="*" component={NotFound} />
    </Router>
  );
};

export default App;
