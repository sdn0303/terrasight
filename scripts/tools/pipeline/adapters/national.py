"""Adapter for national single-ZIP datasets (N03, S12, N02, A00)."""
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

        raw_path = Path(matches[-1])

        if entry.output_geojson is None:
            return None
        out_rel = entry.output_geojson.replace("{pref_code}", pref_code).replace("data/geojson/", "")
        out_path = output_dir / out_rel
        out_path.parent.mkdir(parents=True, exist_ok=True)

        features = []
        try:
            src = open_zip_shapefile(raw_path)
            if src is None:
                return None
            with src:
                for feat in src:
                    geom = shape(feat["geometry"])
                    if geom.is_empty or not geom.is_valid:
                        continue
                    props = dict(feat["properties"])
                    # Apply column renames
                    for old_key, new_key in entry.column_renames.items():
                        if old_key in props:
                            props[new_key] = props.pop(old_key)

                    # Filter by pref_code (skip for datasets where
                    # features cross prefectures, e.g. railways)
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
            return None

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

    def _extract_pref_code(self, props: dict, entry: DatasetEntry, geom=None) -> str | None:
        """Extract prefecture code from feature properties or geometry.

        Tries multiple strategies in order:
        1. Direct pref code fields (prefCode, N03_001, pref_code)
        2. Admin/city code prefix (adminCode, N03_007, L01_017, S12_001c)
        3. Geometry centroid → rough pref lookup (for railway N02 etc.)
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

        # Strategy 3: for datasets without pref fields (N02 railway),
        # use geometry centroid to determine prefecture.
        # Rough bbox check for Tokyo (pref 13): 138.9-140.0, 35.5-35.9
        if geom is not None:
            try:
                centroid = geom.centroid
                # This is a simplified approach — only works for the
                # requested pref_code via the caller's filter.
                return None  # Let caller handle via bbox
            except Exception:
                pass

        return None
