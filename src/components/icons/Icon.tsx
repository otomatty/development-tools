/**
 * Icon Component
 *
 * Wraps lucide-solid icons to provide a consistent icon interface.
 * Maps existing icon names to lucide-solid icon components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/137
 *   - lucide-solid: https://lucide.dev/guide/packages/lucide-solid
 */

import { Component } from 'solid-js';
import * as LucideIcons from 'lucide-solid';
import type { IconProps } from '@/types/ui';

/**
 * Maps existing icon names to lucide-solid icon component names
 */
const iconNameMap: Record<string, keyof typeof LucideIcons> = {
  'shield': 'Shield',
  'search': 'Search',
  'code': 'Code',
  'package': 'Package',
  'terminal': 'Terminal',
  'settings': 'Settings',
  'chart': 'BarChart3',
  'file': 'File',
  'alert-circle': 'AlertCircle',
  'alert-triangle': 'AlertTriangle',
  'file-warning': 'FileWarning',
  'play': 'Play',
  'stop': 'Square',
  'folder': 'Folder',
  'check': 'Check',
  'x': 'X',
  'loader': 'Loader2',
  'wrench': 'Wrench',
  'home': 'Home',
  'zap': 'Zap',
  'user': 'User',
  'trophy': 'Trophy',
  'fire': 'Flame',
  'star': 'Star',
  'badge': 'Badge',
  'bell': 'Bell',
  'refresh-cw': 'RefreshCw',
  'palette': 'Palette',
  'database': 'Database',
  'info': 'Info',
  'check-square': 'CheckSquare',
  'tool': 'Wrench',
  'file-text': 'FileText',
  'chevron-down': 'ChevronDown',
  'chevron-up': 'ChevronUp',
  'download': 'Download',
  'trash': 'Trash2',
  'external-link': 'ExternalLink',
  'more-vertical': 'MoreVertical',
  'logout': 'LogOut',
  'kanban': 'LayoutGrid',
  'plus': 'Plus',
  'git-branch': 'GitBranch',
  'github': 'Github',
  'link': 'Link',
  'refresh': 'RefreshCw',
  'circle': 'Circle',
  'arrow-right': 'ArrowRight',
  'clipboard-copy': 'Copy',
  'radio': 'Radio',
  'expand': 'Maximize2',
  'list': 'List',
  'server': 'Server',
};

/**
 * Icon Component
 *
 * Renders an icon from lucide-solid based on the provided name.
 * Falls back to a default icon if the name is not found.
 */
export const Icon: Component<IconProps> = (props) => {
  const iconName = () => iconNameMap[props.name];
  
  const getIconComponent = () => {
    const name = iconName();
    if (!name) {
      return LucideIcons.AlertCircle;
    }
    const Component = LucideIcons[name];
    if (!Component) {
      return LucideIcons.AlertCircle;
    }
    return Component;
  };

  const defaultClass = 'w-5 h-5';
  const iconClass = () => props.class || defaultClass;
  const size = () => {
    if (props.size) return props.size;
    // Extract size from class if present (e.g., "w-6 h-6" -> 24)
    const match = iconClass().match(/w-(\d+)/);
    if (match) {
      return parseInt(match[1]) * 4; // Tailwind spacing unit (1 = 4px)
    }
    return 20; // Default size
  };

  const IconComponent = getIconComponent();

  return (
    <IconComponent
      class={iconClass()}
      size={size()}
      stroke-width={props.strokeWidth || 2}
    />
  );
};

