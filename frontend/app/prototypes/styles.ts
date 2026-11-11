// Throwaway: the visual-style prototypes under /prototypes. See app/pages/prototypes/README.md.

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
]

/** The six screens every style copies, as paths relative to /prototypes/<id>. */
export const prototypeScreens = [
  { label: 'Events', path: '/' },
  { label: 'Swipe', path: '/events/1' },
  { label: 'Overview', path: '/events/1/overview' },
  { label: 'Library', path: '/library' },
  { label: 'Login', path: '/login' },
  { label: 'Register', path: '/register' },
] as const

/** Screens that are viewable without a (mock) user. */
export const publicPrototypeScreens = ['/login', '/register']
