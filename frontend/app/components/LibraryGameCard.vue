<template>
  <article class="arcade-panel flex flex-col overflow-hidden">
    <MediaCarousel
      :media="carouselMedia"
      class="shrink-0 border-b-2 border-arcade-line"
    />

    <div class="p-4 space-y-3 flex-1 flex flex-col">
      <div class="flex items-start justify-between gap-3">
        <h2 class="arcade-display text-sm sm:text-base leading-relaxed min-w-0 text-arcade-yellow">
          {{ game.name }}
        </h2>
        <span
          v-if="game.rating != null"
          class="arcade-display shrink-0 border-2 border-arcade-green text-arcade-green text-[10px] px-2 py-1 tabular-nums"
          :title="`IGDB rating ${game.rating}`"
        >
          {{ game.rating }}
        </span>
      </div>

      <div v-if="game.events.length" class="flex flex-wrap gap-1.5">
        <NuxtLink
          v-for="ev in game.events"
          :key="ev.id"
          :to="`/events/${ev.id}/overview`"
          class="arcade-tag text-arcade-cyan hover:bg-arcade-cyan hover:text-black"
        >
          {{ ev.name }}
        </NuxtLink>
      </div>

      <p v-if="companyLine" class="text-arcade-muted" :title="companyLine">
        {{ companyLine }}
      </p>

      <p class="text-arcade-text">
        <span class="text-arcade-dim">WW RELEASE</span>
        {{ ' · ' }}
        <span :class="formattedRelease ? 'text-arcade-text' : 'text-arcade-dim'">
          {{ formattedRelease || 'Unknown' }}
        </span>
      </p>

      <div v-if="game.summary" class="text-arcade-text">
        <p :class="expanded || !needsToggle ? '' : 'line-clamp-4'">
          {{ game.summary }}
        </p>
        <button
          v-if="needsToggle"
          type="button"
          class="arcade-display mt-2 text-[10px] text-arcade-cyan hover:text-arcade-yellow"
          @click="expanded = !expanded"
        >
          {{ expanded ? '▲ LESS' : '▼ MORE' }}
        </button>
      </div>

      <div v-if="game.genres.length" class="flex flex-wrap gap-1.5">
        <span v-for="g in game.genres" :key="`g-${g}`" class="arcade-tag text-arcade-magenta">
          {{ g }}
        </span>
      </div>
      <div v-if="game.platforms.length" class="flex flex-wrap gap-1.5">
        <span v-for="p in game.platforms" :key="`p-${p}`" class="arcade-tag text-arcade-muted">
          {{ p }}
        </span>
      </div>

      <div v-if="game.links.length" class="flex flex-wrap gap-x-4 gap-y-1 pt-1 mt-auto">
        <a
          v-for="link in game.links"
          :key="link.url"
          :href="link.url"
          target="_blank"
          rel="noopener noreferrer"
          class="arcade-link"
        >
          ▸ {{ link.label }}
        </a>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import type { LibraryGame, MediaItem } from '../shared/api'

const props = defineProps<{
  game: LibraryGame
}>()

const expanded = ref(false)

const needsToggle = computed(() => (props.game.summary?.length ?? 0) > 220)

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

const formattedRelease = computed(() => {
  const raw = props.game.releaseDate
  if (!raw) return null
  const d = new Date(raw)
  if (Number.isNaN(d.getTime())) return null
  return d.toLocaleDateString(undefined, {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  })
})

/** Screenshots first; cover art only as fallback when there are no screenshots. */
const carouselMedia = computed((): MediaItem[] => {
  const media = [...(props.game.media || [])]
  const screenshots = media.filter(
    (m) => m.kind === 'image' && !isCoverArtUrl(m.url),
  )
  const videos = media.filter((m) => m.kind === 'video')
  const covers = media.filter((m) => m.kind === 'image' && isCoverArtUrl(m.url))

  if (props.game.coverUrl && !covers.some((m) => m.url === props.game.coverUrl)) {
    covers.push({ kind: 'image', url: props.game.coverUrl, title: props.game.name })
  }

  if (screenshots.length) {
    return [...screenshots, ...videos]
  }
  // No screenshots: trailer first, cover only as last resort behind video.
  if (videos.length) {
    return [...videos, ...covers]
  }
  return covers
})

function isCoverArtUrl(url: string) {
  return /\/t_cover[_a-z0-9]*\//i.test(url)
}
</script>
