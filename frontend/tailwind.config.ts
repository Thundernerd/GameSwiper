import type { Config } from 'tailwindcss'

// The arcade palette lives as CSS variables in app/assets/css/main.css; these names map to it,
// so markup writes `text-arcade-yellow` instead of `text-[color:var(--arcade-yellow)]`.
const arcade = (name: string) => `var(--arcade-${name})`

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
