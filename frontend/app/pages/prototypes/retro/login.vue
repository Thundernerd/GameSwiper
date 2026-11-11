<template>
  <Shell>
    <div class="mx-auto max-w-md px-4 py-16">
      <p class="retro-display retro-glow text-center text-2xl sm:text-3xl text-[color:var(--r-magenta)] mb-4">GAME<br class="sm:hidden" />SWIPER</p>
      <p class="retro-display retro-blink text-center text-[10px] text-[color:var(--r-yellow)] mb-10">INSERT COIN</p>
      <div class="retro-panel p-5 sm:p-6">
        <h1 class="retro-display text-base sm:text-lg text-[color:var(--r-cyan)] mb-3">WELCOME BACK</h1>
        <p class="text-[color:var(--r-muted)] mb-6">Sign in to continue swiping games from finished events.</p>
        <form class="space-y-4" @submit.prevent="onSubmit">
          <label class="block">
            <span class="retro-display text-[10px] text-[color:var(--r-text)]">USERNAME</span>
            <input
              v-model="username"
              required
              class="retro-input mt-2"
              autocomplete="username"
            />
          </label>
          <label class="block">
            <span class="retro-display text-[10px] text-[color:var(--r-text)]">PASSWORD</span>
            <input
              v-model="password"
              type="password"
              required
              minlength="6"
              class="retro-input mt-2"
              autocomplete="current-password"
            />
          </label>
          <p v-if="error" class="border-2 border-[color:var(--r-red)] px-3 py-2 text-[color:var(--r-red)]">
            <span class="retro-display text-[10px]">ERROR:</span> {{ error }}
          </p>
          <button
            type="submit"
            :disabled="loading"
            class="retro-btn retro-btn-primary w-full py-3 text-xs"
          >
            {{ loading ? 'SIGNING IN…' : '▶ SIGN IN' }}
          </button>
        </form>
      </div>
      <p class="mt-6 text-[color:var(--r-muted)]">
        No account?
        <NuxtLink :to="link('/register')" class="text-[color:var(--r-cyan)] underline underline-offset-4 hover:text-[color:var(--r-yellow)]">Register</NuxtLink>
      </p>
      <p v-if="useMock" class="mt-3 text-[color:var(--r-dim)]">Mock mode: any username/password works.</p>
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
