/**
 * Form Components
 *
 * Solid.js implementation of Input, TextArea, LabeledInput, and ToggleSwitch components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Input.spec.md (to be created)
 *   - Original (Leptos): ./input.rs, ./toggle_switch.rs
 */

export { Input, TextArea, LabeledInput } from './Input';
export { ToggleSwitch } from './ToggleSwitch';
export type {
  InputType,
  InputSize,
  InputProps,
  TextAreaProps,
  LabeledInputProps,
  ToggleSwitchSize,
  ToggleSwitchProps,
} from '../../../types/ui';

