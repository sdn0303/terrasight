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
from shapely.geometry import shape

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
}


def get_db_url() -> str:
    """Get database URL from environment."""
    return os.environ.get(
        "DATABASE_URL",
        "postgresql://postgres:postgres@localhost:5432/realestate",
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
        geom_wkt = shape(geom).wkt
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


def _build_row(table_name: str, props: dict, geom_wkt: str, pref_code: str) -> tuple | None:
    """Build a row tuple matching the column spec for the given table."""
    if table_name == "land_prices":
        return (
            pref_code,
            props.get("address", ""),
            props.get("price_per_sqm", 0),
            props.get("land_use"),
            props.get("zone_type"),
            props.get("survey_year", 2024),
            geom_wkt,
        )
    if table_name == "zoning":
        return (
            pref_code,
            props.get("zone_code", ""),
            props.get("zone_type", ""),
            props.get("floor_area_ratio"),
            props.get("building_coverage"),
            geom_wkt,
        )
    if table_name == "admin_boundaries":
        return (
            pref_code,
            props.get("prefName", ""),
            props.get("cityCode"),
            props.get("cityName"),
            props.get("adminCode", ""),
            "municipality" if props.get("cityCode") else "prefecture",
            geom_wkt,
        )
    if table_name == "schools":
        return (
            pref_code,
            props.get("school_name", props.get("name", "")),
            props.get("school_type", ""),
            props.get("address"),
            geom_wkt,
        )
    if table_name == "medical_facilities":
        return (
            pref_code,
            props.get("facility_name", props.get("name", "")),
            props.get("facility_type", ""),
            props.get("beds", props.get("bed_count")),
            props.get("address"),
            geom_wkt,
        )
    if table_name == "stations":
        return (
            pref_code,
            props.get("station_name", ""),
            props.get("station_code"),
            props.get("line_name"),
            props.get("operator_name"),
            props.get("passenger_count"),
            geom_wkt,
        )
    if table_name == "railways":
        return (
            pref_code,
            props.get("line_name", ""),
            props.get("operator_name"),
            props.get("railway_type"),
            geom_wkt,
        )
    if table_name == "flood_risk":
        return (pref_code, props.get("depth_rank"), props.get("river_name"), geom_wkt)
    if table_name == "steep_slope":
        return (pref_code, props.get("area_name"), geom_wkt)

    logger.warning(f"No import handler for table: {table_name}")
    return None


def main() -> None:
    parser = argparse.ArgumentParser(description="Import GeoJSON to PostGIS")
    parser.add_argument("--pref", required=True, help="Prefecture code")
    parser.add_argument("--priority", default=None, help="Filter by priority")
    parser.add_argument("--dataset", default=None, help="Filter by dataset ID")
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
        for entry in entries:
            if entry.output_geojson is None:
                continue
            geojson_rel = entry.output_geojson.replace("{pref_code}", pref_code)
            geojson_path = Path(geojson_rel)
            if not geojson_path.exists():
                logger.debug(f"GeoJSON not found: {geojson_path}")
                continue
            import_geojson_to_table(conn, geojson_path, entry.db_table, pref_code)
    finally:
        conn.close()

    logger.info("Import complete")


if __name__ == "__main__":
    main()
