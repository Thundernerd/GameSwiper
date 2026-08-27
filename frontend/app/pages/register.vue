<template>
  <div class="mx-auto max-w-md px-4 py-16">
    <h1 class="text-3xl font-bold mb-2">Create account</h1>
    <p class="text-slate-400 mb-8">Save your likes and pick up where you left off.</p>
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
          autocomplete="new-password"
        />
      </label>
      <p v-if="error" class="text-sm text-rose-400">{{ error }}</p>
      <button
        type="submit"
        :disabled="loading"
        class="w-full rounded-xl bg-violet-600 hover:bg-violet-500 disabled:opacity-60 py-2.5 font-medium transition"
      >
        {{ loading ? 'Creating…' : 'Register' }}
      </button>
    </form>
    <p class="mt-6 text-sm text-slate-400">
      Already have an account?
      <NuxtLink to="/login" class="text-violet-300 hover:underline">Sign in</NuxtLink>
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
