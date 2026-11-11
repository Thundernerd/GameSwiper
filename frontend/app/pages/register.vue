<template>
  <div class="mx-auto max-w-md px-4 py-16">
    <p class="arcade-display arcade-glow text-center text-2xl sm:text-3xl text-arcade-magenta mb-4">GAME<br class="sm:hidden" />SWIPER</p>
    <p class="arcade-display arcade-blink text-center text-[10px] text-arcade-yellow mb-10">PLAYER 1 · JOIN IN</p>
    <div class="arcade-panel p-5 sm:p-6">
      <h1 class="arcade-display text-base sm:text-lg text-arcade-cyan mb-3">NEW PLAYER</h1>
      <p class="text-arcade-muted mb-6">Save your likes and pick up where you left off.</p>
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
            autocomplete="new-password"
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
          {{ loading ? 'CREATING…' : '▶ REGISTER' }}
        </button>
      </form>
    </div>
    <p class="mt-6 text-arcade-muted">
      Already have an account?
      <NuxtLink to="/login" class="text-arcade-cyan underline underline-offset-4 hover:text-arcade-yellow">Sign in</NuxtLink>
    </p>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ middleware: [] })

const { register, user } = useAuth()
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
    await register(username.value, password.value)
    await navigateTo('/')
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Registration failed'
  } finally {
    loading.value = false
  }
}
</script>
