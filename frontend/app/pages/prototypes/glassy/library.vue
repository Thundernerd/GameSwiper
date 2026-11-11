<template>
  <Shell>
    <div class="mx-auto max-w-6xl px-4 py-8">
      <h1 class="text-4xl font-black tracking-tight">
        <span class="bg-gradient-to-r from-fuchsia-300 via-violet-200 to-cyan-200 bg-clip-text text-transparent">Library</span>
      </h1>
      <p class="text-slate-200 mt-2 mb-6">
        All games you liked or disliked across events.
      </p>

      <StatusPanel v-if="pending" kind="loading">Loading…</StatusPanel>
      <StatusPanel v-else-if="error" kind="error">{{ error }}</StatusPanel>

      <template v-else>
        <div class="flex flex-wrap items-center justify-between gap-3 mb-6">
          <div class="flex gap-1 rounded-full border border-white/15 bg-slate-950/45 p-1 backdrop-blur-xl" role="tablist">
            <button
              v-for="tab in tabs"
              :key="tab.key"
              type="button"
              class="rounded-full px-4 py-1.5 text-sm font-semibold transition"
              role="tab"
              :aria-selected="activeTab === tab.key"
              :class="
                activeTab === tab.key
                  ? tab.activeClass
                  : 'text-slate-200 hover:bg-white/10 hover:text-white'
              "
              @click="activeTab = tab.key"
            >
              {{ tab.label }} ({{ tab.count }})
            </button>
          </div>

          <label class="flex items-center gap-2 text-sm text-slate-200">
            <span class="shrink-0">Sort</span>
            <select
              v-model="sortBy"
              class="rounded-full border border-white/15 bg-slate-950/45 backdrop-blur-xl text-slate-50 px-3.5 py-1.5 text-sm transition focus:outline-none focus:border-fuchsia-400/70 focus:ring-4 focus:ring-fuchsia-500/20 [&>option]:bg-slate-900"
            >
              <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </label>
        </div>

        <StatusPanel v-if="!activeGames.length" kind="empty">
          {{ emptyMessage }}
          <NuxtLink :to="link('/')" class="font-semibold text-fuchsia-300 hover:text-fuchsia-200 hover:underline ml-1">Browse events</NuxtLink>
        </StatusPanel>

        <div v-else class="grid gap-6 md:grid-cols-2">
          <LibraryGameCard
            v-for="game in activeGames"
            :key="`${activeTab}-${game.id}`"
            :game="game"
          />
        </div>
      </template>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import type { LibraryGame } from '~/shared/api'
import LibraryGameCard from './-components/LibraryGameCard.vue'
import Shell from './-components/Shell.vue'
import StatusPanel from './-components/StatusPanel.vue'

type SortKey = 'name-asc' | 'name-desc' | 'release-desc' | 'release-asc'

const link = usePrototypeLink()
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
    activeClass: 'bg-gradient-to-r from-emerald-300 to-cyan-300 text-slate-950 shadow-lg shadow-emerald-500/30',
  },
  {
    key: 'disliked' as const,
    label: 'Disliked',
    count: disliked.value.length,
    activeClass: 'bg-gradient-to-r from-rose-300 to-orange-300 text-slate-950 shadow-lg shadow-rose-500/30',
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
