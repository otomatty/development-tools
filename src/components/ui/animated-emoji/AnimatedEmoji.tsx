/**
 * Animated Emoji Component
 *
 * Solid.js implementation of AnimatedEmoji component.
 * Displays an emoji with optional CSS animation based on the animation context.
 * Supports hover-only animation mode for better UX.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ../animated_emoji.rs
 *   - Spec: ../animated_emoji.spec.md
 */

import { Component, createSignal, createMemo, splitProps } from 'solid-js';
import { useAnimation } from '../../../stores/animationStore';
import { buildEmojiClasses, EMOJI_METADATA, type EmojiType, type AnimationIntensity } from './types';

export interface AnimatedEmojiProps {
  /** The type of emoji to display */
  emoji: EmojiType;
  /** CSS size class (e.g., "text-2xl", "text-4xl") */
  size?: string;
  /** Only animate when hovered */
  hoverOnly?: boolean;
  /** Animation intensity */
  intensity?: AnimationIntensity;
  /** Additional CSS classes */
  class?: string;
}

export const AnimatedEmoji: Component<AnimatedEmojiProps> = (props) => {
  const [local, others] = splitProps(props, ['emoji', 'size', 'hoverOnly', 'intensity', 'class']);
  const animation = useAnimation();
  const [isHovered, setIsHovered] = createSignal(false);

  const size = () => local.size ?? 'text-2xl';
  const hoverOnly = () => local.hoverOnly ?? false;
  const intensity = () => (local.intensity ?? 'Normal') as AnimationIntensity;

  const computedClass = createMemo(() => {
    return buildEmojiClasses(
      animation.store.enabled,
      isHovered(),
      hoverOnly(),
      local.emoji,
      intensity(),
      size(),
      local.class
    );
  });

  const metadata = () => EMOJI_METADATA[local.emoji];

  return (
    <span
      class={computedClass()}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      role="img"
      aria-label={metadata().ariaLabel}
      {...others}
    >
      {metadata().emoji}
    </span>
  );
};

