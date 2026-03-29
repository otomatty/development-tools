/**
 * Confirm Dialog Component
 *
 * React implementation of ConfirmDialog component.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Original (Leptos): ./confirm_dialog.rs
 */

import { Modal, ModalHeader, ModalBody, ModalFooter } from './Modal';
import type { ConfirmDialogProps } from '../../../types/ui';

export const ConfirmDialog = ({
  visible,
  title,
  message,
  confirmLabel,
  cancelLabel,
  onConfirm,
  onCancel,
  closeOnOverlay = false,
}: ConfirmDialogProps) => {
  return (
    <Modal
      visible={visible}
      onClose={onCancel}
      size="md"
      closeOnOverlay={closeOnOverlay}
      closeOnEscape={false}
    >
      <ModalHeader onClose={onCancel}>
        <h3 id="confirm-dialog-title" className="text-xl font-gaming font-bold text-white">
          {title}
        </h3>
      </ModalHeader>
      <ModalBody>
        <p id="confirm-dialog-message" className="text-dt-text-sub">
          {message}
        </p>
      </ModalBody>
      <ModalFooter>
        <button
          type="button"
          className="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
          onClick={onCancel}
        >
          {cancelLabel}
        </button>
        <button
          type="button"
          className="px-4 py-2 rounded-lg bg-gm-error hover:bg-red-600 text-white transition-colors"
          onClick={onConfirm}
        >
          {confirmLabel}
        </button>
      </ModalFooter>
    </Modal>
  );
};
