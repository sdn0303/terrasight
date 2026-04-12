"""Adapter for pre-processed static files (geology, landform, etc.)."""
from __future__ import annotations

import logging
from pathlib import Path

from .base import BaseAdapter, ConvertResult, DatasetEntry

logger = logging.getLogger(__name__)


class ManualStaticAdapter(BaseAdapter):
    """Handles pre-processed static FGB files that are already in place."""

    def convert(
        self,
        entry: DatasetEntry,
        pref_code: str,
        raw_dir: Path,
        output_dir: Path,
    ) -> ConvertResult | None:
        # Static files are pre-placed in data/fgb/
        # This adapter just validates they exist
        if entry.output_fgb is None:
            return None

        fgb_path = raw_dir.parent.parent / entry.output_fgb.replace("{pref_code}", pref_code)
        if fgb_path.exists():
            logger.info(f"Static layer {entry.id} pref={pref_code} exists: {fgb_path}")
            return ConvertResult(
                dataset_id=entry.id,
                pref_code=pref_code,
                output_path=fgb_path,
                feature_count=0,  # Unknown without reading
            )

        logger.debug(f"Static layer {entry.id} pref={pref_code} not found: {fgb_path}")
        return None
