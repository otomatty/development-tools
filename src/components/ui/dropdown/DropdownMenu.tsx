/**
 * DropdownMenu Components
 *
 * Solid.js implementation of DropdownMenu, DropdownMenuItem, and DropdownMenuDivider components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./DropdownMenu.spec.md
 *   - Original (Leptos): ./dropdown_menu.rs
 */

import {
  Component,
  createSignal,
  createContext,
  useContext,
  Show,
  onMount,
  onCleanup,
  splitProps,
  JSX,
} from 'solid-js';
import type {
  DropdownMenuProps,
  DropdownMenuItemProps,
  DropdownMenuDividerProps,
} from '../../../types/ui';
import { useAnimation } from '../../../stores/animationStore';

// ============================================================================
// DropdownMenu Context
// ============================================================================

interface DropdownMenuContextValue {
  isOpen: () => boolean;
  setIsOpen: (value: boolean) => void;
  closeMenu: () => void;
}

const DropdownMenuContext = createContext<DropdownMenuContextValue>();

// ============================================================================
// DropdownMenu Component
// ============================================================================

export const DropdownMenu: Component<DropdownMenuProps> = (props) => {
  const [local, others] = splitProps(props, ['trigger', 'children', 'align', 'class']);
  const [isOpen, setIsOpen] = createSignal(false);
  const animation = useAnimation();

  const align = () => local.align ?? 'right';
  const closeMenu = () => setIsOpen(false);
  const toggleMenu = () => setIsOpen((prev) => !prev);

  // Handle ESC key
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen()) {
        closeMenu();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    onCleanup(() => window.removeEventListener('keydown', handleKeyDown));
  });

  // Provide context for child components
  const contextValue: DropdownMenuContextValue = {
    isOpen,
    setIsOpen,
    closeMenu,
  };

  // Menu classes
  const alignClass = align() === 'left' ? 'left-0' : 'right-0';
  const baseMenuClass = `absolute ${alignClass} top-full mt-2 min-w-[160px] bg-gm-bg-card/95 backdrop-blur-sm border border-gm-accent-cyan/20 rounded-lg shadow-lg z-50`;
  const menuClass = animation.store.enabled
    ? `${baseMenuClass} transition-all duration-200 ease-out`
    : baseMenuClass;

  // Menu style (for animation)
  const menuStyle = () => {
    if (isOpen()) {
      return 'opacity: 1; transform: translateY(0);';
    } else if (animation.store.enabled) {
      return 'opacity: 0; transform: translateY(-8px); pointer-events: none;';
    } else {
      return 'display: none;';
    }
  };

  return (
    <DropdownMenuContext.Provider value={contextValue}>
      <div class={`relative ${local.class || ''}`.trim()} {...others}>
        {/* Overlay for click outside detection */}
        <Show when={isOpen()}>
          <div class="fixed inset-0 z-40" onClick={closeMenu} />
        </Show>

        {/* Trigger button */}
        <button
          type="button"
          class="p-2 text-dt-text-sub hover:text-gm-accent-cyan transition-colors rounded-lg"
          aria-expanded={isOpen()}
          aria-haspopup="true"
          onClick={toggleMenu}
        >
          {typeof local.trigger === 'function' ? local.trigger() : local.trigger}
        </button>

        {/* Dropdown menu */}
        <div class={menuClass} style={menuStyle()} role="menu" aria-orientation="vertical">
          <div class="py-1">{local.children}</div>
        </div>
      </div>
    </DropdownMenuContext.Provider>
  );
};

// ============================================================================
// DropdownMenuItem Component
// ============================================================================

export const DropdownMenuItem: Component<DropdownMenuItemProps> = (props) => {
  const [local, others] = splitProps(props, ['children', 'onClick', 'disabled', 'class', 'danger']);
  const context = useContext(DropdownMenuContext);

  if (!context) {
    console.warn('DropdownMenuItem must be used inside DropdownMenu');
    return null;
  }

  const baseClasses =
    'flex items-center gap-3 px-4 py-2 text-sm transition-colors cursor-pointer w-full text-left';
  const dangerClasses = local.danger
    ? 'text-gm-error hover:bg-gm-error/10'
    : 'text-dt-text-main hover:bg-gm-accent-cyan/10';
  const disabledClasses = local.disabled ? 'opacity-50 cursor-not-allowed' : '';
  const itemClasses = `${baseClasses} ${dangerClasses} ${disabledClasses} ${local.class || ''}`.trim();

  const handleClick = (e: MouseEvent) => {
    if (local.disabled) return;
    if (local.onClick) {
      local.onClick();
    }
    // Close menu after item click
    context.closeMenu();
  };

  return (
    <button
      type="button"
      class={itemClasses}
      role="menuitem"
      disabled={local.disabled}
      onClick={handleClick}
      {...others}
    >
      {local.children}
    </button>
  );
};

// ============================================================================
// DropdownMenuDivider Component
// ============================================================================

export const DropdownMenuDivider: Component<DropdownMenuDividerProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div
      class={`my-1 border-t border-gm-accent-cyan/10 ${local.class || ''}`.trim()}
      role="separator"
      {...others}
    />
  );
};

