export default defineNuxtRouteMiddleware(async (to) => {
  // Auth check runs in the browser so session cookies work via the /api proxy.
  if (import.meta.server) return

  const publicPaths = ['/login', '/register']
  if (publicPaths.includes(to.path)) return
  // Throwaway style prototypes handle their own auth (middleware/prototype-auth.ts).
  if (to.path === '/prototypes' || to.path.startsWith('/prototypes/')) return

  const { user, ready, fetchMe } = useAuth()
  if (!ready.value) await fetchMe()
  if (!user.value) return navigateTo('/login')
})
