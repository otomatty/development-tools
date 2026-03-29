/**
 * Animated Emoji Component
 *
 * React implementation of AnimatedEmoji component.
 * Displays an emoji with optional CSS animation based on the animation context.
 * Supports hover-only animation mode for better UX.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ../animated_emoji.rs
 *   - Spec: ../animated_emoji.spec.md
 */

import { useState, useMemo } from 'react';
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
  className?: string;
}

export const AnimatedEmoji = ({
  emoji,
  size = 'text-2xl',
  hoverOnly = false,
  intensity = 'Normal',
  className,
}: AnimatedEmojiProps) => {
  const { enabled } = useAnimation();
  const [isHovered, setIsHovered] = useState(false);

  const computedClass = useMemo(() => {
    return buildEmojiClasses(
      enabled,
      isHovered,
      hoverOnly,
      emoji,
      intensity,
      size,
      className
    );
  }, [enabled, isHovered, hoverOnly, emoji, intensity, size, className]);

  const metadata = EMOJI_METADATA[emoji];

  return (
    <span
      className={computedClass}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      role="img"
      aria-label={metadata.ariaLabel}
    >
      {metadata.emoji}
    </span>
  );
};
