# web

React + TypeScript + Tailwind, bundled with Bun. There is no dev server: the Rust server serves `web/dist` (see the root `readme.md`).

```
bun run build       # bundle to web/dist (build.ts)
bun run dev         # rebuild on change
bun run test        # bun test (happy-dom + testing-library)
bun run typecheck
```
