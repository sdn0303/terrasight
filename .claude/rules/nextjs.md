# Next.js Rules

## Tech Stack

| Layer | Choice | Purpose |
|-------|--------|---------|
| Framework | Next.js 16 (App Router) | RSC, Server Actions, Turbopack, React Compiler |
| Runtime | React 19.2 | View Transitions, `useEffectEvent()` |
| UI | shadcn/ui + Tailwind CSS v4 | Customizable Radix-based components |
| Server State | TanStack Query v5 | Cache, optimistic updates, retry |
| Client State | Zustand | Lightweight, TypeScript-friendly |
| URL State | nuqs | Filters, pagination in URL |
| Forms | React Hook Form + Zod | Uncontrolled + schema validation |
| HTTP | ky (client) / fetch (server) | Client: lightweight, Server: Next.js cache |
| Testing | Vitest + Testing Library + Playwright | Unit/integration + E2E |

## Configuration (next.config.ts)

- `cacheComponents: true` -- Enable `"use cache"` directive
- `experimental.typedRoutes: true` -- Type-safe routing
- `experimental.serverActions.bodySizeLimit: '2mb'` -- Action payload limit
- `images.remotePatterns` -- Whitelist external image domains
- Turbopack is the default bundler (use `--webpack` flag to opt out)

## Project Structure

- `src/app/` -- App Router (pages, layouts, errors)
- `src/components/ui/` -- shadcn/ui components; `src/components/shared/` -- App-wide components
- `src/features/[feature]/{api,components,schemas,types,actions}/` -- Feature modules
- `src/lib/` -- Shared utilities; `src/stores/` -- Zustand; `src/hooks/` -- Shared hooks

## App Router

- **Route Groups**: `(auth)/`, `(dashboard)/` -- each with own `layout.tsx` and `error.tsx`
- **`"use cache"`**: Apply at file/component/function level; use `cacheLife()` for expiration (`seconds`, `minutes`, `hours`, `days`, `max`); read `cookies()`/`headers()` outside cached scope
- **`proxy.ts`**: Replaces `middleware.ts` for network boundary operations

## State Management

- **Query Key Factory**: `const keys = { all: ['entity'], list: (f) => [...], detail: (id) => [...] }`
- **TanStack Query defaults**: `staleTime: 60_000`, `gcTime: 300_000`, `retry: 1`
- **Zustand**: Use `devtools` + `persist` middleware, `partialize` for selective persistence
- **URL state**: Use `nuqs` for filters and pagination

## UI

- **shadcn/ui**: Style `new-york`, RSC-compatible, import from `@/components/ui/`
- **Tailwind CSS v4**: Use `@theme` for custom tokens, CSS variables for dark mode
- **Accessibility**: Semantic HTML, ARIA attributes, keyboard navigation

## Forms & Validation

- Define Zod schema in `schemas/`, derive type with `z.infer<typeof schema>`, use `zodResolver` with React Hook Form
- Always validate again in Server Actions (never trust client)

## Data Fetching & Caching

- **RSC**: Use `fetch` with `"use cache"` or `next: { tags, revalidate }`
- **Client**: Use `ky` wrapped with TanStack Query hooks
- **Always validate** response with Zod schema on both server and client
- **Cache policy**: `cacheLife('days')` for master data, `cacheLife('minutes')` for lists, `tags` for detail views, `revalidate: 0` for real-time

## Error Handling

- Error boundary hierarchy: `app/error.tsx` > `app/(group)/error.tsx` > page-level `error.tsx`
- `error.tsx` must be `'use client'` with retry button and home link
- Create `not-found.tsx` at app root and in route groups

## Security

- No PII in logs (never log email, phone, personal data)
- No detailed errors to client (stack traces in server logs only)
- `NEXT_PUBLIC_` prefix only for truly public env vars
- **Server Actions**: Check session -> check permissions -> validate input with Zod -> execute logic -> revalidate cache

## Anti-patterns

- **Unnecessary `'use client'`**: Only add when using hooks, event handlers, or browser APIs
- **Missing auth in Server Actions**: Always check session first
- **No Error Boundary**: Add `error.tsx` at appropriate levels
- **Direct API calls without Query**: Use TanStack Query for caching and retry
- **Form without Zod / No `"use cache"` strategy**: Always use schema validation and define cache policy

## CI Commands

```bash
pnpm tsc --noEmit
pnpm biome check .
pnpm vitest run
pnpm playwright test
```
