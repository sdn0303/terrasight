# Terrasight

不動産投資データ可視化プラットフォーム (47 都道府県対応)

国土交通省の公示地価・災害リスク・施設データを統合し、24 レイヤーの空間データを 3D マップ上に重畳表示。独自の投資スコア (TLS: 0-100) と WASM 空間エンジンで不動産投資の意思決定を支援する。

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust (Axum + Tokio + SQLx + PostGIS) |
| Frontend | Vite SPA + React 19 + Mapbox GL JS + shadcn/ui + Tailwind CSS v4 |
| WASM | Rust → wasm-bindgen → Web Worker (R-tree spatial queries) |
| Database | PostgreSQL 16 + PostGIS 3.4 |
| Infra | Docker Compose (nginx + Rust backend + PostGIS) |

## Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- [Rust](https://rustup.rs/) (stable 1.94+)
- [Node.js](https://nodejs.org/) 22+ / [pnpm](https://pnpm.io/)
- [uv](https://docs.astral.sh/uv/) (Python data pipeline)
- [lefthook](https://github.com/evilmartians/lefthook) (git hooks)

### Setup

```bash
# Clone and configure
git clone https://github.com/sdn0303/terrasight.git && cd terrasight
cp .env.example .env   # Edit DB_PASSWORD, optional REINFOLIB_API_KEY

# Start all services
docker compose up -d --build

# Import data (Tokyo, priority P0)
./scripts/commands/pipeline.sh 13 P0
```

| Service | URL |
|---------|-----|
| Frontend | http://localhost:3001 |
| Backend API | http://localhost:8000 |
| PostgreSQL | localhost:5432 |

### Local Development (without Docker)

```bash
# 1. Start DB only
docker compose up -d db

# 2. Backend
cd services/backend && cargo run

# 3. Frontend (separate terminal)
cd services/frontend && pnpm install && pnpm dev

# 4. Import data
./scripts/commands/pipeline.sh 13 P0
```

## Project Structure

```
services/
├── backend/    # Rust Axum API (Clean Architecture: handler/usecase/domain/infra)
├── frontend/   # Vite SPA (features/components/stores/hooks)
└── wasm/       # Rust WASM spatial engine (R-tree, FlatGeobuf)
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DB_PASSWORD` | Yes | PostgreSQL password (default: `devpass`) |
| `REINFOLIB_API_KEY` | No | MLIT Reinfolib API key (enables live transaction data) |
| `ESTAT_APP_ID` | No | e-Stat API key (population data) |
| `RUST_LOG` | No | Log level filter (default: `info`) |
| `VITE_API_URL` | No | Backend URL for frontend (default: `http://localhost:8000`) |

## Build & Test

```bash
# Backend
cd services/backend && cargo build && cargo test && cargo clippy -- -D warnings

# Frontend
cd services/frontend && pnpm tsc --noEmit && pnpm biome check src/ && pnpm vitest run

# WASM
cd services/wasm && cargo test
```

## Git Hooks

```bash
lefthook install
```

| Hook | Backend | Frontend | WASM |
|------|---------|----------|------|
| pre-commit | `cargo fmt`, `cargo clippy` | `biome check`, `tsc` | `cargo fmt`, `cargo clippy` |
| pre-push | `cargo build`, `cargo test` | `vitest run`, `vite build` | `cargo test` |

## Data Pipeline

```bash
# Full pipeline (convert + FlatGeobuf + import + validate)
./scripts/commands/pipeline.sh <pref_code> <priority>

# Examples
./scripts/commands/pipeline.sh 13 P0    # Tokyo
./scripts/commands/pipeline.sh 27 P0    # Osaka

# Individual steps
uv run scripts/tools/pipeline/convert.py --pref 13 --priority P0
uv run scripts/tools/pipeline/build_fgb.py --pref 13
uv run scripts/tools/pipeline/import_db.py --pref 13 --priority P0
```

## Documentation

| Document | Description |
|----------|-------------|
| [docs/API_SPEC.md](docs/API_SPEC.md) | REST API specification (v4.0.0) |
| [docs/WASM_SPEC.md](docs/WASM_SPEC.md) | WASM spatial engine specification (v2.0.0) |
| [docs/DATA_STRUCTURE.md](docs/DATA_STRUCTURE.md) | Database schema and data models |
| [docs/DESIGN.md](docs/DESIGN.md) | Design system (colors, typography, components) |
| [docs/UIUX_SPEC.md](docs/UIUX_SPEC.md) | UI/UX design specification |

## License

MIT
