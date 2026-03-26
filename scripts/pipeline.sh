#!/usr/bin/env bash
# scripts/pipeline.sh
# Orchestrates the full government data pipeline: download -> convert -> import
# Usage: ./pipeline.sh [--skip-download]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKIP_DOWNLOAD="${1:-}"

echo "=== Government Data Pipeline ==="
echo "Started: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# ─── Step 1: Download raw data ───────────────────────────
if [ "$SKIP_DOWNLOAD" != "--skip-download" ]; then
  echo "─── Step 1: Download Raw Data ───"
  echo ""

  echo "--- N03: Administrative Boundaries ---"
  bash "$SCRIPT_DIR/download-n03-boundaries.sh"
  echo ""

  echo "--- L01: Land Prices ---"
  bash "$SCRIPT_DIR/download-l01-landprice.sh"
  echo ""

  # Future scripts (uncomment as they are created):
  # echo "--- J-SHIS: Seismic Hazard ---"
  # bash "$SCRIPT_DIR/download-jshis-surface.sh"
  #
  # echo "--- Land Survey: Terrain Classification ---"
  # bash "$SCRIPT_DIR/download-land-survey.sh"
  #
  # echo "--- Tokyo: Liquefaction Data ---"
  # bash "$SCRIPT_DIR/download-tokyo-liquefaction.sh"
  #
  # echo "--- PLATEAU: 3D City Models ---"
  # bash "$SCRIPT_DIR/download-plateau-tokyo.sh"
else
  echo "─── Step 1: Download skipped (--skip-download) ───"
fi

echo ""

# ─── Step 2: Convert to GeoJSON ──────────────────────────
echo "─── Step 2: Convert to GeoJSON ───"
echo ""

# Future scripts (uncomment as they are created):
# echo "--- Converting GML to GeoJSON ---"
# python3 "$SCRIPT_DIR/convert-gml-to-geojson.py"
#
# echo "--- Converting SHP to GeoJSON ---"
# python3 "$SCRIPT_DIR/convert-shp-to-geojson.py"
#
# echo "--- Splitting N03 into prefectures/municipalities ---"
# python3 "$SCRIPT_DIR/convert-n03-boundaries.py"

echo "(Conversion scripts not yet implemented)"
echo ""

# ─── Step 3: Import to PostGIS ────────────────────────────
echo "─── Step 3: Import to PostGIS ───"
echo ""

# Future scripts (uncomment as they are created):
# echo "--- Importing to PostGIS ---"
# python3 "$SCRIPT_DIR/import-to-postgis.py"

echo "(Import scripts not yet implemented)"
echo ""

# ─── Summary ──────────────────────────────────────────────
echo "=== Pipeline Complete ==="
echo "Finished: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""
echo "Data directories:"
echo "  Raw:       data/raw/"
echo "  Processed: data/processed/"
echo "  Frontend:  services/frontend/public/geojson/"
