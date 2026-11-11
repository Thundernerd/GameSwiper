<template>
  <div class="mx-auto max-w-md px-4 py-16">
    <p class="arcade-display arcade-glow text-center text-2xl sm:text-3xl text-arcade-magenta mb-4">GAME<br class="sm:hidden" />SWIPER</p>
    <p class="arcade-display arcade-blink text-center text-[10px] text-arcade-yellow mb-10">INSERT COIN</p>
    <div class="arcade-panel p-5 sm:p-6">
      <h1 class="arcade-display text-base sm:text-lg text-arcade-cyan mb-3">WELCOME BACK</h1>
      <p class="text-arcade-muted mb-6">Sign in to continue swiping games from finished events.</p>
      <form class="space-y-4" @submit.prevent="onSubmit">
        <label class="block">
          <span class="arcade-display text-[10px] text-arcade-text">USERNAME</span>
          <input
            v-model="username"
            required
            class="arcade-input mt-2"
            autocomplete="username"
          />
        </label>
        <label class="block">
          <span class="arcade-display text-[10px] text-arcade-text">PASSWORD</span>
          <input
            v-model="password"
            type="password"
            required
            minlength="6"
            class="arcade-input mt-2"
            autocomplete="current-password"
          />
        </label>
        <p v-if="error" class="border-2 border-arcade-red px-3 py-2 text-arcade-red">
          <span class="arcade-display text-[10px]">ERROR:</span> {{ error }}
        </p>
        <button
          type="submit"
          :disabled="loading"
          class="arcade-btn arcade-btn-primary w-full py-3 text-xs"
        >
          {{ loading ? 'SIGNING IN…' : '▶ SIGN IN' }}
        </button>
      </form>
    </div>
    <p class="mt-6 text-arcade-muted">
      No account?
      <NuxtLink to="/register" class="text-arcade-cyan underline underline-offset-4 hover:text-arcade-yellow">Register</NuxtLink>
    </p>
    <p v-if="useMock" class="mt-3 text-arcade-dim">Mock mode: any username/password works.</p>
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
