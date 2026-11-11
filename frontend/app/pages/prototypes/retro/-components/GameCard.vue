<template>
  <article
    class="retro-panel mx-auto w-full max-w-md flex flex-col overflow-hidden select-none touch-none !border-[color:var(--r-cyan)] !shadow-[6px_6px_0_#000,0_0_24px_rgba(62,240,255,0.25)]"
    :style="cardStyle"
    @pointerdown="$emit('pointerdown', $event)"
  >
    <MediaCarousel :media="game.media" :active="active" class="shrink-0 border-b-2 border-[color:var(--r-cyan)]" />

    <div class="p-4 space-y-2.5">
      <div class="flex items-start justify-between gap-3">
        <h2 class="retro-display text-sm sm:text-base leading-relaxed min-w-0 text-[color:var(--r-yellow)]">
          {{ game.name }}
        </h2>
        <span
          v-if="game.rating != null"
          class="retro-display shrink-0 border-2 border-[color:var(--r-green)] text-[color:var(--r-green)] text-[10px] px-2 py-1 tabular-nums"
          :title="`IGDB rating ${game.rating}`"
        >
          {{ game.rating }}
        </span>
      </div>

      <p v-if="companyLine" class="text-[color:var(--r-muted)] truncate" :title="companyLine">
        {{ companyLine }}
      </p>

      <p v-if="game.summary" class="text-[color:var(--r-text)] line-clamp-3">
        {{ game.summary }}
      </p>

      <div class="flex flex-wrap gap-1.5">
        <span v-for="g in game.genres" :key="`g-${g}`" class="retro-tag text-[color:var(--r-magenta)]">
          {{ g }}
        </span>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span v-for="p in game.platforms" :key="`p-${p}`" class="retro-tag text-[color:var(--r-muted)]">
          {{ p }}
        </span>
      </div>
    </div>

    <div
      v-if="overlay"
      class="pointer-events-none absolute top-6 inset-x-0 flex justify-center"
    >
      <span
        class="retro-display flex flex-col items-center gap-1 border-4 bg-black/85 px-4 py-2 -rotate-6 shadow-[4px_4px_0_#000]"
        :class="overlay.class"
      >
        <span class="text-[10px] text-[color:var(--r-text)]">PLAYER 1</span>
        <span class="retro-glow text-xl">{{ overlay.label }}</span>
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
    transition: props.leaving || (!x && !y) ? 'transform 0.22s steps(6, end)' : 'none',
  }
})

const overlay = computed(() => {
  const x = props.dragX || 0
  const y = props.dragY || 0
  if (props.leaving === 'like' || x > 80)
    return { label: 'LIKE!', class: 'border-[color:var(--r-green)] text-[color:var(--r-green)]' }
  if (props.leaving === 'dislike' || x < -80)
    return { label: 'NOPE!', class: 'border-[color:var(--r-red)] text-[color:var(--r-red)]' }
  if (props.leaving === 'defer' || y > 80)
    return { label: 'LATER', class: 'border-[color:var(--r-yellow)] text-[color:var(--r-yellow)]' }
  return null
})
</script>
