<template>
  <AuthFrame
    v-model:username="username"
    v-model:password="password"
    tagline="PLAYER 1 · JOIN IN"
    heading="NEW PLAYER"
    intro="Save your likes and pick up where you left off."
    submit-label="▶ REGISTER"
    busy-label="CREATING…"
    password-autocomplete="new-password"
    :error="error"
    :loading="loading"
    @submit="onSubmit"
  >
    <p class="mt-6 text-arcade-muted">
      Already have an account?
      <NuxtLink to="/login" class="text-arcade-cyan underline underline-offset-4 hover:text-arcade-yellow">Sign in</NuxtLink>
    </p>
  </AuthFrame>
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
