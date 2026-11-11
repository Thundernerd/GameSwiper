<template>
  <div class="min-h-screen flex flex-col">
    <header
      v-if="user"
      class="sticky top-0 z-40 border-b-4 border-arcade-magenta bg-arcade-bg"
    >
      <div class="mx-auto max-w-6xl px-4 py-3 flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <div class="flex items-center gap-4 min-w-0">
          <NuxtLink
            to="/"
            class="arcade-display arcade-glow text-sm sm:text-base text-arcade-magenta hover:text-arcade-yellow"
          >
            HYPE CHECK
          </NuxtLink>
          <NuxtLink
            to="/library"
            class="arcade-display text-[10px] sm:text-xs text-arcade-cyan hover:text-arcade-yellow"
            active-class="!text-arcade-yellow underline underline-offset-4"
          >
            LIBRARY
          </NuxtLink>
        </div>
        <div class="flex items-center gap-3 min-w-0">
          <span class="arcade-display text-[10px] sm:text-xs text-arcade-text truncate">
            <span class="text-arcade-green">P1</span> {{ user.username }}
          </span>
          <button type="button" class="arcade-btn arcade-btn-ghost text-[10px] px-2.5 py-2" @click="onLogout">
            LOG OUT
          </button>
        </div>
      </div>
    </header>
    <main class="flex-1">
      <NuxtPage />
    </main>
    <div class="arcade-scanlines" aria-hidden="true" />
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
