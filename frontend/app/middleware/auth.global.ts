export default defineNuxtRouteMiddleware(async (to) => {
  // Auth check runs in the browser so session cookies work via the /api proxy.
  if (import.meta.server) return

  const publicPaths = ['/login', '/register']
  if (publicPaths.includes(to.path)) return

  const { user, ready, fetchMe } = useAuth()
  if (!ready.value) await fetchMe()
  if (!user.value) return navigateTo('/login')
})
