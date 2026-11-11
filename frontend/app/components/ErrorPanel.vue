<template>
  <div class="arcade-panel !border-arcade-red" :class="large ? 'p-5' : 'p-4'">
    <p class="arcade-display text-xs text-arcade-red" :class="large ? 'mb-3' : 'mb-2'">ERROR</p>
    <p class="text-arcade-text">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  message: string
  /** More padding, for a panel that stands in for a whole screen (the swipe deck). */
  large?: boolean
}>()
</script>
