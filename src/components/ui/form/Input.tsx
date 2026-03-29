/**
 * Input Components
 *
 * React implementation of Input, TextArea, and LabeledInput components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Input.spec.md
 *   - Original (Leptos): ./input.rs
 */

import { useId } from 'react';
import type {
  InputProps,
  TextAreaProps,
  LabeledInputProps,
  InputSize,
} from '../../../types/ui';

// ============================================================================
// Input Size Classes
// ============================================================================

const sizeClasses: Record<InputSize, string> = {
  sm: 'px-2 py-1 text-sm',
  md: 'px-3 py-2 text-base',
  lg: 'px-4 py-3 text-lg',
};

// ============================================================================
// Input Component
// ============================================================================

const inputBaseClass =
  'w-full bg-gm-bg-secondary border border-gm-border rounded-md text-dt-text-main placeholder-dt-text-sub/50 focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan transition-colors duration-200';

export const Input = ({
  value,
  onInput,
  inputType = 'text',
  size = 'md',
  placeholder,
  disabled = false,
  name,
  id,
  className,
  ...others
}: InputProps) => {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (onInput) {
      onInput(e.target.value);
    }
  };

  const combinedClass = `${inputBaseClass} ${sizeClasses[size]} ${disabled ? 'opacity-50 cursor-not-allowed' : ''} ${className || ''}`.trim();

  return (
    <input
      type={inputType}
      className={combinedClass}
      placeholder={placeholder}
      disabled={disabled}
      name={name}
      id={id}
      value={value}
      onChange={handleChange}
      {...others}
    />
  );
};

// ============================================================================
// TextArea Component
// ============================================================================

const textAreaBaseClass =
  'w-full bg-gm-bg-secondary border border-gm-border rounded-md text-dt-text-main placeholder-dt-text-sub/50 focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan transition-colors duration-200 px-3 py-2';

export const TextArea = ({
  value,
  onInput,
  placeholder,
  disabled = false,
  rows = 3,
  resizable = true,
  className,
  ...others
}: TextAreaProps) => {
  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    if (onInput) {
      onInput(e.target.value);
    }
  };

  const resizeClass = resizable ? 'resize-y' : 'resize-none';
  const combinedClass = `${textAreaBaseClass} ${disabled ? 'opacity-50 cursor-not-allowed' : ''} ${resizeClass} ${className || ''}`.trim();

  return (
    <textarea
      className={combinedClass}
      placeholder={placeholder}
      rows={rows}
      disabled={disabled}
      value={value}
      onChange={handleChange}
      {...others}
    />
  );
};

// ============================================================================
// LabeledInput Component
// ============================================================================

export const LabeledInput = ({
  value,
  onInput,
  label,
  inputType,
  placeholder,
  required = false,
  disabled,
  description,
  size,
  ...others
}: LabeledInputProps) => {
  const inputId = useId();

  return (
    <div className="flex flex-col gap-1">
      <label htmlFor={inputId} className="text-sm font-medium text-dt-text-main">
        {label}
        {required && <span className="text-red-500 ml-1">*</span>}
      </label>

      {description && (
        <span className="text-xs text-dt-text-sub">{description}</span>
      )}

      <Input
        value={value}
        onInput={onInput}
        inputType={inputType}
        placeholder={placeholder}
        disabled={disabled}
        size={size}
        id={inputId}
        {...others}
      />
    </div>
  );
};
