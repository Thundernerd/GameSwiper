import type { User } from '../shared/api'

export function useAuth() {
  const user = useState<User | null>('auth-user', () => null)
  const ready = useState('auth-ready', () => false)
  const api = useApi()

  async function fetchMe() {
    try {
      const res = await api.me()
      user.value = res.user
    } catch {
      user.value = null
    } finally {
      ready.value = true
    }
  }

  async function login(username: string, password: string) {
    const res = await api.login({ username, password })
    user.value = res.user
    return res.user
  }

  async function register(username: string, password: string) {
    const res = await api.register({ username, password })
    user.value = res.user
    return res.user
  }

  async function logout() {
    await api.logout()
    user.value = null
  }

  return { user, ready, fetchMe, login, register, logout }
}
