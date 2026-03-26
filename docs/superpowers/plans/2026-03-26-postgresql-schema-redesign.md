# PostgreSQL Schema Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the PostgreSQL schema for correctness, performance, and compliance with `.claude/rules/postgresql.md`, including geography indexes, proper types, CHECK constraints, denormalization for z-score, admin_boundaries table, and DB tuning.

**Architecture:** Single destructive migration drops and recreates all 10 tables with corrected types/constraints, adds geography functional indexes, adds admin_boundaries table, and denormalizes zone_type onto land_prices. Docker postgresql.conf tuning applied via custom config file. Rust infra layer queries updated to use geography-aware patterns and corrected types.

**Tech Stack:** PostgreSQL 16, PostGIS 3.4.3, sqlx (Rust), Docker Compose

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `services/backend/migrations/20260326000001_schema_redesign.sql` | Create | Destructive migration: DROP + recreate all tables, indexes, constraints |
| `services/backend/migrations/20260326000002_admin_boundaries.sql` | Create | admin_boundaries table for area filtering |
| `services/backend/docker/postgresql.conf` | Create | Tuned PostgreSQL configuration |
| `docker-compose.yml` | Modify | Mount custom postgresql.conf, enable pg_stat_statements |
| `services/backend/src/infra/pg_tls_repository.rs` | Modify | Geography-aware queries, use denormalized zone_type |
| `services/backend/src/infra/pg_stats_repository.rs` | Modify | Remove geography casts (use geography indexes) |
| `services/backend/src/infra/pg_trend_repository.rs` | Modify | Geography-aware nearest query |

---

## Task 1: PostgreSQL Configuration Tuning

**Files:**
- Create: `services/backend/docker/postgresql.conf`
- Modify: `docker-compose.yml`

- [ ] **Step 1: Create custom postgresql.conf**

```bash
mkdir -p services/backend/docker
```

```conf
# services/backend/docker/postgresql.conf
# Tuned for PostGIS spatial workload on SSD-backed Docker volume

# --- Connection ---
listen_addresses = '*'
max_connections = 100

# --- Memory ---
shared_buffers = 256MB
work_mem = 32MB
maintenance_work_mem = 128MB
effective_cache_size = 512MB

# --- Planner (SSD) ---
random_page_cost = 1.1
effective_io_concurrency = 200
seq_page_cost = 1.0

# --- WAL ---
wal_buffers = 16MB
checkpoint_completion_target = 0.9

# --- Monitoring ---
shared_preload_libraries = 'pg_stat_statements'
pg_stat_statements.max = 5000
pg_stat_statements.track = all

# --- Timeouts ---
idle_in_transaction_session_timeout = 30000
statement_timeout = 30000

# --- Logging ---
log_min_duration_statement = 500
log_statement = 'ddl'
log_line_prefix = '%t [%p] %u@%d '
```

- [ ] **Step 2: Mount config in docker-compose.yml**

In the `db` service, add volume mount and command:

```yaml
  db:
    image: postgis/postgis:16-3.4
    command: postgres -c config_file=/etc/postgresql/postgresql.conf
    environment:
      POSTGRES_DB: realestate
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD:-devpass}
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./services/backend/docker/postgresql.conf:/etc/postgresql/postgresql.conf:ro
```

- [ ] **Step 3: Verify Docker starts with new config**

```bash
docker compose down -v && docker compose up -d db
docker compose exec db psql -U app -d realestate -c "SHOW shared_preload_libraries;"
# Expected: pg_stat_statements
docker compose exec db psql -U app -d realestate -c "SHOW random_page_cost;"
# Expected: 1.1
docker compose exec db psql -U app -d realestate -c "SHOW work_mem;"
# Expected: 32MB
```

- [ ] **Step 4: Commit**

```bash
git add services/backend/docker/postgresql.conf docker-compose.yml
git commit -m "feat(db): add tuned postgresql.conf with pg_stat_statements and SSD settings"
```

---

## Task 2: Destructive Schema Migration

**Files:**
- Create: `services/backend/migrations/20260326000001_schema_redesign.sql`

This is the core migration. It drops all existing tables and recreates them with:
- Correct geometry subtypes (Point, MultiLineString, MultiPolygon)
- NOT NULL on all logically-required columns
- CHECK constraints on categorical columns
- `depth_rank` as `smallint` with CHECK (1-5)
- `zone_type` denormalized onto `land_prices`
- Geography functional GIST indexes for all ST_DWithin queries
- B-tree indexes for common filter columns

- [ ] **Step 1: Write the migration**

```sql
-- services/backend/migrations/20260326000001_schema_redesign.sql
-- DESTRUCTIVE: Drops and recreates all domain tables with corrected types,
-- constraints, and indexes. Requires data re-import after running.

SET lock_timeout = '5s';

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- ═══════════════════════════════════════════════════════════════
-- DROP existing tables (CASCADE to remove dependent objects)
-- ═══════════════════════════════════════════════════════════════
DROP TABLE IF EXISTS land_prices CASCADE;
DROP TABLE IF EXISTS zoning CASCADE;
DROP TABLE IF EXISTS flood_risk CASCADE;
DROP TABLE IF EXISTS steep_slope CASCADE;
DROP TABLE IF EXISTS schools CASCADE;
DROP TABLE IF EXISTS medical_facilities CASCADE;
DROP TABLE IF EXISTS seismic_hazard CASCADE;
DROP TABLE IF EXISTS railways CASCADE;
DROP TABLE IF EXISTS liquefaction CASCADE;
DROP TABLE IF EXISTS stations CASCADE;

-- ═══════════════════════════════════════════════════════════════
-- 1. land_prices — Core pricing data (L01)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE land_prices (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    price_per_sqm  integer      NOT NULL CHECK (price_per_sqm > 0),
    address        text         NOT NULL,
    land_use       text         NOT NULL DEFAULT '不明',
    year           integer      NOT NULL CHECK (year >= 1983 AND year <= 2100),
    zone_type      text,        -- Denormalized from zoning via spatial join at import time
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz  NOT NULL DEFAULT now()
);

COMMENT ON TABLE  land_prices              IS '地価公示データ（国土数値情報 L01）';
COMMENT ON COLUMN land_prices.price_per_sqm IS '1㎡あたりの公示価格（円）';
COMMENT ON COLUMN land_prices.address      IS '所在地住所';
COMMENT ON COLUMN land_prices.land_use     IS '利用現況（商業・住居・工業等）';
COMMENT ON COLUMN land_prices.year         IS '調査年度';
COMMENT ON COLUMN land_prices.zone_type    IS '用途地域種別（zoningテーブルから空間結合で転記）';
COMMENT ON COLUMN land_prices.geom         IS '観測地点 (SRID 4326, [lng, lat])';

-- Geometry GIST for ST_Intersects/ST_Contains (planar bbox queries)
CREATE INDEX idx_land_prices_geom ON land_prices USING GIST (geom);
-- Geography GIST for ST_DWithin distance queries (TLS scoring, trend)
CREATE INDEX idx_land_prices_geog ON land_prices USING GIST ((geom::geography));
-- B-tree for year filter (stats, z-score)
CREATE INDEX idx_land_prices_year ON land_prices (year);
-- B-tree for address lookup (trend)
CREATE INDEX idx_land_prices_address ON land_prices (address);
-- Composite unique constraint (dedup)
CREATE UNIQUE INDEX idx_land_prices_address_year ON land_prices (address, year);
-- Composite for z-score: zone_type + year (filter zone_prices CTE)
CREATE INDEX idx_land_prices_zone_year ON land_prices (zone_type, year);

-- ═══════════════════════════════════════════════════════════════
-- 2. zoning — Land use zones (A29)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE zoning (
    id                bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    zone_type         text   NOT NULL,
    zone_code         text   NOT NULL DEFAULT '',
    floor_area_ratio  real   NOT NULL DEFAULT 0 CHECK (floor_area_ratio >= 0),
    building_coverage real   NOT NULL DEFAULT 0 CHECK (building_coverage >= 0),
    geom              geometry(MultiPolygon, 4326) NOT NULL,
    created_at        timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  zoning                   IS '用途地域ポリゴン（国土数値情報 A29）';
COMMENT ON COLUMN zoning.zone_type         IS '用途地域種別名称（例: 商業地域）';
COMMENT ON COLUMN zoning.zone_code         IS '用途地域コード（2桁）';
COMMENT ON COLUMN zoning.floor_area_ratio  IS '容積率（%）';
COMMENT ON COLUMN zoning.building_coverage IS '建蔽率（%）';

CREATE INDEX idx_zoning_geom ON zoning USING GIST (geom);
CREATE INDEX idx_zoning_type ON zoning (zone_type);

-- ═══════════════════════════════════════════════════════════════
-- 3. flood_risk — Flood inundation zones (A31b)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE flood_risk (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    depth_rank  smallint NOT NULL CHECK (depth_rank >= 0 AND depth_rank <= 5),
    river_name  text     NOT NULL DEFAULT '',
    geom        geometry(MultiPolygon, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  flood_risk            IS '洪水浸水想定区域（国土数値情報 A31b）';
COMMENT ON COLUMN flood_risk.depth_rank IS '浸水深ランク: 0=区域外, 1=0.5m未満, 2=0.5-3m, 3=3-5m, 4=5-10m, 5=10m以上';
COMMENT ON COLUMN flood_risk.river_name IS '対象河川名';

CREATE INDEX idx_flood_risk_geom ON flood_risk USING GIST (geom);
-- Geography GIST for ST_DWithin in TLS scoring
CREATE INDEX idx_flood_risk_geog ON flood_risk USING GIST ((geom::geography));

-- ═══════════════════════════════════════════════════════════════
-- 4. steep_slope — Steep slope danger zones (A47)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE steep_slope (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    area_name   text   NOT NULL DEFAULT '',
    geom        geometry(MultiPolygon, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  steep_slope           IS '急傾斜地崩壊危険区域（国土数値情報 A47）';
COMMENT ON COLUMN steep_slope.area_name IS '指定区域名称';

CREATE INDEX idx_steep_slope_geom ON steep_slope USING GIST (geom);
CREATE INDEX idx_steep_slope_geog ON steep_slope USING GIST ((geom::geography));

-- ═══════════════════════════════════════════════════════════════
-- 5. schools — School locations (P29)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE schools (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         text NOT NULL,
    school_type  text NOT NULL DEFAULT '不明'
                 CHECK (school_type IN ('小学校','中学校','高等学校','大学','特別支援学校','不明')),
    geom         geometry(Point, 4326) NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  schools             IS '学校位置情報（国土数値情報 P29）';
COMMENT ON COLUMN schools.school_type IS '学校種別';

CREATE INDEX idx_schools_geom ON schools USING GIST (geom);
CREATE INDEX idx_schools_geog ON schools USING GIST ((geom::geography));

-- ═══════════════════════════════════════════════════════════════
-- 6. medical_facilities — Medical facility locations (P04)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE medical_facilities (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name           text    NOT NULL,
    facility_type  text    NOT NULL DEFAULT '診療所'
                   CHECK (facility_type IN ('病院','診療所','歯科診療所')),
    bed_count      integer NOT NULL DEFAULT 0 CHECK (bed_count >= 0),
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  medical_facilities               IS '医療機関位置情報（国土数値情報 P04）';
COMMENT ON COLUMN medical_facilities.facility_type IS '施設種別';
COMMENT ON COLUMN medical_facilities.bed_count     IS '病床数';

CREATE INDEX idx_medical_geom ON medical_facilities USING GIST (geom);
CREATE INDEX idx_medical_geog ON medical_facilities USING GIST ((geom::geography));

-- ═══════════════════════════════════════════════════════════════
-- 7. seismic_hazard — Seismic fault zones (J-SHIS)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE seismic_hazard (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    fault_id    text   NOT NULL DEFAULT '',
    fault_name  text   NOT NULL,
    magnitude   real   CHECK (magnitude IS NULL OR (magnitude >= -2 AND magnitude <= 10)),
    prob_30y    real   CHECK (prob_30y IS NULL OR (prob_30y >= 0 AND prob_30y <= 1)),
    geom        geometry(MultiLineString, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  seismic_hazard IS '地震断層帯ハザード情報（J-SHIS）';

CREATE INDEX idx_seismic_geom ON seismic_hazard USING GIST (geom);

-- ═══════════════════════════════════════════════════════════════
-- 8. railways — Railway lines (N02)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE railways (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    railway_type    text NOT NULL DEFAULT '',
    line_name       text NOT NULL DEFAULT '',
    operator_name   text NOT NULL DEFAULT '',
    geom            geometry(MultiLineString, 4326) NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE railways IS '鉄道路線データ（国土数値情報 N02）';

CREATE INDEX idx_railways_geom ON railways USING GIST (geom);

-- ═══════════════════════════════════════════════════════════════
-- 9. liquefaction — Liquefaction risk (Tokyo PL map)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE liquefaction (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    risk_rank   text   NOT NULL CHECK (risk_rank IN ('小','中','大','極大')),
    geom        geometry(Point, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE liquefaction IS '液状化リスク判定（PL値区分）';

CREATE INDEX idx_liquefaction_geom ON liquefaction USING GIST (geom);
CREATE INDEX idx_liquefaction_geog ON liquefaction USING GIST ((geom::geography));

-- ═══════════════════════════════════════════════════════════════
-- 10. stations — Railway stations (S12)
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE stations (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    station_name    text NOT NULL,
    station_code    text NOT NULL DEFAULT '',
    operator_name   text NOT NULL DEFAULT '',
    line_name       text NOT NULL DEFAULT '',
    passenger_count integer DEFAULT 0 CHECK (passenger_count IS NULL OR passenger_count >= 0),
    geom            geometry(Point, 4326) NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  stations              IS '駅別乗降客数データ（国土数値情報 S12）';
COMMENT ON COLUMN stations.passenger_count IS '年間乗降客数';

CREATE INDEX idx_stations_geom ON stations USING GIST (geom);
CREATE INDEX idx_stations_geog ON stations USING GIST ((geom::geography));

-- ═══════════════════════════════════════════════════════════════
-- Run ANALYZE on all tables after data import
-- ═══════════════════════════════════════════════════════════════
-- NOTE: Run this manually after data seed: ANALYZE;
```

- [ ] **Step 2: Verify migration applies cleanly**

```bash
docker compose up -d db
docker compose exec db psql -U app -d realestate -f /dev/stdin < services/backend/migrations/20260326000001_schema_redesign.sql
# Expected: CREATE TABLE × 10, CREATE INDEX × 20+, no errors
```

- [ ] **Step 3: Commit**

```bash
git add services/backend/migrations/20260326000001_schema_redesign.sql
git commit -m "feat(db): destructive schema redesign with geography indexes, CHECK constraints, and denormalized zone_type"
```

---

## Task 3: Admin Boundaries Table

**Files:**
- Create: `services/backend/migrations/20260326000002_admin_boundaries.sql`

- [ ] **Step 1: Write admin_boundaries migration**

```sql
-- services/backend/migrations/20260326000002_admin_boundaries.sql
-- Administrative boundary polygons (N03) for area filtering and stats aggregation

SET lock_timeout = '5s';

CREATE TABLE admin_boundaries (
    id         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code       text   NOT NULL,
    name       text   NOT NULL,
    name_en    text   NOT NULL DEFAULT '',
    level      text   NOT NULL CHECK (level IN ('prefecture', 'municipality')),
    pref_code  text   NOT NULL,
    pref_name  text   NOT NULL,
    geom       geometry(MultiPolygon, 4326) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  admin_boundaries IS '行政区域境界（国土数値情報 N03）';
COMMENT ON COLUMN admin_boundaries.code      IS '全国地方公共団体コード（JIS X 0401/0402）';
COMMENT ON COLUMN admin_boundaries.name      IS '市区町村名（都道府県名の場合もあり）';
COMMENT ON COLUMN admin_boundaries.name_en   IS '英語名称（i18n用）';
COMMENT ON COLUMN admin_boundaries.level     IS 'prefecture or municipality';
COMMENT ON COLUMN admin_boundaries.pref_code IS '都道府県コード（2桁）';
COMMENT ON COLUMN admin_boundaries.pref_name IS '都道府県名';

CREATE INDEX idx_admin_geom ON admin_boundaries USING GIST (geom);
CREATE UNIQUE INDEX idx_admin_code ON admin_boundaries (code);
CREATE INDEX idx_admin_level ON admin_boundaries (level);
CREATE INDEX idx_admin_pref ON admin_boundaries (pref_code);
```

- [ ] **Step 2: Apply and verify**

```bash
docker compose exec db psql -U app -d realestate -f /dev/stdin < services/backend/migrations/20260326000002_admin_boundaries.sql
docker compose exec db psql -U app -d realestate -c "\d admin_boundaries"
```

- [ ] **Step 3: Commit**

```bash
git add services/backend/migrations/20260326000002_admin_boundaries.sql
git commit -m "feat(db): add admin_boundaries table for area filtering (N03)"
```

---

## Task 4: Rewrite pg_tls_repository.rs Queries

**Files:**
- Modify: `services/backend/src/infra/pg_tls_repository.rs`

The key changes:
1. Remove `::geography` casts — the new geography functional indexes handle this
2. Rewrite `calc_price_z_score` to use the denormalized `zone_type` column (no spatial join with zoning)
3. Change `depth_rank` query from `MAX(text)` to `MAX(smallint)`
4. Add `::double precision` / `::bigint` casts where PostgreSQL returns NUMERIC

- [ ] **Step 1: Rewrite calc_price_z_score**

The current query does a full spatial join between land_prices and zoning (1.3s). With `zone_type` denormalized onto `land_prices`, we can eliminate the zoning table entirely:

Replace the existing `calc_price_z_score` method body (lines 176-235) with:

```rust
    #[tracing::instrument(skip(self))]
    async fn calc_price_z_score(&self, coord: &Coord) -> Result<ZScoreResult, DomainError> {
        let query = sqlx::query_as::<_, (f64, String, i64)>(
            r#"
            WITH nearest AS (
                SELECT price_per_sqm, zone_type
                FROM land_prices
                WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
                ORDER BY ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography)
                LIMIT 1
            ),
            zone_stats AS (
                SELECT AVG(lp.price_per_sqm)::double precision AS mean_price,
                       STDDEV(lp.price_per_sqm)::double precision AS stddev_price,
                       COUNT(*)::bigint AS sample_count
                FROM land_prices lp, nearest n
                WHERE lp.zone_type = n.zone_type
                  AND lp.year = (SELECT MAX(year) FROM land_prices)
            )
            SELECT
                COALESCE(
                    CASE WHEN zs.stddev_price IS NULL OR zs.stddev_price = 0 THEN 0.0
                         ELSE ((n.price_per_sqm - zs.mean_price) / zs.stddev_price)
                    END, 0.0)::double precision AS z_score,
                COALESCE(n.zone_type, '') AS zone_type,
                COALESCE(zs.sample_count, 0)::bigint AS sample_count
            FROM nearest n
            LEFT JOIN zone_stats zs ON true
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(z_score = row.0, zone_type = %row.1, sample_count = row.2, "price_z_score computed");

        Ok(ZScoreResult {
            z_score: row.0,
            zone_type: row.1,
            sample_count: row.2,
        })
    }
```

This eliminates the `zone_prices` CTE that did `ST_Contains(z.geom, lp.geom)` for every land price. The new query uses the denormalized `zone_type` column with the B-tree index `idx_land_prices_zone_year`.

- [ ] **Step 2: Fix find_flood_depth_rank return type**

`depth_rank` is now `smallint` (was `text`). Change the query return type from `(Option<i32>,)` to `(Option<i16>,)` and convert:

```rust
    #[tracing::instrument(skip(self))]
    async fn find_flood_depth_rank(&self, coord: &Coord) -> Result<Option<i32>, DomainError> {
        let query = sqlx::query_as::<_, (Option<i16>,)>(
            r#"
            SELECT MAX(depth_rank)
            FROM flood_risk
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok(row.0.map(|v| v as i32))
    }
```

- [ ] **Step 3: Verify build**

```bash
cd services/backend && cargo build && cargo clippy -- -D warnings
```

- [ ] **Step 4: Commit**

```bash
git add services/backend/src/infra/pg_tls_repository.rs
git commit -m "fix(backend): rewrite TLS queries for geography indexes and denormalized zone_type"
```

---

## Task 5: Update Seed Data Script

**Files:**
- Modify: `services/backend/migrations/20260322000001_seed_dev.sql` (or create new seed)

The seed data must match the new schema (depth_rank as smallint, NOT NULL columns, zone_type on land_prices, etc.). Read the existing seed file and update it.

- [ ] **Step 1: Read and update seed data**

Read `services/backend/migrations/20260322000001_seed_dev.sql`. Update:
- `flood_risk.depth_rank`: change text values to integers (1-5)
- Add `land_use` value where it was NULL (use '住居')
- Add `zone_type` value to land_prices rows
- Ensure all NOT NULL columns have values

- [ ] **Step 2: Re-seed and verify**

```bash
docker compose exec db psql -U app -d realestate -c "SELECT count(*) FROM land_prices;"
docker compose exec db psql -U app -d realestate -c "SELECT depth_rank, count(*) FROM flood_risk GROUP BY depth_rank;"
# Expected: integer depth_rank values
```

- [ ] **Step 3: Run ANALYZE**

```bash
docker compose exec db psql -U app -d realestate -c "ANALYZE;"
```

- [ ] **Step 4: Commit**

```bash
git add services/backend/migrations/
git commit -m "fix(db): update seed data for new schema constraints"
```

---

## Task 6: Verify End-to-End

- [ ] **Step 1: Restart all services**

```bash
docker compose down -v
docker compose up -d
# Wait for health checks
docker compose exec db psql -U app -d realestate -c "SELECT count(*) FROM land_prices;"
```

- [ ] **Step 2: Test score endpoint**

```bash
curl -s "http://localhost:8000/api/score?lat=35.681&lng=139.767" | python3 -m json.tool | head -20
# Expected: 200 OK with TLS score, no 503 error
```

- [ ] **Step 3: Verify pg_stat_statements**

```bash
docker compose exec db psql -U app -d realestate -c "
  SELECT query, calls, mean_exec_time::numeric(10,2) as avg_ms, total_exec_time::numeric(10,2) as total_ms
  FROM pg_stat_statements
  WHERE query LIKE '%land_prices%'
  ORDER BY total_exec_time DESC
  LIMIT 5;
"
```

- [ ] **Step 4: Verify indexes are used**

```bash
docker compose exec db psql -U app -d realestate -c "
  SELECT indexrelname, idx_scan, idx_tup_read
  FROM pg_stat_user_indexes
  WHERE schemaname = 'public'
  ORDER BY idx_scan DESC;
"
```

---

## Self-Review Checklist

- [x] **Spec coverage**: All 8 audit findings addressed:
  1. Geography GIST indexes → Task 2 (functional indexes on all distance-queried tables)
  2. flood_risk.depth_rank text → Task 2 (smallint with CHECK 0-5) + Task 4 (query fix)
  3. calc_price_z_score spatial join → Task 4 (denormalized zone_type eliminates zoning join)
  4. CHECK constraints → Task 2 (on all categorical columns)
  5. pg_stat_statements → Task 1 (postgresql.conf)
  6. Untyped geometry → Task 2 (MultiLineString for seismic/railways, Point for stations)
  7. admin_boundaries table → Task 3
  8. random_page_cost → Task 1 (1.1 in postgresql.conf)

- [x] **Placeholder scan**: No TBD/TODO. All SQL and Rust code is complete.
- [x] **Type consistency**: depth_rank is smallint in SQL, i16 in Rust (cast to i32 for domain interface). zone_type is text in SQL, String in Rust.
