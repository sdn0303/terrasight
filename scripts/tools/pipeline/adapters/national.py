"""Adapter for national single-ZIP datasets (N03, S12, N02, L01)."""
from __future__ import annotations

import glob
import json
import logging
from pathlib import Path

from shapely.geometry import box, mapping, shape

from .base import BaseAdapter, ConvertResult, DatasetEntry
from .zip_utils import read_features_from_zip

logger = logging.getLogger(__name__)

# L01 field mapping: version-specific KSJ codes -> canonical names
_L01_FIELD_MAP = {
    "v3.0": {
        "city_code": "L01_017",
        "survey_year": "L01_005",
        "price_per_sqm": "L01_006",
        "address": "L01_019",
        "land_use": "L01_021",
        "zone_type": "L01_029",
        "building_coverage": "L01_030",
        "floor_area_ratio": "L01_031",
        "nearest_station": "L01_027",
        "station_distance": "L01_028",
    },
    "v3.1": {
        "city_code": "L01_001",
        "survey_year": "L01_007",
        "price_per_sqm": "L01_008",
        "address": "L01_025",
        "land_use": "L01_028",
        "zone_type": "L01_051",
        "building_coverage": "L01_057",
        "floor_area_ratio": "L01_058",
        "nearest_station": "L01_048",
        "station_distance": "L01_050",
    },
}


def _detect_l01_version(props: dict) -> str:
    """Detect L01 field version from property keys.

    v3.0 (<=2017): L01_017 = city code (5-digit int), L01_006 = price (int)
    v3.1 (>=2018): L01_001 = city code (5-digit str), L01_008 = price (int)
    """
    # v3.1: L01_001 is city code (5 digits), L01_008 exists
    val_001 = props.get("L01_001")
    if val_001 is not None:
        val_str = str(val_001).strip()
        if len(val_str) == 5 and val_str.isdigit():
            return "v3.1"
    # v3.0: L01_017 is city code
    val_017 = props.get("L01_017")
    if val_017 is not None:
        val_str = str(val_017).strip()
        if val_str.isdigit() and int(val_str) > 0:
            return "v3.0"
    return "unknown"


def _normalize_l01_props(props: dict, version: str) -> dict:
    """Map version-specific L01 fields to canonical names."""
    mapping_table = _L01_FIELD_MAP.get(version)
    if mapping_table is None:
        return props
    result = {}
    for canonical, ksj_key in mapping_table.items():
        val = props.get(ksj_key)
        if val is not None:
            result[canonical] = val
    return result


# Approximate bounding boxes for each prefecture (minlon, minlat, maxlon, maxlat).
# Used for spatial filtering when datasets have no pref_code field (S12, N02).
# Boxes are intentionally generous (~0.1 degree padding) to avoid clipping.
_PREF_BBOX: dict[str, tuple[float, float, float, float]] = {
    "01": (139.3, 41.3, 145.9, 45.6),
    "02": (139.4, 39.5, 141.7, 41.6),
    "03": (139.0, 38.7, 142.1, 40.5),
    "04": (139.8, 37.7, 141.7, 39.0),
    "05": (139.5, 39.0, 140.7, 40.5),
    "06": (139.5, 37.7, 140.7, 39.2),
    "07": (139.1, 36.8, 141.1, 37.9),
    "08": (139.6, 35.7, 140.9, 36.9),
    "09": (139.3, 36.2, 140.3, 37.1),
    "10": (138.6, 36.0, 139.7, 37.1),
    "11": (138.9, 35.7, 139.9, 36.3),
    "12": (139.4, 34.9, 140.9, 36.1),
    "13": (138.9, 35.5, 139.9, 35.9),
    "14": (138.9, 35.1, 139.8, 35.7),
    "15": (137.8, 37.4, 140.0, 38.6),
    "16": (136.7, 36.3, 137.9, 37.0),
    "17": (136.2, 36.0, 137.4, 37.0),
    "18": (135.5, 35.3, 136.9, 36.3),
    "19": (138.1, 35.2, 139.2, 36.0),
    "20": (137.3, 35.2, 138.7, 37.0),
    "21": (136.2, 35.1, 137.7, 36.5),
    "22": (137.4, 34.6, 139.2, 35.6),
    "23": (136.6, 34.5, 137.7, 35.4),
    "24": (135.8, 34.2, 137.0, 35.2),
    "25": (135.7, 34.8, 136.5, 35.6),
    "26": (134.8, 34.8, 136.1, 35.8),
    "27": (135.0, 34.2, 135.8, 35.0),
    "28": (134.2, 34.1, 135.5, 35.7),
    "29": (135.5, 34.1, 136.2, 34.8),
    "30": (135.0, 33.4, 136.1, 34.4),
    "31": (133.2, 35.1, 134.5, 35.7),
    "32": (131.6, 34.3, 133.4, 35.6),
    "33": (133.2, 34.3, 134.4, 35.3),
    "34": (132.0, 34.0, 133.5, 35.0),
    "35": (130.7, 33.7, 132.2, 34.8),
    "36": (133.4, 33.7, 134.8, 34.3),
    "37": (133.5, 34.0, 134.5, 34.5),
    "38": (132.0, 32.9, 133.7, 34.0),
    "39": (132.4, 32.7, 134.3, 33.9),
    "40": (129.9, 33.0, 131.2, 34.0),
    "41": (129.7, 33.0, 130.6, 33.6),
    "42": (128.6, 32.5, 130.4, 34.7),
    "43": (130.0, 32.0, 131.3, 33.3),
    "44": (130.8, 32.7, 132.1, 33.8),
    "45": (130.6, 31.3, 131.9, 32.9),
    "46": (128.4, 27.0, 131.3, 32.1),
    "47": (122.9, 24.0, 131.4, 27.9),
}


def _pref_bbox_polygon(pref_code: str):
    """Return a shapely Polygon for the prefecture bounding box, or None."""
    coords = _PREF_BBOX.get(pref_code)
    if coords is None:
        return None
    return box(*coords)


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
        hint = getattr(entry, "geojson_hint", None)
        raw_features = read_features_from_zip(raw_path, filename_hint=hint)
        if not raw_features:
            return []

        # Detect L01 version once from first feature
        is_l01 = entry.ksj_code == "L01"
        l01_version: str | None = None
        if is_l01 and raw_features:
            first_props = raw_features[0].get("properties", {})
            l01_version = _detect_l01_version(first_props)
            logger.info(f"L01 version detected: {l01_version} for {raw_path.name}")

        # Prepare spatial filter for datasets without pref_code field (S12, N02)
        pref_polygon = _pref_bbox_polygon(pref_code)

        features: list[dict] = []
        for feat in raw_features:
            geom_data = feat.get("geometry")
            if geom_data is None:
                continue
            geom = shape(geom_data)
            if geom.is_empty or not geom.is_valid:
                continue
            props = dict(feat.get("properties", {}))

            # L01: normalize to canonical field names before any other processing
            if is_l01 and l01_version and l01_version != "unknown":
                props = _normalize_l01_props(props, l01_version)

            for old_key, new_key in entry.column_renames.items():
                if old_key in props:
                    props[new_key] = props.pop(old_key)

            feat_pref = self._extract_pref_code(props, entry, geom)
            if feat_pref is not None and feat_pref != pref_code:
                continue

            # Spatial filter: if no pref_code extracted, use bbox intersection
            if feat_pref is None and pref_polygon is not None:
                if not geom.intersects(pref_polygon):
                    continue

            props["pref_code"] = pref_code
            features.append({
                "type": "Feature",
                "geometry": mapping(geom),
                "properties": props,
            })
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
        for key in ("city_code", "adminCode", "N03_007", "L01_017", "S12_001c", "S12_001g"):
            val = props.get(key)
            if val is not None:
                val_str = str(val).strip()
                if len(val_str) >= 2 and val_str[:2].isdigit():
                    code = val_str[:2].zfill(2)
                    num = int(code)
                    if 1 <= num <= 47:
                        return code

        return None
