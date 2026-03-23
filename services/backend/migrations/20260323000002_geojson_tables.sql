-- New tables for GeoJSON datasets: seismic_hazard, railways, liquefaction, stations
-- These complement the 6 tables created in 20260320000001_init.sql.
-- All PKs use bigint GENERATED ALWAYS AS IDENTITY (serial/int4 prohibited per rules)
-- All timestamps use timestamptz (timestamp without tz prohibited)
-- All geometry columns have a GIST index for spatial queries

-- 地震断層帯ハザード (Seismic fault hazard zones from J-SHIS)
CREATE TABLE IF NOT EXISTS seismic_hazard (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    fault_id    text,
    fault_name  text    NOT NULL,
    magnitude   real,
    prob_30y    real,
    geom        geometry(Geometry, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  seismic_hazard            IS '地震断層帯ハザード情報（J-SHIS）';
COMMENT ON COLUMN seismic_hazard.fault_id   IS '断層帯ID（例: F015021_00001）';
COMMENT ON COLUMN seismic_hazard.fault_name IS '断層帯名称（例: 立川断層帯）';
COMMENT ON COLUMN seismic_hazard.magnitude  IS 'マグニチュード（負値は推定値）';
COMMENT ON COLUMN seismic_hazard.prob_30y   IS '30年以内の地震発生確率（平均活動間隔ベース）';
COMMENT ON COLUMN seismic_hazard.geom       IS '断層帯の線形または面形状 (SRID 4326)';

CREATE INDEX IF NOT EXISTS idx_seismic_hazard_geom ON seismic_hazard USING GIST (geom);

-- 鉄道路線 (Railway lines from MLIT N02)
CREATE TABLE IF NOT EXISTS railways (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    railway_type    text,
    line_name       text,
    operator_name   text,
    station_name    text,
    geom            geometry(LineString, 4326) NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  railways               IS '鉄道路線データ（国土数値情報 N02）';
COMMENT ON COLUMN railways.railway_type   IS '鉄道区分（JR・民鉄・地下鉄など）';
COMMENT ON COLUMN railways.line_name      IS '路線名';
COMMENT ON COLUMN railways.operator_name  IS '運営会社名';
COMMENT ON COLUMN railways.station_name   IS '関連駅名（区間の開始駅、NULLの場合あり）';
COMMENT ON COLUMN railways.geom          IS '路線の線形 (SRID 4326)';

CREATE INDEX IF NOT EXISTS idx_railways_geom ON railways USING GIST (geom);

-- 液状化リスク (Liquefaction risk from Tokyo PL map)
CREATE TABLE IF NOT EXISTS liquefaction (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    risk_rank   text    NOT NULL,
    geom        geometry(Point, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  liquefaction            IS '液状化リスク判定（PL値区分）';
COMMENT ON COLUMN liquefaction.risk_rank  IS 'PL値区分（小・中・大・極大）';
COMMENT ON COLUMN liquefaction.geom       IS '判定地点 (SRID 4326)';

CREATE INDEX IF NOT EXISTS idx_liquefaction_geom ON liquefaction USING GIST (geom);

-- 駅 (Stations with passenger data from MLIT S12)
CREATE TABLE IF NOT EXISTS stations (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    station_name    text    NOT NULL,
    station_code    text,
    operator_name   text,
    line_name       text,
    geom            geometry(LineString, 4326) NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE  stations               IS '駅別乗降客数データ（国土数値情報 S12）';
COMMENT ON COLUMN stations.station_name  IS '駅名';
COMMENT ON COLUMN stations.station_code  IS '駅コード';
COMMENT ON COLUMN stations.operator_name IS '運営会社名';
COMMENT ON COLUMN stations.line_name     IS '路線名';
COMMENT ON COLUMN stations.geom         IS '駅の代表線形 (SRID 4326)';

CREATE INDEX IF NOT EXISTS idx_stations_geom ON stations USING GIST (geom);
