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
