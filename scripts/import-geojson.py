#!/usr/bin/env python3
"""
Import converted GeoJSON files into PostGIS tables.

Reads pre-converted GeoJSON files from data/geojson/ and inserts rows into their
corresponding PostGIS tables.  Supports 9 datasets (all non-L01 files):

  a29-zoning          -> zoning             (existing table)
  a31b-flood          -> flood_risk          (existing table, ~643K features)
  a47-steep-slope     -> steep_slope         (existing table)
  jshis-seismic       -> seismic_hazard      (new table)
  n02-railway         -> railways            (new table)
  p04-medical         -> medical_facilities  (existing table)
  p29-schools         -> schools             (existing table)
  pl-liquefaction     -> liquefaction        (new table)
  s12-stations        -> stations            (new table)

Each import is idempotent: DELETE all rows from the target table, then INSERT.
New tables are created with CREATE TABLE IF NOT EXISTS before import.

Attribute mapping is defined in DATASETS dict.  Only mapped columns are imported;
unmapped GeoJSON properties are ignored.

Usage:
    export DATABASE_URL="postgresql://user:pass@localhost:5432/realestate"

    python3 scripts/import-geojson.py                          # import all 9 datasets
    python3 scripts/import-geojson.py --dataset p29-schools    # import single dataset
    python3 scripts/import-geojson.py --dry-run                # preview counts
    python3 scripts/import-geojson.py --dataset a31b-flood --batch-size 2000

Dependencies:
    pip install geopandas psycopg2-binary

Exit codes:
    0  success
    1  missing environment variable or file
    2  database error
"""

from __future__ import annotations

import argparse
import os
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable

import geopandas as gpd

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent.parent
GEOJSON_DIR = ROOT / "data" / "geojson"

DEFAULT_BATCH_SIZE = 1000
LARGE_PROGRESS_INTERVAL = 10_000  # print progress every N rows for large datasets


# ---------------------------------------------------------------------------
# Dataset definitions
# ---------------------------------------------------------------------------


@dataclass
class ColumnMapping:
    """Maps a GeoJSON property to a PostGIS column with optional transform."""

    source: str
    target: str
    transform: Callable[[Any], Any] | None = None


@dataclass
class DatasetConfig:
    """Configuration for one GeoJSON dataset import."""

    file: str
    table: str
    columns: list[ColumnMapping]
    geom_type: str = "Geometry"  # PostGIS geometry type hint for EWKT
    force_multi: bool = False  # Convert Polygon -> MultiPolygon for existing tables
    create_ddl: str | None = None  # DDL for new tables (CREATE TABLE IF NOT EXISTS)
    insert_columns: list[str] = field(default_factory=list)  # computed at init

    def __post_init__(self) -> None:
        self.insert_columns = [c.target for c in self.columns]


def _safe_text(val: Any) -> str | None:
    """Convert a value to stripped text, or None for empty/sentinel values."""
    if val is None:
        return None
    s = str(val).strip()
    return None if s in ("", "_", "nan") else s


def _safe_int(val: Any) -> int | None:
    """Convert a value to int, or None."""
    if val is None:
        return None
    try:
        return int(float(val))
    except (TypeError, ValueError):
        return None


def _safe_float(val: Any) -> float | None:
    """Convert a value to float, or None."""
    if val is None:
        return None
    try:
        v = float(val)
        return None if v == -999.0 else v  # -999 is JSHIS sentinel for missing
    except (TypeError, ValueError):
        return None


# ----- DDL for new tables -----

DDL_SEISMIC_HAZARD = """
CREATE TABLE IF NOT EXISTS seismic_hazard (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    fault_id    text,
    fault_name  text    NOT NULL,
    magnitude   real,
    prob_30y    real,
    geom        geometry(Geometry, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_seismic_hazard_geom ON seismic_hazard USING GIST (geom);
"""

DDL_RAILWAYS = """
CREATE TABLE IF NOT EXISTS railways (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    railway_type    text,
    line_name       text,
    operator_name   text,
    station_name    text,
    geom            geometry(LineString, 4326) NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_railways_geom ON railways USING GIST (geom);
"""

DDL_LIQUEFACTION = """
CREATE TABLE IF NOT EXISTS liquefaction (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    risk_rank   text    NOT NULL,
    geom        geometry(Point, 4326) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_liquefaction_geom ON liquefaction USING GIST (geom);
"""

DDL_STATIONS = """
CREATE TABLE IF NOT EXISTS stations (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    station_name    text    NOT NULL,
    station_code    text,
    operator_name   text,
    line_name       text,
    geom            geometry(LineString, 4326) NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_stations_geom ON stations USING GIST (geom);
"""

# Railway type codes (N02_001)
_RAILWAY_TYPE_MAP = {
    "11": "JR",
    "12": "新幹線",
    "2": "民鉄",
    "3": "地下鉄",
    "4": "路面電車",
    "5": "モノレール・新交通",
}


def _railway_type(val: Any) -> str | None:
    s = _safe_text(val)
    if s is None:
        return None
    return _RAILWAY_TYPE_MAP.get(s, s)


# School type codes (P29_003)
_SCHOOL_TYPE_MAP = {
    "16001": "小学校",
    "16002": "中学校",
    "16003": "中等教育学校",
    "16004": "高等学校",
    "16005": "高等専門学校",
    "16006": "短期大学",
    "16007": "大学",
    "16011": "幼稚園",
    "16012": "特別支援学校",
    "16013": "幼保連携型認定こども園",
    "16014": "義務教育学校",
}


def _school_type(val: Any) -> str | None:
    s = _safe_text(val)
    if s is None:
        return None
    return _SCHOOL_TYPE_MAP.get(s, s)


# Medical facility type codes (P04_001)
_MEDICAL_TYPE_MAP = {
    "1": "病院",
    "2": "診療所",
    "3": "歯科診療所",
}


def _medical_type(val: Any) -> str | None:
    s = _safe_text(val)
    if s is None:
        return None
    return _MEDICAL_TYPE_MAP.get(s, s)


# Flood depth rank codes (A31b_201)
_FLOOD_DEPTH_MAP = {
    1: "0.5m未満",
    2: "0.5-3.0m",
    3: "3.0-5.0m",
    4: "5.0-10.0m",
    5: "10.0-20.0m",
    6: "20.0m以上",
}


def _flood_depth(val: Any) -> str | None:
    if val is None:
        return None
    try:
        code = int(float(val))
        return _FLOOD_DEPTH_MAP.get(code, str(code))
    except (TypeError, ValueError):
        return _safe_text(val)


# ----- Dataset registry -----

DATASETS: dict[str, DatasetConfig] = {
    "a29-zoning": DatasetConfig(
        file="a29-zoning-tokyo.geojson",
        table="zoning",
        geom_type="Geometry",
        force_multi=True,
        columns=[
            ColumnMapping("A29_005", "zone_type", _safe_text),
            ColumnMapping("A29_004", "zone_code", lambda v: _safe_text(str(v)) if v is not None else None),
            ColumnMapping("A29_007", "floor_area_ratio", _safe_float),
            ColumnMapping("A29_006", "building_coverage", _safe_float),
        ],
    ),
    "a31b-flood": DatasetConfig(
        file="a31b-flood-tokyo.geojson",
        table="flood_risk",
        geom_type="Geometry",
        force_multi=True,
        columns=[
            ColumnMapping("A31b_201", "depth_rank", _flood_depth),
            ColumnMapping("A31b_101", "river_name", _safe_text),
        ],
    ),
    "a47-steep-slope": DatasetConfig(
        file="a47-steep-slope-tokyo.geojson",
        table="steep_slope",
        geom_type="Geometry",
        force_multi=True,
        columns=[
            ColumnMapping("A47_004", "area_name", _safe_text),
        ],
    ),
    "jshis-seismic": DatasetConfig(
        file="jshis-seismic-tokyo.geojson",
        table="seismic_hazard",
        geom_type="Geometry",
        create_ddl=DDL_SEISMIC_HAZARD,
        columns=[
            ColumnMapping("FLT_ID", "fault_id", _safe_text),
            ColumnMapping("LTENAME", "fault_name", _safe_text),
            ColumnMapping("MAG", "magnitude", _safe_float),
            ColumnMapping("AVR_T30P", "prob_30y", _safe_float),
        ],
    ),
    "n02-railway": DatasetConfig(
        file="n02-railway-tokyo.geojson",
        table="railways",
        geom_type="LineString",
        create_ddl=DDL_RAILWAYS,
        columns=[
            ColumnMapping("N02_001", "railway_type", _railway_type),
            ColumnMapping("N02_003", "line_name", _safe_text),
            ColumnMapping("N02_004", "operator_name", _safe_text),
            ColumnMapping("N02_005", "station_name", _safe_text),
        ],
    ),
    "p04-medical": DatasetConfig(
        file="p04-medical-tokyo.geojson",
        table="medical_facilities",
        geom_type="Point",
        columns=[
            ColumnMapping("P04_002", "name", _safe_text),
            ColumnMapping("P04_001", "facility_type", _medical_type),
            ColumnMapping("P04_008", "bed_count", _safe_int),
        ],
    ),
    "p29-schools": DatasetConfig(
        file="p29-schools-tokyo.geojson",
        table="schools",
        geom_type="Point",
        columns=[
            ColumnMapping("P29_004", "name", _safe_text),
            ColumnMapping("P29_003", "school_type", _school_type),
        ],
    ),
    "pl-liquefaction": DatasetConfig(
        file="pl-liquefaction-tokyo.geojson",
        table="liquefaction",
        geom_type="Point",
        create_ddl=DDL_LIQUEFACTION,
        columns=[
            ColumnMapping("PL区分", "risk_rank", _safe_text),
        ],
    ),
    "s12-stations": DatasetConfig(
        file="s12-stations-tokyo.geojson",
        table="stations",
        geom_type="LineString",
        create_ddl=DDL_STATIONS,
        columns=[
            ColumnMapping("S12_001", "station_name", _safe_text),
            ColumnMapping("S12_001c", "station_code", _safe_text),
            ColumnMapping("S12_002", "operator_name", _safe_text),
            ColumnMapping("S12_003", "line_name", _safe_text),
        ],
    ),
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def parse_database_url(url: str) -> dict[str, Any]:
    """Parse a postgresql:// URL into psycopg2 connect kwargs."""
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


def geom_to_ewkt(geom: Any, *, force_multi: bool = False) -> str | None:
    """Convert a Shapely geometry to EWKT with SRID 4326.

    Drops Z coordinates to avoid PostGIS dimension mismatch.
    When force_multi=True, converts Polygon to MultiPolygon for tables
    that have a MultiPolygon column constraint.
    """
    if geom is None or geom.is_empty:
        return None
    from shapely.geometry import MultiPolygon
    from shapely.ops import transform

    def _drop_z(x: float, y: float, z: float | None = None) -> tuple[float, float]:
        return (x, y)

    if geom.has_z:
        geom = transform(_drop_z, geom)

    if force_multi and geom.geom_type == "Polygon":
        geom = MultiPolygon([geom])

    return f"SRID=4326;{geom.wkt}"


def extract_rows(
    gdf: gpd.GeoDataFrame,
    config: DatasetConfig,
) -> list[tuple[Any, ...]]:
    """Extract and validate rows from a GeoDataFrame.

    Returns a list of tuples: (col1, col2, ..., ewkt_geom)
    Rows with null geometry are skipped.
    """
    rows: list[tuple[Any, ...]] = []
    skipped = 0

    for _idx, row in gdf.iterrows():
        ewkt = geom_to_ewkt(row.geometry, force_multi=config.force_multi)
        if ewkt is None:
            skipped += 1
            continue

        values: list[Any] = []
        for cm in config.columns:
            raw = row.get(cm.source)
            val = cm.transform(raw) if cm.transform else raw
            values.append(val)

        values.append(ewkt)
        rows.append(tuple(values))

    if skipped > 0:
        print(f"    Skipped {skipped} rows with null/empty geometry")

    return rows


# ---------------------------------------------------------------------------
# Import logic
# ---------------------------------------------------------------------------


def ensure_table(
    conn_kwargs: dict[str, Any],
    config: DatasetConfig,
) -> None:
    """Run CREATE TABLE IF NOT EXISTS DDL for new tables."""
    if config.create_ddl is None:
        return

    import psycopg2

    try:
        with psycopg2.connect(**conn_kwargs) as conn:
            with conn.cursor() as cur:
                cur.execute(config.create_ddl)
            conn.commit()
        print(f"    Ensured table '{config.table}' exists")
    except psycopg2.Error as exc:
        print(f"    ERROR creating table {config.table}: {exc}", file=sys.stderr)
        sys.exit(2)


def import_dataset(
    name: str,
    config: DatasetConfig,
    conn_kwargs: dict[str, Any],
    *,
    dry_run: bool,
    batch_size: int,
) -> int:
    """Import one GeoJSON dataset into its PostGIS table.

    Returns the number of rows inserted (or that would be inserted in dry-run).
    """
    path = GEOJSON_DIR / config.file
    if not path.exists():
        print(f"  [{name}] SKIP: {path.name} not found")
        return 0

    print(f"  [{name}] Reading {path.name} ...")
    gdf = read_geojson(path)
    print(f"    Features loaded: {len(gdf)}")

    rows = extract_rows(gdf, config)
    print(f"    Valid rows: {len(rows)}")

    if dry_run:
        print(
            f"    [dry-run] Would DELETE all from '{config.table}' "
            f"then INSERT {len(rows)} rows."
        )
        return len(rows)

    # Ensure table exists (for new tables only)
    ensure_table(conn_kwargs, config)

    import psycopg2
    import psycopg2.extras

    col_names = config.insert_columns + ["geom"]
    col_list = ", ".join(col_names)
    placeholders = ", ".join(["%s"] * len(config.insert_columns)) + ", ST_GeomFromEWKT(%s)"
    template = f"({placeholders})"

    try:
        with psycopg2.connect(**conn_kwargs) as conn:
            with conn.cursor() as cur:
                # Delete existing rows (idempotent re-import)
                cur.execute(f"DELETE FROM {config.table}")
                deleted = cur.rowcount
                if deleted > 0:
                    print(f"    Deleted {deleted} existing rows")

                # Batch insertion with progress reporting
                total = len(rows)
                inserted = 0

                for batch_start in range(0, total, batch_size):
                    batch_end = min(batch_start + batch_size, total)
                    batch = rows[batch_start:batch_end]

                    psycopg2.extras.execute_values(
                        cur,
                        f"INSERT INTO {config.table} ({col_list}) VALUES %s",
                        batch,
                        template=template,
                        page_size=batch_size,
                    )
                    inserted += len(batch)

                    # Progress reporting for large datasets
                    if (
                        total > LARGE_PROGRESS_INTERVAL
                        and inserted % LARGE_PROGRESS_INTERVAL < batch_size
                    ):
                        pct = inserted * 100 // total
                        print(
                            f"    Progress: {inserted:,}/{total:,} rows ({pct}%)"
                        )

            conn.commit()

    except psycopg2.Error as exc:
        print(f"    ERROR inserting into {config.table}: {exc}", file=sys.stderr)
        sys.exit(2)

    print(f"    Inserted {len(rows):,} rows into '{config.table}'")
    return len(rows)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Import GeoJSON files into PostGIS tables.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Available datasets:\n"
            + "\n".join(
                f"  {name:20s} -> {cfg.table:25s} ({cfg.file})"
                for name, cfg in DATASETS.items()
            )
        ),
    )
    parser.add_argument(
        "--dataset",
        type=str,
        choices=list(DATASETS.keys()),
        default=None,
        metavar="NAME",
        help="Import a single dataset. Omit to import all.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview row counts without writing to the database.",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=DEFAULT_BATCH_SIZE,
        metavar="N",
        help=f"Number of rows per INSERT batch (default: {DEFAULT_BATCH_SIZE}).",
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
        conn_kwargs = {}

    datasets_to_import: dict[str, DatasetConfig]
    if args.dataset is not None:
        datasets_to_import = {args.dataset: DATASETS[args.dataset]}
    else:
        datasets_to_import = DATASETS

    mode_label = "[DRY RUN] " if args.dry_run else ""
    print("=" * 60)
    print(f"{mode_label}Importing GeoJSON datasets ({len(datasets_to_import)} dataset(s))")
    print(f"Source: {GEOJSON_DIR}")
    if not args.dry_run:
        print(
            f"Target: {conn_kwargs.get('host')}:{conn_kwargs.get('port')}"
            f"/{conn_kwargs.get('dbname')}"
        )
    print(f"Batch size: {args.batch_size}")
    print("=" * 60)

    total_rows = 0
    for name, config in datasets_to_import.items():
        count = import_dataset(
            name,
            config,
            conn_kwargs,
            dry_run=args.dry_run,
            batch_size=args.batch_size,
        )
        total_rows += count

    print("=" * 60)
    verb = "would be " if args.dry_run else ""
    print(f"{mode_label}Done. Total rows {verb}imported: {total_rows:,}")
    print("=" * 60)


if __name__ == "__main__":
    main()
