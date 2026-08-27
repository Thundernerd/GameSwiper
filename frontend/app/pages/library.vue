<template>
  <div class="mx-auto max-w-6xl px-4 py-8">
    <h1 class="text-3xl font-bold tracking-tight">Library</h1>
    <p class="text-slate-400 mt-1 mb-6">
      All games you liked or disliked across events.
    </p>

    <p v-if="pending" class="text-slate-400">Loading…</p>
    <p v-else-if="error" class="text-rose-400">{{ error }}</p>

    <template v-else>
      <div class="flex flex-wrap items-end justify-between gap-4 mb-6 border-b border-slate-800">
        <div class="flex gap-2">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            type="button"
            class="px-4 py-2 text-sm font-medium border-b-2 -mb-px transition"
            :class="
              activeTab === tab.key
                ? tab.activeClass
                : 'border-transparent text-slate-400 hover:text-slate-200'
            "
            @click="activeTab = tab.key"
          >
            {{ tab.label }} ({{ tab.count }})
          </button>
        </div>

        <label class="flex items-center gap-2 text-sm text-slate-400 pb-2">
          <span class="shrink-0">Sort</span>
          <select
            v-model="sortBy"
            class="rounded-lg border border-slate-700 bg-slate-900 text-slate-200 px-2.5 py-1.5 text-sm focus:outline-none focus:border-violet-500"
          >
            <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </label>
      </div>

      <p v-if="!activeGames.length" class="text-slate-500 text-sm">
        {{ emptyMessage }}
        <NuxtLink to="/" class="text-violet-300 hover:underline ml-1">Browse events</NuxtLink>
      </p>

      <div v-else class="grid gap-6 md:grid-cols-2">
        <LibraryGameCard
          v-for="game in activeGames"
          :key="`${activeTab}-${game.id}`"
          :game="game"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import type { LibraryGame } from '../shared/api'

type SortKey = 'name-asc' | 'name-desc' | 'release-desc' | 'release-asc'

const api = useApi()
const liked = ref<LibraryGame[]>([])
const disliked = ref<LibraryGame[]>([])
const pending = ref(true)
const error = ref('')
const activeTab = ref<'liked' | 'disliked'>('liked')
const sortBy = ref<SortKey>('name-asc')

const sortOptions: { value: SortKey; label: string }[] = [
  { value: 'name-asc', label: 'Name A–Z' },
  { value: 'name-desc', label: 'Name Z–A' },
  { value: 'release-desc', label: 'Release date (newest)' },
  { value: 'release-asc', label: 'Release date (oldest)' },
]

const tabs = computed(() => [
  {
    key: 'liked' as const,
    label: 'Liked',
    count: liked.value.length,
    activeClass: 'border-emerald-400 text-emerald-300',
  },
  {
    key: 'disliked' as const,
    label: 'Disliked',
    count: disliked.value.length,
    activeClass: 'border-rose-400 text-rose-300',
  },
])

function releaseTime(game: LibraryGame) {
  if (!game.releaseDate) return null
  const t = Date.parse(game.releaseDate)
  return Number.isNaN(t) ? null : t
}

function sortGames(games: LibraryGame[]) {
  const list = [...games]
  const byName = (a: LibraryGame, b: LibraryGame) =>
    a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })

  switch (sortBy.value) {
    case 'name-asc':
      return list.sort(byName)
    case 'name-desc':
      return list.sort((a, b) => byName(b, a))
    case 'release-desc':
    case 'release-asc': {
      const asc = sortBy.value === 'release-asc'
      return list.sort((a, b) => {
        const ta = releaseTime(a)
        const tb = releaseTime(b)
        if (ta == null && tb == null) return byName(a, b)
        if (ta == null) return 1
        if (tb == null) return -1
        const diff = asc ? ta - tb : tb - ta
        return diff || byName(a, b)
      })
    }
    default:
      return list
  }
}

const activeGames = computed(() => {
  const source = activeTab.value === 'liked' ? liked.value : disliked.value
  return sortGames(source)
})

const emptyMessage = computed(() =>
  activeTab.value === 'liked' ? 'No likes yet.' : 'No dislikes yet.',
)

onMounted(async () => {
  try {
    const res = await api.library()
    liked.value = res.liked
    disliked.value = res.disliked
    if (!liked.value.length && disliked.value.length) {
      activeTab.value = 'disliked'
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load library'
  } finally {
    pending.value = false
  }
})
</script>
