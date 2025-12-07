/**
 * Input Components
 *
 * Solid.js implementation of Input, TextArea, and LabeledInput components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
 *   - Spec: ./Input.spec.md
 *   - Original (Leptos): ./input.rs
 */

import { Component, splitProps, createUniqueId, Show } from 'solid-js';
import type {
  InputProps,
  TextAreaProps,
  LabeledInputProps,
  InputType,
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

export const Input: Component<InputProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'onInput',
    'inputType',
    'size',
    'placeholder',
    'disabled',
    'name',
    'id',
    'class',
  ]);

  const inputType = () => local.inputType ?? 'text';
  const size = () => local.size ?? 'md';
  const disabled = () => local.disabled ?? false;

  // Handle value: can be string or Accessor<string>
  const getValue = () => (typeof local.value === 'function' ? local.value() : local.value);
  const setValue = (newValue: string) => {
    if (local.onInput) {
      local.onInput(newValue);
    }
  };

  const handleInput = (e: Event) => {
    const target = e.currentTarget as HTMLInputElement;
    setValue(target.value);
  };

  const combinedClass = `${inputBaseClass} ${sizeClasses[size()]} ${disabled() ? 'opacity-50 cursor-not-allowed' : ''} ${local.class || ''}`.trim();

  return (
    <input
      type={inputType()}
      class={combinedClass}
      placeholder={local.placeholder}
      disabled={disabled()}
      name={local.name}
      id={local.id}
      value={getValue()}
      onInput={handleInput}
      {...others}
    />
  );
};

// ============================================================================
// TextArea Component
// ============================================================================

const textAreaBaseClass =
  'w-full bg-gm-bg-secondary border border-gm-border rounded-md text-dt-text-main placeholder-dt-text-sub/50 focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan transition-colors duration-200 px-3 py-2';

export const TextArea: Component<TextAreaProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'onInput',
    'placeholder',
    'disabled',
    'rows',
    'resizable',
    'class',
  ]);

  const disabled = () => local.disabled ?? false;
  const rows = () => local.rows ?? 3;
  const resizable = () => local.resizable ?? true;

  // Handle value: can be string or Accessor<string>
  const getValue = () => (typeof local.value === 'function' ? local.value() : local.value);
  const setValue = (newValue: string) => {
    if (local.onInput) {
      local.onInput(newValue);
    }
  };

  const handleInput = (e: Event) => {
    const target = e.currentTarget as HTMLTextAreaElement;
    setValue(target.value);
  };

  const resizeClass = resizable() ? 'resize-y' : 'resize-none';
  const combinedClass = `${textAreaBaseClass} ${disabled() ? 'opacity-50 cursor-not-allowed' : ''} ${resizeClass} ${local.class || ''}`.trim();

  return (
    <textarea
      class={combinedClass}
      placeholder={local.placeholder}
      rows={rows()}
      disabled={disabled()}
      value={getValue()}
      onInput={handleInput}
      {...others}
    />
  );
};

// ============================================================================
// LabeledInput Component
// ============================================================================

export const LabeledInput: Component<LabeledInputProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'onInput',
    'label',
    'inputType',
    'placeholder',
    'required',
    'disabled',
    'description',
    'size',
  ]);

  // Generate unique ID for label-input association
  const inputId = createUniqueId();
  const required = () => local.required ?? false;

  return (
    <div class="flex flex-col gap-1">
      <label for={inputId} class="text-sm font-medium text-dt-text-main">
        {local.label}
        <Show when={required()}>
          <span class="text-red-500 ml-1">*</span>
        </Show>
      </label>

      <Show when={local.description}>
        <span class="text-xs text-dt-text-sub">{local.description}</span>
      </Show>

      <Input
        value={local.value}
        onInput={local.onInput}
        inputType={local.inputType}
        placeholder={local.placeholder}
        disabled={local.disabled}
        size={local.size}
        id={inputId}
        {...others}
      />
    </div>
  );
};

