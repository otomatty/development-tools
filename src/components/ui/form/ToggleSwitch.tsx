/**
 * Toggle Switch Component
 *
 * React implementation of ToggleSwitch component.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Original (Leptos): ./toggle_switch.rs
 */

import type { ToggleSwitchProps } from '../../../types/ui';

// ============================================================================
// Toggle Switch Size Classes
// ============================================================================

const sizeClasses = {
  small: {
    button: 'w-10 h-5',
    knob: 'w-3 h-3',
    translate: 'translate-x-5',
  },
  medium: {
    button: 'w-12 h-6',
    knob: 'w-4 h-4',
    translate: 'translate-x-6',
  },
  large: {
    button: 'w-14 h-7',
    knob: 'w-5 h-5',
    translate: 'translate-x-7',
  },
};

// ============================================================================
// ToggleSwitch Component
// ============================================================================

export const ToggleSwitch = ({
  enabled = false,
  onToggle,
  labelId,
  size = 'medium',
  disabled = false,
}: ToggleSwitchProps) => {
  const classes = sizeClasses[size];

  const handleClick = () => {
    if (!disabled) {
      onToggle?.();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!disabled && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      onToggle?.();
    }
  };

  const buttonClass = `relative ${classes.button} rounded-full transition-all duration-300 ease-in-out ${
    enabled
      ? 'bg-gradient-to-r from-gm-accent-cyan to-gm-accent-cyan/80 shadow-[0_0_10px_rgba(0,255,255,0.3)] focus:ring-gm-accent-cyan'
      : 'bg-slate-600 hover:bg-slate-500 focus:ring-slate-500'
  } ${
    disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'
  } focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary`;

  const knobClass = `absolute top-1 left-1 ${classes.knob} bg-white rounded-full shadow-md transition-all duration-300 ease-in-out ${
    enabled ? classes.translate : 'translate-x-0'
  }`;

  return (
    <button
      type="button"
      role="switch"
      aria-checked={enabled}
      aria-labelledby={labelId}
      disabled={disabled}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      className={buttonClass}
    >
      <span className={knobClass}></span>
      {enabled && (
        <span className="absolute inset-0 rounded-full bg-gm-accent-cyan/20 animate-pulse"></span>
      )}
    </button>
  );
};
