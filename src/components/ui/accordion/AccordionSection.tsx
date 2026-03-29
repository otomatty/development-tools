/**
 * Accordion Section Component
 *
 * React implementation of AccordionSection component.
 * A single collapsible section with title, icon, and content.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Original (Leptos): ./accordion.rs
 *   - Spec: ./accordion.spec.md
 */

import { Icon } from '../../icons';
import type { AccordionSectionProps } from '../../../types/ui';

export const AccordionSection = ({
  title,
  icon,
  expanded,
  onToggle,
  children,
  maxHeight = '500px',
  className,
}: AccordionSectionProps) => {
  const sectionId = `accordion-section-${title.replace(/\s+/g, '-').toLowerCase()}`;
  const contentId = `${sectionId}-content`;

  const handleClick = () => {
    onToggle?.();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onToggle?.();
    }
  };

  const combinedClass = `bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-lg overflow-hidden transition-all duration-300 hover:border-gm-accent-cyan/40 hover:shadow-gm-accent-cyan/10 ${className || ''}`.trim();

  return (
    <div className={combinedClass}>
      {/* Header button */}
      <button
        type="button"
        className="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-gm-accent-cyan/10 transition-all duration-200 group focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gm-accent-cyan"
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        aria-expanded={expanded}
        aria-controls={contentId}
        id={sectionId}
      >
        <div className="flex items-center gap-3">
          {icon && (
            <span className="text-gm-accent-cyan group-hover:scale-110 transition-transform duration-200">
              <Icon name={icon} className="w-5 h-5" />
            </span>
          )}
          <span className="text-lg font-gaming font-bold text-white group-hover:text-gm-accent-cyan transition-colors duration-200">
            {title}
          </span>
        </div>
        <span
          className="text-gm-accent-cyan transition-transform duration-300 ease-in-out"
          style={{ transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)' }}
          aria-hidden="true"
        >
          <Icon name="chevron-down" className="w-5 h-5" />
        </span>
      </button>

      {/* Content area */}
      <div
        id={contentId}
        role="region"
        aria-labelledby={sectionId}
        className="overflow-hidden transition-all duration-300 ease-in-out"
        style={{
          maxHeight: expanded ? maxHeight : '0px',
          opacity: expanded ? 1 : 0,
        }}
      >
        <div className="px-6 pb-6 pt-2">{children}</div>
      </div>
    </div>
  );
};
