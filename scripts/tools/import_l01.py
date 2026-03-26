#!/usr/bin/env python3
"""
Import MLIT L01 land price GeoJSON files into the PostGIS land_prices table.

Reads pre-converted GeoJSON files from data/geojson/ and inserts rows into the
land_prices table.  One file per survey year (2022-2026), each containing ~4,900
Tokyo-area Point features with MLIT attribute codes.

Attribute mapping (MLIT L01 data dictionary):
    L01_007  -> year          (integer, e.g. 2026)
    L01_008  -> price_per_sqm (integer, yen/sqm for the survey year)
    L01_025  -> address       (text, full address string)
    L01_028  -> land_use      (text, land-use label e.g. "住宅")

Geometry: GeoJSON Point [longitude, latitude] -> PostGIS POINT(lon lat) SRID 4326.

Usage:
    # Set DATABASE_URL first (or export it in your shell)
    export DATABASE_URL="postgresql://user:pass@localhost:5432/realestate"

    uv run scripts/tools/import_l01.py                    # import all 5 years
    uv run scripts/tools/import_l01.py --year 2025        # import single year
    uv run scripts/tools/import_l01.py --dry-run          # preview counts without inserting
    uv run scripts/tools/import_l01.py --year 2026 --dry-run

Dependencies:
    Managed by uv (see scripts/pyproject.toml)

Exit codes:
    0  success
    1  missing environment variable or file
    2  database error
"""

from __future__ import annotations
from typing import Any

import argparse
import os
import sys
from pathlib import Path

import geopandas as gpd

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent.parent.parent  # scripts/tools/ -> scripts/ -> project root
GEOJSON_DIR = ROOT / "data" / "geojson"

YEARS = [2022, 2023, 2024, 2025, 2026]

# MLIT L01 attribute -> PostGIS column
ATTR_YEAR = "L01_007"
ATTR_PRICE = "L01_008"
ATTR_ADDRESS = "L01_025"
ATTR_LAND_USE = "L01_028"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def geojson_path(year: int) -> Path:
    """Return the expected GeoJSON path for the given survey year."""
    return GEOJSON_DIR / f"l01-{year}-tokyo.geojson"


def parse_database_url(url: str) -> dict[str, Any]:
    """Parse a postgresql:// URL into psycopg2 connect kwargs.

    Handles both ``postgresql://`` and ``postgres://`` schemes.
    """
    import urllib.parse

    parsed = urllib.parse.urlparse(url)
    if parsed.scheme not in ("postgresql", "postgres"):
        raise ValueError(
            f"DATABASE_URL must use postgresql:// scheme, got: {parsed.scheme!r}"
        )

    kwargs: dict[str, Any] = {
        "host": parsed.hostname or "localhost",
        "port": parsed.port or 5432,
        "dbname": parsed.path.lstrip("/"),
        "user": parsed.username or "",
        "password": parsed.password or "",
    }
    return kwargs


def read_geojson(path: Path) -> gpd.GeoDataFrame:
    """Read a GeoJSON file and return a GeoDataFrame in EPSG:4326."""
    gdf = gpd.read_file(path)
    if gdf.crs is None:
        gdf = gdf.set_crs("EPSG:4326")
    elif gdf.crs.to_epsg() != 4326:
        gdf = gdf.to_crs("EPSG:4326")
    return gdf


def extract_rows(
    gdf: gpd.GeoDataFrame, year: int
) -> list[tuple[int, str, str | None, float, float, int]]:
    """Extract and validate rows from a GeoDataFrame.

    Returns a list of tuples:
        (price_per_sqm, address, land_use, longitude, latitude, year)

    Rows with null geometry, null address, or non-positive price are skipped
    with a warning.
    """
    rows: list[tuple[int, str, str | None, float, float, int]] = []
    skipped = 0

    for idx, row in gdf.iterrows():
        geom = row.geometry
        if geom is None or geom.is_empty:
            skipped += 1
            continue

        price_raw = row.get(ATTR_PRICE)
        address_raw = row.get(ATTR_ADDRESS)
        land_use_raw = row.get(ATTR_LAND_USE)

        # Validate price
        try:
            price = int(price_raw)
        except (TypeError, ValueError):
            skipped += 1
            continue
        if price <= 0:
            skipped += 1
            continue

        # Validate address
        if address_raw is None or str(address_raw).strip() in ("", "_"):
            skipped += 1
            continue
        address = str(address_raw).strip()

        # Land use: NOT NULL in new schema, use empty string for missing
        land_use: str
        if land_use_raw is None or str(land_use_raw).strip() in ("", "_"):
            land_use = ""
        else:
            land_use = str(land_use_raw).strip()

        # GeoJSON coordinates are always [longitude, latitude] per RFC 7946
        lon = geom.x
        lat = geom.y

        rows.append((price, address, land_use, lon, lat, year))

    if skipped > 0:
        print(f"    Skipped {skipped} invalid/incomplete rows")

    return rows


# ---------------------------------------------------------------------------
# Import logic
# ---------------------------------------------------------------------------


def import_year(
    year: int,
    conn_kwargs: dict[str, Any],
    *,
    dry_run: bool,
) -> int:
    """Import land price data for one survey year.

    Returns the number of rows inserted (or that would be inserted in dry-run).
    Raises SystemExit on unrecoverable errors.
    """
    path = geojson_path(year)
    if not path.exists():
        print(f"  [{year}] SKIP: {path.name} not found")
        return 0

    print(f"  [{year}] Reading {path.name} ...")
    gdf = read_geojson(path)
    rows = extract_rows(gdf, year)

    print(f"    Valid rows: {len(rows)}")

    if dry_run:
        print(f"    [dry-run] Would delete existing rows for year={year} then insert {len(rows)} rows.")
        return len(rows)

    # Import inside a single transaction: delete existing year data first,
    # then bulk-insert all rows.  If anything fails the transaction rolls back,
    # leaving the table unchanged.
    import psycopg2
    import psycopg2.extras

    try:
        with psycopg2.connect(**conn_kwargs) as conn:
            with conn.cursor() as cur:
                # Delete existing rows for this year (idempotent re-import)
                cur.execute(
                    "DELETE FROM land_prices WHERE year = %s",
                    (year,),
                )
                deleted = cur.rowcount
                if deleted > 0:
                    print(f"    Deleted {deleted} existing rows for year={year}")

                # Bulk-insert using execute_values with EWKT geometry literals.
                # ST_GeomFromEWKT parses "SRID=4326;POINT(lon lat)" into a
                # PostGIS geometry(Point, 4326) value.
                psycopg2.extras.execute_values(
                    cur,
                    """
                    INSERT INTO land_prices
                        (price_per_sqm, address, land_use, geom, year)
                    VALUES %s
                    ON CONFLICT (address, year) DO UPDATE
                        SET price_per_sqm = EXCLUDED.price_per_sqm,
                            land_use = EXCLUDED.land_use,
                            geom = EXCLUDED.geom
                    """,
                    [
                        (
                            price,
                            address,
                            land_use,
                            f"SRID=4326;POINT({lon} {lat})",
                            yr,
                        )
                        for (price, address, land_use, lon, lat, yr) in rows
                    ],
                    template="(%s, %s, %s, ST_GeomFromEWKT(%s), %s)",
                    page_size=500,
                )

            conn.commit()

    except psycopg2.Error as exc:
        print(f"    ERROR inserting year={year}: {exc}", file=sys.stderr)
        sys.exit(2)

    print(f"    Inserted {len(rows)} rows for year={year}")
    return len(rows)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Import MLIT L01 land price GeoJSON files into PostGIS land_prices table.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(__doc__ or "").split("Usage:")[1].split("Dependencies:")[0].strip(),
    )
    parser.add_argument(
        "--year",
        type=int,
        choices=YEARS,
        default=None,
        metavar="YEAR",
        help=f"Import a single survey year ({YEARS[0]}-{YEARS[-1]}). Omit to import all years.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview row counts without writing to the database.",
    )
    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    # Resolve DATABASE_URL
    database_url = os.environ.get("DATABASE_URL", "")
    if not database_url and not args.dry_run:
        print(
            "ERROR: DATABASE_URL environment variable is not set.\n"
            "       Example: export DATABASE_URL=postgresql://user:pass@localhost:5432/realestate",
            file=sys.stderr,
        )
        sys.exit(1)

    if database_url:
        try:
            conn_kwargs = parse_database_url(database_url)
        except ValueError as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            sys.exit(1)
    else:
        # dry-run with no DATABASE_URL: conn_kwargs unused
        conn_kwargs = {}

    years_to_import = [args.year] if args.year is not None else YEARS

    mode_label = "[DRY RUN] " if args.dry_run else ""
    print("=" * 60)
    print(f"{mode_label}Importing L01 land price data ({len(years_to_import)} year(s))")
    print(f"Source: {GEOJSON_DIR}")
    if not args.dry_run:
        print(f"Target: {conn_kwargs.get('host')}:{conn_kwargs.get('port')}/{conn_kwargs.get('dbname')}")
    print("=" * 60)

    total_rows = 0
    for year in years_to_import:
        count = import_year(year, conn_kwargs, dry_run=args.dry_run)
        total_rows += count

    # Post-import: backfill zone_type via spatial join with zoning table.
    # The schema_redesign added land_prices.zone_type as a denormalized column
    # used by the TLS z-score calculation.  Without this, zone_type is NULL and
    # z-scores are incorrect.  See: Codex audit DB-01.
    if not args.dry_run and total_rows > 0:
        import psycopg2

        print("\nBackfilling zone_type via spatial join with zoning table...")
        try:
            with psycopg2.connect(**conn_kwargs) as conn:
                with conn.cursor() as cur:
                    cur.execute("""
                        UPDATE land_prices lp
                        SET zone_type = z.zone_type
                        FROM zoning z
                        WHERE lp.zone_type IS NULL
                          AND ST_Within(lp.geom, z.geom)
                    """)
                    updated = cur.rowcount
                conn.commit()
            print(f"  Updated {updated} rows with zone_type from zoning spatial join.")
            # Report remaining NULLs (land prices outside any zoning polygon)
            with psycopg2.connect(**conn_kwargs) as conn:
                with conn.cursor() as cur:
                    cur.execute("SELECT count(*) FROM land_prices WHERE zone_type IS NULL")
                    remaining = cur.fetchone()[0]
            if remaining > 0:
                print(f"  Note: {remaining} rows still have NULL zone_type (outside zoning polygons).")
        except psycopg2.Error as exc:
            print(f"  WARNING: zone_type backfill failed: {exc}", file=sys.stderr)
            print("  TLS z-scores may be inaccurate. Run manually after fixing zoning data.", file=sys.stderr)

    print("=" * 60)
    print(f"{mode_label}Done. Total rows {'would be ' if args.dry_run else ''}imported: {total_rows}")
    print("=" * 60)


if __name__ == "__main__":
    main()
