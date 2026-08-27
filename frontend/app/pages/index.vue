<template>
  <div class="mx-auto max-w-6xl px-4 py-10">
    <div class="mb-8">
      <h1 class="text-3xl font-bold tracking-tight">Finished events</h1>
      <p class="text-slate-400 mt-1">Pick an event and swipe through its games.</p>
    </div>

    <p v-if="pending" class="text-slate-400">Loading events…</p>
    <p v-else-if="error" class="text-rose-400">{{ error }}</p>
    <p v-else-if="!events.length" class="text-slate-400">
      No finished events yet. Sync IGDB from the backend, or enable mock mode.
    </p>

    <div v-else class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <NuxtLink
        v-for="ev in events"
        :key="ev.id"
        :to="`/events/${ev.id}`"
        class="group rounded-2xl border border-slate-800 bg-slate-900/60 overflow-hidden hover:border-violet-500/50 hover:bg-slate-900 transition"
      >
        <div class="aspect-[16/9] bg-slate-800 relative">
          <img
            v-if="ev.logoUrl"
            :src="ev.logoUrl"
            :alt="ev.name"
            class="h-full w-full object-cover opacity-90 group-hover:opacity-100 transition"
          />
          <div
            v-else
            class="h-full w-full flex items-center justify-center text-slate-500 text-sm"
          >
            No logo
          </div>
        </div>
        <div class="p-4 space-y-2">
          <h2 class="font-semibold text-lg leading-snug">{{ ev.name }}</h2>
          <p class="text-xs text-slate-400">
            {{ formatRange(ev.startTime, ev.endTime) }}
          </p>
          <div class="flex items-center justify-between text-sm">
            <span class="text-slate-300">
              {{ ev.ratedCount }} / {{ ev.totalCount }} rated
            </span>
            <span
              class="rounded-full px-2 py-0.5 text-xs"
              :class="
                ev.ratedCount >= ev.totalCount && ev.totalCount > 0
                  ? 'bg-emerald-500/20 text-emerald-300'
                  : 'bg-violet-500/20 text-violet-200'
              "
            >
              {{
                ev.ratedCount >= ev.totalCount && ev.totalCount > 0
                  ? 'Done'
                  : 'Swipe'
              }}
            </span>
          </div>
          <div class="h-1.5 rounded-full bg-slate-800 overflow-hidden">
            <div
              class="h-full bg-violet-500 transition-all"
              :style="{
                width: `${ev.totalCount ? Math.min(100, (ev.ratedCount / ev.totalCount) * 100) : 0}%`,
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
