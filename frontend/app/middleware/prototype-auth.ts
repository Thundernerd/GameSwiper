// Throwaway: see app/pages/prototypes/README.md.
// Prototype screens are public in auth.global.ts; this sends signed-out visitors of a style's
// non-public screens to that style's own login instead of the real /login.
import { publicPrototypeScreens } from '~/prototypes/styles'

export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.server) return

  const [, , style, ...rest] = to.path.split('/')
  if (!style) return // the /prototypes index
  const screen = `/${rest.join('/')}`.replace(/\/$/, '') || '/'
  if (publicPrototypeScreens.includes(screen)) return

  const { user, ready, fetchMe } = useAuth()
  if (!ready.value) await fetchMe()
  if (!user.value) return navigateTo(usePrototypeLink(style)('/login'))
})
