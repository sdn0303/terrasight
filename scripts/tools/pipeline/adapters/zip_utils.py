"""Utilities for opening nested KSJ ZIP archives with fiona."""
from __future__ import annotations

import logging
import zipfile
from pathlib import Path

import fiona

logger = logging.getLogger(__name__)


def open_zip_shapefile(raw_path: Path, prefer_utf8: bool = True):
    """Open a Shapefile inside a (possibly nested) ZIP archive.

    KSJ ZIP archives have varying structures:
    - Flat: shapefiles at root (L01, P29-21_XX)
    - Subdirectory: files inside a single subfolder (A29, P04, P29-XX)
    - Shift-JIS/UTF-8: dual-encoding dirs (S12, N02)

    This function tries multiple strategies in order:
    1. Direct fiona.open("zip://path.zip")
    2. fiona.open("zip://path.zip!subdir/file.shp") for nested .shp
    3. Prefer UTF-8 directory if available
    """
    # Strategy 1: try direct open (works for flat ZIPs)
    try:
        return fiona.open(f"zip://{raw_path}")
    except Exception:
        pass

    # Strategy 2: inspect ZIP and find .shp files
    try:
        with zipfile.ZipFile(raw_path) as zf:
            # Normalize all paths to forward slashes for consistent matching
            all_names = [n.replace("\\", "/") for n in zf.namelist()]
            shp_files = [n for n in all_names if n.lower().endswith(".shp")]

            if not shp_files:
                logger.warning(f"No .shp files found in {raw_path}")
                return None

            # Prefer UTF-8 directory if available
            if prefer_utf8:
                utf8_shps = [s for s in shp_files if "UTF-8/" in s or "utf-8/" in s]
                if utf8_shps:
                    shp_files = utf8_shps

            # Use the first (or only) shapefile.
            # Normalize backslashes — KSJ ZIPs created on Windows use '\'
            # but GDAL/fiona vsi paths require '/'.
            shp_path = shp_files[0].replace("\\", "/")
            vsi_path = f"zip://{raw_path}!{shp_path}"
            logger.debug(f"Opening nested shapefile: {vsi_path}")
            return fiona.open(vsi_path)
    except Exception:
        logger.exception(f"Failed to open any shapefile in {raw_path}")
        return None
