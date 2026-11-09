/** Where to step to from `i` in a list of `n`, wrapping round at either end. */
export const stepTo = (i: number, n: number, by: 1 | -1) => (n > 0 ? (i + by + n) % n : 0);
