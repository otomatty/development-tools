/**
 * Button Component
 *
 * Solid.js implementation of Button and IconButton components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Button.spec.md
 *   - Original (Leptos): ./button.rs
 */

import { Component, splitProps, Show } from 'solid-js';
import type { ButtonProps, IconButtonProps, ButtonVariant, ButtonSize } from '../../../types/ui';

// ============================================================================
// Button Variant Classes
// ============================================================================

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    'bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white hover:opacity-90 hover:shadow-[0_0_15px_rgba(6,182,212,0.4)] active:opacity-80 focus:ring-gm-accent-cyan',
  secondary:
    'bg-gm-bg-secondary border border-slate-600 text-dt-text hover:bg-slate-700 hover:border-slate-500 active:bg-slate-600 focus:ring-slate-500',
  ghost: 'bg-transparent text-dt-text-sub hover:bg-slate-800 hover:text-dt-text active:bg-slate-700 focus:ring-slate-500',
  danger:
    'bg-red-500/20 border border-red-500/50 text-red-400 hover:bg-red-500/30 hover:border-red-500 hover:text-red-300 active:bg-red-500/40 focus:ring-red-500',
  success:
    'bg-green-500/20 border border-green-500/50 text-green-400 hover:bg-green-500/30 hover:border-green-500 hover:text-green-300 active:bg-green-500/40 focus:ring-green-500',
  outline:
    'bg-transparent border border-gm-accent-cyan/50 text-gm-accent-cyan hover:bg-gm-accent-cyan/10 hover:border-gm-accent-cyan active:bg-gm-accent-cyan/20 focus:ring-gm-accent-cyan',
};

// ============================================================================
// Button Size Classes
// ============================================================================

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm gap-1.5',
  md: 'px-4 py-2 text-base gap-2',
  lg: 'px-6 py-3 text-lg gap-2.5',
};

const iconSizeClasses: Record<ButtonSize, string> = {
  sm: 'p-1.5',
  md: 'p-2',
  lg: 'p-3',
};

// ============================================================================
// Button Component
// ============================================================================

const baseClasses =
  'inline-flex items-center justify-center font-medium rounded-2xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none';

export const Button: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, [
    'variant',
    'size',
    'disabled',
    'fullWidth',
    'isLoading',
    'leftIcon',
    'rightIcon',
    'children',
    'class',
  ]);

  const variant = () => local.variant ?? 'primary';
  const size = () => local.size ?? 'md';
  const disabled = () => local.disabled || local.isLoading;
  const widthClass = local.fullWidth ? 'w-full' : '';

  const combinedClass = `${baseClasses} ${variantClasses[variant()]} ${sizeClasses[size()]} ${widthClass} ${local.class || ''}`.trim();

  return (
    <button
      type={others.type || 'button'}
      class={combinedClass}
      disabled={disabled()}
      onClick={others.onClick}
      {...others}
    >
      <Show when={local.isLoading}>
        <svg class="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
      </Show>
      <Show when={!local.isLoading && local.leftIcon}>{local.leftIcon}</Show>
      {local.children}
      <Show when={!local.isLoading && local.rightIcon}>{local.rightIcon}</Show>
    </button>
  );
};

// ============================================================================
// IconButton Component
// ============================================================================

const iconButtonBaseClasses =
  'inline-flex items-center justify-center rounded-2xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none';

export const IconButton: Component<IconButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['variant', 'size', 'disabled', 'label', 'children', 'class']);

  const variant = () => local.variant ?? 'ghost';
  const size = () => local.size ?? 'md';

  const combinedClass = `${iconButtonBaseClasses} ${variantClasses[variant()]} ${iconSizeClasses[size()]} ${local.class || ''}`.trim();

  return (
    <button
      type="button"
      class={combinedClass}
      disabled={local.disabled}
      aria-label={local.label}
      title={local.label}
      onClick={others.onClick}
      {...others}
    >
      {local.children}
    </button>
  );
};

