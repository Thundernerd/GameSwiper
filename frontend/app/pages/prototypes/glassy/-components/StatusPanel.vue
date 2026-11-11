<!-- Glassy loading / error / empty message: a frosted panel with an icon. -->
<template>
  <div
    class="mx-auto flex w-full max-w-md flex-col items-center gap-3 rounded-3xl border border-white/15 bg-slate-950/45 px-6 py-8 text-center backdrop-blur-xl shadow-[0_20px_60px_-20px_rgba(0,0,0,0.7)]"
    :role="kind === 'error' ? 'alert' : 'status'"
  >
    <span
      v-if="kind === 'loading'"
      class="spinner h-9 w-9 rounded-full"
      aria-hidden="true"
    />
    <span
      v-else
      class="grid h-10 w-10 place-items-center rounded-full text-lg font-bold"
      :class="
        kind === 'error'
          ? 'bg-gradient-to-br from-rose-500 to-orange-500 text-white shadow-lg shadow-rose-500/30'
          : 'bg-gradient-to-br from-violet-500 to-cyan-500 text-white shadow-lg shadow-violet-500/30'
      "
      aria-hidden="true"
    >
      {{ kind === 'error' ? '!' : '∅' }}
    </span>
    <p :class="kind === 'error' ? 'text-rose-200' : 'text-slate-200'">
      <slot />
    </p>
  </div>
</template>

<script setup lang="ts">
defineProps<{ kind: 'loading' | 'error' | 'empty' }>()
</script>

<style scoped>
.spinner {
  background: conic-gradient(from 0deg, #d946ef, #8b5cf6, #22d3ee, #d946ef);
  -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px));
  mask: radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px));
  animation: spin 0.9s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
@media (prefers-reduced-motion: reduce) {
  .spinner {
    animation-duration: 3s;
  }
}
</style>
