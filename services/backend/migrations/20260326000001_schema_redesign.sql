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
