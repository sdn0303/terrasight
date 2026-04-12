-- ============================================================
-- Transaction prices (不動産情報ライブラリ: 取引価格・成約価格)
-- ============================================================
CREATE TABLE transaction_prices (
    id                  bigint GENERATED ALWAYS AS IDENTITY,
    pref_code           text   NOT NULL,
    city_code           text   NOT NULL,
    city_name           text   NOT NULL,
    district_name       text,
    property_type       text   NOT NULL CHECK (property_type IN (
                            'condo', 'land_building', 'land', 'forest', 'agriculture'
                        )),
    price_category      text   NOT NULL CHECK (price_category IN ('transaction', 'contract')),
    total_price         bigint NOT NULL CHECK (total_price > 0),
    price_per_sqm       integer,
    area_sqm            integer,
    floor_plan          text,
    building_year       smallint,
    building_structure  text,
    current_use         text,
    city_planning_zone  text,
    building_coverage   smallint,
    floor_area_ratio    smallint,
    nearest_station     text,
    station_walk_min    smallint,
    front_road_width    real,
    land_shape          text,
    transaction_quarter text   NOT NULL,
    transaction_year    smallint NOT NULL CHECK (transaction_year BETWEEN 2000 AND 2100),
    transaction_q       smallint NOT NULL CHECK (transaction_q BETWEEN 1 AND 4),
    created_at          timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (id, pref_code)
) PARTITION BY LIST (pref_code);

COMMENT ON TABLE transaction_prices IS '不動産取引価格情報（不動産情報ライブラリ）';
COMMENT ON COLUMN transaction_prices.price_category IS 'transaction=取引価格, contract=成約価格';
COMMENT ON COLUMN transaction_prices.property_type IS 'condo/land_building/land/forest/agriculture';

DO $$
DECLARE
    pref text;
BEGIN
    FOR pref IN SELECT lpad(i::text, 2, '0') FROM generate_series(1, 47) i
    LOOP
        EXECUTE format(
            'CREATE TABLE transaction_prices_%s PARTITION OF transaction_prices FOR VALUES IN (%L)',
            pref, pref
        );
    END LOOP;
END $$;

CREATE INDEX idx_tx_city_year ON transaction_prices (city_code, transaction_year);
CREATE INDEX idx_tx_pref_type_year ON transaction_prices (pref_code, property_type, transaction_year);
CREATE INDEX idx_tx_quarter ON transaction_prices (transaction_year, transaction_q);

-- ============================================================
-- Land appraisals (鑑定評価書情報地価公示)
-- ============================================================
CREATE TABLE land_appraisals (
    id                  bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pref_code           text   NOT NULL,
    city_code           text   NOT NULL,
    city_name           text   NOT NULL,
    land_use_code       text   NOT NULL,
    sequence_no         smallint NOT NULL,
    appraiser_no        smallint NOT NULL CHECK (appraiser_no IN (1, 2)),
    survey_year         smallint NOT NULL CHECK (survey_year BETWEEN 2000 AND 2100),
    appraisal_price     bigint NOT NULL CHECK (appraisal_price > 0),
    price_per_sqm       integer NOT NULL CHECK (price_per_sqm > 0),
    address             text   NOT NULL,
    display_address     text,
    lot_area_sqm        real,
    current_use_code    text,
    zone_code           text,
    building_coverage   smallint,
    floor_area_ratio    smallint,
    nearest_station     text,
    station_distance_m  integer,
    front_road_width    real,
    fudosan_id          text,
    comparable_price    integer,
    yield_price         integer,
    cost_price          integer,
    created_at          timestamptz NOT NULL DEFAULT now(),
    UNIQUE (pref_code, city_code, land_use_code, sequence_no, appraiser_no, survey_year)
);

COMMENT ON TABLE land_appraisals IS '鑑定評価書情報（地価公示）';
COMMENT ON COLUMN land_appraisals.fudosan_id IS '不動産ID（17桁、将来の座標紐付け用）';
COMMENT ON COLUMN land_appraisals.comparable_price IS '比準価格（円/㎡）';
COMMENT ON COLUMN land_appraisals.yield_price IS '収益価格（円/㎡）';
COMMENT ON COLUMN land_appraisals.cost_price IS '積算価格（円/㎡）';

CREATE INDEX idx_appr_pref_year ON land_appraisals (pref_code, survey_year);
CREATE INDEX idx_appr_city ON land_appraisals (city_code);
CREATE INDEX idx_appr_fudosan ON land_appraisals (fudosan_id) WHERE fudosan_id IS NOT NULL;

-- ============================================================
-- Materialized view: Transaction summary per municipality/year
-- ============================================================
CREATE MATERIALIZED VIEW mv_transaction_summary AS
SELECT
    pref_code,
    city_code,
    transaction_year,
    property_type,
    count(*)::integer                                                AS tx_count,
    avg(total_price)::bigint                                         AS avg_total_price,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY total_price)::bigint AS median_total_price,
    avg(price_per_sqm)::integer                                      AS avg_price_sqm,
    avg(area_sqm)::integer                                           AS avg_area_sqm,
    avg(station_walk_min)::smallint                                   AS avg_walk_min
FROM transaction_prices
WHERE total_price > 0
GROUP BY pref_code, city_code, transaction_year, property_type;

CREATE UNIQUE INDEX idx_mv_tx_pk ON mv_transaction_summary (pref_code, city_code, transaction_year, property_type);
CREATE INDEX idx_mv_tx_city_year ON mv_transaction_summary (city_code, transaction_year);

-- ============================================================
-- Materialized view: Appraisal summary per municipality
-- ============================================================
CREATE MATERIALIZED VIEW mv_appraisal_summary AS
SELECT
    pref_code,
    city_code,
    land_use_code,
    survey_year,
    count(DISTINCT (sequence_no))::integer  AS parcel_count,
    avg(price_per_sqm)::integer             AS avg_price_sqm,
    avg(lot_area_sqm)::real                 AS avg_lot_area,
    avg(comparable_price)::integer          AS avg_comparable,
    avg(yield_price)::integer               AS avg_yield,
    avg(cost_price)::integer                AS avg_cost
FROM land_appraisals
GROUP BY pref_code, city_code, land_use_code, survey_year;

CREATE UNIQUE INDEX idx_mv_appr_pk ON mv_appraisal_summary (pref_code, city_code, land_use_code, survey_year);
