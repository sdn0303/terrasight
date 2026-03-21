#!/usr/bin/env python3
"""
Pre-process MLIT geodata for the frontend map.

Based on: https://github.com/sdn0303/shape2geojson
Uses pyshp for Shapefile → GeoJSON conversion with cp932 encoding.

Filters nationwide datasets to the Tokyo/Kanto area and outputs
optimized GeoJSON files to services/frontend/public/geojson/.

Usage:
    python3 scripts/process-geodata.py

Dependencies:
    pip3 install pyshp
"""

import json
import os
from pathlib import Path

import shapefile

ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT / "data"
OUT_DIR = ROOT / "services" / "frontend" / "public" / "geojson"

# Tokyo prefecture code & name
TOKYO_ADMIN_PREFIX = "13"
TOKYO_PREF_NAME = "東京都"

# Bounding box for Tokyo metro area [minLng, minLat, maxLng, maxLat]
TOKYO_BBOX = [138.9, 35.4, 140.0, 35.95]
# Wider Kanto bbox for context layers (fault lines, flood)
KANTO_BBOX = [138.5, 34.8, 140.9, 37.0]


# ---------------------------------------------------------------------------
# Core helpers (adapted from sdn0303/shape2geojson)
# ---------------------------------------------------------------------------


def read_shape(file_path: str, encoding: str = "cp932") -> list[dict]:
    """Read a Shapefile and return a list of GeoJSON Feature dicts.

    Follows the same pattern as shape2geojson.read_shape():
    - shapefile.Reader with cp932 encoding for Japanese MLIT data
    - __geo_interface__ for direct GeoJSON geometry conversion
    """
    reader = shapefile.Reader(file_path, encoding=encoding)
    fields = reader.fields[1:]
    field_names = [field[0] for field in fields]

    buffer: list[dict] = []
    for sr in reader.shapeRecords():
        atr = dict(zip(field_names, sr.record))
        geom = sr.shape.__geo_interface__
        buffer.append({"type": "Feature", "geometry": geom, "properties": atr})

    return buffer


def read_shape_safe(file_path: str) -> list[dict]:
    """Read Shapefile with encoding fallback: cp932 → utf-8 → latin-1."""
    for enc in ["cp932", "utf-8", "latin-1"]:
        try:
            return read_shape(file_path, encoding=enc)
        except UnicodeDecodeError:
            continue
    print(f"  WARNING: All encodings failed for {file_path}")
    return []


def write_geojson(out_path: Path, features: list[dict]) -> None:
    """Write a GeoJSON FeatureCollection with compact formatting."""
    collection = {"type": "FeatureCollection", "features": features}
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(collection, f, ensure_ascii=False, separators=(",", ":"))

    size_kb = out_path.stat().st_size / 1024
    print(f"  → {out_path.name}: {len(features)} features, {size_kb:.0f} KB")


# ---------------------------------------------------------------------------
# Geometry utilities
# ---------------------------------------------------------------------------


def round_coords(coords: list, precision: int = 5) -> list:
    """Recursively round coordinate arrays to reduce file size.

    precision=5 ≈ 1.1m — sufficient for city-scale visualization.
    precision=4 ≈ 11m  — for national overview maps.
    """
    if isinstance(coords[0], (int, float)):
        return [round(c, precision) for c in coords]
    return [round_coords(c, precision) for c in coords]


def coords_intersect_bbox(coords: list, bbox: list[float]) -> bool:
    """Check if any coordinate falls within [minLng, minLat, maxLng, maxLat]."""
    min_lng, min_lat, max_lng, max_lat = bbox

    def check(c: list) -> bool:
        if isinstance(c[0], (int, float)):
            return min_lng <= c[0] <= max_lng and min_lat <= c[1] <= max_lat
        return any(check(sub) for sub in c)

    return check(coords)


def filter_by_bbox(features: list[dict], bbox: list[float]) -> list[dict]:
    """Filter features whose geometry intersects the bounding box."""
    result = []
    for feat in features:
        geom = feat.get("geometry")
        if geom and geom.get("coordinates"):
            if coords_intersect_bbox(geom["coordinates"], bbox):
                result.append(feat)
    return result


def strip_z_coords(coords: list) -> list:
    """Remove Z values from PolygonZ/PolylineZ coordinates (keep only [lng, lat])."""
    if isinstance(coords[0], (int, float)):
        return coords[:2]  # Keep only lng, lat
    return [strip_z_coords(c) for c in coords]


# ---------------------------------------------------------------------------
# Dataset processors
# ---------------------------------------------------------------------------


def process_did_tokyo() -> None:
    """Process A16 人口集中地区 (DID) — filter to Tokyo."""
    src = DATA_DIR / "A16-15_GML" / "A16-15_00_DID.geojson"
    if not src.exists():
        print(f"  SKIP: {src} not found")
        return

    print(f"  Loading {src.name} ...")
    with open(src, encoding="utf-8") as f:
        data = json.load(f)

    tokyo_features: list[dict] = []
    for feat in data["features"]:
        code = str(feat["properties"].get("行政コード", ""))
        if not code.startswith(TOKYO_ADMIN_PREFIX):
            continue

        feat["geometry"]["coordinates"] = round_coords(
            feat["geometry"]["coordinates"]
        )
        props = feat["properties"]
        feat["properties"] = {
            "id": props.get("DIDid"),
            "adminCode": str(props.get("行政コード", "")),
            "cityName": props.get("市町村名称", ""),
            "didCode": props.get("DID符号"),
            "population": props.get("人口"),
            "areaSqKm": props.get("面積"),
            "prevPopulation": props.get("前回人口"),
            "prevAreaSqKm": props.get("前回面積"),
            "popRatio": props.get("人口割合"),
            "areaRatio": props.get("面積割合"),
            "surveyYear": props.get("調査年度"),
        }
        tokyo_features.append(feat)

    write_geojson(OUT_DIR / "did-tokyo.geojson", tokyo_features)


def process_did_national() -> None:
    """Process A16 DID — national dataset with minimal properties."""
    src = DATA_DIR / "A16-15_GML" / "A16-15_00_DID.geojson"
    if not src.exists():
        print(f"  SKIP: {src} not found")
        return

    print(f"  Loading {src.name} (national) ...")
    with open(src, encoding="utf-8") as f:
        data = json.load(f)

    features: list[dict] = []
    for feat in data["features"]:
        feat["geometry"]["coordinates"] = round_coords(
            feat["geometry"]["coordinates"], precision=4
        )
        props = feat["properties"]
        feat["properties"] = {
            "id": props.get("DIDid"),
            "adminCode": str(props.get("行政コード", "")),
            "cityName": props.get("市町村名称", ""),
            "population": props.get("人口"),
            "areaSqKm": props.get("面積"),
        }
        features.append(feat)

    write_geojson(OUT_DIR / "did-national.geojson", features)


def process_landform_tokyo() -> None:
    """Process landform 地形分類 Shapefile — filter to Tokyo area."""
    shp = DATA_DIR / "landform" / "landform"
    if not shp.with_suffix(".shp").exists():
        print(f"  SKIP: {shp}.shp not found")
        return

    print(f"  Reading landform.shp (cp932) ...")
    all_features = read_shape(str(shp))
    print(f"  Total: {len(all_features)} features")

    tokyo_features: list[dict] = []
    for feat in all_features:
        pref = feat["properties"].get("PREF", "")
        if pref == TOKYO_PREF_NAME:
            keep = True
        elif feat["geometry"] and feat["geometry"].get("coordinates"):
            keep = coords_intersect_bbox(
                feat["geometry"]["coordinates"], TOKYO_BBOX
            )
        else:
            keep = False

        if keep:
            feat["geometry"]["coordinates"] = round_coords(
                feat["geometry"]["coordinates"]
            )
            props = feat["properties"]
            feat["properties"] = {
                "id": props.get("ID"),
                "prefCode": props.get("NO", ""),
                "prefName": props.get("PREF", ""),
                "landformType": props.get("地形区分名", ""),
                "landformCategory": props.get("地形大区分", ""),
            }
            tokyo_features.append(feat)

    print(f"  Filtered: {len(tokyo_features)} features")
    write_geojson(OUT_DIR / "landform-tokyo.geojson", tokyo_features)


def process_geology_tokyo() -> None:
    """Process geology 表層地質 Shapefile — filter to Tokyo area."""
    shp = DATA_DIR / "geology" / "geology"
    if not shp.with_suffix(".shp").exists():
        print(f"  SKIP: {shp}.shp not found")
        return

    print(f"  Reading geology.shp (cp932) ...")
    all_features = read_shape(str(shp))
    print(f"  Total: {len(all_features)} features")

    tokyo_features: list[dict] = []
    for feat in all_features:
        pref = feat["properties"].get("PREF", "")
        if pref == TOKYO_PREF_NAME:
            keep = True
        elif feat["geometry"] and feat["geometry"].get("coordinates"):
            keep = coords_intersect_bbox(
                feat["geometry"]["coordinates"], TOKYO_BBOX
            )
        else:
            keep = False

        if keep:
            feat["geometry"]["coordinates"] = round_coords(
                feat["geometry"]["coordinates"]
            )
            props = feat["properties"]
            feat["properties"] = {
                "id": props.get("ID"),
                "prefCode": props.get("NO", ""),
                "prefName": props.get("PREF", ""),
                "geologySymbol": props.get("地質記号", ""),
                "geologyType": props.get("地質区分名", ""),
                "geologyCategory2": props.get("地質区分２", ""),
                "geologyCategory3": props.get("地質区分３", ""),
                "geologyAge1": props.get("地質時代１", ""),
                "geologyAge2": props.get("地質時代２", ""),
                "geologyAge3": props.get("地質時代３", ""),
            }
            tokyo_features.append(feat)

    print(f"  Filtered: {len(tokyo_features)} features")
    write_geojson(OUT_DIR / "geology-tokyo.geojson", tokyo_features)


def process_n03_tokyo() -> None:
    """Process N03 行政区域 (市町村境界) — filter to Tokyo."""
    src = DATA_DIR / "N03-20240101_GML" / "N03-20240101.geojson"
    if not src.exists():
        print(f"  SKIP: {src} not found")
        return

    print(f"  Loading {src.name} (475MB, may take a moment) ...")
    with open(src, encoding="utf-8") as f:
        data = json.load(f)

    print(f"  Total: {len(data['features'])} features")

    tokyo_features: list[dict] = []
    for feat in data["features"]:
        props = feat["properties"]
        pref = props.get("N03_001", "")
        if pref != TOKYO_PREF_NAME:
            continue

        # Round coordinates for smaller output
        if feat["geometry"] and feat["geometry"].get("coordinates"):
            feat["geometry"]["coordinates"] = round_coords(
                feat["geometry"]["coordinates"]
            )

        # Map to English keys
        feat["properties"] = {
            "prefName": props.get("N03_001", ""),
            "subPrefName": props.get("N03_002", ""),
            "countyName": props.get("N03_003", ""),
            "cityName": props.get("N03_004", ""),
            "wardName": props.get("N03_005", ""),
            "adminCode": props.get("N03_007", ""),
        }
        tokyo_features.append(feat)

    print(f"  Filtered: {len(tokyo_features)} features")
    write_geojson(OUT_DIR / "admin-boundary-tokyo.geojson", tokyo_features)


def process_fault_tokyo() -> None:
    """Process fault 断層線 Shapefile — filter to Kanto area."""
    shp = DATA_DIR / "others" / "fault"
    if not shp.with_suffix(".shp").exists():
        print(f"  SKIP: {shp}.shp not found")
        return

    print(f"  Reading fault.shp (cp932) ...")
    all_features = read_shape(str(shp))
    print(f"  Total: {len(all_features)} features")

    # Fault lines use wider Kanto bbox for seismic context
    kanto_features: list[dict] = []
    for feat in all_features:
        geom = feat.get("geometry")
        if not geom or not geom.get("coordinates"):
            continue
        if coords_intersect_bbox(geom["coordinates"], KANTO_BBOX):
            feat["geometry"]["coordinates"] = round_coords(
                feat["geometry"]["coordinates"]
            )
            props = feat["properties"]
            feat["properties"] = {
                "id": props.get("ID"),
                "prefCode": props.get("NO", ""),
                "prefName": props.get("PREF", ""),
                "faultType1": props.get("断層区分１", ""),
                "faultType2": props.get("断層区分２", ""),
            }
            kanto_features.append(feat)

    print(f"  Filtered: {len(kanto_features)} features (Kanto)")
    write_geojson(OUT_DIR / "fault-kanto.geojson", kanto_features)


def process_flood_history_tokyo() -> None:
    """Process 水害GIS (1896-2019) — merge all Tokyo-area flood events.

    Aggregates 60 shapefiles into a single GeoJSON with flood event metadata.
    Each feature includes the disaster name and date for temporal analysis.
    """
    flood_dir = DATA_DIR / "1896_2019_sinsui_add24"
    if not flood_dir.exists():
        print(f"  SKIP: {flood_dir} not found")
        return

    shp_files = sorted(
        [f.stem for f in flood_dir.glob("*.shp")]
    )
    print(f"  Found {len(shp_files)} flood shapefiles (1896-2019)")

    tokyo_features: list[dict] = []
    skipped = 0

    for shp_name in shp_files:
        path = str(flood_dir / shp_name)
        features = read_shape_safe(path)
        if not features:
            skipped += 1
            continue

        for feat in features:
            geom = feat.get("geometry")
            if not geom or not geom.get("coordinates"):
                continue

            # Strip Z coordinates if PolygonZ
            coords = strip_z_coords(geom["coordinates"])

            if not coords_intersect_bbox(coords, TOKYO_BBOX):
                continue

            feat["geometry"]["coordinates"] = round_coords(coords)
            # Normalize geometry type (PolygonZ → Polygon)
            geom_type = geom.get("type", "")
            if geom_type.endswith("Z"):
                feat["geometry"]["type"] = geom_type[:-1]

            props = feat["properties"]
            date_str = str(props.get("date", ""))
            feat["properties"] = {
                "id": props.get("id"),
                "code": props.get("code", ""),
                "typeName": props.get("name", ""),
                "date": date_str,
                "year": int(date_str[:4]) if len(date_str) >= 4 else None,
                "disasterName": props.get("disastName", ""),
                "source": props.get("source", ""),
            }
            tokyo_features.append(feat)

    if skipped:
        print(f"  Skipped {skipped} files (encoding issues)")
    print(f"  Tokyo flood features: {len(tokyo_features)}")
    write_geojson(OUT_DIR / "flood-history-tokyo.geojson", tokyo_features)


def process_soil_tokyo() -> None:
    """Process soil 土壌図 Shapefile — filter to Tokyo area."""
    shp = DATA_DIR / "soil" / "soil"
    if not shp.with_suffix(".shp").exists():
        print(f"  SKIP: {shp}.shp not found")
        return

    print(f"  Reading soil.shp (cp932) ...")
    all_features = read_shape(str(shp))
    print(f"  Total: {len(all_features)} features")

    tokyo_features: list[dict] = []
    for feat in all_features:
        pref = feat["properties"].get("PREF", "")
        if pref == TOKYO_PREF_NAME:
            keep = True
        elif feat["geometry"] and feat["geometry"].get("coordinates"):
            keep = coords_intersect_bbox(
                feat["geometry"]["coordinates"], TOKYO_BBOX
            )
        else:
            keep = False

        if keep:
            feat["geometry"]["coordinates"] = round_coords(
                feat["geometry"]["coordinates"]
            )
            props = feat["properties"]
            feat["properties"] = {
                "id": props.get("ID"),
                "prefCode": props.get("NO", ""),
                "prefName": props.get("PREF", ""),
                "soilType": props.get("土壌区分名", ""),
                "soilTypeEn": props.get("SOILNAME11", ""),
                "soilCategory": props.get("土壌大区分", ""),
                "soilCategoryEn": props.get("SOILNAME21", ""),
            }
            tokyo_features.append(feat)

    print(f"  Filtered: {len(tokyo_features)} features")
    write_geojson(OUT_DIR / "soil-tokyo.geojson", tokyo_features)


def process_volcano_kanto() -> None:
    """Process volcano 火山 Shapefile — filter to Kanto area."""
    shp = DATA_DIR / "others" / "volcano"
    if not shp.with_suffix(".shp").exists():
        print(f"  SKIP: {shp}.shp not found")
        return

    print(f"  Reading volcano.shp (cp932) ...")
    all_features = read_shape(str(shp))
    print(f"  Total: {len(all_features)} features")

    kanto_features: list[dict] = []
    for feat in all_features:
        geom = feat.get("geometry")
        if not geom or not geom.get("coordinates"):
            continue
        if coords_intersect_bbox(geom["coordinates"], KANTO_BBOX):
            feat["geometry"]["coordinates"] = round_coords(
                feat["geometry"]["coordinates"]
            )
            props = feat["properties"]
            feat["properties"] = {
                "id": props.get("ID"),
                "prefCode": props.get("NO", ""),
                "prefName": props.get("PREF", ""),
                "volcanoType": props.get("火山区分", ""),
            }
            kanto_features.append(feat)

    print(f"  Filtered: {len(kanto_features)} features (Kanto)")
    write_geojson(OUT_DIR / "volcano-kanto.geojson", kanto_features)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("Processing MLIT geodata for frontend map")
    print("=" * 60)

    print("\n[1/9] DID 人口集中地区 (Tokyo)")
    process_did_tokyo()

    print("\n[2/9] DID 人口集中地区 (National)")
    process_did_national()

    print("\n[3/9] Landform 地形分類 (Tokyo area)")
    process_landform_tokyo()

    print("\n[4/9] Geology 表層地質 (Tokyo area)")
    process_geology_tokyo()

    print("\n[5/9] Soil 土壌図 (Tokyo area)")
    process_soil_tokyo()

    print("\n[6/9] N03 行政区域 市町村境界 (Tokyo)")
    process_n03_tokyo()

    print("\n[7/9] Fault 断層線 (Kanto)")
    process_fault_tokyo()

    print("\n[8/9] Flood History 水害GIS (Tokyo, 1896-2019)")
    process_flood_history_tokyo()

    print("\n[9/9] Volcano 火山 (Kanto)")
    process_volcano_kanto()

    print("\n" + "=" * 60)
    print(f"Done! Output: {OUT_DIR}")
    for f in sorted(OUT_DIR.glob("*.geojson")):
        size_mb = f.stat().st_size / (1024 * 1024)
        print(f"  {f.name}: {size_mb:.1f} MB")
    print("=" * 60)


if __name__ == "__main__":
    main()
