/**
 * Main App Component
 *
 * Sets up routing using react-router-dom.
 * All pages are lazy-loaded for better performance.
 */

import { BrowserRouter, Routes, Route, useLocation, matchPath } from 'react-router-dom';
import { lazy, Suspense, useEffect } from 'react';
import { syncNavigationFromUrl } from './stores/navigationStore';
import { MainLayout } from './components/layouts';

// Loading fallback component
const PageLoading = () => (
  <div className="flex-1 flex items-center justify-center">
    <div className="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-cyan border-t-transparent"></div>
  </div>
);

// Lazy-load all pages
const Home = lazy(() => import('./pages/Home'));
const Projects = lazy(() => import('./pages/Projects'));
const ProjectDashboard = lazy(() => import('./pages/ProjectDashboard'));
const Issues = lazy(() => import('./pages/Issues'));
const Settings = lazy(() => import('./pages/Settings'));
const XpHistory = lazy(() => import('./pages/XpHistory'));
const NotFound = lazy(() => import('./pages/NotFound'));

/**
 * Navigation sync component - syncs URL with navigation store
 */
const NavigationSync = ({ children }: { children: React.ReactNode }) => {
  const location = useLocation();

  useEffect(() => {
    const projectMatch = matchPath('/projects/:id', location.pathname);
    syncNavigationFromUrl(
      location.pathname,
      projectMatch?.params.id ? { id: projectMatch.params.id } : undefined,
    );
  }, [location.pathname]);

  return <>{children}</>;
};

/**
 * Root Layout Component
 */
const RootLayout = () => {
  return (
    <NavigationSync>
      <MainLayout>
        <Suspense fallback={<PageLoading />}>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/projects" element={<Projects />} />
            <Route path="/projects/:id" element={<ProjectDashboard />} />
            <Route path="/issues" element={<Issues />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/xp-history" element={<XpHistory />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </Suspense>
      </MainLayout>
    </NavigationSync>
  );
};

const App = () => {
  return (
    <BrowserRouter>
      <RootLayout />
    </BrowserRouter>
  );
};

export default App;
