#!/usr/bin/env python3
"""Validate pipeline output."""
from __future__ import annotations

import argparse
import json
import logging
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent.parent))

from scripts.tools.pipeline.registry import load_catalog

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s: %(message)s")
logger = logging.getLogger(__name__)


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate pipeline output")
    parser.add_argument("--pref", required=True, help="Prefecture code")
    args = parser.parse_args()

    pref_code = args.pref.zfill(2)
    entries = load_catalog()
    errors = []

    for entry in entries:
        if entry.output_geojson:
            path = Path(entry.output_geojson.replace("{pref_code}", pref_code))
            if path.exists():
                with open(path) as f:
                    data = json.load(f)
                count = len(data.get("features", []))
                logger.info(f"  {entry.id}: {count} features ({path})")
            else:
                logger.debug(f"  {entry.id}: not found ({path})")

        if entry.output_fgb:
            fgb = Path(entry.output_fgb.replace("{pref_code}", pref_code))
            if fgb.exists():
                logger.info(f"  {entry.id} FGB: {fgb.stat().st_size} bytes ({fgb})")

    # Validate manifest
    manifest = Path("data/fgb/manifest.json")
    if manifest.exists():
        with open(manifest) as f:
            m = json.load(f)
        for scope, data in m.get("prefectures", {}).items():
            for layer in data.get("layers", []):
                fgb_path = Path("data/fgb") / layer["path"]
                if not fgb_path.exists():
                    errors.append(f"Manifest references missing FGB: {fgb_path}")

    if errors:
        for e in errors:
            logger.error(e)
        sys.exit(1)
    else:
        logger.info("Validation passed")


if __name__ == "__main__":
    main()
