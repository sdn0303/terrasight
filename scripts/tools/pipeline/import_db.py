#!/usr/bin/env python3
"""Import GeoJSON data into PostGIS database.

Usage:
    uv run scripts/tools/pipeline/import_db.py --pref 13 --priority P0
"""
from __future__ import annotations

import argparse
import json
import logging
import os
import sys
from pathlib import Path

import psycopg2
from psycopg2 import sql
from psycopg2.extras import execute_values
from shapely.geometry import MultiLineString, shape

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent.parent))

from scripts.tools.pipeline.registry import load_catalog

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(name)s: %(message)s",
)
logger = logging.getLogger(__name__)

BATCH_SIZE = 1000

# Allowlist of valid table names to prevent SQL injection.
VALID_TABLES = frozenset({
    "admin_boundaries", "land_prices", "zoning", "flood_risk",
    "steep_slope", "schools", "medical_facilities", "stations",
    "railways", "seismic_hazard", "liquefaction",
    "transaction_prices", "land_appraisals",
})

# Column specs per table: (column_names, value_template)
# The template uses %s placeholders matching the tuple order from _build_row.
_TABLE_SPECS: dict[str, tuple[list[str], str]] = {
    "land_prices": (
        ["pref_code", "address", "price_per_sqm", "land_use", "zone_type", "survey_year", "geom"],
        "(%s, %s, %s, %s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "zoning": (
        ["pref_code", "zone_code", "zone_type", "floor_area_ratio", "building_coverage", "geom"],
        "(%s, %s, %s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "admin_boundaries": (
        ["pref_code", "pref_name", "city_code", "city_name", "admin_code", "level", "geom"],
        "(%s, %s, %s, %s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "schools": (
        ["pref_code", "school_name", "school_type", "address", "geom"],
        "(%s, %s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "medical_facilities": (
        ["pref_code", "facility_name", "facility_type", "beds", "address", "geom"],
        "(%s, %s, %s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "stations": (
        ["pref_code", "station_name", "station_code", "line_name", "operator_name", "passenger_count", "geom"],
        "(%s, %s, %s, %s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "railways": (
        ["pref_code", "line_name", "operator_name", "railway_type", "geom"],
        "(%s, %s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "flood_risk": (
        ["pref_code", "depth_rank", "river_name", "geom"],
        "(%s, %s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "steep_slope": (
        ["pref_code", "area_name", "geom"],
        "(%s, %s, ST_GeomFromText(%s, 4326))",
    ),
    "transaction_prices": (
        [
            "pref_code", "city_code", "city_name", "district_name",
            "property_type", "price_category", "total_price", "price_per_sqm",
            "area_sqm", "floor_plan", "building_year", "building_structure",
            "current_use", "city_planning_zone", "building_coverage",
            "floor_area_ratio", "nearest_station", "station_walk_min",
            "front_road_width", "land_shape", "transaction_quarter",
            "transaction_year", "transaction_q",
        ],
        "(%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)",
    ),
    "land_appraisals": (
        [
            "pref_code", "city_code", "city_name", "land_use_code",
            "sequence_no", "appraiser_no", "survey_year", "appraisal_price",
            "price_per_sqm", "address", "display_address", "lot_area_sqm",
            "current_use_code", "zone_code", "building_coverage",
            "floor_area_ratio", "nearest_station", "station_distance_m",
            "front_road_width", "fudosan_id", "comparable_price",
            "yield_price", "cost_price",
        ],
        "(%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)",
    ),
}


def get_db_url() -> str:
    """Get database URL from environment."""
    return os.environ.get(
        "DATABASE_URL",
        f"postgresql://app:{os.environ.get('DB_PASSWORD', 'devpass')}@localhost:5432/realestate",
    )


def import_geojson_to_table(
    conn,
    geojson_path: Path,
    table_name: str,
    pref_code: str,
) -> int:
    """Import GeoJSON features into a PostGIS table.

    Idempotent: DELETE WHERE pref_code = X, then bulk INSERT via execute_values.
    """
    if table_name not in VALID_TABLES:
        raise ValueError(f"table_name '{table_name}' not in allowlist")

    with open(geojson_path, encoding="utf-8") as f:
        data = json.load(f)

    features = data.get("features", [])
    if not features:
        return 0

    cur = conn.cursor()

    # Delete existing data for this pref_code (safe identifier)
    cur.execute(
        sql.SQL("DELETE FROM {} WHERE pref_code = %s").format(sql.Identifier(table_name)),
        (pref_code,),
    )
    deleted = cur.rowcount
    if deleted:
        logger.info(f"Deleted {deleted} existing rows from {table_name} for pref={pref_code}")

    # Build rows for bulk insert
    rows: list[tuple] = []
    for feat in features:
        props = feat.get("properties", {})
        geom = feat.get("geometry")
        if geom is None:
            continue
        geom_shape = shape(geom)
        # Stations table expects Point — convert LineString to centroid
        if table_name == "stations" and geom_shape.geom_type != "Point":
            geom_shape = geom_shape.centroid
        # Railways table expects MultiLineString
        if table_name == "railways" and geom_shape.geom_type == "LineString":
            geom_shape = MultiLineString([geom_shape])
        geom_wkt = geom_shape.wkt
        row = _build_row(table_name, props, geom_wkt, pref_code)
        if row is not None:
            rows.append(row)

    if not rows:
        return 0

    # Bulk insert using execute_values
    cols, template = _TABLE_SPECS[table_name]
    insert_sql = sql.SQL("INSERT INTO {} ({}) VALUES %s").format(
        sql.Identifier(table_name),
        sql.SQL(", ").join(sql.Identifier(c) for c in cols),
    )
    execute_values(cur, insert_sql.as_string(conn), rows, template=template, page_size=BATCH_SIZE)
    conn.commit()

    logger.info(f"Imported {len(rows)} rows into {table_name} for pref={pref_code}")
    return len(rows)


def _prop(props: dict, *keys: str, default=None):
    """Return the first present, non-empty property value from candidate keys."""
    for key in keys:
        if key in props:
            value = props[key]
            if value is not None and value != "":
                return value
    return default


def _build_row(table_name: str, props: dict, geom_wkt: str, pref_code: str) -> tuple | None:
    """Build a row tuple matching the column spec for the given table.

    Uses _prop() to fall back to KSJ-coded field names when canonical
    names are not present (e.g. L01_006 for price_per_sqm).
    """
    if table_name == "land_prices":
        return (
            pref_code,
            _prop(props, "address", default=""),
            _prop(props, "price_per_sqm", default=0),
            _prop(props, "land_use"),
            _prop(props, "zone_type"),
            _prop(props, "survey_year", default=2024),
            geom_wkt,
        )
    if table_name == "zoning":
        return (
            pref_code,
            _prop(props, "zone_code", "A29_004", default=""),
            _prop(props, "zone_type", "A29_005", default=""),
            _prop(props, "floor_area_ratio", "A29_006"),
            _prop(props, "building_coverage", "A29_007"),
            geom_wkt,
        )
    if table_name == "admin_boundaries":
        # Derive city_code from adminCode (5-digit JIS code → municipality)
        admin_code = _prop(props, "adminCode", default="")
        city_code = admin_code if len(admin_code) == 5 else None
        return (
            pref_code,
            _prop(props, "prefName", default=""),
            city_code,
            _prop(props, "cityName"),
            admin_code,
            "municipality" if city_code else "prefecture",
            geom_wkt,
        )
    if table_name == "schools":
        return (
            pref_code,
            _prop(props, "school_name", "name", "P29_004", default=""),
            _prop(props, "school_type", "P29_001", default=""),
            _prop(props, "address", "P29_003"),
            geom_wkt,
        )
    if table_name == "medical_facilities":
        return (
            pref_code,
            _prop(props, "facility_name", "name", "P04_002", default=""),
            _prop(props, "facility_type", "P04_004", default=""),
            _prop(props, "beds", "bed_count", "P04_008"),
            _prop(props, "address", "P04_003"),
            geom_wkt,
        )
    if table_name == "stations":
        return (
            pref_code,
            _prop(props, "station_name", "S12_001", default=""),
            _prop(props, "station_code", "S12_001c"),
            _prop(props, "line_name", "S12_003"),
            _prop(props, "operator_name", "S12_002"),
            _prop(props, "passenger_count", "S12_004"),
            geom_wkt,
        )
    if table_name == "railways":
        return (
            pref_code,
            _prop(props, "line_name", "N02_003", default=""),
            _prop(props, "operator_name", "N02_004"),
            _prop(props, "railway_type", "N02_002"),
            geom_wkt,
        )
    if table_name == "flood_risk":
        return (pref_code, props.get("depth_rank"), props.get("river_name"), geom_wkt)
    if table_name == "steep_slope":
        return (pref_code, props.get("area_name"), geom_wkt)

    logger.warning(f"No import handler for table: {table_name}")
    return None


def import_reinfolib_data(conn, pref_code: str) -> None:
    """Import 不動産情報ライブラリ data for a prefecture."""
    from scripts.tools.pipeline.adapters.reinfolib_csv import (
        read_appraisal_csv,
        read_transaction_csv,
    )

    raw_dir = Path("data/raw/不動産情報ライブラリ")

    # Transaction prices
    tx_zip = raw_dir / "不動産価格（取引価格・成約価格）情報" / "All_20053_20253.zip"
    if tx_zip.exists():
        rows = read_transaction_csv(tx_zip, pref_code)
        if rows:
            _import_dict_rows(conn, "transaction_prices", rows, pref_code, on_conflict="ON CONFLICT DO NOTHING")

    # Appraisals
    appr_dir = raw_dir / "鑑定評価書情報地価公示"
    if appr_dir.exists():
        rows = read_appraisal_csv(appr_dir, pref_code)
        if rows:
            _import_dict_rows(conn, "land_appraisals", rows, pref_code, on_conflict="ON CONFLICT DO NOTHING")

    # Refresh materialized views
    cur = conn.cursor()
    cur.execute("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_transaction_summary")
    cur.execute("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_appraisal_summary")
    conn.commit()
    logger.info(f"Refreshed materialized views for pref={pref_code}")


def _import_dict_rows(
    conn,
    table_name: str,
    rows: list[dict],
    pref_code: str,
    on_conflict: str = "",
) -> None:
    """Bulk import dict rows into a table. Idempotent: DELETE + INSERT."""
    if table_name not in VALID_TABLES:
        raise ValueError(f"table_name '{table_name}' not in allowlist")

    cur = conn.cursor()
    cur.execute(
        sql.SQL("DELETE FROM {} WHERE pref_code = %s").format(sql.Identifier(table_name)),
        (pref_code,),
    )
    deleted = cur.rowcount
    if deleted:
        logger.info(f"Deleted {deleted} existing rows from {table_name} for pref={pref_code}")

    cols, template = _TABLE_SPECS[table_name]
    tuples = [tuple(row.get(c) for c in cols) for row in rows]

    insert_sql = sql.SQL("INSERT INTO {} ({}) VALUES %s").format(
        sql.Identifier(table_name),
        sql.SQL(", ").join(sql.Identifier(c) for c in cols),
    )
    suffix = f" {on_conflict}" if on_conflict else ""
    execute_values(
        cur,
        insert_sql.as_string(conn) + suffix,
        tuples,
        template=template,
        page_size=5000,
    )
    conn.commit()
    logger.info(f"Imported {len(tuples)} rows into {table_name} for pref={pref_code}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Import GeoJSON to PostGIS")
    parser.add_argument("--pref", required=True, help="Prefecture code")
    parser.add_argument("--priority", default=None, help="Filter by priority")
    parser.add_argument("--dataset", default=None, help="Filter by dataset ID")
    parser.add_argument("--reinfolib", action="store_true", help="Import REINFOLIB data")
    args = parser.parse_args()

    pref_code = args.pref.zfill(2)
    entries = load_catalog()

    # Filter entries with db_table (API-layer datasets)
    entries = [e for e in entries if e.db_table is not None]

    if args.priority:
        entries = [e for e in entries if e.priority == args.priority]
    if args.dataset:
        entries = [e for e in entries if e.id == args.dataset]

    conn = psycopg2.connect(get_db_url())
    try:
        # Skip GeoJSON catalog loop when running reinfolib-only import
        if not (args.reinfolib and args.priority is None and args.dataset is None):
            for entry in entries:
                if entry.output_geojson is None:
                    continue
                geojson_rel = entry.output_geojson.replace("{pref_code}", pref_code)
                geojson_path = Path(geojson_rel)
                if not geojson_path.exists():
                    logger.debug(f"GeoJSON not found: {geojson_path}")
                    continue
                import_geojson_to_table(conn, geojson_path, entry.db_table, pref_code)

        if args.reinfolib:
            import_reinfolib_data(conn, pref_code)
    finally:
        conn.close()

    logger.info("Import complete")


if __name__ == "__main__":
    main()
