<template>
  <Shell>
    <div class="mx-auto max-w-md px-5 sm:px-8 py-14 sm:py-24">
      <p class="ed-caps text-stone-600">New reader</p>
      <h1 class="ed-serif mt-3 text-5xl font-medium tracking-tight leading-[1.05]">Create account</h1>
      <p class="mt-4 mb-10 text-lg text-stone-600 leading-relaxed">Save your likes and pick up where you left off.</p>
      <form class="space-y-6" @submit.prevent="onSubmit">
        <label class="block">
          <span class="ed-caps text-stone-600">Username</span>
          <input
            v-model="username"
            required
            class="mt-2 w-full border-0 border-b border-stone-400 bg-transparent px-0 py-2 text-lg outline-none focus:border-stone-900 focus:ring-0"
            autocomplete="username"
          />
        </label>
        <label class="block">
          <span class="ed-caps text-stone-600">Password</span>
          <input
            v-model="password"
            type="password"
            required
            minlength="6"
            class="mt-2 w-full border-0 border-b border-stone-400 bg-transparent px-0 py-2 text-lg outline-none focus:border-stone-900 focus:ring-0"
            autocomplete="new-password"
          />
        </label>
        <p v-if="error" class="text-sm text-rose-700">{{ error }}</p>
        <button
          type="submit"
          :disabled="loading"
          class="w-full bg-stone-900 text-[#f7f4ee] hover:bg-stone-700 disabled:opacity-60 py-3.5 ed-caps transition"
        >
          {{ loading ? 'Creating…' : 'Register' }}
        </button>
      </form>
      <p class="mt-8 border-t border-stone-300 pt-6 text-sm text-stone-600">
        Already have an account?
        <NuxtLink :to="link('/login')" class="font-semibold text-stone-900 underline underline-offset-4 hover:opacity-70">Sign in</NuxtLink>
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
