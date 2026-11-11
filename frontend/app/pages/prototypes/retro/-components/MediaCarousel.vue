<template>
  <div class="overflow-hidden bg-black">
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
          class="h-full w-full flex flex-col items-center justify-center gap-3 bg-black text-[color:var(--r-text)] hover:bg-[color:var(--r-panel)]"
          @click.stop="playVideo"
        >
          <span
            class="retro-display flex h-12 w-12 items-center justify-center border-2 border-[color:var(--r-magenta)] text-[color:var(--r-magenta)] text-base shadow-[4px_4px_0_#000]"
            aria-hidden="true"
          >
            ▶
          </span>
          <span class="retro-display text-[10px] uppercase px-4 text-center">{{ current.title || 'Play trailer' }}</span>
        </button>
        <img
          v-else-if="current"
          :src="current.url"
          :alt="current.title || 'Game media'"
          class="h-full w-full object-cover [image-rendering:pixelated]"
        />
      </div>

      <div
        v-if="media.length > 1"
        class="flex items-center justify-center gap-3 py-2 bg-black"
        @pointerdown.stop
      >
        <button
          type="button"
          class="retro-display px-2 py-1 text-[10px] text-[color:var(--r-cyan)] hover:text-[color:var(--r-yellow)]"
          aria-label="Previous media"
          @click.stop="prev"
        >
          ◀
        </button>
        <div class="flex gap-1.5">
          <button
            v-for="(m, i) in media"
            :key="i"
            type="button"
            class="h-2 w-2"
            :class="i === index ? 'bg-[color:var(--r-yellow)]' : 'bg-[color:var(--r-line)]'"
            :aria-label="`Media ${i + 1}`"
            @click.stop="goTo(i)"
          />
        </div>
        <button
          type="button"
          class="retro-display px-2 py-1 text-[10px] text-[color:var(--r-cyan)] hover:text-[color:var(--r-yellow)]"
          aria-label="Next media"
          @click.stop="next"
        >
          ▶
        </button>
      </div>
    </template>
    <div v-else class="retro-display aspect-[16/10] w-full flex items-center justify-center text-[10px] text-[color:var(--r-dim)]">
      NO SIGNAL
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
