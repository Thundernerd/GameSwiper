<template>
  <div class="min-h-screen flex flex-col">
    <header
      v-if="user"
      class="border-b border-slate-800 bg-slate-950/80 backdrop-blur sticky top-0 z-40"
    >
      <div class="mx-auto max-w-6xl px-4 py-3 flex items-center justify-between gap-4">
        <div class="flex items-center gap-4 min-w-0">
          <NuxtLink to="/" class="font-semibold tracking-tight text-lg text-violet-300 hover:text-violet-200">
            GameSwiper
          </NuxtLink>
          <NuxtLink
            to="/library"
            class="text-sm text-slate-300 hover:text-white transition"
            active-class="text-violet-200"
          >
            Library
          </NuxtLink>
        </div>
        <div class="flex items-center gap-3 text-sm text-slate-300">
          <span>{{ user.username }}</span>
          <button
            type="button"
            class="rounded-lg border border-slate-700 px-3 py-1.5 hover:bg-slate-800 transition"
            @click="onLogout"
          >
            Log out
          </button>
        </div>
      </div>
    </header>
    <main class="flex-1">
      <NuxtPage />
    </main>
  </div>
</template>

<script setup lang="ts">
const { user, fetchMe, logout } = useAuth()

onMounted(() => {
  fetchMe()
})

async function onLogout() {
  await logout()
  await navigateTo('/login')
}
</script>
