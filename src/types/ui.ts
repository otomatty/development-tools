/**
 * UI Component Type Definitions
 *
 * Type definitions for Solid.js UI components migrated from Leptos.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Original (Leptos): src/components/ui/
 */

import type { JSX, Accessor } from 'solid-js';

// ============================================================================
// Button Types
// ============================================================================

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success' | 'outline';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  fullWidth?: boolean;
  isLoading?: boolean;
  leftIcon?: JSX.Element;
  rightIcon?: JSX.Element;
}

export interface IconButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  label: string; // Required for accessibility (aria-label)
}

// ============================================================================
// Input Types
// ============================================================================

export type InputType = 'text' | 'password' | 'number' | 'email' | 'url' | 'search';
export type InputSize = 'sm' | 'md' | 'lg';

export interface InputProps extends Omit<JSX.InputHTMLAttributes<HTMLInputElement>, 'value' | 'onInput'> {
  value: string | Accessor<string>;
  onInput?: (value: string) => void;
  inputType?: InputType;
  size?: InputSize;
  placeholder?: string;
  disabled?: boolean;
  name?: string;
  id?: string;
}

export interface TextAreaProps extends Omit<JSX.TextareaHTMLAttributes<HTMLTextAreaElement>, 'value' | 'onInput'> {
  value: string | Accessor<string>;
  onInput?: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  rows?: number;
  resizable?: boolean;
  class?: string;
}

export interface LabeledInputProps extends Omit<InputProps, 'id'> {
  label: string;
  required?: boolean;
  description?: string;
  inputType?: InputType;
  size?: InputSize;
}

// ============================================================================
// Modal Types
// ============================================================================

export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';

export interface ModalProps {
  visible: Accessor<boolean> | boolean;
  onClose: () => void;
  size?: ModalSize;
  borderClass?: string;
  closeOnOverlay?: boolean;
  closeOnEscape?: boolean;
  children: JSX.Element;
}

export interface ModalHeaderProps {
  children: JSX.Element;
  onClose?: () => void;
}

export interface ModalBodyProps {
  children: JSX.Element;
  class?: string;
}

export interface ModalFooterProps {
  children: JSX.Element;
}

// ============================================================================
// DropdownMenu Types
// ============================================================================

export interface DropdownMenuItemProps {
  children: JSX.Element;
  onClick?: () => void;
  disabled?: boolean;
  danger?: boolean;
  class?: string;
}

export interface DropdownMenuDividerProps {
  class?: string;
}

export interface DropdownMenuProps {
  trigger: JSX.Element | (() => JSX.Element);
  children: JSX.Element;
  align?: 'left' | 'right';
  class?: string;
}

// ============================================================================
// Toast Types
// ============================================================================

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastProps {
  message: string;
  type?: ToastType;
  duration?: number; // milliseconds (0 = no auto-hide)
  onClose?: () => void;
}

export interface InlineToastProps {
  message: string;
  type?: ToastType;
  visible: boolean | (() => boolean);
  class?: string;
}

export interface ToastContextValue {
  showToast: (message: string, type?: ToastType, duration?: number) => void;
}

