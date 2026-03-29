/**
 * Animated Emoji With Intensity Component
 *
 * React implementation of AnimatedEmojiWithIntensity component.
 * Useful for showing different animation intensities based on streak days, XP, etc.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ../animated_emoji.rs
 *   - Spec: ../animated_emoji.spec.md
 */

import { useState, useMemo } from 'react';
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
  value: number;
  /** Thresholds for intensity levels [subtle, normal, strong] */
  thresholds?: [number, number, number];
  /** Additional CSS classes */
  className?: string;
}

export const AnimatedEmojiWithIntensity = ({
  emoji,
  size = 'text-2xl',
  hoverOnly = false,
  value,
  thresholds = [1, 7, 30],
  className,
}: AnimatedEmojiWithIntensityProps) => {
  const { enabled } = useAnimation();
  const [isHovered, setIsHovered] = useState(false);

  const intensity = useMemo((): AnimationIntensity => {
    const [subtle, normal, strong] = thresholds;
    if (value >= strong) {
      return 'Strong';
    } else if (value >= normal) {
      return 'Normal';
    } else if (value >= subtle) {
      return 'Subtle';
    } else {
      return 'None';
    }
  }, [value, thresholds]);

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
