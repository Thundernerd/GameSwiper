<template>
  <article
    class="mx-auto w-full max-w-md flex flex-col rounded-2xl border border-slate-700 bg-slate-900 shadow-2xl overflow-hidden select-none touch-none"
    :style="cardStyle"
    @pointerdown="$emit('pointerdown', $event)"
  >
    <MediaCarousel :media="game.media" :active="active" class="shrink-0 bg-black" />

    <div class="p-4 space-y-2.5">
      <div class="flex items-start justify-between gap-3">
        <h2 class="text-xl font-bold leading-tight min-w-0">{{ game.name }}</h2>
        <span
          v-if="game.rating != null"
          class="shrink-0 rounded-full bg-emerald-500/20 text-emerald-200 text-sm font-semibold px-2.5 py-1 tabular-nums"
          :title="`IGDB rating ${game.rating}`"
        >
          {{ game.rating }}
        </span>
      </div>

      <p v-if="companyLine" class="text-sm text-slate-400 truncate" :title="companyLine">
        {{ companyLine }}
      </p>

      <p v-if="game.summary" class="text-sm text-slate-300 leading-relaxed line-clamp-3">
        {{ game.summary }}
      </p>

      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="g in game.genres"
          :key="`g-${g}`"
          class="rounded-full bg-violet-500/20 text-violet-200 text-xs px-2.5 py-1"
        >
          {{ g }}
        </span>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="p in game.platforms"
          :key="`p-${p}`"
          class="rounded-full bg-slate-800 text-slate-300 text-xs px-2.5 py-1"
        >
          {{ p }}
        </span>
      </div>
    </div>

    <div
      v-if="overlayLabel"
      class="pointer-events-none absolute top-4 inset-x-0 flex justify-center"
    >
      <span
        class="rounded-lg border-2 px-4 py-1 text-lg font-bold uppercase tracking-wider bg-slate-950/70"
        :class="overlayClass"
      >
        {{ overlayLabel }}
      </span>
    </div>
  </article>
</template>

<script setup lang="ts">
import type { GameCard } from '../shared/api'

const props = defineProps<{
  game: GameCard
  active: boolean
  dragX?: number
  dragY?: number
  leaving?: 'like' | 'dislike' | 'defer' | null
}>()

defineEmits<{ pointerdown: [PointerEvent] }>()

const companyLine = computed(() => {
  const developers = props.game.developers ?? []
  const publishers = props.game.publishers ?? []
  if (!developers.length && !publishers.length) return ''

  const same =
    developers.length === publishers.length &&
    developers.every((name, i) => name === publishers[i])

  if (same || !publishers.length) return developers.join(', ')
  if (!developers.length) return publishers.join(', ')
  return `${developers.join(', ')} · ${publishers.join(', ')}`
})

const cardStyle = computed(() => {
  const x = props.dragX || 0
  const y = props.dragY || 0
  const rot = x / 28
  return {
    transform: `translate(${x}px, ${y}px) rotate(${rot}deg)`,
    transition: props.leaving || (!x && !y) ? 'transform 0.22s ease-out' : 'none',
  }
})

const overlayLabel = computed(() => {
  const x = props.dragX || 0
  const y = props.dragY || 0
  if (props.leaving === 'like' || x > 80) return 'Like'
  if (props.leaving === 'dislike' || x < -80) return 'Nope'
  if (props.leaving === 'defer' || y > 80) return 'Later'
  return ''
})

const overlayClass = computed(() => {
  if (overlayLabel.value === 'Like') return 'border-emerald-400 text-emerald-300'
  if (overlayLabel.value === 'Nope') return 'border-rose-400 text-rose-300'
  return 'border-amber-400 text-amber-300'
})
</script>
