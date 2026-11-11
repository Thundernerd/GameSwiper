<template>
  <Shell>
    <div class="mx-auto max-w-6xl px-5 sm:px-8 py-12 sm:py-16">
      <header class="mb-10 sm:mb-14 max-w-2xl">
        <p class="ed-caps text-stone-600">The archive</p>
        <h1 class="ed-serif mt-3 text-4xl sm:text-6xl font-medium leading-[1.05] tracking-tight">
          Finished events
        </h1>
        <p class="mt-4 text-lg text-stone-600 leading-relaxed">
          Pick an event and swipe through its games.
        </p>
      </header>

      <p v-if="pending" class="ed-serif italic text-xl text-stone-600 border-t border-stone-300 pt-6">
        Loading events…
      </p>
      <p v-else-if="error" class="border-t border-rose-700 pt-6 text-rose-700">{{ error }}</p>
      <p v-else-if="!events.length" class="border-t border-stone-300 pt-6 text-stone-600">
        No finished events yet. Sync IGDB from the backend, or enable mock mode.
      </p>

      <div v-else class="grid gap-x-8 gap-y-14 sm:grid-cols-2 lg:grid-cols-3">
        <NuxtLink
          v-for="ev in events"
          :key="ev.id"
          :to="link(`/events/${ev.id}`)"
          class="group block border-t border-stone-900 pt-4"
        >
          <div class="aspect-[4/5] bg-stone-200 overflow-hidden">
            <img
              v-if="ev.logoUrl"
              :src="ev.logoUrl"
              :alt="ev.name"
              class="h-full w-full object-cover transition duration-500 group-hover:scale-[1.03]"
            />
            <div
              v-else
              class="h-full w-full flex items-center justify-center ed-serif italic text-stone-600"
            >
              No logo
            </div>
          </div>
          <p class="ed-caps mt-5 text-stone-600">
            {{ formatRange(ev.startTime, ev.endTime) }}
          </p>
          <h2
            class="ed-serif mt-2 text-2xl font-medium leading-snug group-hover:underline decoration-1 underline-offset-4"
          >
            {{ ev.name }}
          </h2>
          <div class="mt-4 flex items-center justify-between text-sm">
            <span class="text-stone-600 tabular-nums">
              {{ ev.ratedCount }} / {{ ev.totalCount }} rated
            </span>
            <span
              class="ed-caps"
              :class="isDone(ev) ? 'text-emerald-700' : 'text-stone-900'"
            >
              {{ isDone(ev) ? 'Done' : 'Swipe →' }}
            </span>
          </div>
          <div class="mt-2 h-px bg-stone-300">
            <div
              class="h-px transition-all"
              :class="isDone(ev) ? 'bg-emerald-700' : 'bg-stone-900'"
              :style="{
                width: `${ev.totalCount ? Math.min(100, (ev.ratedCount / ev.totalCount) * 100) : 0}%`,
              }"
            />
          </div>
        </NuxtLink>
      </div>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import type { EventSummary } from '~/shared/api'
import Shell from './-components/Shell.vue'

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
