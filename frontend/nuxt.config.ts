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
  // main.css carries the @tailwind directives itself. Without this the module also injects its
  // default tailwind.css after it, so utilities would win over the arcade-* classes in main.css.
  tailwindcss: { cssPath: false },
  app: {
    head: {
      title: 'Hype Check',
      // Retro arcade type: Press Start 2P for display text, VT323 for body text.
      link: [
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        {
          rel: 'stylesheet',
          href: 'https://fonts.googleapis.com/css2?family=Press+Start+2P&family=VT323&display=swap',
        },
      ],
    },
  },
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
