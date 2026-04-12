"""Utilities for reading features from KSJ ZIP archives.

Prefers GeoJSON over Shapefile when available inside the ZIP.
Handles nested dirs, backslash paths, Shift-JIS/UTF-8 dual dirs.
"""
from __future__ import annotations

import json
import logging
import zipfile
from pathlib import Path

import fiona
from shapely.geometry import shape

logger = logging.getLogger(__name__)


def read_features_from_zip(
    raw_path: Path,
    prefer_utf8: bool = True,
    filename_hint: str | None = None,
) -> list[dict] | None:
    """Read GeoJSON features from a ZIP archive.

    Strategy (in order):
    1. Find .geojson file inside ZIP -> json.load -> return features
    2. Fall back to .shp via fiona if no GeoJSON found

    For ZIPs with Shift-JIS/UTF-8 dirs, prefer UTF-8.
    Normalizes backslash paths in ZIP entries.

    Args:
        raw_path: Path to the ZIP file.
        prefer_utf8: If True, prefer files in UTF-8/ directory over Shift-JIS/.
        filename_hint: If provided, prefer GeoJSON files whose name contains this string.

    Returns:
        List of raw GeoJSON feature dicts, or None if nothing readable.
    """
    try:
        with zipfile.ZipFile(raw_path) as zf:
            # Normalize all paths: backslash -> forward slash
            all_names = [n.replace("\\", "/") for n in zf.namelist()]

            # --- Strategy 1: GeoJSON ---
            geojson_files = [n for n in all_names if n.lower().endswith(".geojson")]

            if geojson_files:
                selected = _select_file(geojson_files, prefer_utf8, filename_hint)
                if selected:
                    # Find the original name in the ZIP (may have backslashes)
                    original_name = _find_original_name(zf, selected)
                    logger.debug(f"Reading GeoJSON from ZIP: {raw_path}!{selected}")
                    with zf.open(original_name) as f:
                        data = json.load(f)
                    features = data.get("features", [])
                    if features:
                        return features
                    logger.debug(f"GeoJSON {selected} had 0 features, trying next")

            # --- Strategy 2: Shapefile via fiona ---
            shp_files = [n for n in all_names if n.lower().endswith(".shp")]
            if not shp_files:
                logger.warning(f"No .geojson or .shp files found in {raw_path}")
                return None

            selected_shp = _select_file(shp_files, prefer_utf8, filename_hint)
            if not selected_shp:
                selected_shp = shp_files[0]

            return _read_shp_from_zip(raw_path, selected_shp)

    except Exception:
        logger.exception(f"Failed to read any data from {raw_path}")
        return None


def _select_file(
    candidates: list[str],
    prefer_utf8: bool,
    filename_hint: str | None,
) -> str | None:
    """Select the best file from candidates based on preferences."""
    if not candidates:
        return None

    filtered = candidates

    # Apply filename hint filter
    if filename_hint:
        matching = [c for c in filtered if filename_hint in c]
        if matching:
            filtered = matching

    # Prefer UTF-8 directory
    if prefer_utf8:
        utf8 = [c for c in filtered if "UTF-8/" in c or "utf-8/" in c]
        if utf8:
            filtered = utf8

    # Avoid Shift-JIS if we have non-Shift-JIS alternatives
    non_sjis = [c for c in filtered if "Shift-JIS/" not in c and "shift-jis/" not in c]
    if non_sjis:
        filtered = non_sjis

    return filtered[0] if filtered else candidates[0]


def _find_original_name(zf: zipfile.ZipFile, normalized: str) -> str:
    """Find the original ZIP entry name matching a normalized path."""
    for name in zf.namelist():
        if name.replace("\\", "/") == normalized:
            return name
    return normalized


def _normalize_props_encoding(props: dict) -> dict:
    """Fix mojibake in Shapefile properties.

    fiona reads Japanese Shapefiles without .cpg as Latin-1 raw bytes.
    Re-encode Latin-1 → CP932 to recover correct Japanese text.
    """
    fixed = {}
    for key, value in props.items():
        if isinstance(value, str) and value:
            try:
                raw = value.encode("latin-1")
                # Only attempt decode if there are high bytes (>0x7F)
                if any(b > 0x7F for b in raw):
                    value = raw.decode("cp932")
            except (UnicodeEncodeError, UnicodeDecodeError):
                pass  # Already valid UTF-8 or other encoding
        fixed[key] = value
    return fixed


def _read_shp_features(src) -> list[dict]:
    """Read all features from an open fiona source, fixing encoding."""
    features = []
    for feat in src:
        features.append({
            "type": "Feature",
            "geometry": feat["geometry"],
            "properties": _normalize_props_encoding(dict(feat["properties"])),
        })
    return features


def _read_shp_from_zip(raw_path: Path, shp_path: str) -> list[dict] | None:
    """Read features from a Shapefile inside a ZIP via fiona."""
    vsi_path = f"zip://{raw_path}!{shp_path}"
    logger.debug(f"Opening Shapefile: {vsi_path}")
    try:
        with fiona.open(vsi_path) as src:
            features = _read_shp_features(src)
    except Exception:
        # Try direct open (flat ZIPs)
        try:
            with fiona.open(f"zip://{raw_path}") as src:
                features = _read_shp_features(src)
        except Exception:
            logger.exception(f"Failed to open shapefile in {raw_path}")
            return None

    return features if features else None


# Keep backward compat alias for any remaining callers
open_zip_shapefile = None  # Removed — use read_features_from_zip
