<template>
  <Shell>
    <div class="mx-auto max-w-6xl px-4 py-8">
      <p class="retro-display text-[10px] text-[color:var(--r-cyan)] mb-3">— HIGH SCORES —</p>
      <h1 class="retro-display retro-glow text-xl sm:text-3xl text-[color:var(--r-magenta)]">LIBRARY</h1>
      <p class="text-[color:var(--r-muted)] mt-3 mb-6">
        All games you liked or disliked across events.
      </p>

      <p v-if="pending" class="retro-display retro-blink text-xs text-[color:var(--r-cyan)]">LOADING…</p>
      <div v-else-if="error" class="retro-panel !border-[color:var(--r-red)] p-4">
        <p class="retro-display text-xs text-[color:var(--r-red)] mb-2">ERROR</p>
        <p class="text-[color:var(--r-text)]">{{ error }}</p>
      </div>

      <template v-else>
        <div class="flex flex-wrap items-end justify-between gap-4 mb-6 border-b-2 border-[color:var(--r-line)]">
          <div class="flex gap-2">
            <button
              v-for="tab in tabs"
              :key="tab.key"
              type="button"
              class="retro-display px-3 py-3 text-[10px] sm:text-xs border-b-4 -mb-[2px]"
              :class="
                activeTab === tab.key
                  ? tab.activeClass
                  : 'border-transparent text-[color:var(--r-muted)] hover:text-[color:var(--r-text)]'
              "
              @click="activeTab = tab.key"
            >
              {{ tab.label }} <span class="tabular-nums">({{ tab.count }})</span>
            </button>
          </div>

          <label class="flex items-center gap-2 text-[color:var(--r-muted)] pb-2">
            <span class="retro-display shrink-0 text-[10px]">SORT</span>
            <select
              v-model="sortBy"
              class="bg-black border-2 border-[color:var(--r-line)] text-[color:var(--r-text)] px-2 py-1 focus:outline-none focus:border-[color:var(--r-cyan)]"
            >
              <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </label>
        </div>

        <div v-if="!activeGames.length" class="retro-panel p-6 text-center">
          <p class="retro-display text-xs text-[color:var(--r-yellow)] mb-3">NO SCORES YET</p>
          <p class="text-[color:var(--r-muted)]">
            {{ emptyMessage }}
            <NuxtLink :to="link('/')" class="text-[color:var(--r-cyan)] underline underline-offset-4 hover:text-[color:var(--r-yellow)] ml-1">Browse events</NuxtLink>
          </p>
        </div>

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
    label: 'LIKED',
    count: liked.value.length,
    activeClass: 'border-[color:var(--r-green)] text-[color:var(--r-green)]',
  },
  {
    key: 'disliked' as const,
    label: 'DISLIKED',
    count: disliked.value.length,
    activeClass: 'border-[color:var(--r-red)] text-[color:var(--r-red)]',
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
