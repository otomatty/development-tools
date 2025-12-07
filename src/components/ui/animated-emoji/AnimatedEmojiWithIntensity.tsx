/**
 * Animated Emoji With Intensity Component
 *
 * Solid.js implementation of AnimatedEmojiWithIntensity component.
 * Useful for showing different animation intensities based on streak days, XP, etc.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ../animated_emoji.rs
 *   - Spec: ../animated_emoji.spec.md
 */

import { Component, createSignal, createMemo, splitProps, Accessor } from 'solid-js';
import { useAnimation } from '../../../stores/animationStore';
import { buildEmojiClasses, EMOJI_METADATA, type EmojiType, type AnimationIntensity } from './types';

export interface AnimatedEmojiWithIntensityProps {
  /** The type of emoji to display */
  emoji: EmojiType;
  /** CSS size class */
  size?: string;
  /** Only animate when hovered */
  hoverOnly?: boolean;
  /** Value to determine intensity (higher = stronger animation) */
  value: number | Accessor<number>;
  /** Thresholds for intensity levels [subtle, normal, strong] */
  thresholds?: [number, number, number];
  /** Additional CSS classes */
  class?: string;
}

export const AnimatedEmojiWithIntensity: Component<AnimatedEmojiWithIntensityProps> = (props) => {
  const [local, others] = splitProps(props, ['emoji', 'size', 'hoverOnly', 'value', 'thresholds', 'class']);
  const animation = useAnimation();
  const [isHovered, setIsHovered] = createSignal(false);

  const size = () => local.size ?? 'text-2xl';
  const hoverOnly = () => local.hoverOnly ?? false;
  const thresholds = () => local.thresholds ?? [1, 7, 30];

  const value = (): number => {
    const v = typeof local.value === 'function' ? local.value() : local.value;
    return v;
  };

  const intensity = createMemo((): AnimationIntensity => {
    const v = value();
    const [subtle, normal, strong] = thresholds();
    if (v >= strong) {
      return 'Strong';
    } else if (v >= normal) {
      return 'Normal';
    } else if (v >= subtle) {
      return 'Subtle';
    } else {
      return 'None';
    }
  });

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

