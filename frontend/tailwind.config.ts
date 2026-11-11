import type { Config } from 'tailwindcss'

// The arcade palette lives as CSS variables in app/assets/css/main.css; these names map to it,
// so markup writes `text-arcade-yellow` instead of `text-[color:rgb(var(--arcade-yellow))]`.
// The variables hold RGB channels, so `<alpha-value>` makes opacity modifiers such as
// `bg-arcade-cyan/20` work.
const arcade = (name: string) => `rgb(var(--arcade-${name}) / <alpha-value>)`

export default {
  theme: {
    extend: {
      colors: {
        arcade: {
          bg: arcade('bg'),
          panel: arcade('panel'),
          line: arcade('line'),
          text: arcade('text'),
          muted: arcade('muted'),
          dim: arcade('dim'),
          magenta: arcade('magenta'),
          cyan: arcade('cyan'),
          yellow: arcade('yellow'),
          green: arcade('green'),
          red: arcade('red'),
        },
      },
    },
  },
} satisfies Partial<Config>
