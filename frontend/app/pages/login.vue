<template>
  <AuthFrame
    v-model:username="username"
    v-model:password="password"
    tagline="INSERT COIN"
    heading="WELCOME BACK"
    intro="Sign in to continue swiping games from finished events."
    submit-label="▶ SIGN IN"
    busy-label="SIGNING IN…"
    password-autocomplete="current-password"
    :error="error"
    :loading="loading"
    @submit="onSubmit"
  >
    <p class="mt-6 text-arcade-muted">
      No account?
      <NuxtLink to="/register" class="arcade-link">Register</NuxtLink>
    </p>
    <p v-if="useMock" class="mt-3 text-arcade-dim">Mock mode: any username/password works.</p>
  </AuthFrame>
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
