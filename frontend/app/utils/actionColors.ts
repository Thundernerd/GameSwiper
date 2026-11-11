import type { SwipeAction } from '../shared/api'

/**
 * The arcade colour of each swipe action: like is green, dislike red, defer yellow.
 * Every screen that colours something by action (swipe buttons, card overlay,
 * library tabs, overview columns) reads it from here.
 *
 * Full class names, not built strings, so Tailwind finds them (app/utils is in its content).
 *
 * These are plain utilities, and the arcade-* classes in main.css come after the utilities,
 * so an arcade-* class that sets its own colour (arcade-btn-ghost, arcade-btn-primary) wins
 * over them. Pair them with bare arcade-btn, as the swipe and overview buttons do.
 */
export const actionColors: Record<SwipeAction, { text: string; border: string }> = {
  like: { text: 'text-arcade-green', border: 'border-arcade-green' },
  dislike: { text: 'text-arcade-red', border: 'border-arcade-red' },
  defer: { text: 'text-arcade-yellow', border: 'border-arcade-yellow' },
}
