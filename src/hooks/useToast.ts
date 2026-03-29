/**
 * Toast Hook
 *
 * React hook for managing toast notifications.
 * Provides a simple API to show toast messages.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Toast Component: src/components/ui/feedback/Toast.tsx
 */

import { useState, useCallback } from 'react';
import type { ToastType } from '../types/ui';

export interface ToastMessage {
  message: string;
  type: ToastType;
  duration: number;
}

export interface UseToastReturn {
  current: ToastMessage | null;
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
  const [current, setCurrent] = useState<ToastMessage | null>(null);

  const show = useCallback((message: string, type: ToastType = 'info', duration: number = 3000) => {
    setCurrent({ message, type, duration });
  }, []);

  const success = useCallback((message: string, duration: number = 3000) => {
    show(message, 'success', duration);
  }, [show]);

  const error = useCallback((message: string, duration: number = 3000) => {
    show(message, 'error', duration);
  }, [show]);

  const warning = useCallback((message: string, duration: number = 3000) => {
    show(message, 'warning', duration);
  }, [show]);

  const info = useCallback((message: string, duration: number = 3000) => {
    show(message, 'info', duration);
  }, [show]);

  const hide = useCallback(() => {
    setCurrent(null);
  }, []);

  return { current, show, success, error, warning, info, hide };
};
