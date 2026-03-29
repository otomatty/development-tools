/**
 * Animated Emoji Types
 *
 * Type definitions for animated emoji components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ../animated_emoji.rs
 */

/**
 * Supported animated emoji types
 */
export type EmojiType =
  | 'Fire' // 🔥 Fire - for streaks
  | 'Trophy' // 🏆 Trophy - for achievements, best records
  | 'Star' // ⭐ Star - for ratings, star counts
  | 'Target' // 🎯 Target - for goals, badges
  | 'Muscle' // 💪 Muscle - for streak milestones
  | 'Crown' // 👑 Crown - for highest level badges
  | 'Party' // 🎉 Party - for level ups, badge unlocks
  | 'Sparkles' // ✨ Sparkles - for quality badges, XP notifications
  | 'Rocket'; // 🚀 Rocket - for growth, progress

/**
 * Animation intensity levels
 */
export type AnimationIntensity = 'None' | 'Subtle' | 'Normal' | 'Strong';

/**
 * Emoji metadata
 */
export interface EmojiMetadata {
  emoji: string;
  animationClass: string;
  ariaLabel: string;
}

/**
 * Emoji type to metadata mapping
 */
export const EMOJI_METADATA: Record<EmojiType, EmojiMetadata> = {
  Fire: {
    emoji: '\uD83D\uDD25',
    animationClass: 'animate-emoji-flame',
    ariaLabel: 'streak fire',
  },
  Trophy: {
    emoji: '\uD83C\uDFC6',
    animationClass: 'animate-emoji-shine',
    ariaLabel: 'trophy achievement',
  },
  Star: {
    emoji: '\u2B50',
    animationClass: 'animate-emoji-twinkle',
    ariaLabel: 'star rating',
  },
  Target: {
    emoji: '\uD83C\uDFAF',
    animationClass: 'animate-emoji-pulse-scale',
    ariaLabel: 'goal target',
  },
  Muscle: {
    emoji: '\uD83D\uDCAA',
    animationClass: 'animate-emoji-flex',
    ariaLabel: 'strength milestone',
  },
  Crown: {
    emoji: '\uD83D\uDC51',
    animationClass: 'animate-emoji-float',
    ariaLabel: 'crown achievement',
  },
  Party: {
    emoji: '\uD83C\uDF89',
    animationClass: 'animate-emoji-bounce',
    ariaLabel: 'celebration',
  },
  Sparkles: {
    emoji: '\u2728',
    animationClass: 'animate-emoji-sparkle',
    ariaLabel: 'sparkles effect',
  },
  Rocket: {
    emoji: '\uD83D\uDE80',
    animationClass: 'animate-emoji-launch',
    ariaLabel: 'progress rocket',
  },
};

/**
 * Get CSS class modifier for animation intensity
 */
export function getIntensityModifier(intensity: AnimationIntensity): string {
  switch (intensity) {
    case 'None':
      return '';
    case 'Subtle':
      return 'animation-subtle';
    case 'Normal':
      return '';
    case 'Strong':
      return 'animation-strong';
  }
}

/**
 * Build CSS classes for animated emoji
 */
export function buildEmojiClasses(
  isAnimationEnabled: boolean,
  isHovered: boolean,
  hoverOnly: boolean,
  emojiType: EmojiType,
  intensity: AnimationIntensity,
  size: string,
  customClass?: string
): string {
  const shouldAnimate =
    isAnimationEnabled && intensity !== 'None' && (!hoverOnly || isHovered);

  const classes: string[] = [size];

  if (shouldAnimate) {
    const metadata = EMOJI_METADATA[emojiType];
    classes.push(metadata.animationClass);
    const modifier = getIntensityModifier(intensity);
    if (modifier) {
      classes.push(modifier);
    }
  }

  // Add transition for smooth animation start/stop
  classes.push('transition-transform duration-200');

  if (customClass) {
    classes.push(customClass);
  }

  return classes.join(' ');
}
