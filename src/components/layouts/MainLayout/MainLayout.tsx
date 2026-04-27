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

import type { ReactNode } from 'react';
import { Sidebar } from '../Sidebar';
import { OfflineBanner } from '../OfflineBanner';
import { SessionExpiredBanner } from '../../features/auth';

/**
 * MainLayout Component
 *
 * Wraps the entire application with:
 * - Sidebar (navigation)
 * - OfflineBanner (network status)
 * - SessionExpiredBanner (GitHub auth-expired prompt — Issue #181)
 * - Main content area (children)
 *
 * Responsive design will be implemented in a later phase.
 */
export const MainLayout = ({ children }: { children: ReactNode }) => {
  return (
    <div className="flex h-screen bg-dt-bg">
      <Sidebar />
      <main className="flex-1 flex flex-col overflow-hidden">
        <OfflineBanner />
        <SessionExpiredBanner />
        {children}
      </main>
    </div>
  );
};
