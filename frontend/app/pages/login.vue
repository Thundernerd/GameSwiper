<template>
  <div class="mx-auto max-w-md px-4 py-16">
    <h1 class="text-3xl font-bold mb-2">Welcome back</h1>
    <p class="text-slate-400 mb-8">Sign in to continue swiping games from finished events.</p>
    <form class="space-y-4" @submit.prevent="onSubmit">
      <label class="block">
        <span class="text-sm text-slate-300">Username</span>
        <input
          v-model="username"
          required
          class="mt-1 w-full rounded-xl border border-slate-700 bg-slate-900 px-3 py-2 outline-none focus:border-violet-500"
          autocomplete="username"
        />
      </label>
      <label class="block">
        <span class="text-sm text-slate-300">Password</span>
        <input
          v-model="password"
          type="password"
          required
          minlength="6"
          class="mt-1 w-full rounded-xl border border-slate-700 bg-slate-900 px-3 py-2 outline-none focus:border-violet-500"
          autocomplete="current-password"
        />
      </label>
      <p v-if="error" class="text-sm text-rose-400">{{ error }}</p>
      <button
        type="submit"
        :disabled="loading"
        class="w-full rounded-xl bg-violet-600 hover:bg-violet-500 disabled:opacity-60 py-2.5 font-medium transition"
      >
        {{ loading ? 'Signing in…' : 'Sign in' }}
      </button>
    </form>
    <p class="mt-6 text-sm text-slate-400">
      No account?
      <NuxtLink to="/register" class="text-violet-300 hover:underline">Register</NuxtLink>
    </p>
    <p v-if="useMock" class="mt-4 text-xs text-slate-500">Mock mode: any username/password works.</p>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ middleware: [] })

const { login, user } = useAuth()
const api = useApi()
const useMock = api.useMock
const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

onMounted(async () => {
  if (user.value) await navigateTo('/')
})

async function onSubmit() {
  error.value = ''
  loading.value = true
  try {
    await login(username.value, password.value)
    await navigateTo('/')
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Login failed'
  } finally {
    loading.value = false
  }
}
</script>
