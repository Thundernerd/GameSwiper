<template>
  <Shell>
    <div class="mx-auto max-w-md px-4 py-12 sm:py-20">
      <div class="rounded-3xl border border-white/15 bg-slate-950/50 p-6 sm:p-8 backdrop-blur-2xl shadow-[0_30px_80px_-24px_rgba(0,0,0,0.8)] ring-1 ring-inset ring-white/5">
        <h1 class="text-3xl font-black tracking-tight mb-2">Welcome back</h1>
        <p class="text-slate-200 mb-8">Sign in to continue swiping games from finished events.</p>
        <form class="space-y-4" @submit.prevent="onSubmit">
          <label class="block">
            <span class="text-sm font-medium text-slate-100">Username</span>
            <input
              v-model="username"
              required
              class="mt-1.5 w-full rounded-xl border border-white/15 bg-white/10 px-3.5 py-2.5 text-white placeholder:text-slate-400 outline-none transition focus:border-fuchsia-400/70 focus:bg-white/15 focus:ring-4 focus:ring-fuchsia-500/20"
              autocomplete="username"
            />
          </label>
          <label class="block">
            <span class="text-sm font-medium text-slate-100">Password</span>
            <input
              v-model="password"
              type="password"
              required
              minlength="6"
              class="mt-1.5 w-full rounded-xl border border-white/15 bg-white/10 px-3.5 py-2.5 text-white placeholder:text-slate-400 outline-none transition focus:border-fuchsia-400/70 focus:bg-white/15 focus:ring-4 focus:ring-fuchsia-500/20"
              autocomplete="current-password"
            />
          </label>
          <p v-if="error" role="alert" class="rounded-xl border border-rose-300/30 bg-rose-500/15 px-3 py-2 text-sm text-rose-100">{{ error }}</p>
          <button
            type="submit"
            :disabled="loading"
            class="w-full rounded-xl bg-gradient-to-r from-indigo-600 via-violet-600 to-fuchsia-700 py-3 font-bold text-white shadow-lg shadow-fuchsia-600/30 transition hover:brightness-110 hover:shadow-fuchsia-500/50 active:scale-[0.98] disabled:opacity-60"
          >
            {{ loading ? 'Signing in…' : 'Sign in' }}
          </button>
        </form>
        <p class="mt-6 text-sm text-slate-200">
          No account?
          <NuxtLink :to="link('/register')" class="font-semibold text-fuchsia-300 hover:text-fuchsia-200 hover:underline">Register</NuxtLink>
        </p>
        <p v-if="useMock" class="mt-4 text-xs text-slate-300">Mock mode: any username/password works.</p>
      </div>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import Shell from './-components/Shell.vue'

const link = usePrototypeLink()

const { login, user } = useAuth()
const api = useApi()
const useMock = api.useMock
const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

onMounted(async () => {
  if (user.value) await navigateTo(link('/'))
})

async function onSubmit() {
  error.value = ''
  loading.value = true
  try {
    await login(username.value, password.value)
    await navigateTo(link('/'))
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Login failed'
  } finally {
    loading.value = false
  }
}
</script>
