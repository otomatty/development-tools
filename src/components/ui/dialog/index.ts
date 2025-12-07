/**
 * Dialog Components
 *
 * Solid.js implementation of Modal, ModalHeader, ModalBody, ModalFooter, and ConfirmDialog components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Modal.spec.md (to be created)
 *   - Original (Leptos): ./modal.rs, ./confirm_dialog.rs
 */

export { Modal, ModalHeader, ModalBody, ModalFooter } from './Modal';
export { ConfirmDialog } from './ConfirmDialog';
export type {
  ModalSize,
  ModalProps,
  ModalHeaderProps,
  ModalBodyProps,
  ModalFooterProps,
  ConfirmDialogProps,
} from '../../../types/ui';

