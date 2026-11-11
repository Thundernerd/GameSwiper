// Throwaway: see app/pages/prototypes/README.md.
import { parsePrototypePath, prototypePath } from '~/prototypes/styles'

/**
 * Returns `link(path)`, which maps a real-app path ('/', '/events/1', '/login', …) to the same
 * screen in the style prototype currently being viewed, so copied pages never jump back into
 * the real app. The style comes from the URL, so a copied style folder needs no edits.
 */
export function usePrototypeLink() {
  const route = useRoute()
  return (path: string) => {
    const style = parsePrototypePath(route.path)?.style
    if (!style) throw new Error(`usePrototypeLink used outside a style prototype: ${route.path}`)
    return prototypePath(style, path)
  }
}
