<template>
  <div class="mx-auto max-w-6xl px-4 py-10">
    <div class="mb-8">
      <p class="arcade-display text-[10px] text-arcade-cyan mb-3">— SELECT STAGE —</p>
      <h1 class="arcade-display arcade-glow text-xl sm:text-3xl text-arcade-magenta">FINISHED EVENTS</h1>
      <p class="text-arcade-muted mt-3">Pick an event and swipe through its games.</p>
    </div>

    <p v-if="pending" class="arcade-display arcade-blink text-xs text-arcade-cyan">LOADING EVENTS…</p>
    <div v-else-if="error" class="arcade-panel !border-arcade-red p-4">
      <p class="arcade-display text-xs text-arcade-red mb-2">ERROR</p>
      <p class="text-arcade-text">{{ error }}</p>
    </div>
    <p v-else-if="!events.length" class="arcade-panel p-4 text-arcade-muted">
      No finished events yet. Sync IGDB from the backend, or enable mock mode.
    </p>

    <div v-else class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
      <NuxtLink
        v-for="ev in events"
        :key="ev.id"
        :to="`/events/${ev.id}`"
        class="group arcade-panel overflow-hidden hover:!border-arcade-yellow hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-[6px_6px_0_#000]"
      >
        <div class="aspect-[16/9] bg-black relative border-b-2 border-arcade-line">
          <img
            v-if="ev.logoUrl"
            :src="ev.logoUrl"
            :alt="ev.name"
            class="h-full w-full object-cover opacity-80 group-hover:opacity-100"
          />
          <div
            v-else
            class="arcade-display h-full w-full flex items-center justify-center text-[10px] text-arcade-dim"
          >
            NO SIGNAL
          </div>
        </div>
        <div class="p-4 space-y-3">
          <h2 class="arcade-display text-xs sm:text-sm leading-relaxed text-arcade-text group-hover:text-arcade-yellow">
            {{ ev.name }}
          </h2>
          <p class="text-arcade-muted">
            {{ formatRange(ev.startTime, ev.endTime) }}
          </p>
          <div class="flex items-center justify-between gap-2">
            <span class="text-arcade-text tabular-nums">
              {{ ev.ratedCount }} / {{ ev.totalCount }} rated
            </span>
            <span
              class="arcade-display text-[10px] px-1.5 py-1 border-2"
              :class="
                isDone(ev)
                  ? 'border-arcade-green text-arcade-green'
                  : 'border-arcade-yellow text-arcade-yellow'
              "
            >
              {{ isDone(ev) ? 'DONE' : 'SWIPE' }}
            </span>
          </div>
          <div class="h-3 border-2 border-arcade-line bg-black p-px" aria-hidden="true">
            <div
              class="h-full"
              :class="isDone(ev) ? 'bg-arcade-green' : 'bg-arcade-magenta'"
              :style="{
                width: `${ev.totalCount ? Math.min(100, (ev.ratedCount / ev.totalCount) * 100) : 0}%`,
                backgroundImage: 'repeating-linear-gradient(90deg, transparent 0 6px, rgba(0,0,0,0.55) 6px 8px)',
              }"
            />
          </div>
        </div>
      </NuxtLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { EventSummary } from '../shared/api'

const api = useApi()
const events = ref<EventSummary[]>([])
const pending = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    const res = await api.listEvents()
    events.value = res.events
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load events'
  } finally {
    pending.value = false
  }
})

function isDone(ev: EventSummary) {
  return ev.ratedCount >= ev.totalCount && ev.totalCount > 0
}

function formatRange(start: string | null, end: string | null) {
  const fmt = (iso: string) =>
    new Date(iso).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  if (start && end) return `${fmt(start)} – ${fmt(end)}`
  if (end) return `Ended ${fmt(end)}`
  return 'Date unknown'
}
</script>
