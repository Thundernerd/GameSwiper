# Style prototypes (throwaway)

Everything under `/prototypes` exists to compare visual styles side by side before we pick
one. It is throwaway: once a style is chosen, delete all of it (see the end of this file).

Each style is a full copy of the app's six screens under `/prototypes/<style>/`. The copies
run on the mock fixtures (`NUXT_PUBLIC_USE_MOCK=true`, the default) and leave the production
pages alone. `/prototypes` lists every style with links to its screens.

## Add a style

1. Copy the `baseline/` folder next to it, named after your style id (for example `neon/`).
   The copies need no edits to work: links take the style from the URL.
2. Add one entry to `prototypeStyles` in `app/prototypes/styles.ts`:
   `{ id: 'neon', name: 'Neon', description: 'One sentence about the look.' }`.
3. Restyle the copies however you like. Open `/prototypes` and check all six screens.

That's it: a new folder plus one list entry.

## Rules for a style folder

- **Routes.** The folder mirrors the real app: `index.vue` (events), `events/[id]/index.vue`
  (swipe), `events/[id]/overview.vue`, `library.vue`, `login.vue`, `register.vue`.
- **Links.** Never hard-code an app path like `'/'` or `/events/${id}`. Wrap it:
  `const link = usePrototypeLink()`, then `:to="link(`/events/${id}`)"` or
  `navigateTo(link('/'))`. `link` maps the path into whichever style is in the URL. Only the
  shared "Back to all styles" link may leave `/prototypes/<style>/`.
- **Components.** Put the style's components in its `-components/` folder and import them
  with a relative path (`import GameCard from './-components/GameCard.vue'`). Nuxt does not
  turn that folder into routes (see `ignore` in `nuxt.config.ts`), and it does not
  auto-import those files, so they never clash with the production components.
- **Chrome.** The global header is hidden on `/prototypes/**`. Each style draws its own
  header and sets its own full-page background (baseline does both in `-components/Shell.vue`).
  Don't rely on the `body` styles in `app/assets/css/main.css`.
- **Auth.** Login and register are public. Every other screen needs a mock user; the
  `prototype-auth` middleware (`app/middleware/prototype-auth.ts`) sends signed-out visitors
  to `/prototypes/<style>/login`. The mock user is shared with the real app (same
  `localStorage`), so signing in on one style signs you in everywhere.
- **Back link.** `app/pages/prototypes.vue` wraps every prototype screen and renders the
  "Back to all styles" link in the bottom-left corner. Keep that corner clear.
- Duplicated logic (swipe/drag, sorting, …) is fine. This code won't live long.

## Delete it all

When a style is chosen:

- delete `app/pages/prototypes/`, `app/pages/prototypes.vue`, `app/prototypes/`,
  `app/composables/usePrototypeLink.ts` and `app/middleware/prototype-auth.ts`;
- remove the `isPrototypePath` checks in `app/app.vue` and `app/middleware/auth.global.ts`,
  and the `ignore` entry in `nuxt.config.ts`.
