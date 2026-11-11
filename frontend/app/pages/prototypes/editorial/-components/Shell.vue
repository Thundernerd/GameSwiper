<!--
  Editorial page chrome: paper background, masthead header, and the style's type and colour
  tokens. Colour is reserved for like (green), dislike (red) and defer (amber); everything else
  is ink on paper. All text colours pass WCAG AA (4.5:1) on the paper background.
-->
<template>
  <div class="ed-root min-h-screen flex flex-col bg-[#f7f4ee] text-stone-900 antialiased">
    <header class="sticky top-0 z-40 border-b border-stone-900 bg-[#f7f4ee]/95 backdrop-blur">
      <div class="mx-auto max-w-6xl px-5 sm:px-8 h-16 flex items-center justify-between gap-4">
        <NuxtLink
          :to="link('/')"
          class="ed-serif text-2xl font-semibold tracking-tight italic hover:opacity-70 transition"
        >
          GameSwiper
        </NuxtLink>
        <!-- The mock user lives in localStorage, so the signed-in nav is client-only. -->
        <ClientOnly>
          <nav v-if="user" class="flex items-center gap-4 sm:gap-6 text-sm min-w-0">
            <NuxtLink
              :to="link('/library')"
              class="ed-caps hover:underline underline-offset-4"
              active-class="underline"
            >
              Library
            </NuxtLink>
            <span class="inline-block align-bottom text-stone-600 truncate max-w-[5.5rem] sm:max-w-[12rem]" :title="user.username">
              {{ user.username }}
            </span>
            <button
              type="button"
              class="ed-caps border-b border-stone-900 hover:opacity-70 transition"
              @click="onLogout"
            >
              Log out
            </button>
          </nav>
        </ClientOnly>
      </div>
    </header>
    <!-- Bottom padding keeps the last row clear of the "Back to all styles" pill. -->
    <div class="flex-1 pb-16">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
const link = usePrototypeLink()
const { user, logout } = useAuth()

useHead({
  link: [
    { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
    { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
    {
      rel: 'stylesheet',
      href: 'https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,400..700;1,9..144,400..700&family=Inter:wght@400;500;600&display=swap',
    },
  ],
})

async function onLogout() {
  await logout()
  await navigateTo(link('/login'))
}
</script>

<style>
/* Not scoped: the classes are prefixed and shared by every editorial screen. */
.ed-root {
  font-family: 'Inter', ui-sans-serif, system-ui, sans-serif;
}
.ed-serif {
  font-family: 'Fraunces', Georgia, 'Times New Roman', serif;
  font-optical-sizing: auto;
}
.ed-caps {
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}
</style>
