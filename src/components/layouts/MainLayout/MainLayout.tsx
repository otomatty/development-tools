/**
 * MainLayout Component
 *
 * Main layout wrapper for the application.
 * Provides Sidebar, OfflineBanner, and main content area.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/137
 *   - Sidebar: ../Sidebar/Sidebar.tsx
 *   - OfflineBanner: ../OfflineBanner/OfflineBanner.tsx
 */

import { ParentComponent } from 'solid-js';
import { Sidebar } from '../Sidebar';
import { OfflineBanner } from '../OfflineBanner';

/**
 * MainLayout Component
 *
 * Wraps the entire application with:
 * - Sidebar (navigation)
 * - OfflineBanner (network status)
 * - Main content area (children)
 *
 * Responsive design will be implemented in a later phase.
 */
export const MainLayout: ParentComponent = (props) => {
  return (
    <div class="flex h-screen bg-dt-bg">
      <Sidebar />
      <main class="flex-1 flex flex-col overflow-hidden">
        <OfflineBanner />
        {props.children}
      </main>
    </div>
  );
};

