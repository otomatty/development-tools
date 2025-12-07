/**
 * Animated Emoji Components
 *
 * Solid.js implementation of AnimatedEmoji and AnimatedEmojiWithIntensity components.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/149
 *   - Original (Leptos): ../animated_emoji.rs
 *   - Spec: ../animated_emoji.spec.md
 */

export { AnimatedEmoji } from './AnimatedEmoji';
export { AnimatedEmojiWithIntensity } from './AnimatedEmojiWithIntensity';
export type { AnimatedEmojiProps, AnimatedEmojiWithIntensityProps } from './AnimatedEmoji';
export type { EmojiType, AnimationIntensity } from './types';
export { EMOJI_METADATA, buildEmojiClasses, getIntensityModifier } from './types';

