import type {
  AuthCredentials,
  AuthResponse,
  EventRef,
  EventsListResponse,
  GameCard,
  LibraryGame,
  LibraryResponse,
  OverviewResponse,
  QueueResponse,
  SwipeAction,
  SwipeResponse,
  User,
} from '../shared/api'

import eventsFixture from '../shared/fixtures/events.json'
import queue1 from '../shared/fixtures/queue-1.json'
import queue2 from '../shared/fixtures/queue-2.json'
import queue3 from '../shared/fixtures/queue-3.json'
import overview1 from '../shared/fixtures/overview-1.json'

const MOCK_USER_KEY = 'gameswiper_mock_user'
const MOCK_SWIPES_KEY = 'gameswiper_mock_swipes'

type SwipeMap = Record<string, Record<number, SwipeAction>>

const queueFixtures: Record<number, QueueResponse> = {
  1: queue1 as QueueResponse,
  2: queue2 as QueueResponse,
  3: queue3 as QueueResponse,
}

function readMockUser(): User | null {
  if (!import.meta.client) return null
  const raw = localStorage.getItem(MOCK_USER_KEY)
  if (!raw) return null
  try {
    return JSON.parse(raw) as User
  } catch {
    return null
  }
}

function writeMockUser(user: User | null) {
  if (!import.meta.client) return
  if (!user) localStorage.removeItem(MOCK_USER_KEY)
  else localStorage.setItem(MOCK_USER_KEY, JSON.stringify(user))
}

function readSwipes(): SwipeMap {
  if (!import.meta.client) return {}
  const raw = localStorage.getItem(MOCK_SWIPES_KEY)
  if (!raw) return {}
  try {
    return JSON.parse(raw) as SwipeMap
  } catch {
    return {}
  }
}

function writeSwipes(map: SwipeMap) {
  if (!import.meta.client) return
  localStorage.setItem(MOCK_SWIPES_KEY, JSON.stringify(map))
}

function shuffle<T>(items: T[]): T[] {
  const arr = [...items]
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[arr[i], arr[j]] = [arr[j], arr[i]]
  }
  return arr
}

function buildMockQueue(eventId: number): QueueResponse {
  const base = queueFixtures[eventId]
  if (!base) throw createError({ statusCode: 404, statusMessage: 'Event not found' })
  const eventSwipes = readSwipes()[String(eventId)] || {}
  const unseen = base.games.filter((g) => !eventSwipes[g.id])
  const deferred = base.games.filter((g) => eventSwipes[g.id] === 'defer')
  return {
    event: base.event,
    games: [...shuffle(unseen), ...deferred],
  }
}

function findGameAcrossQueues(gameId: number): { game: GameCard; event: EventRef } | null {
  for (const queue of Object.values(queueFixtures)) {
    const game = queue.games.find((g) => g.id === gameId)
    if (game) return { game, event: queue.event }
  }
  return null
}

function mockGameLinks(game: GameCard) {
  if (game.links?.length) return game.links
  return [
    {
      label: 'IGDB',
      url: `https://www.igdb.com/games/${game.name.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`,
    },
  ]
}

function buildMockLibrary(): LibraryResponse {
  const swipes = readSwipes()
  const likedMap = new Map<number, LibraryGame>()
  const dislikedMap = new Map<number, LibraryGame>()

  for (const [eventIdStr, eventSwipes] of Object.entries(swipes)) {
    const eventId = Number(eventIdStr)
    const queue = queueFixtures[eventId]
    if (!queue) continue

    for (const [gameIdStr, action] of Object.entries(eventSwipes)) {
      if (action !== 'like' && action !== 'dislike') continue
      const gameId = Number(gameIdStr)
      const found = queue.games.find((g) => g.id === gameId) || findGameAcrossQueues(gameId)?.game
      if (!found) continue

      const target = action === 'like' ? likedMap : dislikedMap
      const existing = target.get(gameId)
      if (existing) {
        if (!existing.events.some((e) => e.id === queue.event.id)) {
          existing.events.push(queue.event)
        }
      } else {
        target.set(gameId, {
          id: found.id,
          name: found.name,
          genres: found.genres,
          platforms: found.platforms,
          media: found.media,
          coverUrl: found.coverUrl,
          summary: found.summary,
          rating: found.rating,
          developers: found.developers,
          publishers: found.publishers,
          links: mockGameLinks(found),
          releaseDate: found.releaseDate ?? null,
          events: [queue.event],
        })
      }
    }
  }

  const byName = (a: LibraryGame, b: LibraryGame) =>
    a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })

  return {
    liked: [...likedMap.values()].sort(byName),
    disliked: [...dislikedMap.values()].sort(byName),
  }
}

async function mockApi() {
  return {
    async register(creds: AuthCredentials): Promise<AuthResponse> {
      const user = { id: 1, username: creds.username.trim() || 'demo' }
      writeMockUser(user)
      return { user }
    },
    async login(creds: AuthCredentials): Promise<AuthResponse> {
      const user = { id: 1, username: creds.username.trim() || 'demo' }
      writeMockUser(user)
      return { user }
    },
    async logout(): Promise<void> {
      writeMockUser(null)
    },
    async me(): Promise<AuthResponse> {
      const user = readMockUser()
      if (!user) throw createError({ statusCode: 401, statusMessage: 'Unauthorized' })
      return { user }
    },
    async listEvents(): Promise<EventsListResponse> {
      const swipes = readSwipes()
      const events = (eventsFixture as EventsListResponse).events.map((ev) => {
        const eventSwipes = swipes[String(ev.id)] || {}
        const ratedCount = Object.values(eventSwipes).filter(
          (a) => a === 'like' || a === 'dislike',
        ).length
        return { ...ev, ratedCount }
      })
      return { events }
    },
    async eventQueue(eventId: number): Promise<QueueResponse> {
      return buildMockQueue(eventId)
    },
    async swipe(eventId: number, gameId: number, action: SwipeAction): Promise<SwipeResponse> {
      const map = readSwipes()
      const key = String(eventId)
      map[key] = { ...(map[key] || {}), [gameId]: action }
      writeSwipes(map)
      const queue = buildMockQueue(eventId)
      return { ok: true, remaining: queue.games.length }
    },
    async overview(eventId: number): Promise<OverviewResponse> {
      const base = queueFixtures[eventId]
      if (!base) throw createError({ statusCode: 404, statusMessage: 'Event not found' })
      const eventSwipes = readSwipes()[String(eventId)] || {}
      const toOverview = (id: number) => {
        const g = base.games.find((x) => x.id === id)
        if (!g) return null
        return {
          id: g.id,
          name: g.name,
          coverUrl: g.coverUrl,
          platforms: g.platforms,
          links: mockGameLinks(g),
        }
      }
      const byName = (a: { name: string }, b: { name: string }) =>
        a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })
      if (eventId === 1 && Object.keys(eventSwipes).length === 0) {
        const fixture = overview1 as OverviewResponse
        return {
          ...fixture,
          liked: [...fixture.liked].sort(byName),
          disliked: [...fixture.disliked].sort(byName),
        }
      }
      const liked = Object.entries(eventSwipes)
        .filter(([, a]) => a === 'like')
        .map(([id]) => toOverview(Number(id)))
        .filter(Boolean) as OverviewResponse['liked']
      const disliked = Object.entries(eventSwipes)
        .filter(([, a]) => a === 'dislike')
        .map(([id]) => toOverview(Number(id)))
        .filter(Boolean) as OverviewResponse['disliked']
      liked.sort(byName)
      disliked.sort(byName)
      return { event: base.event, liked, disliked }
    },
    async library(): Promise<LibraryResponse> {
      return buildMockLibrary()
    },
  }
}

async function liveFetch<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    Accept: 'application/json',
    ...(options.body ? { 'Content-Type': 'application/json' } : {}),
    ...(options.headers as Record<string, string> | undefined),
  }

  // useRequestFetch forwards cookies during SSR; browser uses same-origin /api proxy
  const doFetch = import.meta.server ? useRequestFetch() : $fetch

  try {
    return (await doFetch(`/api${path}`, {
      method: (options.method as 'GET' | 'POST' | undefined) || 'GET',
      body: options.body ? JSON.parse(options.body as string) : undefined,
      headers,
      credentials: 'include',
    })) as T
  } catch (e: unknown) {
    const err = e as { statusCode?: number; statusMessage?: string; data?: { error?: string } }
    throw createError({
      statusCode: err.statusCode || 500,
      statusMessage: err.data?.error || err.statusMessage || 'Request failed',
    })
  }
}

export function useApi() {
  const config = useRuntimeConfig()
  const useMock = Boolean(config.public.useMock)

  return {
    useMock,
    async register(creds: AuthCredentials) {
      if (useMock) return (await mockApi()).register(creds)
      return liveFetch<AuthResponse>('/auth/register', {
        method: 'POST',
        body: JSON.stringify(creds),
      })
    },
    async login(creds: AuthCredentials) {
      if (useMock) return (await mockApi()).login(creds)
      return liveFetch<AuthResponse>('/auth/login', {
        method: 'POST',
        body: JSON.stringify(creds),
      })
    },
    async logout() {
      if (useMock) return (await mockApi()).logout()
      return liveFetch<void>('/auth/logout', { method: 'POST' })
    },
    async me() {
      if (useMock) return (await mockApi()).me()
      return liveFetch<AuthResponse>('/auth/me')
    },
    async listEvents() {
      if (useMock) return (await mockApi()).listEvents()
      return liveFetch<EventsListResponse>('/events')
    },
    async eventQueue(eventId: number) {
      if (useMock) return (await mockApi()).eventQueue(eventId)
      return liveFetch<QueueResponse>(`/events/${eventId}/queue`)
    },
    async swipe(eventId: number, gameId: number, action: SwipeAction) {
      if (useMock) return (await mockApi()).swipe(eventId, gameId, action)
      return liveFetch<SwipeResponse>(`/events/${eventId}/swipes`, {
        method: 'POST',
        body: JSON.stringify({ gameId, action }),
      })
    },
    async overview(eventId: number) {
      if (useMock) return (await mockApi()).overview(eventId)
      return liveFetch<OverviewResponse>(`/events/${eventId}/overview`)
    },
    async library() {
      if (useMock) return (await mockApi()).library()
      return liveFetch<LibraryResponse>('/library')
    },
  }
}
