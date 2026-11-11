import { isPrototypePath } from '~/prototypes/styles'

export default defineNuxtRouteMiddleware(async (to) => {
  const publicPaths = ['/login', '/register']
  if (publicPaths.includes(to.path)) return
  // Throwaway style prototypes handle their own auth (middleware/prototype-auth.ts).
  if (isPrototypePath(to.path)) return

  const { user, ready, fetchMe } = useAuth()
  if (!ready.value) await fetchMe()
  if (!user.value) return navigateTo('/login')
})
