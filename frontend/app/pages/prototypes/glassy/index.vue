<template>
  <Shell>
    <div class="mx-auto max-w-6xl px-4 py-10">
      <div class="mb-8">
        <h1 class="text-4xl font-black tracking-tight">
          <span class="bg-gradient-to-r from-fuchsia-300 via-violet-200 to-cyan-200 bg-clip-text text-transparent">
            Finished events
          </span>
        </h1>
        <p class="text-slate-200 mt-2">Pick an event and swipe through its games.</p>
      </div>

      <StatusPanel v-if="pending" kind="loading">Loading events…</StatusPanel>
      <StatusPanel v-else-if="error" kind="error">{{ error }}</StatusPanel>
      <StatusPanel v-else-if="!events.length" kind="empty">
        No finished events yet. Sync IGDB from the backend, or enable mock mode.
      </StatusPanel>

      <div v-else class="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
        <NuxtLink
          v-for="ev in events"
          :key="ev.id"
          :to="link(`/events/${ev.id}`)"
          class="group relative rounded-3xl border border-white/15 bg-slate-950/45 backdrop-blur-xl overflow-hidden shadow-[0_24px_60px_-24px_rgba(0,0,0,0.8)] transition duration-300 hover:-translate-y-1 hover:border-white/30 hover:shadow-[0_30px_70px_-20px_rgba(217,70,239,0.45)]"
        >
          <div class="aspect-[16/9] relative overflow-hidden bg-gradient-to-br from-indigo-900/60 to-fuchsia-900/60">
            <img
              v-if="ev.logoUrl"
              :src="ev.logoUrl"
              :alt="ev.name"
              class="h-full w-full object-cover transition duration-500 group-hover:scale-105"
            />
            <div
              v-else
              class="h-full w-full flex items-center justify-center text-slate-200 text-sm"
            >
              No logo
            </div>
            <div class="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-slate-950/70 to-transparent" />
          </div>
          <div class="p-5 space-y-3">
            <h2 class="font-bold text-lg leading-snug">{{ ev.name }}</h2>
            <p class="text-xs text-slate-300">
              {{ formatRange(ev.startTime, ev.endTime) }}
            </p>
            <div class="flex items-center justify-between text-sm">
              <span class="text-slate-100 tabular-nums">
                {{ ev.ratedCount }} / {{ ev.totalCount }} rated
              </span>
              <span
                class="rounded-full px-2.5 py-0.5 text-xs font-bold"
                :class="
                  isDone(ev)
                    ? 'bg-gradient-to-r from-emerald-300 to-cyan-300 text-slate-950'
                    : 'bg-gradient-to-r from-fuchsia-600 to-violet-600 text-white shadow-md shadow-fuchsia-500/30'
                "
              >
                {{ isDone(ev) ? 'Done' : 'Swipe' }}
              </span>
            </div>
            <div class="h-2 rounded-full bg-white/10 overflow-hidden">
              <div
                class="h-full rounded-full bg-gradient-to-r from-fuchsia-500 via-violet-500 to-cyan-400 shadow-[0_0_12px_rgba(217,70,239,0.7)] transition-all duration-700"
                :style="{
                  width: `${ev.totalCount ? Math.min(100, (ev.ratedCount / ev.totalCount) * 100) : 0}%`,
                }"
              />
            </div>
          </div>
        </NuxtLink>
      </div>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import type { EventSummary } from '~/shared/api'
import Shell from './-components/Shell.vue'
import StatusPanel from './-components/StatusPanel.vue'

const link = usePrototypeLink()

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
