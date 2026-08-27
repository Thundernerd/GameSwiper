<template>
  <div class="mx-auto max-w-5xl px-4 py-8">
    <NuxtLink :to="`/events/${eventId}`" class="text-sm text-slate-400 hover:text-slate-200">
      ← Back to deck
    </NuxtLink>
    <h1 class="text-3xl font-bold mt-2 mb-1">{{ title }}</h1>
    <p class="text-slate-400 mb-8">Your likes and dislikes for this event.</p>

    <p v-if="pending" class="text-slate-400">Loading…</p>
    <p v-else-if="error" class="text-rose-400">{{ error }}</p>

    <div v-else class="grid gap-8 md:grid-cols-2">
      <section v-for="column in columns" :key="column.key">
        <h2 :class="['text-lg font-semibold mb-4', column.headingClass]">
          {{ column.title }} ({{ column.games.length }})
        </h2>
        <p v-if="!column.games.length" class="text-slate-500 text-sm">{{ column.empty }}</p>
        <ul class="space-y-3">
          <li
            v-for="game in column.games"
            :key="game.id"
            class="flex gap-3 rounded-xl border border-slate-800 bg-slate-900/50 p-3"
          >
            <img
              v-if="game.coverUrl"
              :src="game.coverUrl"
              :alt="game.name"
              class="h-20 w-14 object-cover rounded-lg bg-slate-800"
            />
            <div class="min-w-0 flex-1">
              <p class="font-medium truncate">{{ game.name }}</p>
              <p class="text-xs text-slate-400 mt-0.5 truncate">
                {{ game.platforms.join(', ') || '—' }}
              </p>
              <div class="flex flex-wrap gap-2 mt-2">
                <a
                  v-for="link in game.links"
                  :key="link.url"
                  :href="link.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-xs text-violet-300 hover:underline"
                >
                  {{ link.label }}
                </a>
              </div>
              <p v-if="itemError[game.id]" class="text-xs text-rose-400 mt-2">
                {{ itemError[game.id] }}
              </p>
            </div>
            <button
              type="button"
              class="shrink-0 self-center h-10 w-10 rounded-full border-2 text-lg font-bold disabled:opacity-50"
              :class="column.moveButtonClass"
              :title="column.moveTitle"
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
    title: 'Liked',
    games: liked.value,
    empty: 'No likes yet.',
    headingClass: 'text-emerald-300',
    moveAction: 'dislike' as SwipeAction,
    moveLabel: '✕',
    moveTitle: 'Move to disliked',
    moveButtonClass: 'border-rose-500 text-rose-400 hover:bg-rose-500/10',
  },
  {
    key: 'disliked',
    title: 'Disliked',
    games: disliked.value,
    empty: 'No dislikes yet.',
    headingClass: 'text-rose-300',
    moveAction: 'like' as SwipeAction,
    moveLabel: '♥',
    moveTitle: 'Move to liked',
    moveButtonClass: 'border-emerald-500 text-emerald-400 hover:bg-emerald-500/10',
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
