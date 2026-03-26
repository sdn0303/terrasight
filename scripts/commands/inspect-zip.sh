#!/usr/bin/env bash
# =============================================================================
# inspect-zip.sh — Inspect contents of a raw data ZIP file
# =============================================================================
# Usage:
#   ./scripts/commands/inspect-zip.sh data/raw/A29-19_13_GML.zip
#   ./scripts/commands/inspect-zip.sh data/raw/N03-20250101_GML.zip
#   ./scripts/commands/inspect-zip.sh --all    # inspect all ZIPs (summary)
# =============================================================================
set -euo pipefail

if [ "${1:-}" = "--all" ]; then
  echo "=== All ZIPs in data/raw/ ==="
  for f in data/raw/*.zip; do
    name=$(basename "$f")
    count=$(unzip -l "$f" 2>/dev/null | grep -cE "\.(geojson|shp|gml|csv|xml)" || echo "0")
    size=$(du -h "$f" | cut -f1)
    printf "  %-50s %6s  %3s data files\n" "$name" "$size" "$count"
  done
  exit 0
fi

ZIP="${1:?Usage: inspect-zip.sh <path-to-zip>}"

if [ ! -f "$ZIP" ]; then
  echo "ERROR: $ZIP not found" >&2
  exit 1
fi

echo "=== $(basename "$ZIP") ==="
echo "Size: $(du -h "$ZIP" | cut -f1)"
echo ""

echo "--- Geospatial files ---"
unzip -l "$ZIP" 2>/dev/null | grep -E "\.(geojson|shp|gml|csv)" | grep -v "__MACOSX" | head -20

echo ""
echo "--- All files ---"
unzip -l "$ZIP" 2>/dev/null | grep -v "__MACOSX" | tail -20
