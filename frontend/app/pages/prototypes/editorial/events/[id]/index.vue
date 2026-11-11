<template>
  <Shell>
    <div class="mx-auto max-w-lg px-5 sm:px-8 pt-6 sm:pt-10 flex flex-col min-h-[calc(100vh-8rem)]">
      <div class="flex items-end justify-between gap-4 mb-6 border-b border-stone-300 pb-4">
        <div class="min-w-0">
          <NuxtLink :to="link('/')" class="ed-caps text-stone-600 hover:text-stone-900">← Events</NuxtLink>
          <h1 class="ed-serif text-3xl font-medium leading-tight mt-2">{{ eventName }}</h1>
        </div>
        <NuxtLink
          :to="link(`/events/${eventId}/overview`)"
          class="shrink-0 ed-caps border-b border-stone-900 pb-0.5 hover:opacity-70"
        >
          Overview
        </NuxtLink>
      </div>

      <p v-if="pending" class="ed-serif italic text-xl text-stone-600 py-20 text-center">Loading deck…</p>
      <p v-else-if="error" class="ed-serif text-xl italic py-20 text-center">{{ error }}</p>

      <div v-else-if="!deck.length" class="flex-1 flex flex-col items-center justify-center text-center py-16">
        <p class="ed-serif text-5xl font-medium tracking-tight">Deck cleared</p>
        <p class="mt-4 text-lg text-stone-600">You have rated every game in this event.</p>
        <NuxtLink
          :to="link(`/events/${eventId}/overview`)"
          class="mt-8 bg-stone-900 text-[#f7f4ee] hover:bg-stone-700 px-6 py-3.5 ed-caps transition"
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

        <div class="relative z-30 flex items-center justify-center gap-6 py-6 shrink-0 bg-[#f7f4ee]">
          <button
            type="button"
            class="h-14 w-14 rounded-full border border-rose-700 bg-white text-rose-700 text-xl hover:bg-rose-700 hover:text-white transition"
            title="Dislike"
            aria-label="Dislike"
            @click="decide('dislike')"
          >
            ✕
          </button>
          <button
            type="button"
            class="h-12 w-12 rounded-full border border-amber-800 bg-white text-amber-800 ed-serif text-lg italic hover:bg-amber-800 hover:text-white transition"
            title="Not sure yet"
            aria-label="Not sure yet"
            @click="decide('defer')"
          >
            ?
          </button>
          <button
            type="button"
            class="h-14 w-14 rounded-full border border-emerald-700 bg-white text-emerald-700 text-xl hover:bg-emerald-700 hover:text-white transition"
            title="Like"
            aria-label="Like"
            @click="decide('like')"
          >
            ♥
          </button>
        </div>
        <p class="text-center text-sm text-stone-600 leading-relaxed">
          Swipe right to like · left to dislike · down or ? to defer
        </p>
        <p class="text-center ed-serif italic text-stone-900 pt-1 pb-4">{{ deck.length }} left</p>
      </div>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import type { GameCard as GameCardType, SwipeAction } from '~/shared/api'
import GameCard from '../../-components/GameCard.vue'
import Shell from '../../-components/Shell.vue'

const route = useRoute()
const link = usePrototypeLink()
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
  const game = deck.value[0]
  if (busy.value || !game) return
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
    dragX.value = 0
    dragY.value = 0
    busy.value = false
  }
}
</script>
