#!/usr/bin/env python3
"""Convert raw geodata to canonical GeoJSON.

Usage:
    uv run scripts/tools/pipeline/convert.py --pref 13 --priority P0
    uv run scripts/tools/pipeline/convert.py --dataset admin-boundary --pref 13
"""
from __future__ import annotations

import argparse
import json
import logging
import sys
from datetime import datetime, timezone
from pathlib import Path

# Add project root to path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent.parent))

from scripts.tools.pipeline.registry import get_adapter, load_catalog

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(name)s: %(message)s",
)
logger = logging.getLogger(__name__)


def main() -> None:
    parser = argparse.ArgumentParser(description="Convert raw data to GeoJSON")
    parser.add_argument("--pref", required=True, help="Prefecture code (e.g., 13) or 'all'")
    parser.add_argument("--priority", default=None, help="Filter by priority (P0, P1, P2)")
    parser.add_argument("--dataset", default=None, help="Filter by dataset ID")
    parser.add_argument("--catalog", default=None, help="Path to catalog JSON")
    args = parser.parse_args()

    catalog_path = Path(args.catalog) if args.catalog else None
    entries = load_catalog(catalog_path)

    # Filter by priority
    if args.priority:
        entries = [e for e in entries if e.priority == args.priority]

    # Filter by dataset ID
    if args.dataset:
        entries = [e for e in entries if e.id == args.dataset]

    # Filter to convertible datasets (have output_geojson)
    entries = [e for e in entries if e.output_geojson is not None]

    if not entries:
        logger.warning("No datasets match the given filters")
        return

    # Determine prefectures
    if args.pref == "all":
        pref_codes = [f"{i:02d}" for i in range(1, 48)]
    else:
        pref_codes = [args.pref.zfill(2)]

    raw_dir = Path("data/raw")
    output_dir = Path("data/geojson")
    results = []

    for entry in entries:
        adapter = get_adapter(entry.adapter)
        for pref in pref_codes:
            if entry.scope == "national" and pref != pref_codes[0]:
                continue  # National datasets only processed once
            result = adapter.convert(entry, pref, raw_dir, output_dir)
            if result:
                results.append({
                    "dataset_id": result.dataset_id,
                    "pref_code": result.pref_code,
                    "output_path": str(result.output_path),
                    "feature_count": result.feature_count,
                    "bbox": list(result.bbox) if result.bbox else None,
                })

    # Write conversion log
    log_path = Path("data/catalog/convert_log.json")
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_data = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "results": results,
    }
    log_path.write_text(json.dumps(log_data, indent=2, ensure_ascii=False), encoding="utf-8")
    logger.info(f"Conversion complete: {len(results)} datasets processed -> {log_path}")


if __name__ == "__main__":
    main()
