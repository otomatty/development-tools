/**
 * SidebarItem Component
 *
 * Individual navigation item in the sidebar.
 * Highlights when the current route matches the item's path.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/137
 *   - Sidebar: ./Sidebar.tsx
 */

import { Component, Show } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
import { Icon } from '@/components/icons';

export interface SidebarItemProps {
  path: string;
  label: string;
  icon: string;
  exact?: boolean; // If true, only match exact path
}

/**
 * SidebarItem Component
 *
 * Renders a navigation item with icon and label.
 * Automatically highlights when the current route matches.
 */
export const SidebarItem: Component<SidebarItemProps> = (props) => {
  const location = useLocation();

  const isActive = () => {
    if (props.exact) {
      return location.pathname === props.path;
    }
    // For non-exact matching, check if pathname starts with the path
    // Special case for root path
    if (props.path === '/') {
      return location.pathname === '/';
    }
    return location.pathname.startsWith(props.path);
  };

  const activeClass = () => {
    if (isActive()) {
      return 'bg-gradient-to-r from-gm-accent-cyan/20 to-gm-accent-purple/20 text-gm-accent-cyan border-l-2 border-gm-accent-cyan';
    }
    return 'text-slate-400 hover:bg-slate-800 hover:text-dt-text';
  };

  return (
    <A
      href={props.path}
      class={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 ${activeClass()}`}
    >
      <Icon name={props.icon} class="w-5 h-5" />
      <span class="font-medium">{props.label}</span>
    </A>
  );
};

