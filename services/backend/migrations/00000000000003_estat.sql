-- e-Stat 統計データテーブル (国勢調査人口 + 住宅空き家)
-- Applied by: pipeline.sh Step 0c

-- ── 市区町村レベル人口（国勢調査） ──

CREATE TABLE IF NOT EXISTS population_municipality (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code   text NOT NULL,
    city_code   text NOT NULL,
    category    text NOT NULL,       -- '0010'=総人口, '0020'=男, '0030'=女, '0040'=世帯数
    value       integer NOT NULL,
    census_year smallint NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (city_code, category, census_year)
);

COMMENT ON TABLE population_municipality IS '市区町村レベル人口統計（国勢調査, e-Stat）';
COMMENT ON COLUMN population_municipality.pref_code IS '都道府県コード（2桁ゼロ埋め）';
COMMENT ON COLUMN population_municipality.city_code IS '市区町村コード（5桁）';
COMMENT ON COLUMN population_municipality.category IS '統計区分: 0010=総人口, 0020=男, 0030=女, 0040=世帯数';
COMMENT ON COLUMN population_municipality.value IS '統計値（人数または世帯数）';
COMMENT ON COLUMN population_municipality.census_year IS '国勢調査年（西暦）';

CREATE INDEX IF NOT EXISTS idx_pop_muni_pref ON population_municipality (pref_code);
CREATE INDEX IF NOT EXISTS idx_pop_muni_city_year ON population_municipality (city_code, census_year);

-- ── 住宅空き家数 ──

CREATE TABLE IF NOT EXISTS vacancy_rates (
    id            bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code     text NOT NULL,
    city_code     text NOT NULL,
    vacancy_count integer NOT NULL,
    total_houses  integer,           -- 別テーブルから取得が必要（将来対応）
    survey_year   smallint NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    UNIQUE (city_code, survey_year)
);

COMMENT ON TABLE vacancy_rates IS '住宅空き家数（住宅・土地統計調査, e-Stat）';
COMMENT ON COLUMN vacancy_rates.pref_code IS '都道府県コード（2桁ゼロ埋め）';
COMMENT ON COLUMN vacancy_rates.city_code IS '市区町村コード（5桁）';
COMMENT ON COLUMN vacancy_rates.vacancy_count IS '空き家数（戸）';
COMMENT ON COLUMN vacancy_rates.total_houses IS '総住宅数（戸）。将来対応: 別統計から結合予定';
COMMENT ON COLUMN vacancy_rates.survey_year IS '調査年（西暦）';

CREATE INDEX IF NOT EXISTS idx_vacancy_pref ON vacancy_rates (pref_code);
CREATE INDEX IF NOT EXISTS idx_vacancy_city_year ON vacancy_rates (city_code, survey_year);

-- ── マテリアライズドビュー: 人口集計 ──

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_population_summary AS
SELECT
    pm.pref_code,
    pm.city_code,
    pm.census_year,
    MAX(CASE WHEN pm.category = '0010' THEN pm.value END) AS population,
    MAX(CASE WHEN pm.category = '0020' THEN pm.value END) AS male,
    MAX(CASE WHEN pm.category = '0030' THEN pm.value END) AS female,
    MAX(CASE WHEN pm.category = '0040' THEN pm.value END) AS households
FROM population_municipality pm
GROUP BY pm.pref_code, pm.city_code, pm.census_year;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_pop_pk
    ON mv_population_summary (pref_code, city_code, census_year);

-- ── マテリアライズドビュー: 空き家集計 ──

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_vacancy_summary AS
SELECT
    vr.pref_code,
    vr.city_code,
    vr.vacancy_count,
    vr.total_houses,
    CASE
        WHEN vr.total_houses > 0
        THEN ROUND((vr.vacancy_count::numeric / vr.total_houses) * 100, 1)
        ELSE NULL
    END AS vacancy_rate_pct,
    vr.survey_year
FROM vacancy_rates vr;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_vac_pk
    ON mv_vacancy_summary (pref_code, city_code, survey_year);
