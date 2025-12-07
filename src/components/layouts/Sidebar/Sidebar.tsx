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

import { Component, Show } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
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
  { path: '/mock-server', label: 'Mock Server', icon: 'server' },
  { path: '/settings', label: '設定', icon: 'settings' },
];

/**
 * Sidebar Component
 *
 * Main navigation sidebar with:
 * - App header (logo, app name)
 * - Navigation items (Home, Projects, Issues, Mock Server, Settings)
 * - Footer (version, settings button)
 */
export const Sidebar: Component = () => {
  const location = useLocation();

  const isSettingsActive = () => location.pathname === '/settings';

  return (
    <aside class="w-64 bg-slate-900 border-r border-slate-700/50 flex flex-col h-full">
      {/* Header */}
      <div class="p-4 border-b border-slate-700/50">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-lg">
            <Icon name="zap" class="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 class="text-lg font-semibold text-dt-text font-gaming">Dev Tools</h1>
            <p class="text-xs text-dt-text-sub">Level Up Your Dev</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <div class="p-3 space-y-1">
        {navItems.map((item) => (
          <SidebarItem
            path={item.path}
            label={item.label}
            icon={item.icon}
            exact={item.exact}
          />
        ))}
      </div>

      {/* Spacer */}
      <div class="flex-1" />

      {/* Footer */}
      <div class="p-3 border-t border-slate-700/50">
        <div class="flex items-center justify-between">
          <div class="text-xs text-dt-text-sub">v0.1.0</div>
          {/* Settings button */}
          <A
            href="/settings"
            class={`p-2 rounded-lg transition-all duration-200 ${
              isSettingsActive()
                ? 'bg-gm-accent-cyan/20 text-gm-accent-cyan'
                : 'text-slate-400 hover:bg-slate-800 hover:text-dt-text'
            }`}
            title="Settings"
          >
            <Icon name="settings" class="w-5 h-5" />
          </A>
        </div>
      </div>
    </aside>
  );
};

