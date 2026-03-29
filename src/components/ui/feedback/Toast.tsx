/**
 * Toast Components
 *
 * React implementation of Toast and InlineToast components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Toast.spec.md
 *   - Original (Leptos): ./toast.rs
 */

import { useEffect } from 'react';
import type { ToastProps, ToastType, InlineToastProps } from '../../../types/ui';

// ============================================================================
// Toast Type Styles
// ============================================================================

interface ToastStyles {
  icon: string;
  bgClass: string;
  borderClass: string;
  textClass: string;
  glowClass: string;
}

const toastStyles: Record<ToastType, ToastStyles> = {
  success: {
    icon: '\u2713',
    bgClass: 'bg-green-900/90',
    borderClass: 'border-green-500/50',
    textClass: 'text-green-200',
    glowClass: 'shadow-[0_0_15px_rgba(34,197,94,0.3)]',
  },
  error: {
    icon: '\u2717',
    bgClass: 'bg-red-900/90',
    borderClass: 'border-red-500/50',
    textClass: 'text-red-200',
    glowClass: 'shadow-[0_0_15px_rgba(239,68,68,0.3)]',
  },
  info: {
    icon: '\u2139',
    bgClass: 'bg-gm-accent-cyan/20',
    borderClass: 'border-gm-accent-cyan/50',
    textClass: 'text-gm-accent-cyan',
    glowClass: 'shadow-[0_0_15px_rgba(6,182,212,0.3)]',
  },
  warning: {
    icon: '\u26A0',
    bgClass: 'bg-amber-900/90',
    borderClass: 'border-amber-500/50',
    textClass: 'text-amber-200',
    glowClass: 'shadow-[0_0_15px_rgba(245,158,11,0.3)]',
  },
};

const inlineToastStyles: Record<ToastType, Omit<ToastStyles, 'glowClass'>> = {
  success: {
    icon: '\u2713',
    bgClass: 'bg-green-900/30',
    borderClass: 'border-green-500/50',
    textClass: 'text-green-200',
  },
  error: {
    icon: '\u2717',
    bgClass: 'bg-red-900/30',
    borderClass: 'border-red-500/50',
    textClass: 'text-red-200',
  },
  info: {
    icon: '\u2139',
    bgClass: 'bg-gm-accent-cyan/10',
    borderClass: 'border-gm-accent-cyan/30',
    textClass: 'text-gm-accent-cyan',
  },
  warning: {
    icon: '\u26A0',
    bgClass: 'bg-amber-900/30',
    borderClass: 'border-amber-500/50',
    textClass: 'text-amber-200',
  },
};

// ============================================================================
// Toast Component
// ============================================================================

export const Toast = ({
  message,
  type = 'info',
  duration = 3000,
  onClose,
}: ToastProps) => {
  const styles = toastStyles[type];

  // Auto-hide after duration
  useEffect(() => {
    if (duration > 0 && onClose) {
      const timer = setTimeout(() => {
        onClose();
      }, duration);
      return () => clearTimeout(timer);
    }
  }, [message, type, duration, onClose]);

  const toastClass = `fixed bottom-6 right-6 z-50 flex items-center gap-3 px-5 py-3 rounded-xl ${styles.bgClass} border ${styles.borderClass} backdrop-blur-sm animate-slideInUp ${styles.glowClass}`;

  return (
    <div className={toastClass} role="alert" aria-live="polite">
      <span className={`text-lg font-bold ${styles.textClass}`}>{styles.icon}</span>
      <span className={`font-gaming ${styles.textClass}`}>{message}</span>
    </div>
  );
};

// ============================================================================
// InlineToast Component
// ============================================================================

export const InlineToast = ({
  message,
  type = 'success',
  visible,
  className,
}: InlineToastProps) => {
  const styles = inlineToastStyles[type];

  const toastClass = `flex items-center gap-2 px-4 py-2.5 rounded-lg ${styles.bgClass} border ${styles.borderClass} animate-fadeIn ${className || ''}`.trim();

  if (!visible) return null;

  return (
    <div className={toastClass} role="alert" aria-live="polite">
      <span className={`text-sm font-bold ${styles.textClass}`}>{styles.icon}</span>
      <span className={`text-sm ${styles.textClass}`}>{message}</span>
    </div>
  );
};
