// Throwaway: see app/pages/prototypes/README.md.
// Prototype screens are public in auth.global.ts; this sends signed-out visitors of a style's
// non-public screens to that style's own login instead of the real /login.
import { parsePrototypePath, prototypePath, prototypeScreens, prototypeStyles } from '~/prototypes/styles'

export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.server) return

  const parsed = parsePrototypePath(to.path)
  if (!parsed) return // the /prototypes index
  if (!prototypeStyles.some((s) => s.id === parsed.style)) return // unknown style: let it 404
  if (prototypeScreens.some((s) => s.public && s.path === parsed.screen)) return

  const { user, ready, fetchMe } = useAuth()
  if (!ready.value) await fetchMe()
  if (!user.value) return navigateTo(prototypePath(parsed.style, '/login'))
})
