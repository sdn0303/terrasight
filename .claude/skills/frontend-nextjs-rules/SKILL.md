---
name: frontend-nextjs-rules
description: "Frontend development rules for Next.js 16, React 19, TanStack Query, Zustand, Zod, Tailwind v4, and shadcn/ui in services/frontend. Use when writing, reviewing, or refactoring React components, data fetching hooks, state management, forms, or UI styling."
metadata:
  version: "1.0.0"
  filePattern:
    - "services/frontend/**/*.ts"
    - "services/frontend/**/*.tsx"
    - "services/frontend/src/**"
---

# Frontend Next.js Rules

Consolidated frontend guidance for this project. Each reference file covers
a specific domain — open only the ones relevant to your task.

## Core Principles

1. **Server Components by default** — add `'use client'` only when hooks,
   event handlers, or browser APIs are needed. Push the boundary down.
2. **Validate at boundaries** — every external response parsed with Zod
   `safeParse`. Schema is the API contract source of truth.
3. **Derive, don't duplicate** — `z.infer<typeof Schema>` for types,
   query key factories for cache keys, `typeof` / `keyof` for TS types.
4. **Eliminate waterfalls** — `Promise.all` for independent fetches,
   `Suspense` boundaries for streaming, `"use cache"` for deduplication.
5. **Measure before optimizing** — React DevTools Profiler and Lighthouse
   before adding `useMemo`, `useCallback`, or dynamic imports.

## Quick Reference by Task

- **New component**: [react-patterns], [ui-styling]
- **Data fetching / hooks**: [data-fetching], [react-patterns]
- **Forms / validation**: [validation], [data-fetching]
- **State management**: [state-management]
- **Styling / accessibility**: [ui-styling]
- **Code review**: [react-patterns], [data-fetching], [validation]

## Reference Files

Open only the domains you need:

- **[react-patterns]**: RSC vs Client, performance rules, rendering patterns, anti-patterns
- **[data-fetching]**: TanStack Query hooks, query key factory, mutations, SSR hydration
- **[state-management]**: Zustand store patterns, nuqs URL state, selector discipline
- **[validation]**: Zod schemas, React Hook Form integration, Server Action validation
- **[ui-styling]**: Tailwind v4 `@theme`, shadcn/ui, accessibility, design tokens

## Review Checklist

1. `'use client'` only where needed, pushed as far down as possible?
2. External data validated with Zod `safeParse` at boundary?
3. TanStack Query hooks wrapped in custom hooks with key factory?
4. No `any` type, no `!` non-null assertion?
5. Error boundaries at route group level?
6. Accessibility: semantic HTML, ARIA, keyboard navigation?
7. No PII in logs?

## Verification

```bash
pnpm tsc --noEmit && pnpm biome check . && pnpm vitest run
```

[react-patterns]: references/react-patterns.md
[data-fetching]: references/data-fetching.md
[state-management]: references/state-management.md
[validation]: references/validation.md
[ui-styling]: references/ui-styling.md
