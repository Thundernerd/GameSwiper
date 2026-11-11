<template>
  <article
    class="mx-auto w-full max-w-md flex flex-col rounded-3xl border border-white/20 bg-slate-900/60 backdrop-blur-2xl overflow-hidden select-none touch-none ring-1 ring-inset ring-white/5"
    :style="cardStyle"
    @pointerdown="$emit('pointerdown', $event)"
  >
    <MediaCarousel :media="game.media" :active="active" class="shrink-0" />

    <div class="p-5 space-y-3 bg-gradient-to-b from-white/[0.04] to-transparent">
      <div class="flex items-start justify-between gap-3">
        <h2 class="text-xl font-extrabold leading-tight tracking-tight min-w-0">{{ game.name }}</h2>
        <span
          v-if="game.rating != null"
          class="shrink-0 rounded-full bg-gradient-to-r from-emerald-300 to-cyan-300 text-slate-950 text-sm font-bold px-2.5 py-1 tabular-nums shadow-lg shadow-emerald-500/25"
          :title="`IGDB rating ${game.rating}`"
        >
          {{ game.rating }}
        </span>
      </div>

      <p v-if="companyLine" class="text-sm text-slate-300 truncate" :title="companyLine">
        {{ companyLine }}
      </p>

      <p v-if="game.summary" class="text-sm text-slate-100 leading-relaxed line-clamp-3">
        {{ game.summary }}
      </p>

      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="g in game.genres"
          :key="`g-${g}`"
          class="rounded-full bg-gradient-to-r from-fuchsia-500/30 to-violet-500/30 ring-1 ring-inset ring-fuchsia-300/30 text-fuchsia-50 text-xs font-medium px-2.5 py-1"
        >
          {{ g }}
        </span>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="p in game.platforms"
          :key="`p-${p}`"
          class="rounded-full bg-white/10 ring-1 ring-inset ring-white/15 text-slate-100 text-xs px-2.5 py-1"
        >
          {{ p }}
        </span>
      </div>
    </div>

    <div
      v-if="overlayLabel"
      class="pointer-events-none absolute top-5 inset-x-0 flex justify-center"
    >
      <span
        class="overlay-pop rounded-full px-5 py-1.5 text-lg font-black uppercase tracking-widest text-white backdrop-blur-md"
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
  // Tint the soft shadow toward the decision the drag is heading for.
  const strength = Math.min(1, Math.max(Math.abs(x), Math.max(y, 0)) / 120)
  const glow =
    y > Math.abs(x) ? '251, 191, 36' : x > 0 ? '52, 211, 153' : x < 0 ? '251, 113, 133' : '0, 0, 0'
  return {
    transform: `translate(${x}px, ${y}px) rotate(${rot}deg)`,
    transition:
      props.leaving || (!x && !y)
        ? 'transform 0.22s ease-out, box-shadow 0.22s ease-out'
        : 'box-shadow 0.1s linear',
    boxShadow: `0 30px 80px -20px rgba(0, 0, 0, 0.75), 0 0 ${24 + 40 * strength}px ${
      -4 + 6 * strength
    }px rgba(${glow}, ${0.55 * strength})`,
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
  if (overlayLabel.value === 'Like')
    return 'bg-gradient-to-r from-emerald-700/90 to-teal-700/90 shadow-[0_0_30px_rgba(52,211,153,0.6)] ring-2 ring-emerald-300/70'
  if (overlayLabel.value === 'Nope')
    return 'bg-gradient-to-r from-rose-600/90 to-pink-700/90 shadow-[0_0_30px_rgba(251,113,133,0.6)] ring-2 ring-rose-300/70'
  return 'bg-gradient-to-r from-amber-700/90 to-orange-700/90 shadow-[0_0_30px_rgba(251,191,36,0.6)] ring-2 ring-amber-300/70'
})
</script>

<style scoped>
.overlay-pop {
  animation: pop 0.18s ease-out;
}
@keyframes pop {
  from {
    transform: scale(0.7);
    opacity: 0;
  }
}
@media (prefers-reduced-motion: reduce) {
  .overlay-pop {
    animation: none;
  }
}
</style>
