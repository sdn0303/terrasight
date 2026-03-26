-- Dev seed data: minimal sample around Tokyo Station (東京駅) for local development.
-- Run with: psql $DATABASE_URL -f migrations/20260322000001_seed_dev.sql
--
-- This is NOT a migration — it is idempotent INSERT data for development only.
-- Each INSERT uses ON CONFLICT DO NOTHING (via a guard subquery) to be re-runnable.

-- ============================================================
-- 地価公示 (Land Prices) — 5 years × 3 locations = 15 rows
-- Source: 国土数値情報 L01 (MLIT National Land Value Survey)
-- ============================================================
INSERT INTO land_prices (price_per_sqm, address, land_use, zone_type, year, geom)
SELECT v.price, v.addr, v.use_type, v.zone, v.yr, ST_SetSRID(ST_MakePoint(v.lng, v.lat), 4326)
FROM (VALUES
  -- 丸の内 (Marunouchi) — prime commercial
  (5200000, '千代田区丸の内1-9-1',  '商業', '商業地域',     2024, 139.76714, 35.68123),
  (5000000, '千代田区丸の内1-9-1',  '商業', '商業地域',     2023, 139.76714, 35.68123),
  (4800000, '千代田区丸の内1-9-1',  '商業', '商業地域',     2022, 139.76714, 35.68123),
  (4500000, '千代田区丸の内1-9-1',  '商業', '商業地域',     2021, 139.76714, 35.68123),
  (4600000, '千代田区丸の内1-9-1',  '商業', '商業地域',     2020, 139.76714, 35.68123),
  -- 銀座 (Ginza) — high-end commercial
  (4100000, '中央区銀座4-5-6',      '商業', '商業地域',     2024, 139.76493, 35.67170),
  (3900000, '中央区銀座4-5-6',      '商業', '商業地域',     2023, 139.76493, 35.67170),
  (3700000, '中央区銀座4-5-6',      '商業', '商業地域',     2022, 139.76493, 35.67170),
  (3500000, '中央区銀座4-5-6',      '商業', '商業地域',     2021, 139.76493, 35.67170),
  (3600000, '中央区銀座4-5-6',      '商業', '商業地域',     2020, 139.76493, 35.67170),
  -- 神田 (Kanda) — mixed residential/commercial
  (1200000, '千代田区神田須田町1-2', '住居', '近隣商業地域', 2024, 139.77120, 35.69410),
  (1150000, '千代田区神田須田町1-2', '住居', '近隣商業地域', 2023, 139.77120, 35.69410),
  (1100000, '千代田区神田須田町1-2', '住居', '近隣商業地域', 2022, 139.77120, 35.69410),
  (1050000, '千代田区神田須田町1-2', '住居', '近隣商業地域', 2021, 139.77120, 35.69410),
  (1000000, '千代田区神田須田町1-2', '住居', '近隣商業地域', 2020, 139.77120, 35.69410)
) AS v(price, addr, use_type, zone, yr, lng, lat)
WHERE NOT EXISTS (SELECT 1 FROM land_prices LIMIT 1);

-- ============================================================
-- 用途地域 (Zoning) — 5 polygons around Tokyo Station
-- Source: 国土数値情報 A29
-- ============================================================
INSERT INTO zoning (zone_type, zone_code, floor_area_ratio, building_coverage, geom)
SELECT v.ztype, v.zcode, v.far, v.bc, ST_SetSRID(ST_GeomFromText(v.wkt), 4326)
FROM (VALUES
  ('商業地域', '09', 8.0, 0.8,
   'MULTIPOLYGON(((139.764 35.679, 139.770 35.679, 139.770 35.684, 139.764 35.684, 139.764 35.679)))'),
  ('近隣商業地域', '08', 4.0, 0.8,
   'MULTIPOLYGON(((139.770 35.679, 139.775 35.679, 139.775 35.684, 139.770 35.684, 139.770 35.679)))'),
  ('第一種住居地域', '05', 3.0, 0.6,
   'MULTIPOLYGON(((139.764 35.690, 139.770 35.690, 139.770 35.695, 139.764 35.695, 139.764 35.690)))'),
  ('準工業地域', '11', 3.0, 0.6,
   'MULTIPOLYGON(((139.775 35.679, 139.780 35.679, 139.780 35.684, 139.775 35.684, 139.775 35.679)))'),
  ('第二種住居地域', '06', 2.0, 0.6,
   'MULTIPOLYGON(((139.770 35.690, 139.775 35.690, 139.775 35.695, 139.770 35.695, 139.770 35.690)))')
) AS v(ztype, zcode, far, bc, wkt)
WHERE NOT EXISTS (SELECT 1 FROM zoning LIMIT 1);

-- ============================================================
-- 洪水浸水想定区域 (Flood Risk) — 4 polygons along Nihonbashi River
-- Source: 国土数値情報 A31
-- ============================================================
INSERT INTO flood_risk (depth_rank, river_name, geom)
SELECT v.depth, v.river, ST_SetSRID(ST_GeomFromText(v.wkt), 4326)
FROM (VALUES
  (2, '日本橋川',
   'MULTIPOLYGON(((139.770 35.683, 139.774 35.683, 139.774 35.686, 139.770 35.686, 139.770 35.683)))'),
  (1, '日本橋川',
   'MULTIPOLYGON(((139.774 35.683, 139.778 35.683, 139.778 35.686, 139.774 35.686, 139.774 35.683)))'),
  (3, '神田川',
   'MULTIPOLYGON(((139.765 35.693, 139.770 35.693, 139.770 35.696, 139.765 35.696, 139.765 35.693)))'),
  (2, '隅田川',
   'MULTIPOLYGON(((139.780 35.680, 139.785 35.680, 139.785 35.685, 139.780 35.685, 139.780 35.680)))')
) AS v(depth, river, wkt)
WHERE NOT EXISTS (SELECT 1 FROM flood_risk LIMIT 1);

-- ============================================================
-- 急傾斜地 (Steep Slope) — 3 small zones (Tokyo is mostly flat, so sparse)
-- Source: 国土数値情報 A47
-- ============================================================
INSERT INTO steep_slope (area_name, geom)
SELECT v.name, ST_SetSRID(ST_GeomFromText(v.wkt), 4326)
FROM (VALUES
  ('本郷台地東縁',
   'MULTIPOLYGON(((139.762 35.700, 139.764 35.700, 139.764 35.702, 139.762 35.702, 139.762 35.700)))'),
  ('駿河台北側',
   'MULTIPOLYGON(((139.764 35.698, 139.766 35.698, 139.766 35.700, 139.764 35.700, 139.764 35.698)))'),
  ('飯田橋斜面',
   'MULTIPOLYGON(((139.745 35.700, 139.748 35.700, 139.748 35.702, 139.745 35.702, 139.745 35.700)))')
) AS v(name, wkt)
WHERE NOT EXISTS (SELECT 1 FROM steep_slope LIMIT 1);

-- ============================================================
-- 学校 (Schools) — 8 schools around Tokyo Station
-- Source: 国土数値情報 P29
-- ============================================================
INSERT INTO schools (name, school_type, geom)
SELECT v.name, v.stype, ST_SetSRID(ST_MakePoint(v.lng, v.lat), 4326)
FROM (VALUES
  ('千代田区立千代田小学校',   '小学校', 139.76100, 35.69200),
  ('千代田区立お茶の水小学校', '小学校', 139.76500, 35.69700),
  ('中央区立泰明小学校',       '小学校', 139.76350, 35.67200),
  ('中央区立城東小学校',       '小学校', 139.77600, 35.67850),
  ('千代田区立麹町中学校',     '中学校', 139.74400, 35.68600),
  ('千代田区立神田一橋中学校', '中学校', 139.76300, 35.69500),
  ('開成高等学校',             '高等学校', 139.77100, 35.73000),
  ('都立日比谷高等学校',       '高等学校', 139.75100, 35.67100)
) AS v(name, stype, lng, lat)
WHERE NOT EXISTS (SELECT 1 FROM schools LIMIT 1);

-- ============================================================
-- 医療機関 (Medical Facilities) — 6 facilities around Tokyo Station
-- Source: 国土数値情報 P04
-- ============================================================
INSERT INTO medical_facilities (name, facility_type, bed_count, geom)
SELECT v.name, v.ftype, v.beds, ST_SetSRID(ST_MakePoint(v.lng, v.lat), 4326)
FROM (VALUES
  ('聖路加国際病院',           '病院', 520,  139.77200, 35.66900),
  ('三井記念病院',             '病院', 482,  139.77050, 35.69400),
  ('東京逓信病院',             '病院', 477,  139.75500, 35.69400),
  ('日本橋室町クリニック',     '診療所', 0,  139.77400, 35.68700),
  ('丸の内中央ビル診療所',     '診療所', 0,  139.76700, 35.68200),
  ('銀座メディカルクリニック', '診療所', 0,  139.76500, 35.67300)
) AS v(name, ftype, beds, lng, lat)
WHERE NOT EXISTS (SELECT 1 FROM medical_facilities LIMIT 1);
