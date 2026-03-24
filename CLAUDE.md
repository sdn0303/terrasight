# Real Estate Investment Data Visualizer

不動産投資データ可視化プラットフォーム（東京23区）。MLIT API → Rust Axum → GeoJSON → MapLibre GL 3D Map。

## Tech Stack

- **Backend**: Rust (Axum + Tokio + SQLx + PostGIS)
- **Frontend**: Next.js 16 (App Router) + React 19 + MapLibre GL + shadcn/ui + Tailwind CSS v4
- **Database**: PostgreSQL + PostGIS
- **Cache**: SQLite (24h TTL for MLIT API responses)
- **Infra**: Docker Compose

## Project Structure

```
services/
├── backend/    # Rust Axum (Clean Architecture: handler/usecase/domain/infra)
└── frontend/   # Next.js 16 (features/components/stores/hooks)
```

## Build & Test

```bash
# Backend
cd services/backend && cargo build && cargo test && cargo clippy -- -D warnings

# Frontend
cd services/frontend && pnpm install && pnpm tsc --noEmit && pnpm biome check . && pnpm vitest run
```

## Absolute Rules (MUST follow)

- **No secrets in code**: API keys via env vars only. `.env` in `.gitignore`
- **No `.unwrap()` in Rust non-test code**: Use `?` or `.expect("reason")`
- **No `any` in TypeScript**: Use `unknown` + narrowing
- **No `SELECT *`**: Specify columns explicitly
- **No OFFSET pagination**: Use cursor-based
- **Validate at boundaries**: Zod (frontend) + Axum extractors (backend)
- **Server Components by default**: `'use client'` only when necessary
- **Domain layer is pure**: Zero external dependencies in `src/domain/`
- **GeoJSON coordinates**: Always `[longitude, latitude]` (RFC 7946)
- **No raw innerHTML**: Always sanitize with DOMPurify if rendering user content

## Performance Rules

- **Profile before optimizing**: `chrome://tracing` or DevTools Performance タブでボトルネックを計測してから着手。推測で最適化しない
- **No store-derived query keys without debounce**: Zustand の `viewState` 等の高頻度更新値から TanStack Query の `queryKey` を導出する場合、必ずデバウンス済み state 経由にする。直接購読すると毎フレーム再フェッチが発生する
- **WASM は O(n log n) 以上が対象**: 単純 O(n) ループ（加減算・オブジェクト生成）は JS で十分高速。WASM の FFI シリアライズコストが計算コストを上回る場合が多い
- **リクエスト削減 > 計算高速化**: パフォーマンス問題の大半は「計算が遅い」ではなく「不要な処理が多すぎる」。まずリクエスト数・レンダリング回数を疑う

## Anti-patterns (MUST avoid)

- Secrets in source code, Docker ENV, or CI logs
- `useEffect` for data fetching (use TanStack Query)
- Syncing query data to local state
- Inline `queryFn` without custom hook wrapper
- Zustand store を直接購読して TanStack Query の `queryKey` を生成（デバウンスなしのリクエスト洪水）
- 計測なしのパフォーマンス最適化（WASM・Web Worker 等の導入判断含む）
- 3+ table JOINs without `EXPLAIN ANALYZE`
- `ubuntu-latest` in GitHub Actions (pin `ubuntu-24.04`)
- Floating action tags in CI (pin to full SHA)

## Detailed Rules

See `.claude/rules/` for comprehensive guidelines:
architecture, nextjs, typescript, rust, docker, postgresql, rest-api, security, github-actions, terraform, workflow
