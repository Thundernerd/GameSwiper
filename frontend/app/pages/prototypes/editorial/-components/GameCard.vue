<template>
  <article
    class="mx-auto w-full max-w-md flex flex-col bg-white border border-stone-300 shadow-[0_1px_2px_rgba(28,25,23,0.06),0_12px_32px_-12px_rgba(28,25,23,0.18)] overflow-hidden select-none touch-none"
    :style="cardStyle"
    @pointerdown="$emit('pointerdown', $event)"
  >
    <MediaCarousel :media="game.media" :active="active" class="shrink-0" />

    <div class="px-5 pt-4 pb-5 space-y-3">
      <div class="flex items-baseline justify-between gap-4">
        <h2 class="ed-serif text-2xl font-medium leading-tight min-w-0">{{ game.name }}</h2>
        <span
          v-if="game.rating != null"
          class="shrink-0 ed-serif text-xl italic tabular-nums text-stone-900"
          :title="`IGDB rating ${game.rating}`"
        >
          {{ game.rating }}<span class="sr-only"> IGDB rating</span>
        </span>
      </div>

      <p v-if="companyLine" class="ed-caps text-stone-600 truncate" :title="companyLine">
        {{ companyLine }}
      </p>

      <p v-if="game.summary" class="text-[0.95rem] text-stone-700 leading-relaxed line-clamp-3">
        {{ game.summary }}
      </p>

      <div class="border-t border-stone-200 pt-3 space-y-1 text-sm">
        <p v-if="game.genres.length" class="text-stone-900">
          {{ game.genres.join(' · ') }}
        </p>
        <p v-if="game.platforms.length" class="text-stone-600">
          {{ game.platforms.join(' · ') }}
        </p>
      </div>
    </div>

    <div
      v-if="overlayLabel"
      class="pointer-events-none absolute top-5 inset-x-0 flex justify-center"
    >
      <span
        class="ed-serif border-2 bg-white/95 px-5 py-1 text-2xl font-semibold italic tracking-wide -rotate-3"
        :class="overlayClass"
      >
        {{ overlayLabel }}
      </span>
    </div>
  </article>
</template>

<script setup lang="ts">
import type { GameCard } from '~/shared/api'
import MediaCarousel from './MediaCarousel.vue'

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
  if (overlayLabel.value === 'Like') return 'border-emerald-700 text-emerald-700'
  if (overlayLabel.value === 'Nope') return 'border-rose-700 text-rose-700'
  return 'border-amber-800 text-amber-800'
})
</script>
