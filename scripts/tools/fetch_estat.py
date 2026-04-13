#!/usr/bin/env python3
"""
Fetch statistical data from e-Stat API (政府統計の総合窓口).

Downloads census population data (mesh level) and housing vacancy rates
(municipality level) for use in TLS scoring axes S4 (Future) and S5 (Price).

Datasets fetched:
  1. National Census (国勢調査, code 00200521) — population by 500m mesh
  2. Housing & Land Survey (住宅・土地統計調査, code 00200522) — vacancy rates

Usage:
    export ESTAT_APP_ID="your-app-id"

    uv run scripts/tools/fetch_estat.py                    # fetch all
    uv run scripts/tools/fetch_estat.py --dataset census   # census only
    uv run scripts/tools/fetch_estat.py --dataset housing  # housing only
    uv run scripts/tools/fetch_estat.py --dry-run          # list available tables
    uv run scripts/tools/fetch_estat.py --pref 13          # Tokyo only

Dependencies:
    pip install requests  (or managed via uv / pyproject.toml)

Output:
    data/estat/census_population.csv          (all prefectures)
    data/estat/census_population_13.csv       (--pref 13)
    data/estat/housing_vacancy_municipality.csv

API docs: https://www.e-stat.go.jp/api/api-info/e-stat-manual3-0
"""
from __future__ import annotations

import argparse
import csv
import os
import sys
import time
from pathlib import Path
from typing import Any

try:
    import requests
except ImportError:
    print("ERROR: 'requests' package required. Install: pip install requests", file=sys.stderr)
    sys.exit(1)

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent.parent.parent
OUTPUT_DIR = ROOT / "data" / "estat"

API_BASE = "https://api.e-stat.go.jp/rest/3.0/app/json"

# Government statistics codes
CENSUS_CODE = "00200521"      # 国勢調査
HOUSING_CODE = "00200522"     # 住宅・土地統計調査

# Rate limiting: e-Stat asks for reasonable request intervals
REQUEST_INTERVAL_SEC = 1.0
PAGE_SIZE = 100_000


# ---------------------------------------------------------------------------
# API Client
# ---------------------------------------------------------------------------

class EStatClient:
    """Minimal e-Stat API 3.0 client."""

    def __init__(self, app_id: str) -> None:
        self.app_id = app_id
        self.session = requests.Session()
        self.session.headers.update({"Accept": "application/json"})

    def get_stats_list(
        self,
        stats_code: str,
        search_kind: int = 1,
        limit: int = 100,
    ) -> list[dict[str, Any]]:
        """Search for available statistical tables."""
        params = {
            "appId": self.app_id,
            "statsCode": stats_code,
            "searchKind": str(search_kind),
            "limit": str(limit),
            "lang": "J",
        }
        resp = self._get("getStatsList", params)
        data_list = resp.get("GET_STATS_LIST", {}).get("DATALIST_INF", {})
        tables = data_list.get("TABLE_INF", [])
        if isinstance(tables, dict):
            tables = [tables]
        return tables

    def get_stats_data(
        self,
        stats_data_id: str,
        cd_area: str | None = None,
        limit: int = PAGE_SIZE,
        start_position: int = 1,
    ) -> dict[str, Any]:
        """Fetch statistical data for a given table ID."""
        params: dict[str, str] = {
            "appId": self.app_id,
            "statsDataId": stats_data_id,
            "limit": str(limit),
            "startPosition": str(start_position),
            "lang": "J",
        }
        if cd_area is not None:
            params["cdArea"] = cd_area
        return self._get("getStatsData", params)

    def _get(self, endpoint: str, params: dict[str, str]) -> dict[str, Any]:
        """Execute a GET request with rate limiting."""
        url = f"{API_BASE}/{endpoint}"
        resp = self.session.get(url, params=params, timeout=60)
        resp.raise_for_status()
        data = resp.json()

        # Check for API-level errors
        result = data.get("GET_STATS_LIST", data.get("GET_STATS_DATA", {}))
        result_inf = result.get("RESULT", {})
        status = result_inf.get("STATUS", 0)
        if int(status) != 0:
            error_msg = result_inf.get("ERROR_MSG", "Unknown error")
            print(f"  API error (status {status}): {error_msg}", file=sys.stderr)

        time.sleep(REQUEST_INTERVAL_SEC)
        return data


# ---------------------------------------------------------------------------
# Data fetchers
# ---------------------------------------------------------------------------

def fetch_census_population(
    client: EStatClient,
    pref_code: str | None = None,
    dry_run: bool = False,
) -> int:
    """Fetch census population data (mesh or municipality level).

    Searches for the latest census tables and downloads population data.
    Output:
        data/estat/census_population_{pref_code}.csv  (when pref_code specified)
        data/estat/census_population.csv              (all prefectures)
    """
    print("=== Census Population Data (国勢調査) ===")

    # Step 1: Find available tables
    tables = client.get_stats_list(CENSUS_CODE, search_kind=2, limit=50)
    print(f"  Found {len(tables)} table(s)")

    if dry_run:
        for t in tables[:20]:
            title = t.get("TITLE", {})
            if isinstance(title, dict):
                title_str = title.get("$", str(title))
            else:
                title_str = str(title)
            stat_id = t.get("@id", "?")
            print(f"    [{stat_id}] {title_str[:80]}")
        return 0

    # Step 2: Find the best population table
    # Look for tables containing mesh population data
    target_table_id: str | None = None
    for t in tables:
        title = t.get("TITLE", {})
        title_str = title.get("$", str(title)) if isinstance(title, dict) else str(title)
        stat_id = t.get("@id", "")
        # Prefer tables with population by mesh
        if any(kw in title_str for kw in ["人口", "世帯"]):
            target_table_id = stat_id
            print(f"  Selected table: [{stat_id}] {title_str[:80]}")
            break

    if target_table_id is None:
        print("  WARN: No suitable population table found")
        return 0

    # Step 3: Fetch data with pagination
    filename = f"census_population_{pref_code}.csv" if pref_code else "census_population.csv"
    output_file = OUTPUT_DIR / filename
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    total_rows = 0
    start_pos = 1
    all_rows: list[dict[str, str]] = []

    while True:
        print(f"  Fetching from position {start_pos}...")
        resp = client.get_stats_data(
            target_table_id,
            cd_area=pref_code,
            start_position=start_pos,
        )

        stat_data = resp.get("GET_STATS_DATA", {}).get("STATISTICAL_DATA", {})
        data_inf = stat_data.get("DATA_INF", {})
        values = data_inf.get("VALUE", [])

        if not values:
            break

        if isinstance(values, dict):
            values = [values]

        for v in values:
            row: dict[str, str] = {}
            for key, val in v.items():
                if key.startswith("@"):
                    row[key.lstrip("@")] = str(val)
                elif key == "$":
                    row["value"] = str(val)
            all_rows.append(row)

        total_rows += len(values)

        # Check for next page
        result = resp.get("GET_STATS_DATA", {}).get("RESULT", {})
        next_key = result.get("NEXT_KEY")
        if next_key is None or int(next_key) <= start_pos:
            break
        start_pos = int(next_key)

    # Write CSV
    if all_rows:
        fieldnames = list(all_rows[0].keys())
        with open(output_file, "w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(all_rows)
        print(f"  Saved: {output_file} ({total_rows:,} rows)")
    else:
        print("  No data returned")

    return total_rows


def fetch_housing_vacancy(
    client: EStatClient,
    pref_code: str | None = None,
    dry_run: bool = False,
) -> int:
    """Fetch housing vacancy rate data (municipality level).

    Searches for R5 (2023) housing survey tables.
    Output: data/estat/housing_vacancy_municipality.csv
    """
    print("=== Housing Vacancy Data (住宅・土地統計調査) ===")

    # Step 1: Find available tables
    tables = client.get_stats_list(HOUSING_CODE, search_kind=1, limit=50)
    print(f"  Found {len(tables)} table(s)")

    if dry_run:
        for t in tables[:20]:
            title = t.get("TITLE", {})
            if isinstance(title, dict):
                title_str = title.get("$", str(title))
            else:
                title_str = str(title)
            stat_id = t.get("@id", "?")
            print(f"    [{stat_id}] {title_str[:80]}")
        return 0

    # Step 2: Find vacancy rate table
    target_table_id: str | None = None
    for t in tables:
        title = t.get("TITLE", {})
        title_str = title.get("$", str(title)) if isinstance(title, dict) else str(title)
        stat_id = t.get("@id", "")
        if any(kw in title_str for kw in ["空き家", "住宅数"]):
            target_table_id = stat_id
            print(f"  Selected table: [{stat_id}] {title_str[:80]}")
            break

    if target_table_id is None:
        # Fallback: use first table
        if tables:
            target_table_id = tables[0].get("@id", "")
            title = tables[0].get("TITLE", {})
            title_str = title.get("$", str(title)) if isinstance(title, dict) else str(title)
            print(f"  Fallback table: [{target_table_id}] {title_str[:80]}")
        else:
            print("  WARN: No housing survey tables found")
            return 0

    assert target_table_id is not None  # guaranteed by guard above

    # Step 3: Fetch data
    output_file = OUTPUT_DIR / "housing_vacancy_municipality.csv"
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    total_rows = 0
    start_pos = 1
    all_rows: list[dict[str, str]] = []

    while True:
        print(f"  Fetching from position {start_pos}...")
        resp = client.get_stats_data(
            target_table_id,
            cd_area=pref_code,
            start_position=start_pos,
        )

        stat_data = resp.get("GET_STATS_DATA", {}).get("STATISTICAL_DATA", {})
        data_inf = stat_data.get("DATA_INF", {})
        values = data_inf.get("VALUE", [])

        if not values:
            break

        if isinstance(values, dict):
            values = [values]

        for v in values:
            row: dict[str, str] = {}
            for key, val in v.items():
                if key.startswith("@"):
                    row[key.lstrip("@")] = str(val)
                elif key == "$":
                    row["value"] = str(val)
            all_rows.append(row)

        total_rows += len(values)

        result = resp.get("GET_STATS_DATA", {}).get("RESULT", {})
        next_key = result.get("NEXT_KEY")
        if next_key is None or int(next_key) <= start_pos:
            break
        start_pos = int(next_key)

    if all_rows:
        fieldnames = list(all_rows[0].keys())
        with open(output_file, "w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(all_rows)
        print(f"  Saved: {output_file} ({total_rows:,} rows)")
    else:
        print("  No data returned")

    return total_rows


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Fetch statistical data from e-Stat API.",
    )
    parser.add_argument(
        "--dataset",
        choices=["census", "housing", "all"],
        default="all",
        help="Which dataset to fetch (default: all)",
    )
    parser.add_argument(
        "--pref",
        type=str,
        default=None,
        metavar="CODE",
        help="Prefecture code filter (e.g., 13 for Tokyo)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="List available tables without downloading data",
    )
    args = parser.parse_args()

    # Resolve API key
    app_id = os.environ.get("ESTAT_APP_ID", "")
    if not app_id:
        # Try loading from backend .env
        env_file = ROOT / "services" / "backend" / ".env"
        if env_file.exists():
            for line in env_file.read_text().splitlines():
                if line.startswith("ESTAT_APP_ID="):
                    app_id = line.split("=", 1)[1].strip()
                    break

    if not app_id:
        print(
            "ERROR: ESTAT_APP_ID not set.\n"
            "  Set via: export ESTAT_APP_ID=your-app-id\n"
            "  Or add to: services/backend/.env",
            file=sys.stderr,
        )
        sys.exit(1)

    client = EStatClient(app_id)
    total = 0

    if args.dataset in ("census", "all"):
        total += fetch_census_population(client, pref_code=args.pref, dry_run=args.dry_run)

    if args.dataset in ("housing", "all"):
        total += fetch_housing_vacancy(client, pref_code=args.pref, dry_run=args.dry_run)

    print(f"\nTotal rows fetched: {total:,}")


if __name__ == "__main__":
    main()
