"""Adapter for per-prefecture archive datasets (A29, P29, P04, L01)."""
from __future__ import annotations

import glob
import json
import logging
from pathlib import Path

import fiona
from shapely.geometry import mapping, shape

from .base import BaseAdapter, ConvertResult, DatasetEntry

logger = logging.getLogger(__name__)


class PerPrefArchiveAdapter(BaseAdapter):
    """Handles datasets with {pref_code} in ZIP filename."""

    def convert(
        self,
        entry: DatasetEntry,
        pref_code: str,
        raw_dir: Path,
        output_dir: Path,
    ) -> ConvertResult | None:
        pattern = entry.raw_pattern
        if pattern is None:
            return None

        # Resolve pattern with pref_code
        resolved = pattern.replace("{pref_code}", pref_code)
        matches = sorted(glob.glob(str(raw_dir.parent.parent / resolved)))

        if not matches:
            logger.warning(f"No raw files for {entry.id} pref={pref_code}: {resolved}")
            return None

        raw_path = Path(matches[-1])  # Use latest if multiple

        # Determine output path
        if entry.output_geojson is None:
            return None
        out_path = output_dir / entry.output_geojson.replace("{pref_code}", pref_code).replace("data/geojson/", "")
        out_path.parent.mkdir(parents=True, exist_ok=True)

        features = []
        try:
            with fiona.open(f"zip://{raw_path}") as src:
                for feat in src:
                    geom = shape(feat["geometry"])
                    if geom.is_empty or not geom.is_valid:
                        continue
                    props = dict(feat["properties"])
                    # Apply column renames
                    for old_key, new_key in entry.column_renames.items():
                        if old_key in props:
                            props[new_key] = props.pop(old_key)
                    props["pref_code"] = pref_code
                    features.append({
                        "type": "Feature",
                        "geometry": mapping(geom),
                        "properties": props,
                    })
        except Exception:
            logger.exception(f"Failed to read {raw_path} for {entry.id}")
            return None

        if not features:
            logger.warning(f"No features extracted for {entry.id} pref={pref_code}")
            return None

        geojson = {
            "type": "FeatureCollection",
            "features": features,
        }
        out_path.write_text(json.dumps(geojson, ensure_ascii=False), encoding="utf-8")
        logger.info(f"Converted {entry.id} pref={pref_code}: {len(features)} features -> {out_path}")

        bbox = _compute_bbox(features)
        return ConvertResult(
            dataset_id=entry.id,
            pref_code=pref_code,
            output_path=out_path,
            feature_count=len(features),
            bbox=bbox,
        )


def _compute_bbox(features: list[dict]) -> tuple[float, float, float, float] | None:
    """Compute bounding box from GeoJSON features."""
    if not features:
        return None
    from shapely.geometry import shape as shp
    from shapely.ops import unary_union
    geoms = [shp(f["geometry"]) for f in features if f.get("geometry")]
    if not geoms:
        return None
    union = unary_union(geoms)
    return union.bounds  # (minx, miny, maxx, maxy)
