/**
 * Toast Components
 *
 * Solid.js implementation of Toast and InlineToast components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Toast.spec.md
 *   - Original (Leptos): ./toast.rs
 */

import { Component, Show, onMount, onCleanup, splitProps } from 'solid-js';
import type { ToastProps, ToastType } from '../../../types/ui';

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
    icon: '✓',
    bgClass: 'bg-green-900/90',
    borderClass: 'border-green-500/50',
    textClass: 'text-green-200',
    glowClass: 'shadow-[0_0_15px_rgba(34,197,94,0.3)]',
  },
  error: {
    icon: '✗',
    bgClass: 'bg-red-900/90',
    borderClass: 'border-red-500/50',
    textClass: 'text-red-200',
    glowClass: 'shadow-[0_0_15px_rgba(239,68,68,0.3)]',
  },
  info: {
    icon: 'ℹ',
    bgClass: 'bg-gm-accent-cyan/20',
    borderClass: 'border-gm-accent-cyan/50',
    textClass: 'text-gm-accent-cyan',
    glowClass: 'shadow-[0_0_15px_rgba(6,182,212,0.3)]',
  },
  warning: {
    icon: '⚠',
    bgClass: 'bg-amber-900/90',
    borderClass: 'border-amber-500/50',
    textClass: 'text-amber-200',
    glowClass: 'shadow-[0_0_15px_rgba(245,158,11,0.3)]',
  },
};

const inlineToastStyles: Record<ToastType, Omit<ToastStyles, 'glowClass'>> = {
  success: {
    icon: '✓',
    bgClass: 'bg-green-900/30',
    borderClass: 'border-green-500/50',
    textClass: 'text-green-200',
  },
  error: {
    icon: '✗',
    bgClass: 'bg-red-900/30',
    borderClass: 'border-red-500/50',
    textClass: 'text-red-200',
  },
  info: {
    icon: 'ℹ',
    bgClass: 'bg-gm-accent-cyan/10',
    borderClass: 'border-gm-accent-cyan/30',
    textClass: 'text-gm-accent-cyan',
  },
  warning: {
    icon: '⚠',
    bgClass: 'bg-amber-900/30',
    borderClass: 'border-amber-500/50',
    textClass: 'text-amber-200',
  },
};

// ============================================================================
// Toast Component
// ============================================================================

export const Toast: Component<ToastProps> = (props) => {
  const [local, others] = splitProps(props, ['message', 'type', 'duration', 'onClose']);
  const toastType = () => local.type ?? 'info';
  const duration = () => local.duration ?? 3000;
  const styles = () => toastStyles[toastType()];

  // Auto-hide after duration
  onMount(() => {
    if (duration() > 0 && local.onClose) {
      const timer = setTimeout(() => {
        local.onClose?.();
      }, duration());
      onCleanup(() => clearTimeout(timer));
    }
  });

  const toastClass = `fixed bottom-6 right-6 z-50 flex items-center gap-3 px-5 py-3 rounded-xl ${styles().bgClass} border ${styles().borderClass} backdrop-blur-sm animate-slideInUp ${styles().glowClass}`;

  return (
    <div class={toastClass} role="alert" aria-live="polite" {...others}>
      <span class={`text-lg font-bold ${styles().textClass}`}>{styles().icon}</span>
      <span class={`font-gaming ${styles().textClass}`}>{local.message}</span>
    </div>
  );
};

// ============================================================================
// InlineToast Component
// ============================================================================

export interface InlineToastProps {
  message: string;
  type?: ToastType;
  visible: boolean | (() => boolean);
  class?: string;
}

export const InlineToast: Component<InlineToastProps> = (props) => {
  const [local, others] = splitProps(props, ['message', 'type', 'visible', 'class']);
  const toastType = () => local.type ?? 'success';
  const visible = () => (typeof local.visible === 'function' ? local.visible() : local.visible);
  const styles = () => inlineToastStyles[toastType()];

  const toastClass = `flex items-center gap-2 px-4 py-2.5 rounded-lg ${styles().bgClass} border ${styles().borderClass} animate-fadeIn ${local.class || ''}`.trim();

  return (
    <Show when={visible()}>
      <div class={toastClass} role="alert" aria-live="polite" {...others}>
        <span class={`text-sm font-bold ${styles().textClass}`}>{styles().icon}</span>
        <span class={`text-sm ${styles().textClass}`}>{local.message}</span>
      </div>
    </Show>
  );
};

