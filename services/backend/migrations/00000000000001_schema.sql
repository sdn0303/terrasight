-- Enable PostGIS
CREATE EXTENSION IF NOT EXISTS postgis;

-- ============================================================
-- Administrative boundaries (L1/L2 hierarchy)
-- ============================================================
CREATE TABLE admin_boundaries (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    level          text   NOT NULL CHECK (level IN ('prefecture', 'municipality')),
    pref_code      text   NOT NULL,
    pref_name      text   NOT NULL,
    city_code      text,
    city_name      text,
    admin_code     text   NOT NULL,
    geom           geometry(MultiPolygon, 4326) NOT NULL,
    area_sqm       double precision,
    created_at     timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE admin_boundaries IS '行政区画マスタ（都道府県 + 市区町村）';
COMMENT ON COLUMN admin_boundaries.pref_code IS '都道府県コード（2桁 zero-padded）';
COMMENT ON COLUMN admin_boundaries.level IS 'prefecture or municipality';

CREATE INDEX idx_admin_geom ON admin_boundaries USING gist (geom);
CREATE INDEX idx_admin_pref ON admin_boundaries (pref_code);
CREATE INDEX idx_admin_level_pref ON admin_boundaries (level, pref_code);
CREATE INDEX idx_admin_code ON admin_boundaries (admin_code);

-- ============================================================
-- Land prices (L01 公示地価)
-- ============================================================
CREATE TABLE land_prices (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    address        text   NOT NULL,
    price_per_sqm  integer NOT NULL CHECK (price_per_sqm >= 0),
    land_use       text,
    zone_type      text,
    survey_year    smallint NOT NULL CHECK (survey_year BETWEEN 2000 AND 2100),
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE land_prices IS '地価公示・都道府県地価調査';

CREATE INDEX idx_lp_geom ON land_prices USING gist (geom);
CREATE INDEX idx_lp_pref_year ON land_prices (pref_code, survey_year);
CREATE UNIQUE INDEX idx_lp_addr_year ON land_prices (address, survey_year);

-- ============================================================
-- Zoning (A29 用途地域)
-- ============================================================
CREATE TABLE zoning (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    zone_code      text   NOT NULL,
    zone_type      text   NOT NULL,
    floor_area_ratio double precision,
    building_coverage double precision,
    geom           geometry(MultiPolygon, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE zoning IS '用途地域';

CREATE INDEX idx_zoning_geom ON zoning USING gist (geom);
CREATE INDEX idx_zoning_pref ON zoning (pref_code);

-- ============================================================
-- Flood risk (A31b 洪水浸水想定)
-- ============================================================
CREATE TABLE flood_risk (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    depth_rank     text,
    river_name     text,
    geom           geometry(MultiPolygon, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_flood_geom ON flood_risk USING gist (geom);
CREATE INDEX idx_flood_pref ON flood_risk (pref_code);

-- ============================================================
-- Steep slope (A47 急傾斜地)
-- ============================================================
CREATE TABLE steep_slope (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    area_name      text,
    geom           geometry(MultiPolygon, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_steep_geom ON steep_slope USING gist (geom);
CREATE INDEX idx_steep_pref ON steep_slope (pref_code);

-- ============================================================
-- Schools (P29)
-- ============================================================
CREATE TABLE schools (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    school_name    text   NOT NULL,
    school_type    text   NOT NULL,
    address        text,
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_school_geom ON schools USING gist (geom);
CREATE INDEX idx_school_pref ON schools (pref_code);

-- ============================================================
-- Medical facilities (P04)
-- ============================================================
CREATE TABLE medical_facilities (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    facility_name  text   NOT NULL,
    facility_type  text   NOT NULL,
    beds           integer,
    address        text,
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_medical_geom ON medical_facilities USING gist (geom);
CREATE INDEX idx_medical_pref ON medical_facilities (pref_code);

-- ============================================================
-- Stations (S12)
-- ============================================================
CREATE TABLE stations (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    station_name   text   NOT NULL,
    station_code   text,
    line_name      text,
    operator_name  text,
    passenger_count integer,
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_station_geom ON stations USING gist (geom);
CREATE INDEX idx_station_pref ON stations (pref_code);

-- ============================================================
-- Railways (N02)
-- ============================================================
CREATE TABLE railways (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    line_name      text   NOT NULL,
    operator_name  text,
    railway_type   text,
    geom           geometry(MultiLineString, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_railway_geom ON railways USING gist (geom);
CREATE INDEX idx_railway_pref ON railways (pref_code);

-- ============================================================
-- Seismic hazard (J-SHIS)
-- ============================================================
CREATE TABLE seismic_hazard (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    hazard_level   text   NOT NULL,
    probability    double precision,
    geom           geometry(MultiLineString, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_seismic_geom ON seismic_hazard USING gist (geom);
CREATE INDEX idx_seismic_pref ON seismic_hazard (pref_code);

-- ============================================================
-- Liquefaction risk (PL)
-- ============================================================
CREATE TABLE liquefaction (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code      text   NOT NULL,
    pl_rank        text   NOT NULL,
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_liq_geom ON liquefaction USING gist (geom);
CREATE INDEX idx_liq_pref ON liquefaction (pref_code);
