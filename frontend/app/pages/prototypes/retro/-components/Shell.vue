<!-- Retro arcade page chrome: CRT background, scanlines, pixel fonts and the arcade header. -->
<template>
  <div class="retro-root min-h-screen flex flex-col overflow-x-clip">
    <header v-if="user" class="sticky top-0 z-40 border-b-4 border-[color:var(--r-magenta)] bg-[color:var(--r-bg)]">
      <div class="mx-auto max-w-6xl px-4 py-3 flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <div class="flex items-center gap-4 min-w-0">
          <NuxtLink
            :to="link('/')"
            class="retro-display retro-glow text-sm sm:text-base text-[color:var(--r-magenta)] hover:text-[color:var(--r-yellow)]"
          >
            GAMESWIPER
          </NuxtLink>
          <NuxtLink
            :to="link('/library')"
            class="retro-display text-[10px] sm:text-xs text-[color:var(--r-cyan)] hover:text-[color:var(--r-yellow)]"
            active-class="!text-[color:var(--r-yellow)] underline underline-offset-4"
          >
            LIBRARY
          </NuxtLink>
        </div>
        <div class="flex items-center gap-3 min-w-0">
          <span class="retro-display text-[10px] sm:text-xs text-[color:var(--r-text)] truncate">
            <span class="text-[color:var(--r-green)]">P1</span> {{ user.username }}
          </span>
          <button type="button" class="retro-btn retro-btn-ghost text-[10px] px-2.5 py-2" @click="onLogout">
            LOG OUT
          </button>
        </div>
      </div>
    </header>
    <!-- Bottom padding keeps the shared "Back to all styles" corner clear. -->
    <div class="flex-1 pb-14">
      <slot />
    </div>
    <div class="retro-scanlines" aria-hidden="true" />
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
      href: 'https://fonts.googleapis.com/css2?family=Press+Start+2P&family=VT323&display=swap',
    },
  ],
})

async function onLogout() {
  await logout()
  await navigateTo(link('/login'))
}
</script>

<style>
/* Unscoped so the retro pages and components can share these; every class is retro-prefixed. */
.retro-root {
  --r-bg: #08060f;
  --r-panel: #120e1f;
  --r-line: #3a3163;
  --r-text: #efeeff; /* AA: 17.6:1 on --r-bg, 16.5:1 on --r-panel */
  --r-muted: #b4b0d6; /* AA: 9.7:1 on --r-bg, 9.1:1 on --r-panel */
  --r-dim: #8f8bb5; /* AA: 6.3:1 on --r-bg, 5.9:1 on --r-panel */
  --r-magenta: #ff4fd8;
  --r-cyan: #3ef0ff;
  --r-yellow: #ffe24a;
  --r-green: #4dff8a;
  --r-red: #ff5a72;
  background:
    radial-gradient(ellipse at 50% 0%, rgba(255, 79, 216, 0.12), transparent 60%),
    radial-gradient(ellipse at 50% 100%, rgba(62, 240, 255, 0.08), transparent 60%),
    var(--r-bg);
  background-attachment: fixed;
  color: var(--r-text);
  font-family: 'VT323', ui-monospace, monospace;
  font-size: 1.3rem;
  line-height: 1.25;
  -webkit-font-smoothing: none;
}

.retro-root * {
  border-radius: 0 !important;
}

.retro-display {
  font-family: 'Press Start 2P', ui-monospace, monospace;
  line-height: 1.6;
  letter-spacing: 0.02em;
}

.retro-glow {
  text-shadow: 0 0 6px currentColor, 0 0 18px currentColor;
}

.retro-panel {
  background: var(--r-panel);
  border: 2px solid var(--r-line);
  box-shadow: 4px 4px 0 #000;
}

.retro-btn {
  font-family: 'Press Start 2P', ui-monospace, monospace;
  text-transform: uppercase;
  border: 2px solid currentColor;
  box-shadow: 4px 4px 0 #000;
  transition: transform 60ms, box-shadow 60ms, background-color 60ms;
}
.retro-btn:hover {
  background-color: color-mix(in srgb, currentColor 14%, transparent);
}
.retro-btn:active {
  transform: translate(4px, 4px);
  box-shadow: 0 0 0 #000;
}
.retro-btn:disabled {
  opacity: 0.5;
  pointer-events: none;
}
.retro-btn-ghost {
  color: var(--r-muted);
}
.retro-btn-primary {
  color: #08060f;
  background: var(--r-yellow);
  border-color: var(--r-yellow);
}
.retro-btn-primary:hover {
  background: #fff07a;
}

.retro-input {
  width: 100%;
  background: #000;
  border: 2px solid var(--r-line);
  color: var(--r-green);
  padding: 0.4rem 0.75rem;
  font-size: 1.5rem;
  outline: none;
  caret-color: var(--r-green);
}
.retro-input:focus {
  border-color: var(--r-cyan);
  box-shadow: 0 0 0 2px #000, 0 0 12px rgba(62, 240, 255, 0.5);
}

.retro-tag {
  border: 1px solid currentColor;
  padding: 0 0.4rem;
  font-size: 1.05rem;
  line-height: 1.4;
}

.retro-root a:focus-visible,
.retro-root button:focus-visible,
.retro-root select:focus-visible {
  outline: 2px dashed var(--r-yellow);
  outline-offset: 3px;
}

.retro-scanlines {
  pointer-events: none;
  position: fixed;
  inset: 0;
  z-index: 60;
  background:
    repeating-linear-gradient(to bottom, rgba(0, 0, 0, 0.22) 0 1px, transparent 1px 3px),
    radial-gradient(ellipse at center, transparent 60%, rgba(0, 0, 0, 0.45) 100%);
}

.retro-blink {
  animation: retro-blink 1s steps(1) infinite;
}
@keyframes retro-blink {
  50% {
    opacity: 0;
  }
}
@media (prefers-reduced-motion: reduce) {
  .retro-blink {
    animation: none;
  }
}
</style>
