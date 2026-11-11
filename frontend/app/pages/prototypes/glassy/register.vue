<template>
  <Shell>
    <div class="mx-auto max-w-md px-4 py-12 sm:py-20">
      <div class="mb-6 text-center">
        <span class="text-2xl font-black tracking-tight bg-gradient-to-r from-fuchsia-300 via-violet-200 to-cyan-200 bg-clip-text text-transparent">
          GameSwiper
        </span>
      </div>
      <div class="rounded-3xl border border-white/15 bg-slate-950/50 p-6 sm:p-8 backdrop-blur-2xl shadow-[0_30px_80px_-24px_rgba(0,0,0,0.8)] ring-1 ring-inset ring-white/5">
        <h1 class="text-3xl font-black tracking-tight mb-2">Create account</h1>
        <p class="text-slate-200 mb-8">Save your likes and pick up where you left off.</p>
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
              autocomplete="new-password"
            />
          </label>
          <p v-if="error" role="alert" class="rounded-xl border border-rose-300/30 bg-rose-500/15 px-3 py-2 text-sm text-rose-100">{{ error }}</p>
          <button
            type="submit"
            :disabled="loading"
            class="w-full rounded-xl bg-gradient-to-r from-indigo-600 via-violet-600 to-fuchsia-700 py-3 font-bold text-white shadow-lg shadow-fuchsia-600/30 transition hover:brightness-110 hover:shadow-fuchsia-500/50 active:scale-[0.98] disabled:opacity-60"
          >
            {{ loading ? 'Creating…' : 'Register' }}
          </button>
        </form>
        <p class="mt-6 text-sm text-slate-200">
          Already have an account?
          <NuxtLink :to="link('/login')" class="font-semibold text-fuchsia-300 hover:text-fuchsia-200 hover:underline">Sign in</NuxtLink>
        </p>
      </div>
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
