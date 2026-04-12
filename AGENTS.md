# Terrasight

不動産投資データ可視化プラットフォーム（47都道府県対応）。MLIT API → Rust Axum → GeoJSON → MapLibre GL 3D Map。

## Tech Stack

- **Backend**: Rust (Axum + Tokio + SQLx + PostGIS)
- **Frontend**: Next.js 16 (App Router) + React 19 + MapLibre GL + shadcn/ui + Tailwind CSS v4
- **WASM**: Rust → wasm-bindgen → Web Worker (R-tree spatial queries)
- **Database**: PostgreSQL + PostGIS
- **Infra**: Docker Compose

## Project Structure

```text
services/
├── backend/    # Rust Axum (Clean Architecture: handler/usecase/domain/infra)
├── frontend/   # Next.js 16 (features/components/stores/hooks)
└── wasm/       # Rust WASM spatial engine (R-tree, FlatGeobuf)
```

## Build and Test

```bash
# Backend
cd services/backend && cargo build && cargo test && cargo clippy -- -D warnings

# Frontend
cd services/frontend && pnpm install && pnpm tsc --noEmit && pnpm biome check . && pnpm vitest run

# WASM
cd services/wasm && cargo test
```

## Absolute Rules

- No secrets in code — API keys via env vars, `.env` in `.gitignore`
- No `.unwrap()` in Rust non-test code — use `?` or `.expect("reason")`
- No `any` in TypeScript — use `unknown` + narrowing
- No `SELECT *` — specify columns explicitly
- No OFFSET pagination — use cursor-based
- Validate at boundaries — Zod (frontend) + Axum extractors (backend)
- Server Components by default — `'use client'` only when necessary
- Domain layer is pure — zero external dependencies in `src/domain/`
- Frontend Zod schema is API contract source of truth
- GeoJSON coordinates: always `[longitude, latitude]` (RFC 7946)
- Profile before optimizing — no WASM/Worker without measurement

## Key Conventions

- **Layer IDs**: UI uses `underscore_case`, WASM/FGB uses `hyphen-case`. Use `canonicalLayerId()` at boundaries
- **Static data**: FlatGeobuf in `data/fgb/{pref_code}/`, served via symlink `public/data/fgb/`
- **Dataset catalog**: `data/catalog/dataset_catalog.json` drives all pipeline stages
- **Prefecture scope**: All tables have `pref_code` column; API endpoints accept optional `?pref_code=13`
- **WASM stats**: Disabled in Phase 1. Backend `/api/stats` is canonical
- **Design tokens**: `globals.css` `:root` variables, Tailwind `@theme` with `ds-*` prefix

## Performance Rules

- Profile before optimizing — DevTools Performance tab or `chrome://tracing` first
- No store-derived query keys without debounce — Zustand `viewState` → TanStack Query `queryKey` must go through debounced state
- WASM for O(n log n)+ only — simple O(n) loops are faster in JS due to FFI overhead
- Reduce requests before optimizing computation — most perf issues are unnecessary fetches/renders

## API Contract Rules

- Frontend Zod schema is source of truth — backend `Serialize` DTOs must match field names exactly
- `z.record()` rejects `null` — return `json!({})` not `json!(null)` for optional objects
- Integration tests verify API contract — assert field paths matching Zod schema

## Anti-patterns

- Secrets in source code, Docker ENV, or CI logs
- `useEffect` for data fetching (use TanStack Query)
- Syncing query data to local state
- Inline `queryFn` without custom hook wrapper
- Zustand store direct subscription for TanStack Query keys (request flood)
- 3+ table JOINs without `EXPLAIN ANALYZE`
- `ubuntu-latest` in GitHub Actions (pin `ubuntu-24.04`)
- Floating action tags in CI (pin to full SHA)
- Backend DTO / Frontend Zod field name mismatch without integration test
- `#[allow(dead_code)]` on legacy code after new implementation is verified

## Operations

```bash
./scripts/commands/db-full-reset.sh                                    # DB reset + seed + import
./scripts/commands/db-import-all.sh                                    # Data import only
./scripts/commands/pipeline.sh 13 P0                                   # Pipeline v2: convert + build + import + validate
uv run scripts/tools/pipeline/convert.py --pref 13 --priority P0      # RAW → GeoJSON
uv run scripts/tools/pipeline/build_fgb.py --pref 13                  # GeoJSON → FlatGeobuf + manifest
uv run scripts/tools/pipeline/import_db.py --pref 13 --priority P0    # GeoJSON → PostGIS
docker compose up -d --build                                           # Dev environment
```

## Detailed Rules

See `.claude/rules/` for comprehensive guidelines:

| Rule | Scope | Always loaded |
| ---- | ----- | ------------- |
| `architecture.md` | All files | Yes |
| `security.md` | All files | Yes |
| `workflow.md` | All files | Yes |
| `rust.md` | `services/backend/**/*.rs` | No |
| `nextjs.md` | `services/frontend/**` | No |
| `typescript.md` | `**/*.ts`, `**/*.tsx` | No |
| `postgresql.md` | `**/*.sql`, `**/migrations/**` | No |
| `rest-api.md` | `src/handler/**`, `src/app/api/**` | No |
| `docker.md` | `Dockerfile*`, `docker-compose*` | No |
| `github-actions.md` | `.github/**` | No |
| `terraform.md` | `**/*.tf`, `**/*.tfvars` | No |
