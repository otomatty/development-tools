/**
 * Form Components
 *
 * Solid.js implementation of Input, TextArea, and LabeledInput components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Input.spec.md (to be created)
 *   - Original (Leptos): ./input.rs
 */

export { Input, TextArea, LabeledInput } from './Input';
export type {
  InputType,
  InputSize,
  InputProps,
  TextAreaProps,
  LabeledInputProps,
} from '../../../types/ui';

