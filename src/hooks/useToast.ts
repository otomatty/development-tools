/**
 * Toast Hook
 *
 * Solid.js hook for managing toast notifications.
 * Provides a simple API to show toast messages.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Toast Component: src/components/ui/feedback/Toast.tsx
 */

import { createSignal, Accessor } from 'solid-js';
import type { ToastType } from '../types/ui';

export interface ToastMessage {
  message: string;
  type: ToastType;
  duration: number;
}

export interface UseToastReturn {
  current: Accessor<ToastMessage | null>;
  show: (message: string, type?: ToastType, duration?: number) => void;
  success: (message: string, duration?: number) => void;
  error: (message: string, duration?: number) => void;
  warning: (message: string, duration?: number) => void;
  info: (message: string, duration?: number) => void;
  hide: () => void;
}

/**
 * Toast management hook
 *
 * Provides functions to show toast notifications.
 *
 * @example
 * ```tsx
 * const toast = useToast();
 *
 * // Show success message
 * toast.success("保存しました");
 *
 * // Show error message
 * toast.error("エラーが発生しました");
 *
 * // Show custom message
 * toast.show("カスタムメッセージ", "info", 5000);
 * ```
 */
export const useToast = (): UseToastReturn => {
  const [current, setCurrent] = createSignal<ToastMessage | null>(null);

  const show = (message: string, type: ToastType = 'info', duration: number = 3000) => {
    setCurrent({
      message,
      type,
      duration,
    });
  };

  const success = (message: string, duration: number = 3000) => {
    show(message, 'success', duration);
  };

  const error = (message: string, duration: number = 3000) => {
    show(message, 'error', duration);
  };

  const warning = (message: string, duration: number = 3000) => {
    show(message, 'warning', duration);
  };

  const info = (message: string, duration: number = 3000) => {
    show(message, 'info', duration);
  };

  const hide = () => {
    setCurrent(null);
  };

  return {
    current,
    show,
    success,
    error,
    warning,
    info,
    hide,
  };
};

