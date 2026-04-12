#!/usr/bin/env python3
"""Build FlatGeobuf files from GeoJSON + generate manifest.

Usage:
    uv run scripts/tools/pipeline/build_fgb.py --pref 13
"""
from __future__ import annotations

import argparse
import json
import logging
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent.parent))

from scripts.tools.pipeline.registry import load_catalog

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(name)s: %(message)s",
)
logger = logging.getLogger(__name__)


def build_fgb_from_geojson(geojson_path: Path, fgb_path: Path) -> int:
    """Convert GeoJSON to FlatGeobuf. Returns feature count."""
    import fiona

    fgb_path.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with fiona.open(str(geojson_path)) as src:
        schema = src.schema
        crs = src.crs
        with fiona.open(str(fgb_path), "w", driver="FlatGeobuf", schema=schema, crs=crs) as dst:
            for feat in src:
                dst.write(feat)
                count += 1
    return count


def main() -> None:
    parser = argparse.ArgumentParser(description="Build FlatGeobuf + manifest")
    parser.add_argument("--pref", required=True, help="Prefecture code (e.g., 13)")
    parser.add_argument("--catalog", default=None, help="Path to catalog JSON")
    args = parser.parse_args()

    pref_code = args.pref.zfill(2)
    catalog_path = Path(args.catalog) if args.catalog else None
    entries = load_catalog(catalog_path)

    # Only process static_layer entries that have output_fgb
    static_entries = [e for e in entries if e.static_layer and e.output_fgb]

    manifest: dict = {"version": "2.0.0", "generated": "", "prefectures": {}}
    layers: list[dict] = []

    for entry in static_entries:
        fgb_rel = entry.output_fgb.replace("{pref_code}", pref_code)
        fgb_path = Path(fgb_rel)

        # Check if source GeoJSON exists
        if entry.output_geojson:
            geojson_rel = entry.output_geojson.replace("{pref_code}", pref_code)
            geojson_path = Path(geojson_rel)
            if geojson_path.exists():
                count = build_fgb_from_geojson(geojson_path, fgb_path)
                size_bytes = fgb_path.stat().st_size if fgb_path.exists() else 0
                layers.append({
                    "id": entry.id,
                    "path": fgb_rel.replace("data/fgb/", ""),
                    "features": count,
                    "size_bytes": size_bytes,
                })
                logger.info(f"Built FGB {entry.id}: {count} features -> {fgb_path}")
            else:
                logger.debug(f"GeoJSON not found for {entry.id}: {geojson_path}")

        # If FGB already exists (manual static), register it
        if fgb_path.exists() and not any(l["id"] == entry.id for l in layers):
            size_bytes = fgb_path.stat().st_size
            layers.append({
                "id": entry.id,
                "path": fgb_rel.replace("data/fgb/", ""),
                "features": 0,
                "size_bytes": size_bytes,
            })

    # Build manifest
    manifest["generated"] = datetime.now(timezone.utc).isoformat()
    for layer in layers:
        scope_key = "national" if layer["path"].startswith("national/") else pref_code
        if scope_key not in manifest["prefectures"]:
            manifest["prefectures"][scope_key] = {"layers": []}
        manifest["prefectures"][scope_key]["layers"].append(layer)

    manifest_path = Path("data/fgb/manifest.json")
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")
    logger.info(f"Manifest written: {manifest_path}")


if __name__ == "__main__":
    main()
