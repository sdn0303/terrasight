-- Initial schema: PostGIS extension + 6 domain tables
-- All PKs use bigint GENERATED ALWAYS AS IDENTITY (serial/int4 prohibited per rules)
-- All timestamps use timestamptz (timestamp without tz prohibited)
-- All geometry columns have a GIST index for spatial queries

CREATE EXTENSION IF NOT EXISTS postgis;

-- 地価公示 (Land prices from MLIT National Land Value Survey)
CREATE TABLE land_prices (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    price_per_sqm  integer      NOT NULL,
    address        text         NOT NULL,
    land_use       text,
    year           integer      NOT NULL,
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz  NOT NULL DEFAULT now()
);

COMMENT ON TABLE  land_prices              IS '地価公示データ（国土数値情報）';
COMMENT ON COLUMN land_prices.price_per_sqm IS '1平方メートルあたりの公示価格（円）';
COMMENT ON COLUMN land_prices.land_use      IS '地目（商業・住居など）';
COMMENT ON COLUMN land_prices.geom          IS '観測地点座標 (SRID 4326, longitude/latitude)';

CREATE INDEX idx_land_prices_geom ON land_prices USING GIST (geom);
CREATE INDEX idx_land_prices_year ON land_prices (year);

-- 用途地域 (Zoning districts)
CREATE TABLE zoning (
    id                bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    zone_type         text   NOT NULL,
    zone_code         text,
    floor_area_ratio  real,
    building_coverage real,
    geom              geometry(MultiPolygon, 4326) NOT NULL,
    created_at        timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  zoning                  IS '用途地域ポリゴン（国土数値情報）';
COMMENT ON COLUMN zoning.zone_type        IS '用途地域種別名称（例: 商業地域）';
COMMENT ON COLUMN zoning.zone_code        IS '用途地域コード（2桁）';
COMMENT ON COLUMN zoning.floor_area_ratio IS '容積率';
COMMENT ON COLUMN zoning.building_coverage IS '建蔽率';

CREATE INDEX idx_zoning_geom ON zoning USING GIST (geom);

-- 洪水浸水想定区域 (Flood inundation risk zones)
CREATE TABLE flood_risk (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    depth_rank  text,
    river_name  text,
    geom        geometry(MultiPolygon, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  flood_risk            IS '洪水浸水想定区域（国土数値情報）';
COMMENT ON COLUMN flood_risk.depth_rank IS '浸水深ランク（例: 0.5-3.0m）';
COMMENT ON COLUMN flood_risk.river_name IS '対象河川名';

CREATE INDEX idx_flood_risk_geom ON flood_risk USING GIST (geom);

-- 急傾斜地崩壊危険区域 (Steep slope collapse danger zones)
CREATE TABLE steep_slope (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    area_name   text,
    geom        geometry(MultiPolygon, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  steep_slope           IS '急傾斜地崩壊危険区域（国土数値情報）';
COMMENT ON COLUMN steep_slope.area_name IS '指定区域名称';

CREATE INDEX idx_steep_slope_geom ON steep_slope USING GIST (geom);

-- 学校 (Schools)
CREATE TABLE schools (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         text NOT NULL,
    school_type  text,
    geom         geometry(Point, 4326) NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  schools             IS '学校位置情報（国土数値情報）';
COMMENT ON COLUMN schools.school_type IS '学校種別（小学校・中学校・高等学校など）';

CREATE INDEX idx_schools_geom ON schools USING GIST (geom);

-- 医療機関 (Medical facilities)
CREATE TABLE medical_facilities (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name           text    NOT NULL,
    facility_type  text,
    bed_count      integer,
    geom           geometry(Point, 4326) NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  medical_facilities               IS '医療機関位置情報（国土数値情報）';
COMMENT ON COLUMN medical_facilities.facility_type IS '施設種別（病院・診療所など）';
COMMENT ON COLUMN medical_facilities.bed_count     IS '病床数（診療所は0）';

CREATE INDEX idx_medical_facilities_geom ON medical_facilities USING GIST (geom);
