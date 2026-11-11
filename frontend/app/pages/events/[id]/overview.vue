<template>
  <div class="mx-auto max-w-5xl px-4 py-8">
    <NuxtLink :to="`/events/${eventId}`" class="arcade-small-link">
      ◀ BACK TO DECK
    </NuxtLink>
    <p class="arcade-label text-arcade-yellow mt-6 mb-3">— RESULTS —</p>
    <h1 class="arcade-display arcade-glow text-base sm:text-2xl leading-relaxed text-arcade-magenta mb-3">{{ title }}</h1>
    <p class="text-arcade-muted mb-8">Your likes and dislikes for this event.</p>

    <LoadingLine v-if="pending">LOADING…</LoadingLine>
    <ErrorPanel v-else-if="error" :message="error" />

    <div v-else class="grid gap-8 md:grid-cols-2">
      <section v-for="column in columns" :key="column.key" class="min-w-0">
        <h2 :class="['arcade-display text-xs sm:text-sm mb-4 pb-2 border-b-2', ...column.headingClass]">
          {{ column.title }} <span class="tabular-nums">({{ column.games.length }})</span>
        </h2>
        <p v-if="!column.games.length" class="text-arcade-dim">{{ column.empty }}</p>
        <ul class="space-y-3">
          <li
            v-for="game in column.games"
            :key="game.id"
            class="arcade-panel flex gap-3 p-3"
          >
            <img
              v-if="game.coverUrl"
              :src="game.coverUrl"
              :alt="game.name"
              class="h-20 w-14 shrink-0 object-cover bg-black border-2 border-arcade-line"
            />
            <div class="min-w-0 flex-1">
              <p class="text-arcade-text text-2xl leading-none truncate">{{ game.name }}</p>
              <p class="text-arcade-muted mt-1 truncate">
                {{ game.platforms.join(', ') || '—' }}
              </p>
              <div class="flex flex-wrap gap-x-3 gap-y-1 mt-2">
                <a
                  v-for="link in game.links"
                  :key="link.url"
                  :href="link.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="arcade-link"
                >
                  {{ link.label }}
                </a>
              </div>
              <p v-if="itemError[game.id]" class="text-arcade-red mt-2">
                {{ itemError[game.id] }}
              </p>
            </div>
            <button
              type="button"
              class="arcade-btn shrink-0 self-center h-11 w-11 text-xl !font-sans"
              :class="column.moveButtonClass"
              :title="column.moveTitle"
              :aria-label="column.moveTitle"
              :disabled="movingId === game.id"
              @click="moveGame(game, column.moveAction)"
            >
              {{ column.moveLabel }}
            </button>
          </li>
        </ul>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { GameOverview, SwipeAction } from '../../../shared/api'

const route = useRoute()
const api = useApi()
const eventId = computed(() => Number(route.params.id))
const title = ref('Overview')
const liked = ref<GameOverview[]>([])
const disliked = ref<GameOverview[]>([])
const pending = ref(true)
const error = ref('')
const movingId = ref<number | null>(null)
const itemError = ref<Record<number, string>>({})

function byName(a: GameOverview, b: GameOverview) {
  return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })
}

function sortLists() {
  liked.value = [...liked.value].sort(byName)
  disliked.value = [...disliked.value].sort(byName)
}

const columns = computed(() => [
  {
    key: 'liked',
    title: 'LIKED',
    games: liked.value,
    empty: 'No likes yet.',
    headingClass: [actionColors.like.border, actionColors.like.text],
    moveAction: 'dislike' as SwipeAction,
    moveLabel: '✕',
    moveTitle: 'Move to disliked',
    moveButtonClass: actionColors.dislike.text,
  },
  {
    key: 'disliked',
    title: 'DISLIKED',
    games: disliked.value,
    empty: 'No dislikes yet.',
    headingClass: [actionColors.dislike.border, actionColors.dislike.text],
    moveAction: 'like' as SwipeAction,
    moveLabel: '♥',
    moveTitle: 'Move to liked',
    moveButtonClass: actionColors.like.text,
  },
])

async function moveGame(game: GameOverview, action: SwipeAction) {
  if (movingId.value !== null) return
  const fromLiked = action === 'dislike'
  const source = fromLiked ? liked : disliked
  const dest = fromLiked ? disliked : liked
  const index = source.value.findIndex((g) => g.id === game.id)
  if (index < 0) return

  movingId.value = game.id
  itemError.value = { ...itemError.value, [game.id]: '' }
  source.value = source.value.filter((g) => g.id !== game.id)
  dest.value = [...dest.value, game].sort(byName)

  try {
    await api.swipe(eventId.value, game.id, action)
  } catch (e: unknown) {
    dest.value = dest.value.filter((g) => g.id !== game.id)
    source.value = [...source.value, game].sort(byName)
    itemError.value = {
      ...itemError.value,
      [game.id]: e instanceof Error ? e.message : 'Failed to update',
    }
  } finally {
    movingId.value = null
  }
}

onMounted(async () => {
  try {
    const res = await api.overview(eventId.value)
    title.value = res.event.name
    liked.value = res.liked
    disliked.value = res.disliked
    sortLists()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load overview'
  } finally {
    pending.value = false
  }
})
</script>
