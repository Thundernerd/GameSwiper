<template>
  <Shell>
    <div class="mx-auto max-w-6xl px-5 sm:px-8 py-12 sm:py-16">
      <h1 class="ed-serif text-4xl sm:text-6xl font-medium tracking-tight leading-[1.05]">Library</h1>
      <p class="mt-4 mb-10 text-lg text-stone-600">
        All games you liked or disliked across events.
      </p>

      <p v-if="pending" class="ed-serif italic text-xl text-stone-600 border-t border-stone-300 pt-6">Loading…</p>
      <p v-else-if="error" class="border-t-2 border-stone-900 pt-6 ed-serif text-xl italic">{{ error }}</p>

      <template v-else>
        <div class="flex flex-wrap items-end justify-between gap-x-6 gap-y-3 mb-10 border-b border-stone-300">
          <div class="flex gap-6">
            <button
              v-for="tab in tabs"
              :key="tab.key"
              type="button"
              class="ed-serif py-2 text-2xl border-b-2 -mb-px transition"
              :class="
                activeTab === tab.key
                  ? tab.activeClass
                  : 'border-transparent text-stone-600 hover:text-stone-900'
              "
              @click="activeTab = tab.key"
            >
              {{ tab.label }}
              <span class="ed-caps align-middle tabular-nums">{{ tab.count }}</span>
            </button>
          </div>

          <label class="flex items-center gap-2 pb-3">
            <span class="shrink-0 ed-caps text-stone-600">Sort</span>
            <select
              v-model="sortBy"
              class="border-0 border-b border-stone-400 bg-transparent py-1 pl-0 pr-7 text-sm text-stone-900 focus:outline-none focus:ring-0 focus:border-stone-900"
            >
              <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </label>
        </div>

        <p v-if="!activeGames.length" class="ed-serif italic text-xl text-stone-600">
          {{ emptyMessage }}
          <NuxtLink
            :to="link('/')"
            class="ml-1 not-italic text-stone-900 underline underline-offset-4 hover:opacity-70"
          >Browse events</NuxtLink>
        </p>

        <div v-else class="grid gap-x-10 gap-y-14 md:grid-cols-2">
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
    label: 'Liked',
    count: liked.value.length,
    activeClass: 'border-emerald-700 text-emerald-700',
  },
  {
    key: 'disliked' as const,
    label: 'Disliked',
    count: disliked.value.length,
    activeClass: 'border-rose-700 text-rose-700',
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
