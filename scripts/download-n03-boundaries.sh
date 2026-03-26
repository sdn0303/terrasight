#!/usr/bin/env bash
# scripts/download-n03-boundaries.sh
# Downloads N03-2025 administrative boundary data (47 prefectures + municipalities)
# Source: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2025.html
set -euo pipefail

DATA_DIR="data/raw/n03"
OUTPUT_DIR="services/frontend/public/geojson/n03"

mkdir -p "$DATA_DIR" "$OUTPUT_DIR"

echo "[N03] Downloading administrative boundaries (2025)..."

# Full nationwide GeoJSON (preferred — single file ~600MB)
N03_URL="https://nlftp.mlit.go.jp/ksj/gml/data/N03/N03-2025/N03-20250101_GML.zip"
N03_ZIP="$DATA_DIR/N03-2025.zip"

if [ ! -f "$N03_ZIP" ]; then
  curl -L --progress-bar -o "$N03_ZIP" "$N03_URL"
  echo "[N03] Downloaded: $(du -h "$N03_ZIP" | cut -f1)"
else
  echo "[N03] Already downloaded: $N03_ZIP"
fi

echo "[N03] Extracting..."
unzip -oq "$N03_ZIP" -d "$DATA_DIR/extracted/" 2>/dev/null || true

echo "[N03] Done. Raw files in $DATA_DIR/extracted/"
echo "[N03] Next step: Run scripts/convert-n03-boundaries.py to split into prefecture/municipality GeoJSON"
echo ""
echo "Expected output files:"
echo "  $OUTPUT_DIR/prefectures.geojson  (47 prefecture polygons)"
echo "  $OUTPUT_DIR/municipalities.geojson  (all municipality polygons)"
