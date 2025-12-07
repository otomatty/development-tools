/**
 * Confirm Dialog Component
 *
 * Solid.js implementation of ConfirmDialog component.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Original (Leptos): ./confirm_dialog.rs
 */

import { Component, Show } from 'solid-js';
import { Modal, ModalHeader, ModalBody, ModalFooter } from './Modal';
import type { ConfirmDialogProps } from '../../../types/ui';

export const ConfirmDialog: Component<ConfirmDialogProps> = (props) => {
  const visible = () => (typeof props.visible === 'function' ? props.visible() : props.visible);
  const closeOnOverlay = () => props.closeOnOverlay ?? false;

  return (
    <Modal
      visible={visible()}
      onClose={props.onCancel}
      size="md"
      closeOnOverlay={closeOnOverlay()}
      closeOnEscape={false}
    >
      <ModalHeader onClose={props.onCancel}>
        <h3 id="confirm-dialog-title" class="text-xl font-gaming font-bold text-white">
          {props.title}
        </h3>
      </ModalHeader>
      <ModalBody>
        <p id="confirm-dialog-message" class="text-dt-text-sub">
          {props.message}
        </p>
      </ModalBody>
      <ModalFooter>
        <button
          class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
          onClick={props.onCancel}
        >
          {props.cancelLabel}
        </button>
        <button
          class="px-4 py-2 rounded-lg bg-gm-error hover:bg-red-600 text-white transition-colors"
          onClick={props.onConfirm}
        >
          {props.confirmLabel}
        </button>
      </ModalFooter>
    </Modal>
  );
};

