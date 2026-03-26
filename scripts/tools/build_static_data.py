#!/usr/bin/env python3
"""Convert GeoJSON files to FlatGeobuf format for static layer serving."""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

import fiona
import geopandas as gpd

ROOT = Path(__file__).resolve().parent.parent.parent  # scripts/tools/ -> scripts/ -> project root

PREFECTURE_LAYERS: list[tuple[str, str]] = [
    ("services/frontend/public/geojson/admin-boundary-tokyo.geojson", "admin-boundary.fgb"),
    ("services/frontend/public/geojson/did-tokyo.geojson", "did.fgb"),
    ("services/frontend/public/geojson/flood-history-tokyo.geojson", "flood-history.fgb"),
    ("services/frontend/public/geojson/geology-tokyo.geojson", "geology.fgb"),
    ("services/frontend/public/geojson/landform-tokyo.geojson", "landform.fgb"),
    ("services/frontend/public/geojson/soil-tokyo.geojson", "soil.fgb"),
    ("services/frontend/public/geojson/pl-liquefaction-tokyo.geojson", "liquefaction.fgb"),
    ("services/frontend/public/geojson/n02-railway-tokyo.geojson", "railway.fgb"),
]

NATIONAL_LAYERS: list[tuple[str, str]] = [
    ("services/frontend/public/geojson/fault-kanto.geojson", "fault.fgb"),
    ("services/frontend/public/geojson/volcano-kanto.geojson", "volcano.fgb"),
    ("services/frontend/public/geojson/jshis-seismic-tokyo.geojson", "seismic.fgb"),
]


def check_flatgeobuf_support() -> bool:
    """Verify that fiona can write FlatGeobuf files."""
    if "FlatGeobuf" in fiona.supported_drivers:
        return True
    print("ERROR: FlatGeobuf driver not available in fiona.", file=sys.stderr)
    print(f"  Available drivers: {sorted(fiona.supported_drivers.keys())}", file=sys.stderr)
    return False


def convert_file(
    src: Path,
    dst: Path,
) -> tuple[int, int] | None:
    """Convert a single GeoJSON file to FlatGeobuf.

    Returns (feature_count, file_size_bytes) on success, None on failure.
    """
    if not src.exists():
        print(f"  SKIP: {src.relative_to(ROOT)} does not exist")
        return None

    try:
        gdf = gpd.read_file(src)
        dst.parent.mkdir(parents=True, exist_ok=True)
        gdf.to_file(dst, driver="FlatGeobuf")
        features = len(gdf)
        size_bytes = dst.stat().st_size
        print(f"  Converting {dst.name}... {features} features → {size_bytes} bytes")
        return features, size_bytes
    except Exception as exc:
        print(f"  ERROR converting {src.name}: {exc}", file=sys.stderr)
        return None


def create_symlink(fgb_dir: Path) -> None:
    """Create dev symlink from frontend public dir to data/fgb."""
    link = ROOT / "services" / "frontend" / "public" / "data" / "fgb"
    if link.exists() or link.is_symlink():
        print(f"  Symlink already exists: {link}")
        return

    link.parent.mkdir(parents=True, exist_ok=True)
    target = os.path.relpath(fgb_dir, link.parent)
    link.symlink_to(target)
    print(f"  Created symlink: {link} → {target}")


def main() -> None:
    print("=== FlatGeobuf Static Data Pipeline ===\n")

    if not check_flatgeobuf_support():
        sys.exit(1)

    manifest: dict[str, object] = {
        "generated": datetime.now(timezone.utc).isoformat(),
        "prefectures": {
            "13": [],
            "national": [],
        },
    }

    total_files = 0
    total_size = 0

    # Prefecture layers (Tokyo = 13)
    pref_out = ROOT / "data" / "fgb" / "13"
    print("Prefecture layers (13 - Tokyo):")
    for src_rel, out_name in PREFECTURE_LAYERS:
        src = ROOT / src_rel
        dst = pref_out / out_name
        result = convert_file(src, dst)
        if result is not None:
            features, size_bytes = result
            layer_name = Path(out_name).stem
            manifest["prefectures"]["13"].append(  # type: ignore[union-attr]
                {"layer": layer_name, "features": features, "size_bytes": size_bytes}
            )
            total_files += 1
            total_size += size_bytes

    # National layers
    nat_out = ROOT / "data" / "fgb" / "national"
    print("\nNational layers:")
    for src_rel, out_name in NATIONAL_LAYERS:
        src = ROOT / src_rel
        dst = nat_out / out_name
        result = convert_file(src, dst)
        if result is not None:
            features, size_bytes = result
            layer_name = Path(out_name).stem
            manifest["prefectures"]["national"].append(  # type: ignore[union-attr]
                {"layer": layer_name, "features": features, "size_bytes": size_bytes}
            )
            total_files += 1
            total_size += size_bytes

    # Write manifest
    manifest_path = ROOT / "data" / "fgb" / "manifest.json"
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    print(f"\nManifest written: {manifest_path.relative_to(ROOT)}")

    # Dev symlink
    print("\nSymlink:")
    create_symlink(ROOT / "data" / "fgb")

    # Summary
    print(f"\n=== Done: {total_files} files, {total_size:,} bytes total ===")


if __name__ == "__main__":
    main()
