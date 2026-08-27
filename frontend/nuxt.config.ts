// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  modules: ['@nuxtjs/tailwindcss'],
  css: ['~/assets/css/main.css'],
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
