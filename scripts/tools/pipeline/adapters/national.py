"""Adapter for national single-ZIP datasets (N03, S12, N02, L01)."""
from __future__ import annotations

import glob
import json
import logging
from pathlib import Path

from shapely.geometry import mapping, shape

from .base import BaseAdapter, ConvertResult, DatasetEntry
from .zip_utils import open_zip_shapefile

logger = logging.getLogger(__name__)


class NationalArchiveAdapter(BaseAdapter):
    """Handles national ZIPs that need pref_code filtering."""

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

        matches = sorted(glob.glob(str(raw_dir.parent.parent / pattern)))
        if not matches:
            logger.warning(f"No raw files for {entry.id}: {pattern}")
            return None

        if entry.output_geojson is None:
            return None
        out_rel = entry.output_geojson.replace("{pref_code}", pref_code).replace("data/geojson/", "")
        out_path = output_dir / out_rel
        out_path.parent.mkdir(parents=True, exist_ok=True)

        # Try archives from newest to oldest — some newer KSJ Shapefiles
        # have broken attribute encoding. Fall back until we get features.
        features: list[dict] = []
        for raw_match in reversed(matches):
            raw_path = Path(raw_match)
            features = self._read_and_filter(raw_path, entry, pref_code)
            if features:
                break
            logger.debug(f"No usable features from {raw_path}, trying older archive")

        if not features:
            logger.warning(f"No features for {entry.id} pref={pref_code} after filtering")
            return None

        geojson = {"type": "FeatureCollection", "features": features}
        out_path.write_text(json.dumps(geojson, ensure_ascii=False), encoding="utf-8")
        logger.info(f"Converted {entry.id} pref={pref_code}: {len(features)} features -> {out_path}")

        from .per_pref import _compute_bbox
        return ConvertResult(
            dataset_id=entry.id,
            pref_code=pref_code,
            output_path=out_path,
            feature_count=len(features),
            bbox=_compute_bbox(features),
        )

    def _read_and_filter(
        self, raw_path: Path, entry: DatasetEntry, pref_code: str
    ) -> list[dict]:
        """Read features from archive and filter by pref_code."""
        features: list[dict] = []
        try:
            src = open_zip_shapefile(raw_path)
            if src is None:
                return []
            with src:
                for feat in src:
                    geom = shape(feat["geometry"])
                    if geom.is_empty or not geom.is_valid:
                        continue
                    props = dict(feat["properties"])
                    for old_key, new_key in entry.column_renames.items():
                        if old_key in props:
                            props[new_key] = props.pop(old_key)

                    feat_pref = self._extract_pref_code(props, entry, geom)
                    if feat_pref is not None and feat_pref != pref_code:
                        continue

                    props["pref_code"] = pref_code
                    features.append({
                        "type": "Feature",
                        "geometry": mapping(geom),
                        "properties": props,
                    })
        except Exception:
            logger.exception(f"Failed to read {raw_path} for {entry.id}")
        return features

    def _extract_pref_code(self, props: dict, entry: DatasetEntry, geom=None) -> str | None:
        """Extract prefecture code from feature properties.

        Tries multiple strategies:
        1. Direct pref code fields (prefCode, N03_001, pref_code)
        2. Admin/city code prefix (adminCode, N03_007, L01_017, S12_001c)
        """
        # Strategy 1: direct pref code fields
        for key in ("prefCode", "N03_001", "pref_code"):
            val = props.get(key)
            if val and isinstance(val, str) and len(val) >= 2:
                return val[:2].zfill(2)

        # Strategy 2: extract from municipality/admin codes
        for key in ("adminCode", "N03_007", "L01_017", "S12_001c", "S12_001g"):
            val = props.get(key)
            if val is not None:
                val_str = str(val).strip()
                if len(val_str) >= 2 and val_str[:2].isdigit():
                    code = val_str[:2].zfill(2)
                    num = int(code)
                    if 1 <= num <= 47:
                        return code

        return None
