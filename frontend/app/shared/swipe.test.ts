import { describe, expect, it } from 'vitest'
import { swipeDecision } from './swipe'

describe('swipeDecision', () => {
  it.each([
    ['not moved', 0, 0],
    ['right', 99, 0],
    ['left', -99, 0],
    ['down', 0, 99],
    ['up', 0, -99],
    ['diagonally', 99, 99],
  ])('snaps back when dragged %s within the threshold', (_, dx, dy) => {
    expect(swipeDecision(dx, dy)).toBeNull()
  })

  it.each([
    ['right', 100, 0],
    ['left', -100, 0],
    ['down', 0, 100],
  ])('snaps back when dragged %s exactly to the threshold', (_, dx, dy) => {
    expect(swipeDecision(dx, dy)).toBeNull()
  })

  it('likes when dragged right past the threshold', () => {
    expect(swipeDecision(101, 0)).toBe('like')
  })

  it('dislikes when dragged left past the threshold', () => {
    expect(swipeDecision(-101, 0)).toBe('dislike')
  })

  it.each([
    ['straight down', 0, 101],
    ['down and a little right', 150, 151],
    ['down and a little left', -150, 151],
  ])('defers when dragged %s past the threshold', (_, dx, dy) => {
    expect(swipeDecision(dx, dy)).toBe('defer')
  })

  it.each([
    ['right', 151, 150, 'like'],
    ['left', -151, 150, 'dislike'],
  ])(
    'follows the sideways drag when dragged down past the threshold but more to the %s',
    (_, dx, dy, expected) => {
      expect(swipeDecision(dx, dy)).toBe(expected)
    },
  )

  it('snaps back when dragged straight up past the threshold, because upward drags do not defer', () => {
    expect(swipeDecision(0, -101)).toBeNull()
  })
})
