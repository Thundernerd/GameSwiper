<template>
  <article
    class="flex flex-col rounded-3xl border border-white/15 bg-slate-950/55 backdrop-blur-xl overflow-hidden shadow-[0_24px_60px_-24px_rgba(0,0,0,0.8)] transition duration-300 hover:-translate-y-1 hover:border-white/25 hover:shadow-[0_30px_70px_-24px_rgba(139,92,246,0.45)]"
  >
    <MediaCarousel
      :media="carouselMedia"
      class="shrink-0"
    />

    <div class="p-5 space-y-3 flex-1 flex flex-col">
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

      <div v-if="game.events.length" class="flex flex-wrap gap-1.5">
        <NuxtLink
          v-for="ev in game.events"
          :key="ev.id"
          :to="link(`/events/${ev.id}/overview`)"
          class="rounded-full border border-cyan-300/40 bg-cyan-950/50 text-cyan-50 text-xs font-medium px-2.5 py-1 hover:bg-cyan-400/25 transition"
        >
          {{ ev.name }}
        </NuxtLink>
      </div>

      <p v-if="companyLine" class="text-sm text-slate-300" :title="companyLine">
        {{ companyLine }}
      </p>

      <p class="text-sm text-slate-200">
        <span class="text-slate-300">WW release</span>
        {{ ' · ' }}
        <span :class="formattedRelease ? 'text-white font-medium' : 'text-slate-300 italic'">
          {{ formattedRelease || 'Unknown' }}
        </span>
      </p>

      <div v-if="game.summary" class="text-sm text-slate-100 leading-relaxed">
        <p :class="expanded || !needsToggle ? '' : 'line-clamp-4'">
          {{ game.summary }}
        </p>
        <button
          v-if="needsToggle"
          type="button"
          class="mt-1 text-xs font-semibold text-fuchsia-300 hover:text-fuchsia-200 hover:underline"
          @click="expanded = !expanded"
        >
          {{ expanded ? 'Show less' : 'Show more' }}
        </button>
      </div>

      <div v-if="game.genres.length" class="flex flex-wrap gap-1.5">
        <span
          v-for="g in game.genres"
          :key="`g-${g}`"
          class="rounded-full bg-gradient-to-r from-fuchsia-500/30 to-violet-500/30 ring-1 ring-inset ring-fuchsia-300/30 text-fuchsia-50 text-xs font-medium px-2.5 py-1"
        >
          {{ g }}
        </span>
      </div>
      <div v-if="game.platforms.length" class="flex flex-wrap gap-1.5">
        <span
          v-for="p in game.platforms"
          :key="`p-${p}`"
          class="rounded-full bg-white/10 ring-1 ring-inset ring-white/15 text-slate-100 text-xs px-2.5 py-1"
        >
          {{ p }}
        </span>
      </div>

      <div v-if="game.links.length" class="flex flex-wrap gap-2 pt-2 mt-auto">
        <a
          v-for="link in game.links"
          :key="link.url"
          :href="link.url"
          target="_blank"
          rel="noopener noreferrer"
          class="rounded-full bg-white/5 ring-1 ring-inset ring-white/15 px-2.5 py-1 text-xs font-medium text-fuchsia-200 hover:bg-white/15 hover:text-white transition"
        >
          {{ link.label }}
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
