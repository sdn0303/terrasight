# React Patterns

## Contents

- [Server vs Client Components](#server-vs-client-components)
- [Eliminating Waterfalls](#eliminating-waterfalls)
- [Bundle Size Optimization](#bundle-size-optimization)
- [Server-Side Performance](#server-side-performance)
- [Re-render Optimization](#re-render-optimization)
- [Rendering Performance](#rendering-performance)
- [Anti-patterns](#anti-patterns)

---

## Server vs Client Components

- **Default to Server Components** — no `'use client'` unless hooks, events, or browser APIs needed
- Push `'use client'` boundary as far down the tree as possible
- Server Components can `async/await` data directly
- Client Components cannot use `async` — use TanStack Query instead
- React Compiler (auto-memoization) is enabled — remove manual `useMemo`/`useCallback` unless profiler shows need

## Eliminating Waterfalls

- `async-defer-await` — move `await` into branches where actually used
- `async-parallel` — `Promise.all()` for independent operations
- `async-suspense-boundaries` — use `Suspense` to stream content progressively
- `async-api-routes` — start promises early, `await` late in API routes
- `server-parallel-fetching` — restructure components to parallelize fetches

## Bundle Size Optimization

- `bundle-barrel-imports` — import directly from source, avoid barrel files
- `bundle-dynamic-imports` — `next/dynamic` for heavy components (charts, maps, editors)
- `bundle-defer-third-party` — load analytics/logging after hydration
- `bundle-conditional` — load modules only when feature is activated
- `bundle-preload` — preload on hover/focus for perceived speed

## Server-Side Performance

- `server-auth-actions` — authenticate Server Actions like API routes
- `server-cache-react` — `React.cache()` for per-request deduplication
- `server-dedup-props` — avoid duplicate serialization in RSC props
- `server-hoist-static-io` — hoist static I/O (fonts, logos) to module level
- `server-serialization` — minimize data passed to client components
- `server-after-nonblocking` — `after()` for non-blocking operations

## Re-render Optimization

- `rerender-defer-reads` — don't subscribe to state only used in callbacks
- `rerender-derived-state` — subscribe to derived booleans, not raw values
- `rerender-derived-state-no-effect` — derive state during render, not in effects
- `rerender-functional-setstate` — functional `setState` for stable callbacks
- `rerender-lazy-state-init` — pass function to `useState` for expensive initial values
- `rerender-split-combined-hooks` — split hooks with independent dependencies
- `rerender-transitions` — `startTransition` for non-urgent updates
- `rerender-use-deferred-value` — defer expensive renders to keep input responsive
- `rerender-no-inline-components` — never define components inside components

## Rendering Performance

- `rendering-content-visibility` — `content-visibility: auto` for long lists
- `rendering-hoist-jsx` — extract static JSX outside components
- `rendering-conditional-render` — use ternary, not `&&` for conditionals
- `rendering-usetransition-loading` — prefer `useTransition` for loading state
- `rendering-resource-hints` — React DOM resource hints for preloading

## Anti-patterns

- `useEffect` for data fetching — use TanStack Query
- Syncing query data to local state — use query cache directly
- Components inside components — extract to separate files
- Missing error boundaries — add `error.tsx` at route group level
- Unnecessary `'use client'` on leaf components with no interactivity
