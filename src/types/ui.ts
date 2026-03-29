/**
 * UI Component Type Definitions
 *
 * Type definitions for React UI components.
 */

import type { ReactNode, ButtonHTMLAttributes, InputHTMLAttributes, TextareaHTMLAttributes } from 'react';

// Button Types
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success' | 'outline';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  fullWidth?: boolean;
  isLoading?: boolean;
  leftIcon?: ReactNode;
  rightIcon?: ReactNode;
}

export interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  label: string;
}

// Input Types
export type InputType = 'text' | 'password' | 'number' | 'email' | 'url' | 'search';
export type InputSize = 'sm' | 'md' | 'lg';

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'value' | 'onInput' | 'size'> {
  value: string;
  onInput?: (value: string) => void;
  inputType?: InputType;
  size?: InputSize;
  placeholder?: string;
  disabled?: boolean;
  name?: string;
  id?: string;
}

export interface TextAreaProps extends Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, 'value' | 'onInput'> {
  value: string;
  onInput?: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  rows?: number;
  resizable?: boolean;
  className?: string;
}

export interface LabeledInputProps extends Omit<InputProps, 'id'> {
  label: string;
  required?: boolean;
  description?: string;
  inputType?: InputType;
  size?: InputSize;
}

// Modal Types
export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';

export interface ModalProps {
  visible: boolean;
  onClose: () => void;
  size?: ModalSize;
  borderClass?: string;
  closeOnOverlay?: boolean;
  closeOnEscape?: boolean;
  children: ReactNode;
}

export interface ModalHeaderProps {
  children: ReactNode;
  onClose?: () => void;
}

export interface ModalBodyProps {
  children: ReactNode;
  className?: string;
}

export interface ModalFooterProps {
  children: ReactNode;
}

// DropdownMenu Types
export interface DropdownMenuItemProps {
  children: ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  danger?: boolean;
  className?: string;
}

export interface DropdownMenuDividerProps {
  className?: string;
}

export interface DropdownMenuProps {
  trigger: ReactNode | (() => ReactNode);
  children: ReactNode;
  align?: 'left' | 'right';
  className?: string;
}

// Toast Types
export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastProps {
  message: string;
  type?: ToastType;
  duration?: number;
  onClose?: () => void;
}

export interface InlineToastProps {
  message: string;
  type?: ToastType;
  visible: boolean;
  className?: string;
}

export interface ToastContextValue {
  showToast: (message: string, type?: ToastType, duration?: number) => void;
}

// ToggleSwitch Types
export type ToggleSwitchSize = 'small' | 'medium' | 'large';

export interface ToggleSwitchProps {
  enabled: boolean;
  onToggle?: () => void;
  labelId?: string;
  size?: ToggleSwitchSize;
  disabled?: boolean;
}

// ConfirmDialog Types
export interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  visible: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  closeOnOverlay?: boolean;
}

// Accordion Types
export interface AccordionSectionProps {
  title: string;
  icon?: string;
  expanded: boolean;
  onToggle?: () => void;
  children: ReactNode;
  maxHeight?: string;
  className?: string;
}

// Icon Types
export interface IconProps {
  name: string;
  className?: string;
  size?: number | string;
  strokeWidth?: number;
}
