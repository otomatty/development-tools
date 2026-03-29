import type { ButtonProps, IconButtonProps, ButtonVariant, ButtonSize } from '../../../types/ui';

const variantClasses: Record<ButtonVariant, string> = {
  primary: 'bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white hover:opacity-90 hover:shadow-[0_0_15px_rgba(6,182,212,0.4)] active:opacity-80 focus:ring-gm-accent-cyan',
  secondary: 'bg-gm-bg-secondary border border-slate-600 text-dt-text hover:bg-slate-700 hover:border-slate-500 active:bg-slate-600 focus:ring-slate-500',
  ghost: 'bg-transparent text-dt-text-sub hover:bg-slate-800 hover:text-dt-text active:bg-slate-700 focus:ring-slate-500',
  danger: 'bg-red-500/20 border border-red-500/50 text-red-400 hover:bg-red-500/30 hover:border-red-500 hover:text-red-300 active:bg-red-500/40 focus:ring-red-500',
  success: 'bg-green-500/20 border border-green-500/50 text-green-400 hover:bg-green-500/30 hover:border-green-500 hover:text-green-300 active:bg-green-500/40 focus:ring-green-500',
  outline: 'bg-transparent border border-gm-accent-cyan/50 text-gm-accent-cyan hover:bg-gm-accent-cyan/10 hover:border-gm-accent-cyan active:bg-gm-accent-cyan/20 focus:ring-gm-accent-cyan',
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm gap-1.5',
  md: 'px-4 py-2 text-base gap-2',
  lg: 'px-6 py-3 text-lg gap-2.5',
};

const iconSizeClasses: Record<ButtonSize, string> = {
  sm: 'p-1.5',
  md: 'p-2',
  lg: 'p-3',
};

const baseClasses = 'inline-flex items-center justify-center font-medium rounded-2xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none';

export const Button = ({ variant = 'primary', size = 'md', disabled, fullWidth, isLoading, leftIcon, rightIcon, children, className, type = 'button', ...rest }: ButtonProps) => {
  const isDisabled = disabled || isLoading;
  const widthClass = fullWidth ? 'w-full' : '';
  const combinedClass = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${widthClass} ${className || ''}`.trim();

  return (
    <button type={type} className={combinedClass} disabled={isDisabled} {...rest}>
      {isLoading && (
        <svg className="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
      )}
      {!isLoading && leftIcon}
      {children}
      {!isLoading && rightIcon}
    </button>
  );
};

const iconButtonBaseClasses = 'inline-flex items-center justify-center rounded-2xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none';

export const IconButton = ({ variant = 'ghost', size = 'md', disabled, label, children, className, ...rest }: IconButtonProps) => {
  const combinedClass = `${iconButtonBaseClasses} ${variantClasses[variant]} ${iconSizeClasses[size]} ${className || ''}`.trim();

  return (
    <button type="button" className={combinedClass} disabled={disabled} aria-label={label} title={label} {...rest}>
      {children}
    </button>
  );
};
