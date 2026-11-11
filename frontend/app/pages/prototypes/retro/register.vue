<template>
  <Shell>
    <div class="mx-auto max-w-md px-4 py-16">
      <p class="retro-display retro-glow text-center text-2xl sm:text-3xl text-[color:var(--r-magenta)] mb-4">GAME<br class="sm:hidden" />SWIPER</p>
      <p class="retro-display retro-blink text-center text-[10px] text-[color:var(--r-yellow)] mb-10">PLAYER 1 · JOIN IN</p>
      <div class="retro-panel p-5 sm:p-6">
        <h1 class="retro-display text-base sm:text-lg text-[color:var(--r-cyan)] mb-3">NEW PLAYER</h1>
        <p class="text-[color:var(--r-muted)] mb-6">Save your likes and pick up where you left off.</p>
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
              autocomplete="new-password"
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
            {{ loading ? 'CREATING…' : '▶ REGISTER' }}
          </button>
        </form>
      </div>
      <p class="mt-6 text-[color:var(--r-muted)]">
        Already have an account?
        <NuxtLink :to="link('/login')" class="text-[color:var(--r-cyan)] underline underline-offset-4 hover:text-[color:var(--r-yellow)]">Sign in</NuxtLink>
      </p>
    </div>
  </Shell>
</template>

<script setup lang="ts">
import Shell from './-components/Shell.vue'

const link = usePrototypeLink()

const { register, user } = useAuth()
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
    await register(username.value, password.value)
    await navigateTo(link('/'))
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Registration failed'
  } finally {
    loading.value = false
  }
}
</script>
