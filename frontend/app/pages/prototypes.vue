<!-- Throwaway: shared wrapper for every /prototypes route. See prototypes/README.md. -->
<template>
  <div>
    <NuxtPage />
    <NuxtLink
      v-if="route.path.replace(/\/$/, '') !== '/prototypes'"
      to="/prototypes"
      class="fixed bottom-3 left-3 z-[100] rounded-full bg-black/75 px-3 py-1.5 font-sans text-xs text-white shadow-lg ring-1 ring-white/20 hover:bg-black"
    >
      ← Back to all styles
    </NuxtLink>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ middleware: ['prototype-auth'] })

const route = useRoute()
</script>
