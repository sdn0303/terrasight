# Real Estate Investment Data Visualizer

不動産投資データ可視化プラットフォーム（47都道府県対応）。MLIT API → Rust Axum → GeoJSON → MapLibre GL 3D Map。

## Tech Stack

- **Backend**: Rust (Axum + Tokio + SQLx + PostGIS)
- **Frontend**: Next.js 16 (App Router) + React 19 + MapLibre GL + shadcn/ui + Tailwind CSS v4
- **WASM**: Rust → wasm-bindgen → Web Worker (R-tree spatial queries)
- **Database**: PostgreSQL + PostGIS
- **Infra**: Docker Compose

## Project Structure

```
services/
├── backend/    # Rust Axum (Clean Architecture: handler/usecase/domain/infra)
├── frontend/   # Next.js 16 (features/components/stores/hooks)
└── wasm/       # Rust WASM spatial engine (R-tree, FlatGeobuf)
```

## Build & Test

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
- Validate at boundaries — Zod (frontend) + Axum extractors (backend)
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

## Operations

```bash
./scripts/commands/db-full-reset.sh          # DB reset + seed + import
./scripts/commands/db-import-all.sh          # Data import only
./scripts/commands/pipeline.sh 13 P0         # Pipeline v2: convert + build + import + validate
uv run scripts/tools/pipeline_v2/convert.py --pref 13 --priority P0   # RAW → GeoJSON
uv run scripts/tools/pipeline_v2/build_fgb.py --pref 13               # GeoJSON → FlatGeobuf + manifest
uv run scripts/tools/pipeline_v2/import_db.py --pref 13 --priority P0 # GeoJSON → PostGIS
docker compose up -d --build                 # Dev environment
rm -f .git/index.lock                        # Fix git lock
```

## Detailed Rules

See `.claude/rules/` for comprehensive guidelines:
architecture, nextjs, typescript, rust, docker, postgresql, rest-api, security, github-actions, terraform, **workflow** (includes subagent anti-patterns)
