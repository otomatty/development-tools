/**
 * Dialog Components
 *
 * Solid.js implementation of Modal, ModalHeader, ModalBody, and ModalFooter components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Modal.spec.md (to be created)
 *   - Original (Leptos): ./modal.rs
 */

export { Modal, ModalHeader, ModalBody, ModalFooter } from './Modal';
export type {
  ModalSize,
  ModalProps,
  ModalHeaderProps,
  ModalBodyProps,
  ModalFooterProps,
} from '../../../types/ui';

