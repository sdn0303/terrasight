# Pipeline v2: Data Compatibility Fix — KSJ Format Evolution + ZIP Structure Handling

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the pipeline to handle all KSJ data format variations (v3.0→v3.1 field changes, nested ZIPs, GeoJSON-first reading, encoding issues) so `./scripts/commands/pipeline.sh 13 P0` runs end-to-end without errors.

**Architecture:** Replace fiona Shapefile reading with GeoJSON-first strategy (newer KSJ ZIPs include `.geojson` files). Add version-aware field mapping for L01. Fix N03 GeoJSON parsing (uses different structure). Handle S12/N02 dual-encoding (Shift-JIS/UTF-8) directories.

**Tech Stack:** Python 3.12 · fiona · shapely · psycopg2 · GeoJSON (stdlib json)

---

## Data Inventory & Issues Found

### Raw Data on Disk (`data/raw/`)

| Dataset | Code | Files | Format | Structure | Pref Filter | Status |
|---------|------|-------|--------|-----------|-------------|--------|
| Admin boundaries | N03 | 3 (2023-2025) | Shapefile+GeoJSON | Flat | N03_001 (prefName) | **OK** — working |
| Zoning | A29 | 94 (2011+2019, 47 prefs) | Shapefile | Nested dir | Per-pref ZIP | **OK** — working |
| Land prices | L01 | 20 (2007-2026) | Shapefile+GeoJSON | Flat→Nested (v3.1) | L01_017(v3.0) / L01_001(v3.1) | **BROKEN** — field mapping changed |
| Schools | P29 | 50 (13+21, 47 prefs) | Shapefile+GeoJSON | Flat / Nested | Per-pref ZIP | **OK** — working |
| Medical | P04 | 49 (14+20, 47 prefs) | Shapefile+GeoJSON | Nested + backslash | Per-pref ZIP | **OK** — fixed (backslash normalize) |
| Stations | S12 | 13 (2012-2024) | Shapefile+GeoJSON | Shift-JIS/UTF-8 dirs | S12_001c (6-digit) | **BROKEN** — no features after filter |
| Railway | N02 | 18 (2005-2024) | Shapefile+GeoJSON | Shift-JIS/UTF-8 dirs + Station/Section split | No pref field | **BROKEN** — includes all prefs |
| Flood risk | A31b | 104 (mesh-based) | GeoJSON | Nested by scenario | Mesh→pref mapping needed | **NOT IMPL** — adapter needed |
| Steep slope | A47 | 45 (per-pref) | Shapefile+GeoJSON | Nested dir | Per-pref ZIP | **UNTESTED** |

### Key Findings

#### Finding 1: L01 Field Mapping v3.0 → v3.1 (CRITICAL)

L01-18 (2018) and later switched from v3.0 to v3.1 product specification. **ALL field numbers changed:**

| Field | v3.0 (L01-17以前) | v3.1 (L01-18以降) |
|-------|-------------------|-------------------|
| 市区町村コード | `L01_017` | `L01_001` |
| 年度 | `L01_005` | `L01_007` |
| 価格(円/m²) | `L01_006` | `L01_008` |
| 住所 | `L01_019` | `L01_025` |
| 土地利用 | `L01_021` | `L01_028` |
| 用途地域 | `L01_029` | `L01_051` |
| 建蔽率 | `L01_030` | `L01_057` |
| 容積率 | `L01_031` | `L01_058` |
| 最寄駅名 | `L01_027` | `L01_048` |
| 駅距離(m) | `L01_028` | `L01_050` |

Additionally, v3.1 Shapefiles have **broken attributes** when read by fiona (fields return `false` or `0`), because the 144+ columns exceed Shapefile DBF limits. **GeoJSON inside the ZIP works correctly.**

#### Finding 2: ZIP Internal Structure Variations

| Pattern | Examples | Directory | Backslash | GeoJSON included |
|---------|----------|-----------|-----------|------------------|
| Flat | L01-17, P29-21_13, N03 | No | No | No (v3.0), Yes (v3.1+) |
| Nested subdir | L01-24, A29-19, A47-21 | Yes | No | Yes |
| Backslash paths | P04-20 | Yes | **Yes** | Yes |
| Dual encoding dirs | S12-24, N02-24 | Shift-JIS/ + UTF-8/ | No | Yes |
| Multiple shapefiles | N02 (Station + Section) | Yes | No | Yes (separate GeoJSON per type) |
| Garbled Japanese dirs | A29-19 (CP932 encoded) | Yes | No | Yes |

#### Finding 3: S12/N02 Reading Strategy

- **S12 (Stations)**: ZIP contains `UTF-8/S12-24_NumberOfPassengers.{shp,geojson}`. GeoJSON has all fields including `S12_001c` (station code with pref prefix). Shapefile works via zip_utils but encoding issues possible.
- **N02 (Railway)**: ZIP contains `UTF-8/N02-24_RailroadSection.geojson` + `UTF-8/N02-24_Station.geojson`. Two separate feature types. Railway sections have NO pref_code field — geometry-only filtering needed.

#### Finding 4: A31b (Flood) is Mesh-Based GeoJSON

A31b files are named `A31b-24_10_{mesh}_GEOJSON.zip` — mesh-code based, not per-pref. Each contains scenario subdirectories (`10_計画規模/`, `20_想定最大規模/`). Need mesh-to-pref mapping or spatial filter.

---

## Decision Log

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | GeoJSON-first reading: prefer `.geojson` over `.shp` in ZIP | v3.1 Shapefiles have broken attributes; GeoJSON is always correct |
| D2 | Version-aware L01 field mapping | v3.0 and v3.1 have completely different field numbers |
| D3 | Read GeoJSON directly via `json.load` instead of fiona for nested GeoJSON | fiona ZIP support is fragile; stdlib json + shapely is more reliable |
| D4 | For N02 (railway), skip pref filtering entirely | Railways cross prefectures; PostGIS spatial queries handle per-pref at query time |
| D5 | Defer A31b flood adapter to separate task | Mesh-based structure needs dedicated adapter, not a quick fix |
| D6 | Use L01-17 as latest reliable v3.0 archive; use L01-24+ GeoJSON for v3.1 | L01-18→L01-26 Shapefiles are broken but GeoJSON inside same ZIP works |

---

## Phase 1: Core Reading Infrastructure

### Task 1.1: GeoJSON-First ZIP Reader

**Files:**
- Modify: `scripts/tools/pipeline/adapters/zip_utils.py`

- [ ] **Step 1:** Rewrite `open_zip_shapefile` to `read_features_from_zip`:

```python
def read_features_from_zip(raw_path: Path, prefer_utf8: bool = True) -> list[dict] | None:
    """Read GeoJSON features from a ZIP archive.
    
    Strategy (in order):
    1. Find .geojson file inside ZIP → json.load → return features
    2. Fall back to .shp via fiona if no GeoJSON found
    
    For ZIPs with Shift-JIS/UTF-8 dirs, prefer UTF-8.
    Normalizes backslash paths in ZIP entries.
    """
```

Key logic:
- Open ZIP, list all files, normalize `\\` → `/`
- Filter `.geojson` files, prefer `UTF-8/` dir
- If GeoJSON found: `json.load` → return `features` list
- If no GeoJSON: fall back to `fiona.open("zip://...!path.shp")`
- Return list of raw GeoJSON feature dicts

- [ ] **Step 2:** Test with each problem archive:
```bash
python3 -c "
from scripts.tools.pipeline.adapters.zip_utils import read_features_from_zip
from pathlib import Path
for f in ['data/raw/L01-24_GML.zip', 'data/raw/P04-20_13_GML.zip', 'data/raw/S12-24_GML.zip', 'data/raw/N02-24_GML.zip']:
    feats = read_features_from_zip(Path(f))
    print(f'{f}: {len(feats) if feats else 0} features')
"
```

- [ ] **Step 3:** Commit

---

### Task 1.2: Update Adapters to Use New Reader

**Files:**
- Modify: `scripts/tools/pipeline/adapters/per_pref.py`
- Modify: `scripts/tools/pipeline/adapters/national.py`

- [ ] **Step 1:** Replace `_read_features` in per_pref.py to use `read_features_from_zip` instead of `fiona.open`

- [ ] **Step 2:** Replace `_read_and_filter` in national.py to use `read_features_from_zip`

- [ ] **Step 3:** Both adapters should work with raw feature dicts (geometry as dict, not shapely) — apply `shape()` only for validation/bbox

- [ ] **Step 4:** Test: `uv run scripts/tools/pipeline/convert.py --pref 13 --dataset admin-boundary`

- [ ] **Step 5:** Commit

---

## Phase 2: L01 Version-Aware Field Mapping

### Task 2.1: L01 Field Version Detection + Mapping

**Files:**
- Modify: `scripts/tools/pipeline/adapters/national.py`
- Modify: `scripts/tools/pipeline/import_db.py`

- [ ] **Step 1:** Add L01 version detection in NationalArchiveAdapter:

```python
def _detect_l01_version(self, props: dict) -> str:
    """Detect L01 field version from property keys.
    
    v3.0 (≤2017): L01_017 = city code (5-digit int), L01_006 = price (int)
    v3.1 (≥2018): L01_001 = city code (5-digit str), L01_008 = price (int)
    """
    # v3.1: L01_001 is city code (5 digits), L01_008 exists and is large int
    val_001 = props.get('L01_001')
    if val_001 and str(val_001).isdigit() and len(str(val_001)) == 5:
        return 'v3.1'
    # v3.0: L01_017 is city code
    val_017 = props.get('L01_017')
    if val_017 and str(val_017).isdigit() and int(str(val_017)) > 0:
        return 'v3.0'
    return 'unknown'
```

- [ ] **Step 2:** Add field normalization that maps v3.0/v3.1 fields to canonical names:

```python
L01_FIELD_MAP = {
    'v3.0': {
        'city_code': 'L01_017', 'survey_year': 'L01_005', 'price_per_sqm': 'L01_006',
        'address': 'L01_019', 'land_use': 'L01_021', 'zone_type': 'L01_029',
    },
    'v3.1': {
        'city_code': 'L01_001', 'survey_year': 'L01_007', 'price_per_sqm': 'L01_008',
        'address': 'L01_025', 'land_use': 'L01_028', 'zone_type': 'L01_051',
    },
}

def _normalize_l01_props(self, props: dict, version: str) -> dict:
    """Map version-specific L01 fields to canonical names."""
    mapping = L01_FIELD_MAP.get(version, {})
    result = {}
    for canonical, ksj_key in mapping.items():
        result[canonical] = props.get(ksj_key)
    return result
```

- [ ] **Step 3:** Update `_extract_pref_code` to use `city_code` from normalized props (first 2 digits)

- [ ] **Step 4:** Update `_prop()` in import_db.py to handle normalized canonical names from L01

- [ ] **Step 5:** Test: `uv run scripts/tools/pipeline/convert.py --pref 13 --dataset land-price`

Expected: 東京のみの features (25,000前後ではなく, 都道府県フィルタ後の ~2,000-3,000)

- [ ] **Step 6:** Commit

---

## Phase 3: S12/N02 GeoJSON Reading

### Task 3.1: S12 Stations — GeoJSON Direct Read

**Files:**
- Modify: `scripts/tools/pipeline/adapters/national.py`

- [ ] **Step 1:** S12 ZIP has `UTF-8/S12-24_NumberOfPassengers.geojson`. The GeoJSON has `S12_001c` field (6-digit station code, first 2 = pref code). `read_features_from_zip` should prefer this GeoJSON.

- [ ] **Step 2:** `_extract_pref_code` already handles `S12_001c`. Verify it works with GeoJSON reader.

- [ ] **Step 3:** Test: `uv run scripts/tools/pipeline/convert.py --pref 13 --dataset stations`
Expected: ~2,000-3,000 Tokyo stations (not 10,343 = all prefs)

- [ ] **Step 4:** Commit

### Task 3.2: N02 Railway — Multi-GeoJSON Handling

**Files:**
- Modify: `scripts/tools/pipeline/adapters/zip_utils.py`

- [ ] **Step 1:** N02 ZIP has TWO GeoJSON files: `N02-24_RailroadSection.geojson` (lines) + `N02-24_Station.geojson` (points). For railway dataset, we want `RailroadSection`.

`read_features_from_zip` should accept optional `filename_hint` parameter to prefer matching GeoJSON:

```python
def read_features_from_zip(raw_path, prefer_utf8=True, filename_hint=None):
    # If hint provided, prefer GeoJSON matching hint
    if filename_hint:
        matching = [g for g in geojson_files if filename_hint in g]
        if matching:
            geojson_files = matching
```

- [ ] **Step 2:** Catalog entry for railway should specify `"geojson_hint": "RailroadSection"` (add to DatasetEntry).

- [ ] **Step 3:** N02 has NO pref_code field. Current behavior (include all when pref=None) is intentional — railway data is filtered spatially by PostGIS.

- [ ] **Step 4:** Test: `uv run scripts/tools/pipeline/convert.py --pref 13 --dataset railway`

- [ ] **Step 5:** Commit

---

## Phase 4: Import Field Mapping Alignment

### Task 4.1: Update import_db.py for Canonical Field Names

**Files:**
- Modify: `scripts/tools/pipeline/import_db.py`

- [ ] **Step 1:** The convert step should output GeoJSON with **canonical field names** (not KSJ codes). Update adapters to normalize fields during convert, not during import.

This means `_prop()` fallbacks in import_db.py become simpler — the canonical names should be present in the converted GeoJSON.

- [ ] **Step 2:** Update `_build_row` for land_prices to use canonical names directly:
```python
if table_name == "land_prices":
    return (
        pref_code,
        _prop(props, "address", default=""),
        _prop(props, "price_per_sqm", default=0),
        _prop(props, "land_use"),
        _prop(props, "zone_type"),
        _prop(props, "survey_year", default=2024),
        geom_wkt,
    )
```

- [ ] **Step 3:** Full pipeline test: `./scripts/commands/pipeline.sh 13 P0`

- [ ] **Step 4:** Commit

---

## Phase 5: Validation & Edge Cases

### Task 5.1: End-to-End Pipeline Validation

- [ ] **Step 1:** Run `./scripts/commands/pipeline.sh 13 P0`
- [ ] **Step 2:** Verify row counts match expectations:

| Table | Expected Rows (approx) |
|-------|----------------------|
| admin_boundaries | ~62 (Tokyo municipalities) → 6,904 (multi-polygon) |
| zoning | ~11,000 |
| land_prices | ~2,000-3,000 (Tokyo only, latest year) |
| schools | ~4,400 |
| medical_facilities | ~25,000 |
| stations | ~2,000-3,000 (Tokyo) |
| railways | ~800-1,500 (Tokyo area sections) |

- [ ] **Step 3:** Verify `docker compose up -d --build` starts without "relation does not exist" errors
- [ ] **Step 4:** Verify API returns data: `curl localhost:3000/api/health`
- [ ] **Step 5:** Commit final state

---

## Subagent Delegation Guide

| Task | Agent | Model | Max Files |
|------|-------|-------|-----------|
| 1.1 ZIP reader rewrite | general-purpose | sonnet | 1 file |
| 1.2 Adapter updates | general-purpose | sonnet | 2 files |
| 2.1 L01 field mapping | general-purpose | sonnet | 2 files |
| 3.1-3.2 S12/N02 | general-purpose | sonnet | 2 files |
| 4.1 Import alignment | general-purpose | sonnet | 1 file |
| 5.1 Validation | manual | - | - |

---

## Risk Register

| Risk | Mitigation |
|------|-----------|
| L01 v3.1 GeoJSON field order may vary across years | Detect version per-archive, not globally |
| N02 "all features" for railways makes GeoJSON huge | Accept for now; PostGIS filters spatially at query time |
| A29 garbled Japanese dir names in ZIP | `read_features_from_zip` handles by trying all `.geojson`/`.shp` paths |
| `price_per_sqm=0` for some L01 records | Already relaxed CHECK to `>= 0` |
| S12 station count for Tokyo might be high (commuter stations from neighboring prefs) | Accept — spatial query precision can be improved later |
