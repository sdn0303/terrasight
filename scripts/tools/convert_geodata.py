#!/usr/bin/env python3
"""
Convert raw MLIT/J-SHIS geodata to optimized GeoJSON for PostGIS import.

Reads from data/raw/ (ZIP archives), filters to Tokyo 23-ku area,
normalizes properties, and writes to data/geojson/.

Usage:
    uv run scripts/tools/convert_geodata.py                  # all datasets
    uv run scripts/tools/convert_geodata.py l01 a29 a31b     # specific datasets
    uv run scripts/tools/convert_geodata.py --list            # show available datasets

Dependencies:
    Managed by uv (see scripts/pyproject.toml)
"""

from __future__ import annotations

import json
import sys
import tempfile
import zipfile
from pathlib import Path
from typing import Callable

import geopandas as gpd
import pandas as pd
from shapely.geometry import box

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent.parent.parent  # scripts/tools/ -> scripts/ -> project root
RAW_DIR = ROOT / "data" / "raw"
OUT_DIR = ROOT / "data" / "geojson"

# Tokyo 23-ku bbox [minLng, minLat, maxLng, maxLat]
TOKYO_BBOX = box(138.9, 35.4, 140.0, 35.95)

# Kanto wider bbox for context layers
KANTO_BBOX = box(138.5, 34.8, 140.9, 37.0)

TOKYO_PREF_CODE = "13"

TARGET_CRS = "EPSG:4326"

COORD_PRECISION = 5  # ~1.1m — sufficient for city-scale


# ---------------------------------------------------------------------------
# Core helpers
# ---------------------------------------------------------------------------


def find_raw(pattern: str) -> list[Path]:
    """Find raw ZIP files matching a glob pattern."""
    return sorted(RAW_DIR.glob(pattern))


def read_zip(
    zip_path: Path,
    encoding: str = "cp932",
) -> gpd.GeoDataFrame:
    """Read a geospatial file from inside a ZIP archive.

    Auto-detects format with priority: GeoJSON > Shapefile > GML.
    Handles cp932/utf-8 encoding fallback for Japanese MLIT data.
    """
    with zipfile.ZipFile(zip_path) as zf:
        names = zf.namelist()
        has_geojson = any(n.endswith(".geojson") for n in names)
        has_shp = any(n.endswith(".shp") for n in names)

    if has_geojson:
        return _read_json_from_zip(zip_path)
    elif has_shp:
        return _read_shp_from_zip(zip_path, encoding)
    else:
        return _read_gml_from_zip(zip_path, encoding)


def _read_shp_from_zip(
    zip_path: Path, encoding: str = "cp932"
) -> gpd.GeoDataFrame:
    """Read Shapefile(s) from ZIP with encoding fallback."""
    with zipfile.ZipFile(zip_path) as zf:
        shp_names = [n for n in zf.namelist() if n.endswith(".shp")]
        if not shp_names:
            raise FileNotFoundError(f"No .shp found in {zip_path.name}")

    frames: list[gpd.GeoDataFrame] = []
    for shp_name in shp_names:
        uri = f"zip://{zip_path}!{shp_name}"
        for enc in [encoding, "utf-8", "latin-1"]:
            try:
                gdf = gpd.read_file(uri, encoding=enc)
                frames.append(gdf)
                break
            except (UnicodeDecodeError, Exception):
                continue

    if not frames:
        raise RuntimeError(f"Failed to read any .shp from {zip_path.name}")

    return pd.concat(frames, ignore_index=True) if len(frames) > 1 else frames[0]


def _read_json_from_zip(zip_path: Path) -> gpd.GeoDataFrame:
    """Read GeoJSON from inside a ZIP archive."""
    with zipfile.ZipFile(zip_path) as zf:
        json_names = [
            n
            for n in zf.namelist()
            if (n.endswith(".geojson") or n.endswith(".json"))
            and "__MACOSX" not in n
        ]
        if not json_names:
            raise FileNotFoundError(f"No .geojson found in {zip_path.name}")

        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            for name in json_names:
                zf.extract(name, tmp_path)
            frames = [
                gpd.read_file(tmp_path / name) for name in json_names
            ]

    return pd.concat(frames, ignore_index=True) if len(frames) > 1 else frames[0]


def _read_gml_from_zip(
    zip_path: Path, encoding: str = "cp932"
) -> gpd.GeoDataFrame:
    """Read GML from inside a ZIP archive."""
    with zipfile.ZipFile(zip_path) as zf:
        gml_names = [n for n in zf.namelist() if n.endswith(".geojson")]
        if not gml_names:
            gml_names = [n for n in zf.namelist() if n.endswith(".gml")]
        if not gml_names:
            raise FileNotFoundError(f"No .gml/.geojson found in {zip_path.name}")

        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            # Extract all files (GML needs associated .xsd etc.)
            zf.extractall(tmp_path)
            # Try GeoJSON first (some MLIT ZIPs include both)
            target = gml_names[0]
            gdf = gpd.read_file(tmp_path / target, encoding=encoding)

    return gdf


def ensure_wgs84(gdf: gpd.GeoDataFrame) -> gpd.GeoDataFrame:
    """Ensure GeoDataFrame is in WGS84 (EPSG:4326)."""
    if gdf.crs is None:
        gdf = gdf.set_crs(TARGET_CRS)
    elif gdf.crs.to_epsg() != 4326:
        gdf = gdf.to_crs(TARGET_CRS)
    return gdf


def clip_tokyo(gdf: gpd.GeoDataFrame) -> gpd.GeoDataFrame:
    """Spatial clip to Tokyo metro bbox."""
    return gpd.clip(gdf, TOKYO_BBOX)


def clip_kanto(gdf: gpd.GeoDataFrame) -> gpd.GeoDataFrame:
    """Spatial clip to wider Kanto bbox."""
    return gpd.clip(gdf, KANTO_BBOX)


def force_2d(gdf: gpd.GeoDataFrame) -> gpd.GeoDataFrame:
    """Remove Z coordinates (PolygonZ → Polygon)."""
    from shapely import force_2d as shapely_force_2d

    gdf = gdf.copy()
    gdf["geometry"] = gdf["geometry"].apply(
        lambda g: shapely_force_2d(g) if g is not None else None
    )
    return gdf


def set_precision(gdf: gpd.GeoDataFrame, precision: int = COORD_PRECISION) -> gpd.GeoDataFrame:
    """Round coordinates to reduce file size."""
    from shapely import set_precision as shapely_set_precision

    # precision=5 means ~1.1m, which corresponds to ~0.00001 degrees
    grid_size = 10 ** (-precision)
    gdf = gdf.copy()
    gdf["geometry"] = gdf["geometry"].apply(
        lambda g: shapely_set_precision(g, grid_size) if g is not None else None
    )
    return gdf


def write_geojson(gdf: gpd.GeoDataFrame, name: str) -> Path:
    """Write GeoDataFrame as compact GeoJSON to data/geojson/."""
    out_path = OUT_DIR / f"{name}.geojson"
    # Drop rows with no geometry
    gdf = gdf[gdf.geometry.notna()].copy()

    if gdf.empty:
        print(f"  ⚠ {name}: 0 features — skipped")
        return out_path

    gdf.to_file(out_path, driver="GeoJSON")

    size_kb = out_path.stat().st_size / 1024
    print(f"  → {name}.geojson: {len(gdf)} features, {size_kb:.0f} KB")
    return out_path


# ---------------------------------------------------------------------------
# Dataset processors
# ---------------------------------------------------------------------------


def process_l01() -> None:
    """L01 地価公示 — 5 years for Tokyo (sparkline data)."""
    print("\n[L01] 地価公示 (Tokyo, 5 years)")

    for year in ["22", "23", "24", "25", "26"]:
        zips = find_raw(f"L01-{year}_GML.zip")
        if not zips:
            print(f"  SKIP: L01-{year}_GML.zip not found")
            continue

        gdf = read_zip(zips[0])
        gdf = ensure_wgs84(gdf)
        gdf = clip_tokyo(gdf)
        gdf = set_precision(gdf)
        write_geojson(gdf, f"l01-20{year}-tokyo")


def process_a29() -> None:
    """A29 用途地域 — Tokyo (2011 version)."""
    print("\n[A29] 用途地域 (Tokyo, 2011)")

    zips = find_raw("A29-11_13_GML.zip")
    if not zips:
        print("  SKIP: A29-11_13_GML.zip not found")
        return

    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "a29-zoning-tokyo")


def process_a31b() -> None:
    """A31b 洪水浸水想定区域 — Tokyo meshes (5339, 5340)."""
    print("\n[A31b] 洪水浸水想定区域 (Tokyo meshes)")

    tokyo_meshes = ["5339", "5340"]
    frames: list[gpd.GeoDataFrame] = []

    for mesh in tokyo_meshes:
        zips = find_raw(f"A31b-24_10_{mesh}_GEOJSON.zip")
        if not zips:
            print(f"  SKIP: A31b mesh {mesh} not found")
            continue

        print(f"  Reading mesh {mesh} ({zips[0].stat().st_size // (1024*1024)} MB)...")
        gdf = read_zip(zips[0])
        frames.append(gdf)

    if not frames:
        return

    gdf = pd.concat(frames, ignore_index=True)
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = force_2d(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "a31b-flood-tokyo")


def process_a33() -> None:
    """A33 土砂災害警戒区域 — filter to Tokyo from national dataset."""
    print("\n[A33] 土砂災害警戒区域 (Tokyo)")

    zips = find_raw("A33-24_00_GEOJSON.zip")
    if not zips:
        print("  SKIP: A33-24_00_GEOJSON.zip not found")
        return

    print(f"  Reading national dataset ({zips[0].stat().st_size // (1024*1024)} MB)...")
    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = force_2d(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "a33-landslide-tokyo")


def process_a40() -> None:
    """A40 津波浸水想定 — Tokyo."""
    print("\n[A40] 津波浸水想定 (Tokyo)")

    zips = find_raw("A40-23_13_GML.zip")
    if not zips:
        print("  SKIP: A40-23_13_GML.zip not found")
        return

    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = force_2d(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "a40-tsunami-tokyo")


def process_a47() -> None:
    """A47 急傾斜地崩壊危険区域 — Tokyo."""
    print("\n[A47] 急傾斜地崩壊危険区域 (Tokyo)")

    zips = find_raw("A47-21_13_GML.zip")
    if not zips:
        print("  SKIP: A47-21_13_GML.zip not found")
        return

    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = force_2d(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "a47-steep-slope-tokyo")


def process_p29() -> None:
    """P29 学校 — Tokyo (2021)."""
    print("\n[P29] 学校 (Tokyo, 2021)")

    zips = find_raw("P29-21_13_GML.zip")
    if not zips:
        print("  SKIP: P29-21_13_GML.zip not found")
        return

    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "p29-schools-tokyo")


def process_p04() -> None:
    """P04 医療機関 — Tokyo (2020)."""
    print("\n[P04] 医療機関 (Tokyo, 2020)")

    zips = find_raw("P04-20_13_GML.zip")
    if not zips:
        print("  SKIP: P04-20_13_GML.zip not found")
        return

    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "p04-medical-tokyo")


def process_pl() -> None:
    """PL分布図 液状化指数 — Tokyo (Shapefile from 東京都建設局).

    Note: Tokyo Bureau of Construction uses UTF-8 encoding (not cp932).
    """
    print("\n[PL] 液状化指数 (Tokyo)")

    zips = find_raw("PL分布図.zip")
    if not zips:
        print("  SKIP: PL分布図.zip not found")
        return

    gdf = _read_shp_from_zip(zips[0], encoding="utf-8")
    gdf = ensure_wgs84(gdf)
    gdf = force_2d(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "pl-liquefaction-tokyo")


def process_s12() -> None:
    """S12 駅別乗降客数 — Tokyo (2024, latest)."""
    print("\n[S12] 駅別乗降客数 (Tokyo, 2024)")

    zips = find_raw("S12-24_GML.zip")
    if not zips:
        print("  SKIP: S12-24_GML.zip not found")
        return

    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "s12-stations-tokyo")


def process_n02() -> None:
    """N02 鉄道 — Tokyo (2023)."""
    print("\n[N02] 鉄道 (Tokyo, 2023)")

    zips = find_raw("N02-23_GML.zip")
    if not zips:
        print("  SKIP: N02-23_GML.zip not found")
        return

    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = force_2d(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "n02-railway-tokyo")


def process_n03() -> None:
    """N03 行政区域 — Tokyo."""
    print("\n[N03] 行政区域 (Tokyo)")

    zips = find_raw("N03-20250101_GML.zip")
    if not zips:
        print("  SKIP: N03-20250101_GML.zip not found")
        return

    print(f"  Reading national dataset ({zips[0].stat().st_size // (1024*1024)} MB)...")
    gdf = read_zip(zips[0])
    gdf = ensure_wgs84(gdf)

    # Filter by admin code prefix (Tokyo = 13)
    admin_col = None
    for col in gdf.columns:
        if "N03_001" in col or "都道府県" in col:
            admin_col = col
            break

    if admin_col:
        gdf = gdf[gdf[admin_col] == "東京都"]
    else:
        gdf = clip_tokyo(gdf)

    gdf = set_precision(gdf)
    write_geojson(gdf, "n03-admin-boundary-tokyo")


def process_mesh500() -> None:
    """500mメッシュ将来推計人口 — Tokyo (ZIP-in-ZIP: outer contains per-prefecture ZIPs)."""
    print("\n[500m mesh] 将来推計人口 (Tokyo)")

    zips = find_raw("500m_mesh_2024_GEOJSON.zip")
    if not zips:
        print("  SKIP: 500m_mesh_2024_GEOJSON.zip not found")
        return

    outer_zip = zips[0]
    print(f"  Extracting outer ZIP ({outer_zip.stat().st_size // (1024*1024)} MB)...")

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        with zipfile.ZipFile(outer_zip) as zf:
            # Find inner Tokyo ZIP (prefecture code 13)
            inner_names = [
                n for n in zf.namelist()
                if n.endswith(".zip") and "_13_" in n
            ]
            if not inner_names:
                print("  SKIP: No Tokyo (13) inner ZIP found")
                return

            zf.extract(inner_names[0], tmp_path)
            inner_zip = tmp_path / inner_names[0]

        print(f"  Reading inner ZIP: {inner_names[0]}")
        gdf = read_zip(inner_zip)

    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "mesh500-population-tokyo")


def process_seismic() -> None:
    """P-Y2024-PRM-SHAPE J-SHIS 確率論的地震動予測地図 — Tokyo."""
    print("\n[J-SHIS] 確率論的地震動予測地図 (Tokyo)")

    zips = find_raw("P-Y2024-PRM-SHAPE.zip")
    if not zips:
        print("  SKIP: P-Y2024-PRM-SHAPE.zip not found")
        return

    gdf = _read_shp_from_zip(zips[0])
    gdf = ensure_wgs84(gdf)
    gdf = clip_tokyo(gdf)
    gdf = set_precision(gdf)
    write_geojson(gdf, "jshis-seismic-tokyo")


def process_transactions() -> None:
    """Tokyo 不動産取引価格 CSV → GeoJSON (station-based geocoding)."""
    print("\n[Transactions] 不動産取引価格 (Tokyo)")

    csvs = find_raw("Tokyo_*.csv")
    if not csvs:
        print("  SKIP: Tokyo_*.csv not found")
        return

    # This needs station coords for geocoding — skip if not available
    station_coords = ROOT / "services" / "backend" / "data" / "station_coords.json"
    if not station_coords.exists():
        print("  SKIP: station_coords.json not found (needed for geocoding)")
        return

    with open(station_coords, encoding="utf-8") as f:
        stations: dict[str, dict] = json.load(f)

    # Read CSV with cp932 encoding
    df = pd.read_csv(csvs[0], encoding="cp932")
    print(f"  Total records: {len(df)}")

    # Geocode by nearest station
    station_col = "最寄駅：名称"
    if station_col not in df.columns:
        print(f"  SKIP: column '{station_col}' not found")
        return

    lats, lngs = [], []
    for _, row in df.iterrows():
        station_name = str(row.get(station_col, ""))
        coord = stations.get(station_name)
        if coord:
            lats.append(coord["lat"])
            lngs.append(coord["lng"])
        else:
            lats.append(None)
            lngs.append(None)

    df["latitude"] = lats
    df["longitude"] = lngs

    # Drop rows without coordinates
    df = df.dropna(subset=["latitude", "longitude"])

    gdf = gpd.GeoDataFrame(
        df,
        geometry=gpd.points_from_xy(df["longitude"], df["latitude"]),
        crs=TARGET_CRS,
    )
    gdf = gdf.drop(columns=["latitude", "longitude"])
    gdf = set_precision(gdf)
    write_geojson(gdf, "transactions-tokyo")


# ---------------------------------------------------------------------------
# Registry
# ---------------------------------------------------------------------------

PROCESSORS: dict[str, tuple[str, Callable[[], None]]] = {
    "l01": ("L01 地価公示 (5年分)", process_l01),
    "a29": ("A29 用途地域", process_a29),
    "a31b": ("A31b 洪水浸水想定区域", process_a31b),
    "a33": ("A33 土砂災害警戒区域", process_a33),
    "a40": ("A40 津波浸水想定", process_a40),
    "a47": ("A47 急傾斜地崩壊危険区域", process_a47),
    "p29": ("P29 学校", process_p29),
    "p04": ("P04 医療機関", process_p04),
    "pl": ("PL 液状化指数", process_pl),
    "s12": ("S12 駅別乗降客数", process_s12),
    "n02": ("N02 鉄道", process_n02),
    "n03": ("N03 行政区域", process_n03),
    "mesh500": ("500m メッシュ将来推計人口", process_mesh500),
    "seismic": ("J-SHIS 地震動予測地図", process_seismic),
    "transactions": ("不動産取引価格 CSV", process_transactions),
}


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    args = [a.lower() for a in sys.argv[1:]]

    if "--list" in args or "-l" in args:
        print("Available datasets:")
        for key, (desc, _) in PROCESSORS.items():
            print(f"  {key:15s} {desc}")
        return

    if "--help" in args or "-h" in args:
        print(__doc__)
        return

    # Select processors
    if args:
        selected = {k: v for k, v in PROCESSORS.items() if k in args}
        unknown = [a for a in args if a not in PROCESSORS and not a.startswith("-")]
        if unknown:
            print(f"Unknown datasets: {', '.join(unknown)}")
            print("Use --list to see available datasets")
            sys.exit(1)
    else:
        selected = PROCESSORS

    print("=" * 60)
    print(f"Converting {len(selected)} dataset(s): raw/ → geojson/")
    print("=" * 60)

    for key, (desc, func) in selected.items():
        try:
            func()
        except Exception as e:
            print(f"  ✗ {key}: {e}")

    print("\n" + "=" * 60)
    print(f"Output: {OUT_DIR}")
    for f in sorted(OUT_DIR.glob("*.geojson")):
        size_mb = f.stat().st_size / (1024 * 1024)
        print(f"  {f.name}: {size_mb:.1f} MB")
    print("=" * 60)


if __name__ == "__main__":
    main()
