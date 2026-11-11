<template>
  <Shell :backdrop="backdrop">
    <div class="mx-auto max-w-lg px-4 py-6 flex flex-col min-h-[calc(100vh-4rem)]">
      <div class="flex items-center justify-between gap-3 mb-5">
        <div class="min-w-0">
          <NuxtLink :to="link('/')" class="text-sm text-white/90 hover:text-white transition drop-shadow-[0_1px_6px_rgba(0,0,0,0.8)]">← Events</NuxtLink>
          <h1 class="text-xl sm:text-2xl font-black tracking-tight leading-tight mt-1 drop-shadow-[0_2px_12px_rgba(0,0,0,0.6)]">{{ eventName }}</h1>
        </div>
        <NuxtLink
          :to="link(`/events/${eventId}/overview`)"
          class="shrink-0 text-sm rounded-full border border-white/20 bg-slate-950/50 backdrop-blur-md px-4 py-1.5 text-white hover:bg-slate-950/70 transition"
        >
          Overview
        </NuxtLink>
      </div>

      <StatusPanel v-if="pending" kind="loading" class="my-16">Loading deck…</StatusPanel>
      <StatusPanel v-else-if="error" kind="error" class="my-16">{{ error }}</StatusPanel>

      <div v-else-if="!deck.length" class="flex-1 flex flex-col items-center justify-center py-10">
        <div
          class="cleared w-full max-w-sm flex flex-col items-center text-center gap-4 rounded-3xl border border-white/15 bg-slate-950/50 px-6 py-10 backdrop-blur-2xl shadow-[0_30px_80px_-24px_rgba(0,0,0,0.8)]"
        >
          <span
            class="grid h-16 w-16 place-items-center rounded-full bg-gradient-to-br from-emerald-300 via-cyan-300 to-violet-400 text-3xl text-slate-950 shadow-[0_0_40px_-4px_rgba(52,211,153,0.6)]"
            aria-hidden="true"
          >
            ✓
          </span>
          <p class="text-3xl font-black tracking-tight bg-gradient-to-r from-emerald-200 via-cyan-200 to-violet-200 bg-clip-text text-transparent">
            Deck cleared
          </p>
          <p class="text-slate-200">You have rated every game in this event.</p>
          <NuxtLink
            :to="link(`/events/${eventId}/overview`)"
            class="rounded-full bg-gradient-to-r from-indigo-600 via-violet-600 to-fuchsia-700 px-6 py-3 font-bold text-white shadow-lg shadow-fuchsia-600/30 transition hover:brightness-110 hover:shadow-fuchsia-500/50"
          >
            View overview
          </NuxtLink>
        </div>
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

        <div class="relative z-30 flex items-center justify-center gap-5 py-6 shrink-0">
          <button
            type="button"
            class="grid h-16 w-16 place-items-center rounded-full bg-gradient-to-br from-rose-600 to-pink-700 text-2xl font-bold text-white ring-4 ring-white/10 shadow-[0_12px_30px_-6px_rgba(244,63,94,0.7)] transition hover:scale-110 active:scale-95"
            title="Dislike"
            @click="decide('dislike')"
          >
            ✕
          </button>
          <button
            type="button"
            class="grid h-12 w-12 place-items-center rounded-full bg-gradient-to-br from-amber-400 to-orange-600 text-lg font-black text-slate-950 ring-4 ring-white/10 shadow-[0_12px_30px_-6px_rgba(251,146,60,0.7)] transition hover:scale-110 active:scale-95"
            title="Not sure yet"
            @click="decide('defer')"
          >
            ?
          </button>
          <button
            type="button"
            class="grid h-16 w-16 place-items-center rounded-full bg-gradient-to-br from-emerald-600 to-teal-700 text-2xl font-bold text-white ring-4 ring-white/10 shadow-[0_12px_30px_-6px_rgba(16,185,129,0.8)] transition hover:scale-110 active:scale-95"
            title="Like"
            @click="decide('like')"
          >
            ♥
          </button>
        </div>
        <p class="mx-auto mb-4 rounded-2xl border border-white/10 bg-slate-950/55 px-4 py-1.5 text-center text-xs leading-relaxed text-slate-200 backdrop-blur-md">
          Swipe right to like · left to dislike · down or ? to defer ·
          <span class="whitespace-nowrap font-bold text-white tabular-nums">{{ deck.length }} left</span>
        </p>
      </div>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import type { GameCard as GameCardType, SwipeAction } from '~/shared/api'
import GameCard from '../../-components/GameCard.vue'
import Shell from '../../-components/Shell.vue'
import StatusPanel from '../../-components/StatusPanel.vue'

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

/** The active card's cover art, blurred behind the whole screen. */
const backdrop = computed(() => {
  const game = deck.value[0]
  if (!game) return null
  return game.coverUrl || game.media.find((m) => m.kind === 'image')?.url || null
})

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
    transform: 'scale(0.94) translateY(16px)',
    opacity: 0.7,
    transition: 'transform 0.3s ease-out, opacity 0.3s ease-out',
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

<style scoped>
.cleared {
  animation: rise 0.45s cubic-bezier(0.2, 0.8, 0.2, 1);
}
@keyframes rise {
  from {
    opacity: 0;
    transform: translateY(16px) scale(0.96);
  }
}
@media (prefers-reduced-motion: reduce) {
  .cleared {
    animation: none;
  }
}
</style>
