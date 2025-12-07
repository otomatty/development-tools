/**
 * Modal Components
 *
 * Solid.js implementation of Modal, ModalHeader, ModalBody, and ModalFooter components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Modal.spec.md
 *   - Original (Leptos): ./modal.rs
 */

import { Component, Show, onMount, onCleanup, splitProps } from 'solid-js';
import { Portal } from 'solid-js/web';
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

export const Modal: Component<ModalProps> = (props) => {
  const animation = useAnimation();
  const visible = () => (typeof props.visible === 'function' ? props.visible() : props.visible);
  const size = () => props.size ?? 'md';
  const closeOnOverlay = () => props.closeOnOverlay ?? true;
  const closeOnEscape = () => props.closeOnEscape ?? true;
  const borderClass = () => props.borderClass || 'border border-slate-700/50';

  // Handle ESC key
  // TODO: [IMPROVE] 複数のモーダルが同時に開かれている場合、ESCキーで全てのモーダルが閉じてしまう問題
  // 最前面のモーダルのみが反応するように、グローバルなモーダルスタック管理を導入する必要がある
  onMount(() => {
    if (closeOnEscape()) {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && visible()) {
          props.onClose();
        }
      };
      window.addEventListener('keydown', handleKeyDown);
      onCleanup(() => window.removeEventListener('keydown', handleKeyDown));
    }
  });

  const handleOverlayClick = (e: MouseEvent) => {
    if (closeOnOverlay() && e.target === e.currentTarget) {
      props.onClose();
    }
  };

  const handleContentClick = (e: MouseEvent) => {
    e.stopPropagation();
  };

  const overlayClass = `fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm ${getAnimationClass('animate-fade-in', animation.store.enabled)}`;
  const modalClass = `bg-dt-card ${borderClass()} rounded-2xl w-full ${sizeClasses[size()]} mx-4 shadow-xl ${getAnimationClass('animate-scale-in', animation.store.enabled)}`;

  return (
    <Show when={visible()}>
      <Portal>
        <div
          class={overlayClass}
          role="dialog"
          aria-modal="true"
          onClick={handleOverlayClick}
        >
          <div class={modalClass} onClick={handleContentClick}>
            {props.children}
          </div>
        </div>
      </Portal>
    </Show>
  );
};

// ============================================================================
// ModalHeader Component
// ============================================================================

export const ModalHeader: Component<ModalHeaderProps> = (props) => {
  return (
    <div class="p-4 border-b border-slate-700/50 flex items-center justify-between">
      <div class="flex-1 min-w-0">{props.children}</div>
      <Show when={props.onClose}>
        <button
          class="p-1.5 text-dt-text-sub hover:text-white hover:bg-slate-800 rounded-lg transition-colors flex-shrink-0"
          onClick={() => props.onClose?.()}
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </Show>
    </div>
  );
};

// ============================================================================
// ModalBody Component
// ============================================================================

export const ModalBody: Component<ModalBodyProps> = (props) => {
  const [local, others] = splitProps(props, ['children', 'class']);
  const classes = ['p-4 overflow-y-auto', local.class].filter(Boolean).join(' ');

  return (
    <div class={classes} {...others}>
      {local.children}
    </div>
  );
};

// ============================================================================
// ModalFooter Component
// ============================================================================

export const ModalFooter: Component<ModalFooterProps> = (props) => {
  return (
    <div class="p-4 border-t border-slate-700/50 flex items-center justify-end gap-3">
      {props.children}
    </div>
  );
};

