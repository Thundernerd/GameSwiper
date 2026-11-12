<template>
  <div class="mx-auto max-w-lg px-4 py-6 flex flex-col min-h-[calc(100vh-4rem)]">
    <div class="flex items-center justify-between gap-3 mb-5">
      <div class="min-w-0">
        <NuxtLink to="/" class="arcade-small-link">◀ EVENTS</NuxtLink>
        <h1 class="arcade-display text-xs sm:text-sm leading-relaxed mt-2 text-arcade-text">{{ eventName }}</h1>
      </div>
      <NuxtLink
        :to="`/events/${eventId}/overview`"
        class="arcade-btn arcade-btn-ghost shrink-0 text-[10px] px-2.5 py-2 !text-arcade-cyan"
      >
        OVERVIEW
      </NuxtLink>
    </div>

    <LoadingLine v-if="pending" class="py-20 text-center">LOADING DECK…</LoadingLine>
    <ErrorPanel v-else-if="error" :message="error" large class="my-16 text-center" />

    <div v-else-if="!deck.length" class="flex-1 flex flex-col items-center justify-center text-center gap-5">
      <p class="arcade-display arcade-glow text-xl sm:text-2xl leading-relaxed text-arcade-green">DECK<br>CLEARED!</p>
      <p class="arcade-display arcade-blink text-[10px] text-arcade-yellow">STAGE COMPLETE</p>
      <p class="text-arcade-muted">You have rated every game in this event.</p>
      <NuxtLink
        :to="`/events/${eventId}/overview`"
        class="arcade-btn arcade-btn-primary px-5 py-3 text-xs"
      >
        ▶ VIEW OVERVIEW
      </NuxtLink>
    </div>

    <div v-else class="flex-1 flex flex-col">
      <div class="relative mx-auto w-full max-w-md pb-4">
        <GameCard
          v-for="(game, index) in visibleCards"
          :key="game.id"
          :game="game"
          :active="index === 0"
          :style="stackStyle(index)"
          :class="
            index === 0
              ? 'relative z-20'
              : 'absolute inset-0 z-10 pointer-events-none'
          "
          :drag-x="index === 0 ? dragX : 0"
          :drag-y="index === 0 ? dragY : 0"
          :leaving="index === 0 ? leaving : null"
          @pointerdown="index === 0 ? onPointerDown($event) : undefined"
        />
      </div>

      <div class="flex-1 min-h-0" aria-hidden="true" />

      <div class="relative z-30 flex items-end justify-center gap-5 py-6 shrink-0 bg-arcade-bg arcade-scanlines">
        <button
          type="button"
          class="arcade-btn h-16 w-16 text-3xl !font-sans"
          :class="actionColors.dislike.text"
          title="Dislike"
          aria-label="Dislike"
          @click="decide('dislike')"
        >
          ✕
        </button>
        <button
          type="button"
          class="arcade-btn h-12 w-12 text-lg"
          :class="actionColors.defer.text"
          title="Not sure yet"
          aria-label="Not sure yet"
          @click="decide('defer')"
        >
          ?
        </button>
        <button
          type="button"
          class="arcade-btn h-16 w-16 text-3xl !font-sans"
          :class="actionColors.like.text"
          title="Like"
          aria-label="Like"
          @click="decide('like')"
        >
          ♥
        </button>
      </div>
      <p class="text-center text-arcade-dim pb-4">
        Swipe right to like · left to dislike · down or ? to defer ·
        <span class="text-arcade-yellow tabular-nums">{{ deck.length }} left</span>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { GameCard as GameCardType, SwipeAction } from '~/shared/api'
import { swipeDecision } from '~/shared/swipe'

const route = useRoute()
const api = useApi()
const eventId = computed(() => Number(route.params.id))
const eventName = ref('Event')
const deck = ref<GameCardType[]>([])
const pending = ref(true)
const error = ref('')
const dragX = ref(0)
const dragY = ref(0)
const leaving = ref<SwipeAction | null>(null)
const busy = ref(false)

let pointerId: number | null = null
let startX = 0
let startY = 0
let dragging = false

const visibleCards = computed(() => deck.value.slice(0, 2))

onMounted(loadQueue)

async function loadQueue() {
  pending.value = true
  error.value = ''
  try {
    const res = await api.eventQueue(eventId.value)
    eventName.value = res.event.name
    deck.value = res.games
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load queue'
  } finally {
    pending.value = false
  }
}

function stackStyle(index: number) {
  if (index === 0) return {}
  return {
    transform: 'translate(8px, 8px)',
    opacity: 0.6,
  }
}

function onPointerDown(e: PointerEvent) {
  if (busy.value || leaving.value) return
  const target = e.target
  if (
    target instanceof Element &&
    target.closest('button, a, iframe, [data-no-drag]')
  ) {
    return
  }
  pointerId = e.pointerId
  startX = e.clientX
  startY = e.clientY
  dragging = true
  ;(e.currentTarget as HTMLElement)?.setPointerCapture?.(e.pointerId)
  window.addEventListener('pointermove', onPointerMove)
  window.addEventListener('pointerup', onPointerUp)
  window.addEventListener('pointercancel', onPointerCancel)
}

function onPointerMove(e: PointerEvent) {
  if (!dragging || e.pointerId !== pointerId) return
  dragX.value = e.clientX - startX
  dragY.value = e.clientY - startY
}

/** Stops tracking the current drag, if any. Leaves the card where it is. */
function stopDrag() {
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  window.removeEventListener('pointercancel', onPointerCancel)
  dragging = false
  pointerId = null
}

onBeforeUnmount(stopDrag)

function snapBack() {
  dragX.value = 0
  dragY.value = 0
}

/** The user let go of the card: act on where it was released. */
function onPointerUp(e: PointerEvent) {
  if (e.pointerId !== pointerId) return
  stopDrag()
  const action = swipeDecision(dragX.value, dragY.value)
  if (action) void decide(action)
  else snapBack()
}

/**
 * The browser took the pointer away (an OS gesture, rotation, a native drag).
 * The user never released the card, so record nothing, however far it was dragged.
 */
function onPointerCancel(e: PointerEvent) {
  if (e.pointerId !== pointerId) return
  stopDrag()
  snapBack()
}

async function decide(action: SwipeAction) {
  const game = deck.value[0]
  if (busy.value || !game) return
  // A button pressed mid-drag takes over: the held pointer no longer moves,
  // releases or cancels this card (or the next one).
  stopDrag()
  busy.value = true
  leaving.value = action

  if (action === 'like') dragX.value = 420
  else if (action === 'dislike') dragX.value = -420
  else {
    dragY.value = 520
    dragX.value = 0
  }

  await new Promise((r) => setTimeout(r, 220))

  try {
    await api.swipe(eventId.value, game.id, action)
    const rest = deck.value.slice(1)
    if (action === 'defer') {
      deck.value = [...rest, game]
    } else {
      deck.value = rest
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Swipe failed'
  } finally {
    leaving.value = null
    snapBack()
    busy.value = false
  }
}
</script>
