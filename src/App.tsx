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
import type { Component } from 'solid-js';
import { syncNavigationFromUrl } from './stores/navigationStore';
import { MainLayout } from './components/layouts';

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
 * Router Sync Component
 *
 * Syncs navigation store with router URL changes.
 * This component must be inside Router context.
 */
const RouterSync: Component = () => {
  const location = useLocation();
  const params = useParams();

  // Sync navigation store with URL changes
  createEffect(() => {
    // Spread params to ensure reactivity on any param change
    syncNavigationFromUrl(location.pathname, { ...params });
  });

  return null;
};

const App: Component = () => {
  return (
    <Router>
      <RouterSync />
      <MainLayout>
        <Routes>
          <Route path="/" component={Home} />
          <Route path="/projects" component={Projects} />
          <Route path="/projects/:id" component={ProjectDashboard} />
          <Route path="/issues" component={Issues} />
          <Route path="/mock-server" component={MockServer} />
          <Route path="/settings" component={Settings} />
          <Route path="/xp-history" component={XpHistory} />
          <Route path="*" component={NotFound} />
        </Routes>
      </MainLayout>
    </Router>
  );
};

export default App;
