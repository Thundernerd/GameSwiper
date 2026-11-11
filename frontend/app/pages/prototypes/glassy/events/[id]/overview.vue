<template>
  <Shell>
    <div class="mx-auto max-w-5xl px-4 py-8">
      <NuxtLink :to="link(`/events/${eventId}`)" class="text-sm text-slate-200 hover:text-white transition">
        ← Back to deck
      </NuxtLink>
      <h1 class="text-4xl font-black tracking-tight mt-2 mb-2">
        <span class="bg-gradient-to-r from-fuchsia-300 via-violet-200 to-cyan-200 bg-clip-text text-transparent">{{ title }}</span>
      </h1>
      <p class="text-slate-200 mb-8">Your likes and dislikes for this event.</p>

      <StatusPanel v-if="pending" kind="loading">Loading…</StatusPanel>
      <StatusPanel v-else-if="error" kind="error">{{ error }}</StatusPanel>

      <div v-else class="grid grid-cols-1 gap-8 md:grid-cols-2">
        <section
          v-for="column in columns"
          :key="column.key"
          class="min-w-0 rounded-3xl border border-white/15 bg-slate-950/40 p-4 sm:p-5 backdrop-blur-xl shadow-[0_24px_60px_-24px_rgba(0,0,0,0.8)]"
        >
          <h2 class="flex items-center gap-2 text-lg font-bold mb-4">
            <span :class="['h-2.5 w-2.5 rounded-full', column.dotClass]" aria-hidden="true" />
            <span :class="['bg-clip-text text-transparent', column.headingClass]">{{ column.title }}</span>
            <span class="rounded-full bg-white/10 px-2 py-0.5 text-xs font-semibold text-slate-100 tabular-nums">{{ column.games.length }}</span>
          </h2>
          <p v-if="!column.games.length" class="rounded-2xl border border-dashed border-white/20 px-4 py-6 text-center text-sm text-slate-200">
            {{ column.empty }}
          </p>
          <TransitionGroup tag="ul" name="reorder" class="relative space-y-3">
            <li
              v-for="game in column.games"
              :key="game.id"
              class="flex gap-3 rounded-2xl border border-white/10 bg-white/[0.06] p-3 transition hover:bg-white/10"
            >
              <img
                v-if="game.coverUrl"
                :src="game.coverUrl"
                :alt="game.name"
                class="h-20 w-14 shrink-0 object-cover rounded-xl bg-white/10 shadow-lg shadow-black/40"
              />
              <div class="min-w-0 flex-1">
                <p class="font-semibold truncate">{{ game.name }}</p>
                <p class="text-xs text-slate-300 mt-0.5 truncate">
                  {{ game.platforms.join(', ') || '—' }}
                </p>
                <div class="flex flex-wrap gap-2 mt-2">
                  <a
                    v-for="link in game.links"
                    :key="link.url"
                    :href="link.url"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="rounded-full bg-white/5 ring-1 ring-inset ring-white/15 px-2 py-0.5 text-xs font-medium text-fuchsia-200 hover:bg-white/15 hover:text-white transition"
                  >
                    {{ link.label }}
                  </a>
                </div>
                <p v-if="itemError[game.id]" role="alert" class="text-xs text-rose-200 mt-2">
                  {{ itemError[game.id] }}
                </p>
              </div>
              <button
                type="button"
                class="shrink-0 self-center grid h-10 w-10 place-items-center rounded-full text-lg font-bold text-white ring-2 ring-white/10 transition hover:scale-110 active:scale-95 disabled:opacity-50"
                :class="column.moveButtonClass"
                :title="column.moveTitle"
                :disabled="movingId === game.id"
                @click="moveGame(game, column.moveAction)"
              >
                {{ column.moveLabel }}
              </button>
            </li>
          </TransitionGroup>
        </section>
      </div>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import type { GameOverview, SwipeAction } from '~/shared/api'
import Shell from '../../-components/Shell.vue'
import StatusPanel from '../../-components/StatusPanel.vue'

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
    headingClass: 'bg-gradient-to-r from-emerald-200 to-cyan-200',
    dotClass: 'bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.9)]',
    moveAction: 'dislike' as SwipeAction,
    moveLabel: '✕',
    moveTitle: 'Move to disliked',
    moveButtonClass: 'bg-gradient-to-br from-rose-600 to-pink-700 shadow-lg shadow-rose-500/40',
  },
  {
    key: 'disliked',
    title: 'Disliked',
    games: disliked.value,
    empty: 'No dislikes yet.',
    headingClass: 'bg-gradient-to-r from-rose-200 to-orange-200',
    dotClass: 'bg-rose-400 shadow-[0_0_10px_rgba(251,113,133,0.9)]',
    moveAction: 'like' as SwipeAction,
    moveLabel: '♥',
    moveTitle: 'Move to liked',
    moveButtonClass: 'bg-gradient-to-br from-emerald-600 to-teal-700 shadow-lg shadow-emerald-500/40',
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

<style scoped>
.reorder-move,
.reorder-enter-active,
.reorder-leave-active {
  transition: all 0.35s ease;
}
.reorder-enter-from,
.reorder-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
.reorder-leave-active {
  position: absolute;
  inset-inline: 0;
}
@media (prefers-reduced-motion: reduce) {
  .reorder-move,
  .reorder-enter-active,
  .reorder-leave-active {
    transition: none;
  }
}
</style>
