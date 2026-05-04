/**
 * Icon Component
 *
 * Wraps lucide-react icons to provide a consistent icon interface.
 * Maps existing icon names to lucide-react icon components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/137
 *   - lucide-react: https://lucide.dev/guide/packages/lucide-react
 */

import * as LucideIcons from 'lucide-react';
import type { IconProps } from '@/types/ui';

/**
 * Maps existing icon names to lucide-react icon component names
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
  'git-merge': 'GitMerge',
  'git-pull-request': 'GitPullRequest',
  'git-pull-request-closed': 'GitPullRequestClosed',
  'clock': 'Clock',
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
 * Renders an icon from lucide-react based on the provided name.
 * Falls back to a default icon if the name is not found.
 */
export const Icon = ({ name, className, size, strokeWidth }: IconProps) => {
  const iconKey = iconNameMap[name];

  const IconComponent =
    (iconKey ? (LucideIcons[iconKey] as LucideIcons.LucideIcon | undefined) : undefined) ??
    LucideIcons.AlertCircle;

  const defaultClass = 'w-5 h-5';
  const iconClass = className || defaultClass;

  const computedSize = (() => {
    if (size) return size;
    // Extract size from class if present (e.g., "w-6 h-6" -> 24)
    const match = iconClass.match(/w-(\d+(?:\.\d+)?)/);
    if (match) {
      return parseFloat(match[1]) * 4; // Tailwind spacing unit (1 = 4px)
    }
    return 20; // Default size
  })();

  return (
    <IconComponent
      className={iconClass}
      size={computedSize}
      strokeWidth={strokeWidth ?? 2}
    />
  );
};
