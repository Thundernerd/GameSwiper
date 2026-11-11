<template>
  <article class="flex flex-col min-w-0 border-t border-stone-900 pt-4">
    <MediaCarousel :media="carouselMedia" class="shrink-0" />

    <div class="pt-5 space-y-3 flex-1 flex flex-col">
      <div v-if="game.events.length" class="flex flex-wrap gap-x-3 gap-y-1">
        <NuxtLink
          v-for="ev in game.events"
          :key="ev.id"
          :to="link(`/events/${ev.id}/overview`)"
          class="ed-caps text-stone-600 hover:text-stone-900 hover:underline underline-offset-4"
        >
          {{ ev.name }}
        </NuxtLink>
      </div>

      <div class="flex items-baseline justify-between gap-4">
        <h2 class="ed-serif text-3xl font-medium leading-tight min-w-0">{{ game.name }}</h2>
        <span
          v-if="game.rating != null"
          class="shrink-0 ed-serif text-2xl italic tabular-nums"
          :title="`IGDB rating ${game.rating}`"
        >
          {{ game.rating }}<span class="sr-only"> IGDB rating</span>
        </span>
      </div>

      <p class="text-sm text-stone-600">
        <template v-if="companyLine">
          <span :title="companyLine">{{ companyLine }}</span>
          {{ ' · ' }}
        </template>
        <span>WW release</span>
        {{ ' ' }}
        <span :class="formattedRelease ? 'text-stone-900' : 'italic'">
          {{ formattedRelease || 'Unknown' }}
        </span>
      </p>

      <div v-if="game.summary" class="text-[0.95rem] text-stone-700 leading-relaxed">
        <p :class="expanded || !needsToggle ? '' : 'line-clamp-4'">
          {{ game.summary }}
        </p>
        <button
          v-if="needsToggle"
          type="button"
          class="mt-1 ed-serif italic text-stone-900 underline underline-offset-4 hover:opacity-70"
          @click="expanded = !expanded"
        >
          {{ expanded ? 'Show less' : 'Show more' }}
        </button>
      </div>

      <div
        v-if="game.genres.length || game.platforms.length"
        class="border-t border-stone-200 pt-3 space-y-1 text-sm"
      >
        <p v-if="game.genres.length" class="text-stone-900">{{ game.genres.join(' · ') }}</p>
        <p v-if="game.platforms.length" class="text-stone-600">{{ game.platforms.join(' · ') }}</p>
      </div>

      <div v-if="game.links.length" class="flex flex-wrap gap-x-4 gap-y-1 pt-1 mt-auto">
        <a
          v-for="link in game.links"
          :key="link.url"
          :href="link.url"
          target="_blank"
          rel="noopener noreferrer"
          class="ed-caps text-stone-900 border-b border-stone-400 hover:border-stone-900"
        >
          {{ link.label }} ↗
        </a>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import type { LibraryGame, MediaItem } from '~/shared/api'
import MediaCarousel from './MediaCarousel.vue'

const props = defineProps<{
  game: LibraryGame
}>()

const link = usePrototypeLink()
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
