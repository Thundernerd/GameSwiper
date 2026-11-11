<!-- Throwaway: comparison index for the style prototypes. See README.md next to this file. -->
<template>
  <div class="min-h-screen bg-white text-neutral-900 font-sans">
    <div class="mx-auto max-w-4xl px-4 py-12">
      <h1 class="text-3xl font-bold tracking-tight">Style prototypes</h1>
      <p class="mt-1 text-neutral-600">
        Throwaway copies of every screen, one per visual style, running on mock fixtures.
      </p>

      <ul class="mt-8 divide-y divide-neutral-200 border-y border-neutral-200">
        <li v-for="style in prototypeStyles" :key="style.id" class="py-5">
          <h2 class="text-lg font-semibold">{{ style.name }}</h2>
          <p class="mt-0.5 text-sm text-neutral-600">{{ style.description }}</p>
          <nav class="mt-3 flex flex-wrap gap-2">
            <NuxtLink
              v-for="screen in prototypeScreens"
              :key="screen.path"
              :to="usePrototypeLink(style.id)(screen.path)"
              class="rounded-md border border-neutral-300 px-2.5 py-1 text-sm hover:border-neutral-900 hover:bg-neutral-100"
            >
              {{ screen.label }}
            </NuxtLink>
          </nav>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { prototypeScreens, prototypeStyles } from '~/prototypes/styles'
</script>
