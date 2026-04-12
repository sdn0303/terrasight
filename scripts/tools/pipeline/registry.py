"""Adapter registry -- maps catalog adapter names to implementations."""
from __future__ import annotations

import json
import logging
from pathlib import Path

from .adapters.base import BaseAdapter, DatasetEntry
from .adapters.manual_static import ManualStaticAdapter
from .adapters.national import NationalArchiveAdapter
from .adapters.per_pref import PerPrefArchiveAdapter

logger = logging.getLogger(__name__)

ADAPTER_MAP: dict[str, type[BaseAdapter]] = {
    "PerPrefArchiveAdapter": PerPrefArchiveAdapter,
    "NationalArchiveAdapter": NationalArchiveAdapter,
    "ManualStaticAdapter": ManualStaticAdapter,
    "NestedPerPrefArchiveAdapter": PerPrefArchiveAdapter,  # Fallback
    "IrregularPrefArchiveAdapter": PerPrefArchiveAdapter,  # Fallback
}


def load_catalog(catalog_path: Path | None = None) -> list[DatasetEntry]:
    """Load dataset catalog from JSON."""
    if catalog_path is None:
        catalog_path = Path("data/catalog/dataset_catalog.json")

    with open(catalog_path, encoding="utf-8") as f:
        raw = json.load(f)

    entries = []
    for d in raw["datasets"]:
        entries.append(DatasetEntry(
            id=d["id"],
            name=d["name"],
            scope=d.get("scope", "per_prefecture"),
            source_type=d.get("source_type", ""),
            adapter=d.get("adapter", ""),
            raw_pattern=d.get("raw_pattern"),
            output_geojson=d.get("output_geojson"),
            output_fgb=d.get("output_fgb"),
            db_table=d.get("db_table"),
            static_layer=d.get("static_layer", False),
            priority=d.get("priority", "P0"),
            column_renames=d.get("column_renames", {}),
            ksj_code=d.get("ksj_code"),
            update_frequency=d.get("update_frequency"),
        ))
    return entries


def get_adapter(adapter_name: str) -> BaseAdapter:
    """Get adapter instance by name."""
    cls = ADAPTER_MAP.get(adapter_name)
    if cls is None:
        raise ValueError(f"Unknown adapter: {adapter_name}")
    return cls()
