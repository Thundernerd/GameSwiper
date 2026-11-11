// Throwaway: see app/pages/prototypes/README.md.

/**
 * Returns a function that maps a real-app path ('/', '/events/1', '/login', …) to the same
 * screen inside a style prototype ('/prototypes/<style>', '/prototypes/<style>/events/1', …),
 * so copied pages never jump back into the real app.
 */
export function usePrototypeLink(style: string) {
  const base = `/prototypes/${style}`
  return (path: string) => (path === '/' ? base : `${base}${path}`)
}
