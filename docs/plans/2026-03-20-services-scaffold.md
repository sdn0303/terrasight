# Services Scaffold Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** `services/backend/`（Rust Axum + PostGIS）と `services/frontend/`（Next.js 16 + MapLibre GL）のプロジェクトスキャフォールドを作成し、ヘルスチェック疎通まで動作確認する。

**Architecture:** Backend は Clean Architecture 4層（handler/routes → services → models/domain → infra）で構成。Frontend は Next.js 16 App Router + feature-based 構造。両サービスは Docker Compose で接続し、`/api/health` の疎通でスキャフォールド完了を確認する。

**Tech Stack:**
- Backend: Rust 1.94 + Axum 0.8 + Tokio + SQLx 0.8 + PostGIS + thiserror + tracing
- Frontend: Next.js 16 + React 19 + TypeScript + MapLibre GL JS + react-map-gl + Tailwind CSS v4 + shadcn/ui
- Infra: Docker Compose (postgis/postgis:16-3.4 + Rust multi-stage + Node slim)

---

## Phase A: Backend Scaffold (Rust Axum)

### Task A1: Cargo プロジェクト初期化

**Files:**
- Create: `services/backend/Cargo.toml`
- Create: `services/backend/src/main.rs`
- Create: `services/backend/.env.example`
- Create: `services/backend/.gitignore`

**Step 1: Cargo.toml を作成**

```toml
[package]
name = "realestate-api"
version = "0.1.0"
edition = "2024"
rust-version = "1.94"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "json", "migrate"] }
reqwest = { version = "0.12", features = ["json"] }
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
thiserror = "2"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
axum-test = "16"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "json", "migrate"] }

[profile.release]
lto = true
codegen-units = 1
strip = true
```

**Step 2: main.rs 最小スケルトン（/api/health のみ）**

```rust
use axum::{Router, routing::get, Json};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app = Router::new()
        .route("/api/health", get(health))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Step 3: .env.example と .gitignore**

`.env.example`:
```
DATABASE_URL=postgres://app:devpass@localhost:5432/realestate
REINFOLIB_API_KEY=
ESTAT_APP_ID=
RUST_LOG=info
ALLOWED_ORIGINS=http://localhost:3000
```

`.gitignore`:
```
/target
.env
```

**Step 4: ビルド確認**

```bash
cd services/backend && cargo build 2>&1
```
Expected: Compiling → Finished

**Step 5: コミット**

```bash
git add services/backend/
git commit -m "feat(backend): Rust Axum project scaffold with health endpoint"
```

---

### Task A2: モジュール構造の展開（Clean Architecture）

**Files:**
- Create: `services/backend/src/routes/mod.rs`
- Create: `services/backend/src/routes/health.rs`
- Create: `services/backend/src/routes/area_data.rs` (stub)
- Create: `services/backend/src/routes/score.rs` (stub)
- Create: `services/backend/src/routes/stats.rs` (stub)
- Create: `services/backend/src/routes/trend.rs` (stub)
- Create: `services/backend/src/models/mod.rs`
- Create: `services/backend/src/models/error.rs`
- Create: `services/backend/src/models/geojson.rs` (stub)
- Create: `services/backend/src/services/mod.rs`
- Create: `services/backend/src/services/scoring.rs` (stub)
- Modify: `services/backend/src/main.rs` — リファクタ

**Step 1: AppError 型を models/error.rs に定義**

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found")]
    NotFound,

    #[error("Rate limited")]
    RateLimited,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "INVALID_PARAMS",
            Self::NotFound => "NOT_FOUND",
            Self::RateLimited => "RATE_LIMITED",
            Self::Database(_) => "DB_UNAVAILABLE",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
            }
        });

        (status, Json(body)).into_response()
    }
}
```

**Step 2: AppState を定義**

```rust
// src/models/mod.rs
pub mod error;
pub mod geojson;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub reinfolib_key: Option<String>,
}
```

**Step 3: health.rs をモジュールに移動**

```rust
// src/routes/health.rs
use axum::{extract::State, Json};
use serde::Serialize;
use crate::models::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    db_connected: bool,
    reinfolib_key_set: bool,
    version: String,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_connected = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    Json(HealthResponse {
        status: if db_connected { "ok".into() } else { "degraded".into() },
        db_connected,
        reinfolib_key_set: state.reinfolib_key.is_some(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
```

**Step 4: routes/mod.rs でまとめる**

```rust
pub mod health;
pub mod area_data;
pub mod score;
pub mod stats;
pub mod trend;
```

**Step 5: 各 stub ルートを作成**

各ファイル (area_data.rs, score.rs, stats.rs, trend.rs) に同じパターンで stub:

```rust
// src/routes/area_data.rs
use axum::Json;
use serde_json::{json, Value};

pub async fn get_area_data() -> Json<Value> {
    Json(json!({ "status": "not_implemented" }))
}
```

**Step 6: main.rs をリファクタ（AppState + DB接続）**

```rust
use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use std::net::SocketAddr;

mod routes;
mod models;
mod services;

use models::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    let state = AppState {
        db: pool,
        reinfolib_key: std::env::var("REINFOLIB_API_KEY").ok(),
    };

    let app = Router::new()
        .route("/api/health", get(routes::health::health))
        .route("/api/area-data", get(routes::area_data::get_area_data))
        .route("/api/score", get(routes::score::get_score))
        .route("/api/stats", get(routes::stats::get_stats))
        .route("/api/trend", get(routes::trend::get_trend))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Step 7: cargo check でコンパイル確認**

```bash
cd services/backend && cargo check 2>&1
```
Expected: Compiling → Finished

**Step 8: コミット**

```bash
git add services/backend/
git commit -m "feat(backend): Clean Architecture module structure with AppState and AppError"
```

---

### Task A3: PostGIS マイグレーション

**Files:**
- Create: `services/backend/migrations/20260320000001_init.sql`
- Create: `scripts/seed_dev_data.sql`

**Step 1: 初期マイグレーション SQL**

```sql
-- 20260320000001_init.sql
CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE land_prices (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    price_per_sqm integer NOT NULL,
    address text NOT NULL,
    land_use text,
    year integer NOT NULL,
    geom geometry(Point, 4326) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_land_prices_geom ON land_prices USING GIST(geom);
CREATE INDEX idx_land_prices_year ON land_prices (year);

CREATE TABLE zoning (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    zone_type text NOT NULL,
    zone_code text,
    floor_area_ratio real,
    building_coverage real,
    geom geometry(MultiPolygon, 4326) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_zoning_geom ON zoning USING GIST(geom);

CREATE TABLE flood_risk (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    depth_rank text,
    river_name text,
    geom geometry(MultiPolygon, 4326) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_flood_risk_geom ON flood_risk USING GIST(geom);

CREATE TABLE steep_slope (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    area_name text,
    geom geometry(MultiPolygon, 4326) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_steep_slope_geom ON steep_slope USING GIST(geom);

CREATE TABLE schools (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL,
    school_type text,
    geom geometry(Point, 4326) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_schools_geom ON schools USING GIST(geom);

CREATE TABLE medical_facilities (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL,
    facility_type text,
    bed_count integer,
    geom geometry(Point, 4326) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_medical_geom ON medical_facilities USING GIST(geom);
```

**Step 2: 開発用シードデータ（東京駅周辺）**

```sql
-- scripts/seed_dev_data.sql
-- 地価公示（5年分 — Sparkline用）
INSERT INTO land_prices (price_per_sqm, address, land_use, year, geom) VALUES
  (1020000, '千代田区丸の内1-1', '商業', 2020, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1050000, '千代田区丸の内1-1', '商業', 2021, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1100000, '千代田区丸の内1-1', '商業', 2022, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1150000, '千代田区丸の内1-1', '商業', 2023, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1200000, '千代田区丸の内1-1', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (780000, '中央区銀座4-6', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7649, 35.6717), 4326)),
  (620000, '港区新橋2-1', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7586, 35.6660), 4326)),
  (450000, '千代田区神田1-2', '住居', 2024, ST_SetSRID(ST_MakePoint(139.7700, 35.6925), 4326)),
  (380000, '台東区上野1-1', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7745, 35.7135), 4326));

-- 用途地域
INSERT INTO zoning (zone_type, zone_code, floor_area_ratio, building_coverage, geom) VALUES
  ('商業地域', '09', 8.0, 0.8, ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.76 35.68, 139.77 35.68, 139.77 35.69, 139.76 35.69, 139.76 35.68)))'), 4326)),
  ('第一種住居地域', '05', 3.0, 0.6, ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.77 35.69, 139.78 35.69, 139.78 35.70, 139.77 35.70, 139.77 35.69)))'), 4326)),
  ('近隣商業地域', '08', 4.0, 0.8, ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.75 35.66, 139.76 35.66, 139.76 35.67, 139.75 35.67, 139.75 35.66)))'), 4326));

-- 洪水浸水想定区域
INSERT INTO flood_risk (depth_rank, river_name, geom) VALUES
  ('0.5-3.0m', '荒川', ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.78 35.70, 139.80 35.70, 139.80 35.72, 139.78 35.72, 139.78 35.70)))'), 4326)),
  ('0-0.5m', '隅田川', ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.79 35.69, 139.80 35.69, 139.80 35.70, 139.79 35.70, 139.79 35.69)))'), 4326));

-- 急傾斜地
INSERT INTO steep_slope (area_name, geom) VALUES
  ('文京区目白台', ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.72 35.72, 139.73 35.72, 139.73 35.73, 139.72 35.73, 139.72 35.72)))'), 4326));

-- 学校
INSERT INTO schools (name, school_type, geom) VALUES
  ('千代田区立麹町小学校', '小学校', ST_SetSRID(ST_MakePoint(139.7401, 35.6841), 4326)),
  ('千代田区立神田一橋中学校', '中学校', ST_SetSRID(ST_MakePoint(139.7611, 35.6927), 4326)),
  ('東京都立日比谷高等学校', '高等学校', ST_SetSRID(ST_MakePoint(139.7520, 35.6719), 4326));

-- 医療機関
INSERT INTO medical_facilities (name, facility_type, bed_count, geom) VALUES
  ('聖路加国際病院', '病院', 520, ST_SetSRID(ST_MakePoint(139.7722, 35.6697), 4326)),
  ('東京逓信病院', '病院', 461, ST_SetSRID(ST_MakePoint(139.7543, 35.6943), 4326)),
  ('三井記念病院', '病院', 482, ST_SetSRID(ST_MakePoint(139.7785, 35.6930), 4326)),
  ('神田クリニック', '診療所', 0, ST_SetSRID(ST_MakePoint(139.7705, 35.6940), 4326)),
  ('銀座メディカルセンター', '診療所', 0, ST_SetSRID(ST_MakePoint(139.7645, 35.6720), 4326));
```

**Step 3: コミット**

```bash
git add services/backend/migrations/ scripts/
git commit -m "feat(backend): PostGIS initial migration and dev seed data"
```

---

### Task A4: Dockerfile（マルチステージビルド）

**Files:**
- Create: `services/backend/Dockerfile`

**Step 1: Dockerfile**

```dockerfile
FROM rust:1.94-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN useradd -r -s /bin/false appuser
COPY --from=builder /app/target/release/realestate-api /usr/local/bin/
USER appuser
EXPOSE 8000
ENTRYPOINT ["realestate-api"]
```

**Step 2: コミット**

```bash
git add services/backend/Dockerfile
git commit -m "feat(backend): multi-stage Docker build with non-root user"
```

---

## Phase B: Frontend Scaffold (Next.js 16)

### Task B1: Next.js プロジェクト初期化

**Files:**
- Create: `services/frontend/package.json`
- Create: `services/frontend/next.config.ts`
- Create: `services/frontend/tsconfig.json`
- Create: `services/frontend/src/app/layout.tsx`
- Create: `services/frontend/src/app/page.tsx`
- Create: `services/frontend/src/app/globals.css`
- Create: `services/frontend/.gitignore`
- Create: `services/frontend/Dockerfile`

**Step 1: pnpm init + 依存関係インストール**

```bash
cd services/frontend
pnpm init
pnpm add next@latest react@latest react-dom@latest
pnpm add -D typescript @types/react @types/react-dom @types/node
pnpm add maplibre-gl react-map-gl
pnpm add framer-motion recharts
pnpm add -D @biomejs/biome vitest @testing-library/react @testing-library/dom jsdom
```

**Step 2: next.config.ts**

```typescript
import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  experimental: {
    typedRoutes: true,
  },
};

export default nextConfig;
```

**Step 3: tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "esnext",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "plugins": [{ "name": "next" }],
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

**Step 4: globals.css（CRT Dark Theme トークン + Tailwind）**

```css
@import "tailwindcss";

:root {
  --bg-primary: #0a0a0f;
  --bg-secondary: #12121a;
  --bg-tertiary: #1a1a25;
  --text-primary: #e4e4e7;
  --text-secondary: #a1a1aa;
  --text-muted: #52525b;
  --text-heading: #f4f4f5;
  --border-primary: rgba(63, 63, 70, 0.5);
  --accent-cyan: #22d3ee;
  --accent-danger: #e04030;
  --accent-warning: #ffd000;
  --accent-success: #10b981;
  --hover-accent: rgba(34, 211, 238, 0.1);
  --font-mono: 'Geist Mono', monospace, system-ui;
}

body {
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-mono);
  margin: 0;
  overflow: hidden;
}
```

**Step 5: layout.tsx**

```tsx
import type { Metadata } from 'next';
import { GeistMono } from 'geist/font/mono';
import './globals.css';

export const metadata: Metadata = {
  title: 'RealEstate Intelligence',
  description: '不動産投資意思決定プラットフォーム',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="ja" className={GeistMono.variable}>
      <body>{children}</body>
    </html>
  );
}
```

**Step 6: page.tsx（最小プレースホルダ）**

```tsx
export default function Home() {
  return (
    <main
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100vh',
        color: 'var(--accent-cyan)',
        fontFamily: 'var(--font-mono)',
        fontSize: '14px',
      }}
    >
      REALESTATE INTELLIGENCE — INITIALIZING...
    </main>
  );
}
```

**Step 7: .gitignore**

```
node_modules/
.next/
.env*.local
```

**Step 8: pnpm add geist フォント**

```bash
pnpm add geist
```

**Step 9: ビルド確認**

```bash
cd services/frontend && pnpm tsc --noEmit 2>&1
```
Expected: 型チェック通過

**Step 10: コミット**

```bash
git add services/frontend/
git commit -m "feat(frontend): Next.js 16 project scaffold with CRT dark theme"
```

---

### Task B2: フロントエンド ディレクトリ構造 + 共通ユーティリティ

**Files:**
- Create: `services/frontend/src/lib/api.ts`
- Create: `services/frontend/src/lib/constants.ts`
- Create: `services/frontend/src/lib/layers.ts`
- Create: `services/frontend/src/hooks/use-map-data.ts`
- Create: `services/frontend/src/components/ui/` (placeholder)
- Create: `services/frontend/src/components/crt-overlay.tsx`
- Create: `services/frontend/src/components/status-bar.tsx`
- Create: `services/frontend/biome.json`
- Create: `services/frontend/vitest.config.ts`

**Step 1: API クライアント**

```typescript
// src/lib/api.ts
const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8000';

export async function fetchApi<T>(path: string, params?: Record<string, string>): Promise<T> {
  const url = new URL(path, API_BASE);
  if (params) {
    for (const [key, value] of Object.entries(params)) {
      url.searchParams.set(key, value);
    }
  }

  const response = await fetch(url.toString());
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: response.statusText } }));
    throw new Error(error.error?.message ?? `API error: ${response.status}`);
  }
  return response.json() as Promise<T>;
}
```

**Step 2: レイヤー定義**

```typescript
// src/lib/layers.ts
export interface LayerConfig {
  id: string;
  name: string;
  nameJa: string;
  category: 'pricing' | 'urban' | 'disaster' | 'facilities';
  defaultEnabled: boolean;
}

export const LAYERS: LayerConfig[] = [
  { id: 'landprice', name: 'Land Price', nameJa: '地価公示', category: 'pricing', defaultEnabled: true },
  { id: 'zoning', name: 'Zoning', nameJa: '用途地域', category: 'urban', defaultEnabled: true },
  { id: 'flood', name: 'Flood Risk', nameJa: '洪水浸水', category: 'disaster', defaultEnabled: false },
  { id: 'steep_slope', name: 'Steep Slope', nameJa: '急傾斜地', category: 'disaster', defaultEnabled: false },
  { id: 'schools', name: 'Schools', nameJa: '学校', category: 'facilities', defaultEnabled: false },
  { id: 'medical', name: 'Medical', nameJa: '医療機関', category: 'facilities', defaultEnabled: false },
];

export const CATEGORIES = [
  { id: 'pricing', label: 'PRICING' },
  { id: 'urban', label: 'URBAN PLANNING' },
  { id: 'disaster', label: 'DISASTER RISK' },
  { id: 'facilities', label: 'FACILITIES' },
] as const;
```

**Step 3: 定数**

```typescript
// src/lib/constants.ts
export const MAP_CONFIG = {
  center: [139.767, 35.681] as [number, number],
  zoom: 12,
  pitch: 45,
  bearing: 0,
  style: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
} as const;

export const DEBOUNCE_MS = 300;
export const BBOX_MAX_DEGREES = 0.5;
```

**Step 4: CRT Overlay コンポーネント**

```tsx
// src/components/crt-overlay.tsx
export function CRTOverlay() {
  return (
    <>
      <div
        aria-hidden="true"
        className="pointer-events-none fixed inset-0"
        style={{
          zIndex: 200,
          background: 'radial-gradient(circle, transparent 40%, rgba(0,0,0,0.8) 100%)',
        }}
      />
      <div
        aria-hidden="true"
        className="pointer-events-none fixed inset-0"
        style={{
          zIndex: 300,
          opacity: 0.05,
          background: 'linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px)',
          backgroundSize: '100% 4px',
        }}
      />
    </>
  );
}
```

**Step 5: StatusBar コンポーネント**

```tsx
// src/components/status-bar.tsx
interface StatusBarProps {
  lat: number;
  lng: number;
  zoom: number;
  isLoading: boolean;
  isDemoMode: boolean;
}

export function StatusBar({ lat, lng, zoom, isLoading, isDemoMode }: StatusBarProps) {
  return (
    <div
      className="fixed bottom-0 left-0 right-0 flex items-center gap-4 px-4"
      style={{
        height: '28px',
        fontSize: '10px',
        fontFamily: 'var(--font-mono)',
        background: 'var(--bg-primary)',
        borderTop: '1px solid var(--border-primary)',
        color: 'var(--text-muted)',
        zIndex: 20,
      }}
    >
      <span>{lat.toFixed(4)}°N {lng.toFixed(4)}°E</span>
      <span>Z:{zoom.toFixed(1)}</span>
      {isDemoMode && (
        <span style={{ color: 'var(--accent-warning)' }}>● DEMO</span>
      )}
      {isLoading && (
        <span style={{ color: 'var(--accent-cyan)' }}>◌ LOADING...</span>
      )}
    </div>
  );
}
```

**Step 6: biome.json**

```json
{
  "$schema": "https://biomejs.dev/schemas/2.0.0/schema.json",
  "organizeImports": { "enabled": true },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true
    }
  },
  "formatter": {
    "indentStyle": "space",
    "indentWidth": 2
  }
}
```

**Step 7: vitest.config.ts**

```typescript
import { defineConfig } from 'vitest/config';
import { resolve } from 'node:path';

export default defineConfig({
  test: {
    environment: 'jsdom',
    globals: true,
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
});
```

**Step 8: ビルド確認**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check .
```

**Step 9: コミット**

```bash
git add services/frontend/
git commit -m "feat(frontend): directory structure, API client, layer definitions, CRT overlay"
```

---

### Task B3: Dockerfile（フロントエンド）

**Files:**
- Create: `services/frontend/Dockerfile`

**Step 1: Dockerfile**

```dockerfile
FROM node:22-slim AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

FROM base AS deps
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN pnpm build

FROM base AS runner
WORKDIR /app
ENV NODE_ENV=production
RUN addgroup --system --gid 1001 nodejs && adduser --system --uid 1001 nextjs
COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static
USER nextjs
EXPOSE 3000
ENV PORT=3000
CMD ["node", "server.js"]
```

**Step 2: next.config.ts に output: 'standalone' 追加**

```typescript
const nextConfig: NextConfig = {
  output: 'standalone',
  experimental: {
    typedRoutes: true,
  },
};
```

**Step 3: コミット**

```bash
git add services/frontend/Dockerfile services/frontend/next.config.ts
git commit -m "feat(frontend): multi-stage Docker build with standalone output"
```

---

## Phase C: Infrastructure (Docker Compose)

### Task C1: Docker Compose + .env.example

**Files:**
- Create: `docker-compose.yml` (root)
- Update: `.env.example` (root)
- Update: `.gitignore` (root)

**Step 1: docker-compose.yml**

```yaml
services:
  db:
    image: postgis/postgis:16-3.4
    environment:
      POSTGRES_DB: realestate
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD:-devpass}
    volumes:
      - pgdata:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d realestate"]
      interval: 10s
      timeout: 5s
      retries: 5

  backend:
    build: ./services/backend
    depends_on:
      db:
        condition: service_healthy
    environment:
      DATABASE_URL: postgres://app:${DB_PASSWORD:-devpass}@db:5432/realestate
      REINFOLIB_API_KEY: ${REINFOLIB_API_KEY:-}
      RUST_LOG: ${RUST_LOG:-info}
      ALLOWED_ORIGINS: http://localhost:3000
    ports:
      - "8000:8000"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/api/health"]
      interval: 30s
      timeout: 5s
      retries: 3

  frontend:
    build: ./services/frontend
    depends_on:
      - backend
    environment:
      NEXT_PUBLIC_API_URL: http://backend:8000
    ports:
      - "3000:3000"

volumes:
  pgdata:
```

**Step 2: .env.example 更新**

```
# Database
DB_PASSWORD=devpass

# Backend
REINFOLIB_API_KEY=
ESTAT_APP_ID=
RUST_LOG=info

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:8000
```

**Step 3: .gitignore 更新（data/ を除外）**

追加:
```
data/geojson/
.env
.env.local
```

**Step 4: コミット**

```bash
git add docker-compose.yml .env.example .gitignore
git commit -m "feat(infra): Docker Compose with PostGIS, Rust backend, Next.js frontend"
```

---

## Phase D: 疎通確認

### Task D1: ローカル起動 + /api/health 疎通

**Step 1: PostGIS 起動 + マイグレーション**

```bash
docker compose up -d db
# DB起動を待つ
sleep 5
cd services/backend
DATABASE_URL=postgres://app:devpass@localhost:5432/realestate sqlx migrate run
DATABASE_URL=postgres://app:devpass@localhost:5432/realestate psql -f ../../scripts/seed_dev_data.sql
```

**Step 2: Backend 起動**

```bash
cd services/backend
DATABASE_URL=postgres://app:devpass@localhost:5432/realestate RUST_LOG=info cargo run
```

**Step 3: ヘルスチェック確認**

```bash
curl -s http://localhost:8000/api/health | jq .
```
Expected:
```json
{
  "status": "ok",
  "db_connected": true,
  "reinfolib_key_set": false,
  "version": "0.1.0"
}
```

**Step 4: Frontend 起動**

```bash
cd services/frontend
NEXT_PUBLIC_API_URL=http://localhost:8000 pnpm dev
```

**Step 5: ブラウザ確認**

http://localhost:3000 で "REALESTATE INTELLIGENCE — INITIALIZING..." が表示されること。

---

## Summary

| Phase | Tasks | 成果物 |
|-------|-------|--------|
| A (Backend) | A1-A4 | Rust Axum scaffold + PostGIS migration + Dockerfile |
| B (Frontend) | B1-B3 | Next.js 16 scaffold + CRT theme + Dockerfile |
| C (Infra) | C1 | docker-compose.yml |
| D (疎通) | D1 | /api/health 200 OK |

**並行実行可能**: Phase A と Phase B は独立。Phase C は A+B 完了後。Phase D は全完了後。
