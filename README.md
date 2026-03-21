# Real Estate Investment Data Visualizer

不動産投資データ可視化プラットフォーム（東京23区）

MLIT (国土交通省) API の公示地価・災害リスク・施設データを取得し、独自の投資スコア (0-100) を算出。PostGIS 空間クエリと MapLibre GL JS による 3D マップで可視化する。

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust (Axum + Tokio + SQLx + PostGIS) |
| Frontend | Next.js 16 (App Router) + React 19 + MapLibre GL + shadcn/ui + Tailwind CSS v4 |
| Database | PostgreSQL 16 + PostGIS 3.4 |
| Cache | SQLite (24h TTL for MLIT API responses) |
| Infra | Docker Compose |

## Architecture

Clean Architecture 4-layer (handler / usecase / domain / infra):

```
services/
├── backend/                    # Rust Axum API server (:8000)
│   ├── src/
│   │   ├── main.rs             # Entry point + router
│   │   ├── app_state.rs        # DI composition root
│   │   ├── config.rs           # Environment configuration
│   │   ├── logging.rs          # Tracing / structured logging
│   │   ├── handler/            # HTTP handlers (request/response)
│   │   ├── usecase/            # Business logic orchestration
│   │   ├── domain/             # Entities, value objects, traits (pure, no I/O)
│   │   └── infra/              # PostgreSQL repositories (trait implementations)
│   └── lib/
│       ├── api-core/           # Shared middleware (rate limit, request ID, error handling)
│       ├── db/                 # Connection pool management
│       ├── geo-math/           # CAGR, rounding, spatial utilities
│       ├── mlit-client/        # MLIT Reinfolib API client (retry, caching)
│       └── telemetry/          # Tracing subscriber setup
└── frontend/                   # Next.js 16 (:3000)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Service health check (DB connectivity, API key status) |
| GET | `/api/area-data` | GeoJSON layers for a bounding box (flood, slope, zoning, facilities) |
| GET | `/api/score` | Investment score (0-100) for a coordinate |
| GET | `/api/stats` | Aggregated statistics for a bounding box |
| GET | `/api/trend` | Price trend / CAGR analysis for a coordinate |

See [docs/API_SPEC.md](docs/API_SPEC.md) for full request/response specifications.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 22+ / [pnpm](https://pnpm.io/)
- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- [lefthook](https://github.com/evilmartians/lefthook) (git hooks)

### Setup

```bash
# 1. Clone and configure
git clone <repo-url> && cd sample-app
cp .env.example .env   # Edit with your API keys

# 2. Start database
docker compose up -d db

# 3. Seed development data
psql -h localhost -U app -d realestate -f scripts/seed_dev_data.sql

# 4. Run backend
cd services/backend
cargo run

# 5. Run frontend (separate terminal)
cd services/frontend
pnpm install && pnpm dev
```

### Docker Compose (all services)

```bash
docker compose up --build
```

| Service | Port |
|---------|------|
| Frontend | http://localhost:3000 |
| Backend | http://localhost:8000 |
| PostgreSQL | localhost:5432 |

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DB_PASSWORD` | Yes | PostgreSQL password (default: `devpass`) |
| `DATABASE_URL` | Yes | Full connection string (set automatically in Docker) |
| `REINFOLIB_API_KEY` | No | MLIT Reinfolib API key (enables live data fetching) |
| `ESTAT_APP_ID` | No | e-Stat API key |
| `RUST_LOG` | No | Log level filter (default: `info`) |
| `NEXT_PUBLIC_API_URL` | Yes | Backend URL for frontend (default: `http://localhost:8000`) |

## Development

### Build & Test

```bash
# Backend
cd services/backend
cargo build                          # Build
cargo test                           # Run tests (25 unit + doc tests)
cargo clippy -- -D warnings          # Lint (zero warnings policy)
cargo fmt --all -- --check           # Format check

# Frontend
cd services/frontend
pnpm tsc --noEmit                    # Type check
pnpm biome check .                   # Lint + format
pnpm vitest run                      # Unit tests
```

### Git Hooks (lefthook)

```bash
lefthook install
```

| Hook | Checks |
|------|--------|
| pre-commit | `cargo fmt`, `cargo clippy`, SQL lint |
| pre-push | `cargo test`, `cargo build` |

## Documentation

| Document | Description |
|----------|-------------|
| [docs/API_SPEC.md](docs/API_SPEC.md) | REST API specification |
| [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) | Product requirements |
| [docs/UIUX_SPEC.md](docs/UIUX_SPEC.md) | UI/UX design specification |
| [docs/plans/](docs/plans/) | Implementation plans |
| [docs/designs/](docs/designs/) | Architecture design documents |
| [docs/research/](docs/research/) | Technical research notes |

## License

Private repository. All rights reserved.
