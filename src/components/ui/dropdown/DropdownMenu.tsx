/**
 * DropdownMenu Components
 *
 * React implementation of DropdownMenu, DropdownMenuItem, and DropdownMenuDivider components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./DropdownMenu.spec.md
 *   - Original (Leptos): ./dropdown_menu.rs
 */

import React, { createContext, useContext, useState, useEffect } from 'react';
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
  isOpen: boolean;
  setIsOpen: (value: boolean) => void;
  closeMenu: () => void;
}

const DropdownMenuContext = createContext<DropdownMenuContextValue | undefined>(undefined);

// ============================================================================
// DropdownMenu Component
// ============================================================================

export const DropdownMenu = ({
  trigger,
  children,
  align = 'right',
  className,
}: DropdownMenuProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const { enabled } = useAnimation();

  const closeMenu = () => setIsOpen(false);
  const toggleMenu = () => setIsOpen((prev) => !prev);

  // Handle ESC key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        closeMenu();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen]);

  const contextValue: DropdownMenuContextValue = {
    isOpen,
    setIsOpen,
    closeMenu,
  };

  // Menu classes
  const alignClass = align === 'left' ? 'left-0' : 'right-0';
  const baseMenuClass = `absolute ${alignClass} top-full mt-2 min-w-[160px] bg-gm-bg-card/95 backdrop-blur-sm border border-gm-accent-cyan/20 rounded-lg shadow-lg z-50`;
  const menuClass = enabled
    ? `${baseMenuClass} transition-all duration-200 ease-out`
    : baseMenuClass;

  // Menu style (for animation)
  const menuStyle: React.CSSProperties = isOpen
    ? { opacity: 1, transform: 'translateY(0)' }
    : enabled
      ? { opacity: 0, transform: 'translateY(-8px)', pointerEvents: 'none' }
      : { display: 'none' };

  return (
    <DropdownMenuContext.Provider value={contextValue}>
      <div className={`relative ${className || ''}`.trim()}>
        {/* Overlay for click outside detection */}
        {isOpen && (
          <div className="fixed inset-0 z-40" onClick={closeMenu} />
        )}

        {/* Trigger button */}
        <button
          type="button"
          className="p-2 text-dt-text-sub hover:text-gm-accent-cyan transition-colors rounded-lg"
          aria-expanded={isOpen}
          aria-haspopup="true"
          onClick={toggleMenu}
        >
          {typeof trigger === 'function' ? trigger() : trigger}
        </button>

        {/* Dropdown menu */}
        <div className={menuClass} style={menuStyle} role="menu" aria-orientation="vertical">
          <div className="py-1">{children}</div>
        </div>
      </div>
    </DropdownMenuContext.Provider>
  );
};

// ============================================================================
// DropdownMenuItem Component
// ============================================================================

export const DropdownMenuItem = ({
  children,
  onClick,
  disabled,
  danger,
  className,
}: DropdownMenuItemProps) => {
  const context = useContext(DropdownMenuContext);

  if (!context) {
    console.warn('DropdownMenuItem must be used inside DropdownMenu');
    return null;
  }

  const baseClasses =
    'flex items-center gap-3 px-4 py-2 text-sm transition-colors cursor-pointer w-full text-left';
  const dangerClasses = danger
    ? 'text-gm-error hover:bg-gm-error/10'
    : 'text-dt-text-main hover:bg-gm-accent-cyan/10';
  const disabledClasses = disabled ? 'opacity-50 cursor-not-allowed' : '';
  const itemClasses = `${baseClasses} ${dangerClasses} ${disabledClasses} ${className || ''}`.trim();

  const handleClick = () => {
    if (disabled) return;
    if (onClick) {
      onClick();
    }
    context.closeMenu();
  };

  return (
    <button
      type="button"
      className={itemClasses}
      role="menuitem"
      disabled={disabled}
      onClick={handleClick}
    >
      {children}
    </button>
  );
};

// ============================================================================
// DropdownMenuDivider Component
// ============================================================================

export const DropdownMenuDivider = ({ className }: DropdownMenuDividerProps) => {
  return (
    <div
      className={`my-1 border-t border-gm-accent-cyan/10 ${className || ''}`.trim()}
      role="separator"
    />
  );
};
