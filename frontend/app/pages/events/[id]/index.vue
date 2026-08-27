<template>
  <div class="mx-auto max-w-lg px-4 py-6 flex flex-col min-h-[calc(100vh-4rem)]">
    <div class="flex items-center justify-between gap-3 mb-4">
      <div>
        <NuxtLink to="/" class="text-sm text-slate-400 hover:text-slate-200">← Events</NuxtLink>
        <h1 class="text-xl font-semibold mt-1">{{ eventName }}</h1>
      </div>
      <NuxtLink
        :to="`/events/${eventId}/overview`"
        class="text-sm rounded-lg border border-slate-700 px-3 py-1.5 hover:bg-slate-800"
      >
        Overview
      </NuxtLink>
    </div>

    <p v-if="pending" class="text-slate-400 py-20 text-center">Loading deck…</p>
    <p v-else-if="error" class="text-rose-400 py-20 text-center">{{ error }}</p>

    <div v-else-if="!deck.length" class="flex-1 flex flex-col items-center justify-center text-center gap-4">
      <p class="text-2xl font-semibold">Deck cleared</p>
      <p class="text-slate-400">You have rated every game in this event.</p>
      <NuxtLink
        :to="`/events/${eventId}/overview`"
        class="rounded-xl bg-violet-600 hover:bg-violet-500 px-5 py-2.5 font-medium"
      >
        View overview
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

      <div class="relative z-30 flex items-center justify-center gap-4 py-6 shrink-0 bg-slate-950">
        <button
          type="button"
          class="h-14 w-14 rounded-full border-2 border-rose-500 text-rose-400 text-xl font-bold hover:bg-rose-500/10"
          title="Dislike"
          @click="decide('dislike')"
        >
          ✕
        </button>
        <button
          type="button"
          class="h-12 w-12 rounded-full border-2 border-amber-400 text-amber-300 text-sm font-semibold hover:bg-amber-400/10"
          title="Not sure yet"
          @click="decide('defer')"
        >
          ?
        </button>
        <button
          type="button"
          class="h-14 w-14 rounded-full border-2 border-emerald-500 text-emerald-400 text-xl font-bold hover:bg-emerald-500/10"
          title="Like"
          @click="decide('like')"
        >
          ♥
        </button>
      </div>
      <p class="text-center text-xs text-slate-500 pb-4">
        Swipe right to like · left to dislike · down or ? to defer · {{ deck.length }} left
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { GameCard as GameCardType, SwipeAction } from '../../shared/api'

const route = useRoute()
const api = useApi()
const eventId = computed(() => Number(route.params.id))
const eventName = ref('Event')
const deck = ref<GameCardType[]>([])
const pending = ref(true)
const error = ref('')
const dragX = ref(0)
const dragY = ref(0)
const leaving = ref<'like' | 'dislike' | 'defer' | null>(null)
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
    transform: 'scale(0.96) translateY(12px)',
    opacity: 0.85,
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
  window.addEventListener('pointercancel', onPointerUp)
}

function onPointerMove(e: PointerEvent) {
  if (!dragging || e.pointerId !== pointerId) return
  dragX.value = e.clientX - startX
  dragY.value = e.clientY - startY
}

function onPointerUp(e: PointerEvent) {
  if (e.pointerId !== pointerId) return
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  window.removeEventListener('pointercancel', onPointerUp)
  dragging = false
  pointerId = null

  const x = dragX.value
  const y = dragY.value
  const threshold = 100
  if (y > threshold && Math.abs(y) > Math.abs(x)) {
    void decide('defer')
  } else if (x > threshold) {
    void decide('like')
  } else if (x < -threshold) {
    void decide('dislike')
  } else {
    dragX.value = 0
    dragY.value = 0
  }
}

async function decide(action: SwipeAction) {
  if (busy.value || !deck.value.length) return
  busy.value = true
  leaving.value = action
  const game = deck.value[0]

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
    dragX.value = 0
    dragY.value = 0
    busy.value = false
  }
}
</script>
