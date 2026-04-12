"""Base adapter interface for data pipeline v2."""
from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class DatasetEntry:
    """One entry from dataset_catalog.json."""
    id: str
    name: str
    scope: str  # "per_prefecture" | "national"
    source_type: str
    adapter: str
    raw_pattern: str | None = None
    output_geojson: str | None = None
    output_fgb: str | None = None
    db_table: str | None = None
    static_layer: bool = False
    priority: str = "P0"
    column_renames: dict[str, str] = field(default_factory=dict)
    ksj_code: str | None = None
    update_frequency: str | None = None
    geojson_hint: str | None = None


@dataclass
class ConvertResult:
    """Result of a conversion run."""
    dataset_id: str
    pref_code: str
    output_path: Path
    feature_count: int
    bbox: tuple[float, float, float, float] | None = None


class BaseAdapter(ABC):
    """Base class for all data pipeline adapters."""

    @abstractmethod
    def convert(
        self,
        entry: DatasetEntry,
        pref_code: str,
        raw_dir: Path,
        output_dir: Path,
    ) -> ConvertResult | None:
        """Convert raw data to canonical GeoJSON for one prefecture.

        Returns None if no matching raw data found.
        """
        ...
