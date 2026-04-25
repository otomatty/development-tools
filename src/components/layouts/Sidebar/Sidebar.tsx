/**
 * Sidebar Component
 *
 * Main navigation sidebar for the application.
 * Displays navigation items, app header, and footer.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/137
 *   - SidebarItem: ./SidebarItem.tsx
 *   - Navigation Store: src/stores/navigationStore.ts
 */

import { Link, useLocation } from 'react-router-dom';
import { Icon } from '@/components/icons';
import { SidebarItem } from './SidebarItem';

interface NavItem {
  path: string;
  label: string;
  icon: string;
  exact?: boolean;
}

const navItems: NavItem[] = [
  { path: '/', label: 'ホーム', icon: 'home', exact: true },
  { path: '/projects', label: 'プロジェクト', icon: 'folder' },
  { path: '/issues', label: 'Issue', icon: 'list' },
  { path: '/settings', label: '設定', icon: 'settings' },
];

/**
 * Sidebar Component
 *
 * Main navigation sidebar with:
 * - App header (logo, app name)
 * - Navigation items (Home, Projects, Issues, Settings)
 * - Footer (version, settings button)
 */
export const Sidebar = () => {
  const location = useLocation();

  const isSettingsActive = location.pathname === '/settings';

  return (
    <aside className="w-64 bg-slate-900 border-r border-slate-700/50 flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b border-slate-700/50">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-lg">
            <Icon name="zap" className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-dt-text font-gaming">Dev Tools</h1>
            <p className="text-xs text-dt-text-sub">Level Up Your Dev</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <div className="p-3 space-y-1">
        {navItems.map((item) => (
          <SidebarItem
            key={item.path}
            path={item.path}
            label={item.label}
            icon={item.icon}
            exact={item.exact}
          />
        ))}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Footer */}
      <div className="p-3 border-t border-slate-700/50">
        <div className="flex items-center justify-between">
          <div className="text-xs text-dt-text-sub">v0.1.0</div>
          {/* Settings button */}
          <Link
            to="/settings"
            className={`p-2 rounded-lg transition-all duration-200 ${
              isSettingsActive
                ? 'bg-gm-accent-cyan/20 text-gm-accent-cyan'
                : 'text-slate-400 hover:bg-slate-800 hover:text-dt-text'
            }`}
            title="Settings"
            aria-label="Settings"
          >
            <Icon name="settings" className="w-5 h-5" />
          </Link>
        </div>
      </div>
    </aside>
  );
};
