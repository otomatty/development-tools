/**
 * Modal Components
 *
 * React implementation of Modal, ModalHeader, ModalBody, and ModalFooter components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Modal.spec.md
 *   - Original (Leptos): ./modal.rs
 */

import { useEffect } from 'react';
import { createPortal } from 'react-dom';
import type {
  ModalProps,
  ModalHeaderProps,
  ModalBodyProps,
  ModalFooterProps,
  ModalSize,
} from '../../../types/ui';
import { useAnimation } from '../../../stores/animationStore';

// ============================================================================
// Modal Size Classes
// ============================================================================

const sizeClasses: Record<ModalSize, string> = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  '2xl': 'max-w-2xl',
  full: 'max-w-4xl',
};

// ============================================================================
// Animation Helper
// ============================================================================

function getAnimationClass(animationClass: string, enabled: boolean): string {
  return enabled ? animationClass : '';
}

// ============================================================================
// Modal Component
// ============================================================================

export const Modal = ({
  visible,
  onClose,
  size = 'md',
  borderClass,
  closeOnOverlay = true,
  closeOnEscape = true,
  children,
}: ModalProps) => {
  const { enabled } = useAnimation();
  const border = borderClass || 'border border-slate-700/50';

  // Handle ESC key
  useEffect(() => {
    if (closeOnEscape) {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && visible) {
          onClose();
        }
      };
      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }
  }, [closeOnEscape, visible, onClose]);

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (closeOnOverlay && e.target === e.currentTarget) {
      onClose();
    }
  };

  const handleContentClick = (e: React.MouseEvent) => {
    e.stopPropagation();
  };

  const overlayClass = `fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm ${getAnimationClass('animate-fade-in', enabled)}`;
  const modalClass = `bg-dt-card ${border} rounded-2xl w-full ${sizeClasses[size]} mx-4 shadow-xl ${getAnimationClass('animate-scale-in', enabled)}`;

  if (!visible) return null;

  return createPortal(
    <div
      className={overlayClass}
      role="dialog"
      aria-modal="true"
      onClick={handleOverlayClick}
    >
      <div className={modalClass} onClick={handleContentClick}>
        {children}
      </div>
    </div>,
    document.body
  );
};

// ============================================================================
// ModalHeader Component
// ============================================================================

export const ModalHeader = ({ children, onClose }: ModalHeaderProps) => {
  return (
    <div className="p-4 border-b border-slate-700/50 flex items-center justify-between">
      <div className="flex-1 min-w-0">{children}</div>
      {onClose && (
        <button
          className="p-1.5 text-dt-text-sub hover:text-white hover:bg-slate-800 rounded-lg transition-colors flex-shrink-0"
          onClick={() => onClose()}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      )}
    </div>
  );
};

// ============================================================================
// ModalBody Component
// ============================================================================

export const ModalBody = ({ children, className, ...others }: ModalBodyProps) => {
  const classes = ['p-4 overflow-y-auto', className].filter(Boolean).join(' ');

  return (
    <div className={classes} {...others}>
      {children}
    </div>
  );
};

// ============================================================================
// ModalFooter Component
// ============================================================================

export const ModalFooter = ({ children }: ModalFooterProps) => {
  return (
    <div className="p-4 border-t border-slate-700/50 flex items-center justify-end gap-3">
      {children}
    </div>
  );
};
