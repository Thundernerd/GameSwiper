import type { SwipeAction } from './api'

/** How far (px) the card must be dragged before letting go records a decision. */
const SWIPE_THRESHOLD = 100

/**
 * The decision that releasing a card dragged by (dx, dy) performs, or null when
 * the card snaps back. The swipe page acts on this on release and the card
 * labels itself with it while dragging, so the two cannot disagree.
 *
 * A mostly-downward drag past the threshold defers; otherwise a sideways drag
 * past the threshold likes (right) or dislikes (left).
 */
export function swipeDecision(dx: number, dy: number): SwipeAction | null {
  if (dy > SWIPE_THRESHOLD && Math.abs(dy) > Math.abs(dx)) return 'defer'
  if (dx > SWIPE_THRESHOLD) return 'like'
  if (dx < -SWIPE_THRESHOLD) return 'dislike'
  return null
}
