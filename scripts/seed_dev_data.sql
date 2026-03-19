-- Development seed data — Tokyo Station area (東京駅周辺)
-- Run after migration: psql $DATABASE_URL -f scripts/seed_dev_data.sql
--
-- Coordinates are [longitude, latitude] per RFC 7946 / PostGIS convention:
--   ST_MakePoint(longitude, latitude)
--   ST_MakeEnvelope(west, south, east, north, 4326)

-- -----------------------------------------------------------------------
-- 地価公示 (Land prices — 5 years of data at key Tokyo locations for Sparkline charts)
-- -----------------------------------------------------------------------
INSERT INTO land_prices (price_per_sqm, address, land_use, year, geom) VALUES
  -- 丸の内 (5-year series for trend chart)
  (1020000, '千代田区丸の内1-1', '商業', 2020, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1050000, '千代田区丸の内1-1', '商業', 2021, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1100000, '千代田区丸の内1-1', '商業', 2022, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1150000, '千代田区丸の内1-1', '商業', 2023, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (1200000, '千代田区丸の内1-1', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  -- 銀座 (latest year only)
  (780000,  '中央区銀座4-6',     '商業', 2024, ST_SetSRID(ST_MakePoint(139.7649, 35.6717), 4326)),
  -- 新橋
  (620000,  '港区新橋2-1',       '商業', 2024, ST_SetSRID(ST_MakePoint(139.7586, 35.6660), 4326)),
  -- 神田
  (450000,  '千代田区神田1-2',   '住居', 2024, ST_SetSRID(ST_MakePoint(139.7700, 35.6925), 4326)),
  -- 上野
  (380000,  '台東区上野1-1',     '商業', 2024, ST_SetSRID(ST_MakePoint(139.7745, 35.7135), 4326));

-- -----------------------------------------------------------------------
-- 用途地域 (Zoning districts)
-- -----------------------------------------------------------------------
INSERT INTO zoning (zone_type, zone_code, floor_area_ratio, building_coverage, geom) VALUES
  ('商業地域',         '09', 8.0, 0.8,
   ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.76 35.68, 139.77 35.68, 139.77 35.69, 139.76 35.69, 139.76 35.68)))'), 4326)),
  ('第一種住居地域',   '05', 3.0, 0.6,
   ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.77 35.69, 139.78 35.69, 139.78 35.70, 139.77 35.70, 139.77 35.69)))'), 4326)),
  ('近隣商業地域',     '08', 4.0, 0.8,
   ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.75 35.66, 139.76 35.66, 139.76 35.67, 139.75 35.67, 139.75 35.66)))'), 4326));

-- -----------------------------------------------------------------------
-- 洪水浸水想定区域 (Flood risk zones)
-- -----------------------------------------------------------------------
INSERT INTO flood_risk (depth_rank, river_name, geom) VALUES
  ('0.5-3.0m', '荒川',
   ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.78 35.70, 139.80 35.70, 139.80 35.72, 139.78 35.72, 139.78 35.70)))'), 4326)),
  ('0-0.5m',   '隅田川',
   ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.79 35.69, 139.80 35.69, 139.80 35.70, 139.79 35.70, 139.79 35.69)))'), 4326));

-- -----------------------------------------------------------------------
-- 急傾斜地崩壊危険区域 (Steep slope zones)
-- -----------------------------------------------------------------------
INSERT INTO steep_slope (area_name, geom) VALUES
  ('文京区目白台',
   ST_SetSRID(ST_GeomFromText('MULTIPOLYGON(((139.72 35.72, 139.73 35.72, 139.73 35.73, 139.72 35.73, 139.72 35.72)))'), 4326));

-- -----------------------------------------------------------------------
-- 学校 (Schools)
-- -----------------------------------------------------------------------
INSERT INTO schools (name, school_type, geom) VALUES
  ('千代田区立麹町小学校',     '小学校',   ST_SetSRID(ST_MakePoint(139.7401, 35.6841), 4326)),
  ('千代田区立神田一橋中学校', '中学校',   ST_SetSRID(ST_MakePoint(139.7611, 35.6927), 4326)),
  ('東京都立日比谷高等学校',   '高等学校', ST_SetSRID(ST_MakePoint(139.7520, 35.6719), 4326));

-- -----------------------------------------------------------------------
-- 医療機関 (Medical facilities)
-- -----------------------------------------------------------------------
INSERT INTO medical_facilities (name, facility_type, bed_count, geom) VALUES
  ('聖路加国際病院',         '病院',   520, ST_SetSRID(ST_MakePoint(139.7722, 35.6697), 4326)),
  ('東京逓信病院',           '病院',   461, ST_SetSRID(ST_MakePoint(139.7543, 35.6943), 4326)),
  ('三井記念病院',           '病院',   482, ST_SetSRID(ST_MakePoint(139.7785, 35.6930), 4326)),
  ('神田クリニック',         '診療所',   0, ST_SetSRID(ST_MakePoint(139.7705, 35.6940), 4326)),
  ('銀座メディカルセンター', '診療所',   0, ST_SetSRID(ST_MakePoint(139.7645, 35.6720), 4326));
