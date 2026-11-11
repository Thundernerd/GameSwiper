<!--
  Glassy page chrome: a frosted header over a full-page backdrop. With `backdrop` (a cover-art
  URL) the backdrop is that image, heavily blurred, cross-fading when it changes; otherwise it is
  a slowly drifting gradient mesh. A dark scrim keeps text on top at AA contrast.
-->
<template>
  <div class="relative isolate min-h-screen flex flex-col text-slate-50 antialiased [overflow-x:clip]">
    <div class="fixed inset-0 -z-10 overflow-hidden bg-slate-950" aria-hidden="true">
      <div class="mesh absolute inset-0">
        <span class="blob blob-a" />
        <span class="blob blob-b" />
        <span class="blob blob-c" />
        <span class="blob blob-d" />
      </div>
      <Transition name="backdrop">
        <img
          v-if="backdrop"
          :key="backdrop"
          :src="backdrop"
          alt=""
          class="absolute inset-0 h-full w-full object-cover scale-125 blur-2xl saturate-[1.8] brightness-110"
        />
      </Transition>
      <div
        class="absolute inset-0 bg-gradient-to-b"
        :class="
          backdrop
            ? 'from-slate-950/60 via-slate-950/35 to-slate-950/75'
            : 'from-slate-950/10 via-slate-950/25 to-slate-950/60'
        "
      />
    </div>

    <header
      v-if="user"
      class="sticky top-0 z-40 border-b border-white/10 bg-slate-950/60 backdrop-blur-xl shadow-[0_8px_32px_-12px_rgba(0,0,0,0.6)]"
    >
      <div class="mx-auto max-w-6xl px-4 py-3 flex items-center justify-between gap-3">
        <div class="flex items-center gap-3 sm:gap-5 min-w-0">
          <NuxtLink
            :to="link('/')"
            class="shrink-0 font-extrabold tracking-tight text-lg bg-gradient-to-r from-fuchsia-300 via-violet-200 to-cyan-200 bg-clip-text text-transparent hover:brightness-125 transition"
          >
            GameSwiper
          </NuxtLink>
          <NuxtLink
            :to="link('/library')"
            class="rounded-full px-3 py-1 text-sm text-slate-200 hover:bg-white/10 hover:text-white transition"
            active-class="bg-white/15 text-white ring-1 ring-white/20"
          >
            Library
          </NuxtLink>
        </div>
        <div class="flex items-center gap-2 sm:gap-3 text-sm min-w-0">
          <span class="flex items-center gap-2 min-w-0">
            <span
              class="grid h-7 w-7 shrink-0 place-items-center rounded-full bg-gradient-to-br from-fuchsia-500 to-cyan-500 text-xs font-bold uppercase text-white shadow-lg shadow-fuchsia-500/30"
              aria-hidden="true"
            >
              {{ user.username.slice(0, 1) }}
            </span>
            <span class="truncate max-w-[7rem] sm:max-w-[12rem] text-slate-100">{{ user.username }}</span>
          </span>
          <button
            type="button"
            class="shrink-0 rounded-full border border-white/20 bg-white/10 px-3 py-1.5 text-slate-100 hover:bg-white/20 transition"
            @click="onLogout"
          >
            Log out
          </button>
        </div>
      </div>
    </header>
    <div class="flex-1">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  /** Image to blur behind the page (the active card's cover art). */
  backdrop?: string | null
}>()

const link = usePrototypeLink()
const { user, logout } = useAuth()

async function onLogout() {
  await logout()
  await navigateTo(link('/login'))
}
</script>

<style scoped>
.blob {
  position: absolute;
  border-radius: 9999px;
  filter: blur(70px);
  opacity: 0.85;
  animation: drift 24s ease-in-out infinite alternate;
}
.blob-a {
  width: 60vmax;
  height: 60vmax;
  left: -20vmax;
  top: -20vmax;
  background: #c026d3;
}
.blob-b {
  width: 50vmax;
  height: 50vmax;
  right: -18vmax;
  top: 10vh;
  background: #4f46e5;
  animation-duration: 30s;
  animation-delay: -8s;
}
.blob-c {
  width: 45vmax;
  height: 45vmax;
  left: 15vw;
  bottom: -25vmax;
  background: #0891b2;
  animation-duration: 27s;
  animation-delay: -14s;
}
.blob-d {
  width: 30vmax;
  height: 30vmax;
  right: 5vw;
  bottom: -5vmax;
  background: #e11d48;
  opacity: 0.6;
  animation-duration: 34s;
  animation-delay: -4s;
}
@keyframes drift {
  from {
    transform: translate(0, 0) scale(1);
  }
  to {
    transform: translate(6vmax, 4vmax) scale(1.15);
  }
}

.backdrop-enter-active,
.backdrop-leave-active {
  transition: opacity 0.6s ease;
}
.backdrop-enter-from,
.backdrop-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .blob {
    animation: none;
  }
  .backdrop-enter-active,
  .backdrop-leave-active {
    transition: none;
  }
}
</style>
