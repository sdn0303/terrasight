"""Adapter for per-prefecture archive datasets (A29, P29, P04, L01)."""
from __future__ import annotations

import glob
import json
import logging
import re
from pathlib import Path

from shapely.geometry import mapping, shape

from .base import BaseAdapter, ConvertResult, DatasetEntry
from .zip_utils import read_features_from_zip

logger = logging.getLogger(__name__)


class PerPrefArchiveAdapter(BaseAdapter):
    """Handles datasets with {pref_code} in ZIP filename.

    For datasets with {year} in output_geojson (e.g. land-price), each
    matched archive is treated as a separate year and written to its own
    output file. All features are also aggregated into a single
    ConvertResult so the caller gets a combined count.
    """

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

        resolved = pattern.replace("{pref_code}", pref_code)
        matches = sorted(glob.glob(str(raw_dir.parent.parent / resolved)))

        if not matches:
            logger.warning(f"No raw files for {entry.id} pref={pref_code}: {resolved}")
            return None

        if entry.output_geojson is None:
            return None

        has_year_placeholder = "{year}" in entry.output_geojson
        all_features: list[dict] = []
        last_out_path: Path | None = None

        for raw_match in matches:
            raw_path = Path(raw_match)
            year = _extract_year_from_filename(raw_path.name) if has_year_placeholder else None

            out_rel = entry.output_geojson.replace("{pref_code}", pref_code)
            if year and has_year_placeholder:
                out_rel = out_rel.replace("{year}", year)
            out_rel = out_rel.replace("data/geojson/", "")
            out_path = output_dir / out_rel
            out_path.parent.mkdir(parents=True, exist_ok=True)

            features = _read_features(raw_path, entry, pref_code)
            if not features:
                continue

            geojson = {"type": "FeatureCollection", "features": features}
            out_path.write_text(json.dumps(geojson, ensure_ascii=False), encoding="utf-8")
            logger.info(
                f"Converted {entry.id} pref={pref_code}"
                + (f" year={year}" if year else "")
                + f": {len(features)} features -> {out_path}"
            )
            all_features.extend(features)
            last_out_path = out_path

            # For non-year datasets, only process the latest archive
            if not has_year_placeholder:
                break

        if not all_features or last_out_path is None:
            return None

        return ConvertResult(
            dataset_id=entry.id,
            pref_code=pref_code,
            output_path=last_out_path,
            feature_count=len(all_features),
            bbox=_compute_bbox(all_features),
        )


def _read_features(raw_path: Path, entry: DatasetEntry, pref_code: str) -> list[dict]:
    """Read features from a single archive."""
    raw_features = read_features_from_zip(raw_path)
    if not raw_features:
        return []

    features = []
    for feat in raw_features:
        geom_data = feat.get("geometry")
        if geom_data is None:
            continue
        geom = shape(geom_data)
        if geom.is_empty or not geom.is_valid:
            continue
        props = dict(feat.get("properties", {}))
        for old_key, new_key in entry.column_renames.items():
            if old_key in props:
                props[new_key] = props.pop(old_key)
        props["pref_code"] = pref_code
        features.append({
            "type": "Feature",
            "geometry": mapping(geom),
            "properties": props,
        })
    return features


def _extract_year_from_filename(filename: str) -> str | None:
    """Extract a 4-digit year from a filename like L01-25_13_GML.zip."""
    # Try common KSJ patterns: L01-25 means 令和7年=2025, but also check 4-digit
    m = re.search(r"(\d{4})", filename)
    if m:
        return m.group(1)
    # KSJ 2-digit year: L01-{YY} where YY is Japanese fiscal year offset
    m = re.search(r"-(\d{2})[_.]", filename)
    if m:
        yy = int(m.group(1))
        # Heuristic: values < 50 are Reiwa era (2019+), else Heisei
        year = 2000 + yy if yy < 50 else 1988 + yy
        return str(year)
    return None


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
    return union.bounds
