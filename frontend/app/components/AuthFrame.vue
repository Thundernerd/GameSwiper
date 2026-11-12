<template>
  <div class="mx-auto max-w-md px-4 py-16">
    <p class="arcade-display arcade-glow text-center text-2xl sm:text-3xl text-arcade-magenta mb-4">HYPE <br class="sm:hidden" >CHECK</p>
    <p class="arcade-display arcade-blink text-center text-[10px] text-arcade-yellow mb-10">{{ tagline }}</p>
    <div class="arcade-panel p-5 sm:p-6">
      <h1 class="arcade-display text-base sm:text-lg text-arcade-cyan mb-3">{{ heading }}</h1>
      <p class="text-arcade-muted mb-6">{{ intro }}</p>
      <form class="space-y-4" @submit.prevent="$emit('submit')">
        <label class="block">
          <span class="arcade-display text-[10px] text-arcade-text">USERNAME</span>
          <input
            v-model="username"
            required
            class="arcade-input mt-2"
            autocomplete="username"
          >
        </label>
        <label class="block">
          <span class="arcade-display text-[10px] text-arcade-text">PASSWORD</span>
          <input
            v-model="password"
            type="password"
            required
            minlength="6"
            class="arcade-input mt-2"
            :autocomplete="passwordAutocomplete"
          >
        </label>
        <p v-if="error" class="border-2 border-arcade-red px-3 py-2 text-arcade-red">
          <span class="arcade-display text-[10px]">ERROR:</span> {{ error }}
        </p>
        <button
          type="submit"
          :disabled="loading"
          class="arcade-btn arcade-btn-primary w-full py-3 text-xs"
        >
          {{ loading ? busyLabel : submitLabel }}
        </button>
      </form>
    </div>
    <!-- Links under the panel (switch between sign-in and register, mock-mode note). -->
    <slot />
  </div>
</template>

<script setup lang="ts">
/** The frame shared by the sign-in and register pages: title, panel, username/password form. */
defineProps<{
  /** Blinking line under the title. */
  tagline: string
  heading: string
  intro: string
  submitLabel: string
  /** Submit button text while `loading`. */
  busyLabel: string
  passwordAutocomplete: 'current-password' | 'new-password'
  error: string
  loading: boolean
}>()

defineEmits<{ submit: [] }>()

const username = defineModel<string>('username', { required: true })
const password = defineModel<string>('password', { required: true })
</script>
