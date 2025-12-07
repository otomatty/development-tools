/**
 * Accordion Section Component
 *
 * Solid.js implementation of AccordionSection component.
 * A single collapsible section with title, icon, and content.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/136
 *   - Original (Leptos): ./accordion.rs
 *   - Spec: ./accordion.spec.md
 */

import { Component, Show, splitProps } from 'solid-js';
import { Icon } from '../../icons';
import type { AccordionSectionProps } from '../../../types/ui';

export const AccordionSection: Component<AccordionSectionProps> = (props) => {
  const [local, others] = splitProps(props, [
    'title',
    'icon',
    'expanded',
    'onToggle',
    'children',
    'maxHeight',
    'class',
  ]);

  const expanded = () => (typeof local.expanded === 'function' ? local.expanded() : local.expanded);
  const maxHeight = () => local.maxHeight ?? '500px';
  const sectionId = `accordion-section-${local.title.replace(/\s+/g, '-').toLowerCase()}`;
  const contentId = `${sectionId}-content`;

  const handleClick = () => {
    local.onToggle?.();
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      local.onToggle?.();
    }
  };

  const combinedClass = `bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-lg overflow-hidden transition-all duration-300 hover:border-gm-accent-cyan/40 hover:shadow-gm-accent-cyan/10 ${local.class || ''}`.trim();

  return (
    <div class={combinedClass} {...others}>
      {/* Header button */}
      <button
        type="button"
        class="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-gm-accent-cyan/10 transition-all duration-200 group focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gm-accent-cyan"
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        aria-expanded={expanded()}
        aria-controls={contentId}
        id={sectionId}
      >
        <div class="flex items-center gap-3">
          <Show when={local.icon}>
            <span class="text-gm-accent-cyan group-hover:scale-110 transition-transform duration-200">
              <Icon name={local.icon!} class="w-5 h-5" />
            </span>
          </Show>
          <span class="text-lg font-gaming font-bold text-white group-hover:text-gm-accent-cyan transition-colors duration-200">
            {local.title}
          </span>
        </div>
        <span
          class="text-gm-accent-cyan transition-transform duration-300 ease-in-out"
          style={{ transform: expanded() ? 'rotate(180deg)' : 'rotate(0deg)' }}
          aria-hidden="true"
        >
          <Icon name="chevron-down" class="w-5 h-5" />
        </span>
      </button>

      {/* Content area */}
      <div
        id={contentId}
        role="region"
        aria-labelledby={sectionId}
        class="overflow-hidden transition-all duration-300 ease-in-out"
        style={{
          'max-height': expanded() ? maxHeight() : '0px',
          opacity: expanded() ? '1' : '0',
        }}
      >
        <div class="px-6 pb-6 pt-2">{local.children}</div>
      </div>
    </div>
  );
};

