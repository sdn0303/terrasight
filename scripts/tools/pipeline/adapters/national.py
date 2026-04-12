"""Adapter for national single-ZIP datasets (N03, S12, N02, A00)."""
from __future__ import annotations

import glob
import json
import logging
from pathlib import Path

import fiona
from shapely.geometry import mapping, shape

from .base import BaseAdapter, ConvertResult, DatasetEntry

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

                    # Filter by pref_code
                    feat_pref = self._extract_pref_code(props, entry)
                    if feat_pref != pref_code:
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

    def _extract_pref_code(self, props: dict, entry: DatasetEntry) -> str | None:
        """Extract prefecture code from feature properties."""
        # Try common KSJ field patterns
        for key in ("prefCode", "N03_001", "adminCode", "pref_code"):
            val = props.get(key)
            if val and isinstance(val, str) and len(val) >= 2:
                return val[:2].zfill(2)
        # Try adminCode prefix
        admin = props.get("adminCode") or props.get("N03_007")
        if admin and isinstance(admin, str) and len(admin) >= 2:
            return admin[:2].zfill(2)
        return None
