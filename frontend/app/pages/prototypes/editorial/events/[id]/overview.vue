<template>
  <Shell>
    <div class="mx-auto max-w-5xl px-5 sm:px-8 py-10 sm:py-14">
      <NuxtLink :to="link(`/events/${eventId}`)" class="ed-caps text-stone-600 hover:text-stone-900">
        ← Back to deck
      </NuxtLink>
      <h1 class="ed-serif mt-4 text-4xl sm:text-6xl font-medium tracking-tight leading-[1.05]">{{ title }}</h1>
      <p class="mt-4 mb-10 sm:mb-14 text-lg text-stone-600">Your likes and dislikes for this event.</p>

      <p v-if="pending" class="ed-serif italic text-xl text-stone-600 border-t border-stone-300 pt-6">Loading…</p>
      <p v-else-if="error" class="border-t border-rose-700 pt-6 text-rose-700">{{ error }}</p>

      <div v-else class="grid gap-12 md:gap-14 md:grid-cols-2">
        <section v-for="column in columns" :key="column.key" class="min-w-0">
          <h2
            :class="[
              'flex items-baseline justify-between border-b-2 pb-3 mb-1',
              column.headingClass,
            ]"
          >
            <span class="ed-serif text-3xl font-medium text-stone-900">{{ column.title }}</span>
            <span class="ed-caps tabular-nums">{{ column.games.length }}</span>
          </h2>
          <p v-if="!column.games.length" class="ed-serif italic text-stone-600 py-6">{{ column.empty }}</p>
          <ul class="divide-y divide-stone-300">
            <li
              v-for="game in column.games"
              :key="game.id"
              class="flex gap-4 py-4"
            >
              <img
                v-if="game.coverUrl"
                :src="game.coverUrl"
                :alt="game.name"
                class="h-24 w-[4.5rem] shrink-0 object-cover bg-stone-200"
              />
              <div class="min-w-0 flex-1">
                <p class="ed-serif text-xl font-medium leading-snug truncate">{{ game.name }}</p>
                <p class="text-sm text-stone-600 mt-1 truncate">
                  {{ game.platforms.join(' · ') || '—' }}
                </p>
                <div class="flex flex-wrap gap-x-4 gap-y-1 mt-2">
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
                <p v-if="itemError[game.id]" class="text-sm text-rose-700 mt-2">
                  {{ itemError[game.id] }}
                </p>
              </div>
              <button
                type="button"
                class="shrink-0 self-center h-10 w-10 rounded-full border bg-white text-base transition disabled:opacity-50"
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
  </Shell>
</template>

<script setup lang="ts">
import type { GameOverview, SwipeAction } from '~/shared/api'
import Shell from '../../-components/Shell.vue'

const route = useRoute()
const link = usePrototypeLink()
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
    headingClass: 'border-emerald-700 text-emerald-700',
    moveAction: 'dislike' as SwipeAction,
    moveLabel: '✕',
    moveTitle: 'Move to disliked',
    moveButtonClass: 'border-rose-700 text-rose-700 hover:bg-rose-700 hover:text-white',
  },
  {
    key: 'disliked',
    title: 'Disliked',
    games: disliked.value,
    empty: 'No dislikes yet.',
    headingClass: 'border-rose-700 text-rose-700',
    moveAction: 'like' as SwipeAction,
    moveLabel: '♥',
    moveTitle: 'Move to liked',
    moveButtonClass: 'border-emerald-700 text-emerald-700 hover:bg-emerald-700 hover:text-white',
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
