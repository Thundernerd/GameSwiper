// Throwaway: the visual-style prototypes under /prototypes. See app/pages/prototypes/README.md.

export const prototypesBase = '/prototypes'

export interface PrototypeStyle {
  /** Folder name under app/pages/prototypes/ and URL segment: /prototypes/<id>/ */
  id: string
  name: string
  /** One sentence shown on the /prototypes index. */
  description: string
}

/** Add one entry here per style folder. */
export const prototypeStyles: PrototypeStyle[] = [
  {
    id: 'baseline',
    name: 'Baseline',
    description: 'The current look, copied as-is, to compare the other styles against.',
  },
  {
    id: 'glassy',
    name: 'Glassy',
    description:
      'Frosted-glass cards and panels over the active game\'s blurred cover art, with bold gradient accents, soft shadows and motion.',
  },
  {
    id: 'retro',
    name: 'Retro arcade',
    description: 'Arcade cabinet on a CRT: neon on near-black, pixel type, hard edges and scanlines.',
  },
  {
    id: 'editorial',
    name: 'Editorial',
    description:
      'Light magazine layout: serif headlines, roomy cover art, and colour only for like, dislike and defer.',
  },
]

/**
 * The six screens every style copies, as real-app paths. `public` screens are viewable
 * without a (mock) user.
 */
export const prototypeScreens = [
  { label: 'Events', path: '/', public: false },
  { label: 'Swipe', path: '/events/1', public: false },
  { label: 'Overview', path: '/events/1/overview', public: false },
  { label: 'Library', path: '/library', public: false },
  { label: 'Login', path: '/login', public: true },
  { label: 'Register', path: '/register', public: true },
] as const

/** True for /prototypes and everything under it. */
export function isPrototypePath(path: string) {
  return path === prototypesBase || path.startsWith(`${prototypesBase}/`)
}

/** Maps a real-app path ('/', '/events/1', …) to the same screen in a style prototype. */
export function prototypePath(style: string, path: string) {
  const base = `${prototypesBase}/${style}`
  return path === '/' ? base : `${base}${path}`
}

/** Splits '/prototypes/<style>/<rest>' into the style id and the real-app path of the screen. */
export function parsePrototypePath(path: string): { style: string; screen: string } | null {
  if (!isPrototypePath(path)) return null
  const [style = '', ...rest] = path.slice(prototypesBase.length + 1).split('/')
  if (!style) return null
  return { style, screen: `/${rest.join('/')}`.replace(/(.)\/$/, '$1') }
}
