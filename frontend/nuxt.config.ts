// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  // Client-only rendering. Auth state is browser-only (the mock user lives in localStorage),
  // so server-rendering a gated page produced HTML the client then replaced with /login,
  // causing hydration mismatches.
  // Pages fetch their data in onMounted anyway, so SSR only ever rendered a "Loading…" shell.
  ssr: false,
  modules: ['@nuxtjs/tailwindcss'],
  css: ['~/assets/css/main.css'],
  // Throwaway style prototypes keep their components next to their pages; don't route them.
  ignore: ['**/pages/prototypes/*/-components/**'],
  runtimeConfig: {
    public: {
      useMock: true,
      apiBase: '',
    },
  },
  nitro: {
    devProxy: {
      '/api': {
        target: 'http://127.0.0.1:8080/api',
        changeOrigin: true,
        cookieDomainRewrite: 'localhost',
      },
    },
  },
  routeRules: {
    '/api/**': { proxy: 'http://127.0.0.1:8080/api/**' },
  },
})
