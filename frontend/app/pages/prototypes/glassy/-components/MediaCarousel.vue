<template>
  <div class="overflow-hidden bg-slate-950/40">
    <template v-if="media.length">
      <div class="aspect-[16/10] w-full relative">
        <iframe
          v-if="current?.kind === 'video' && videoActive"
          :src="embedUrl(current.url)"
          class="h-full w-full"
          title="Game trailer"
          allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
          allowfullscreen
        />
        <button
          v-else-if="current?.kind === 'video'"
          type="button"
          class="group h-full w-full flex flex-col items-center justify-center gap-3 bg-gradient-to-br from-indigo-950/80 via-slate-900/70 to-fuchsia-950/80 text-slate-100 transition"
          @click.stop="playVideo"
        >
          <span
            class="flex h-14 w-14 items-center justify-center rounded-full bg-gradient-to-br from-fuchsia-500 via-violet-600 to-cyan-500 pl-1 text-white text-xl shadow-[0_0_40px_-4px_rgba(217,70,239,0.7)] ring-4 ring-white/15 group-hover:scale-110 transition"
            aria-hidden="true"
          >
            ▶
          </span>
          <span class="text-sm">{{ current.title || 'Play trailer' }}</span>
        </button>
        <img
          v-else-if="current"
          :src="current.url"
          :alt="current.title || 'Game media'"
          class="h-full w-full object-cover"
        />
      </div>

      <div
        v-if="media.length > 1"
        class="flex items-center justify-center gap-3 py-2"
        @pointerdown.stop
      >
        <button
          type="button"
          class="grid h-7 w-7 place-items-center rounded-full border border-white/15 bg-white/10 text-sm text-slate-100 hover:bg-white/20 transition"
          aria-label="Previous media"
          @click.stop="prev"
        >
          ‹
        </button>
        <div class="flex items-center gap-1.5">
          <button
            v-for="(m, i) in media"
            :key="i"
            type="button"
            class="h-1.5 rounded-full transition-all duration-300"
            :class="i === index ? 'w-5 bg-gradient-to-r from-fuchsia-400 to-cyan-300' : 'w-1.5 bg-white/40 hover:bg-white/70'"
            :aria-label="`Show media ${i + 1}`"
            @click.stop="goTo(i)"
          />
        </div>
        <button
          type="button"
          class="grid h-7 w-7 place-items-center rounded-full border border-white/15 bg-white/10 text-sm text-slate-100 hover:bg-white/20 transition"
          aria-label="Next media"
          @click.stop="next"
        >
          ›
        </button>
      </div>
    </template>
    <div v-else class="aspect-[16/10] w-full flex items-center justify-center bg-gradient-to-br from-indigo-950/70 to-fuchsia-950/70 text-slate-300">
      No media
    </div>
  </div>
</template>

<script setup lang="ts">
import type { MediaItem } from '~/shared/api'

const props = defineProps<{
  media: MediaItem[]
  /** When true and slide is video, auto-embed (swipe deck). Library uses click-to-play. */
  active?: boolean
  /** Prefer cover/image first; useful for library grids to avoid auto-video. */
  preferImageFirst?: boolean
}>()

const index = ref(0)
const userPlayed = ref(false)

const orderedMedia = computed(() => {
  if (!props.preferImageFirst || !props.media.length) return props.media
  const images = props.media.filter((m) => m.kind === 'image')
  const videos = props.media.filter((m) => m.kind === 'video')
  if (!images.length) return props.media
  return [...images, ...videos]
})

const current = computed(() => orderedMedia.value[index.value] || null)

const videoActive = computed(() => {
  if (props.active) return true
  return userPlayed.value
})

watch(
  () => props.media,
  () => {
    index.value = 0
    userPlayed.value = false
  },
)

watch(index, () => {
  userPlayed.value = false
})

function goTo(i: number) {
  index.value = i
}

function prev() {
  index.value = (index.value - 1 + orderedMedia.value.length) % orderedMedia.value.length
}

function next() {
  index.value = (index.value + 1) % orderedMedia.value.length
}

function playVideo() {
  userPlayed.value = true
}

function embedUrl(url: string) {
  if (url.includes('youtube.com/embed')) {
    const sep = url.includes('?') ? '&' : '?'
    return `${url}${sep}rel=0&modestbranding=1`
  }
  return url
}
</script>
